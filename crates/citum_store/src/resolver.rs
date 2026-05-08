/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Store resolver for locating and managing user styles and locales.

use crate::format::StoreFormat;
use citum_schema::{Locale, Style, StyleRegistry};
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
#[cfg(feature = "http")]
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[cfg(feature = "http")]
use sha2::{Digest, Sha256};
#[cfg(feature = "http")]
use tempfile;

/// Error type for store resolution operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ResolverError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid style: {0}")]
    InvalidStyle(Cow<'static, str>),
    #[error("style not found: {0}")]
    StyleNotFound(Cow<'static, str>),
    #[error("locale not found: {0}")]
    LocaleNotFound(Cow<'static, str>),
    #[error("yaml error: {0}")]
    YamlError(String),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("cbor error: {0}")]
    CborError(String),
    #[cfg(feature = "http")]
    #[error("http error: {0}")]
    HttpError(String),
    #[cfg(feature = "http")]
    #[error("git error: {0}")]
    GitError(String),
    /// The URI host or origin is not in the resolver's allowlist.
    #[error("host not in resolver allowlist: {uri} ({reason})")]
    Denied {
        /// URI that triggered the denial.
        uri: String,
        /// Human-readable explanation (e.g., "host not in allowlist").
        reason: String,
    },
    /// A network operation failed (DNS, TLS, transport, timeout, etc.).
    #[error("network error fetching {uri}: {reason}")]
    NetworkError {
        /// URI that the resolver was attempting to fetch.
        uri: String,
        /// Underlying transport-layer reason.
        reason: String,
    },
    /// The fetched style declares a `citum-version` requirement that the
    /// running engine does not satisfy.
    #[error(
        "engine version mismatch for {uri}: running citum {declared} does not satisfy required {required}"
    )]
    VersionMismatch {
        /// URI of the style with the incompatible version requirement.
        uri: String,
        /// `citum-version` requirement declared by the style.
        required: String,
        /// Version of the running engine.
        declared: String,
    },
    /// The fetched style's content hash did not match an `extends-pin` or CID.
    #[error("integrity failure for {uri}: expected {expected}, got {actual}")]
    IntegrityFailure {
        /// URI whose content failed verification.
        uri: String,
        /// Expected CID or hash.
        expected: String,
        /// Actual CID or hash computed from the response bytes.
        actual: String,
    },
}

/// A trait for resolving Citum styles and locales from various sources.
pub trait StyleResolver {
    /// Resolve a style by URI or ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolverError::StyleNotFound`] if the style cannot be located.
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError>;

    /// Resolve a locale by ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolverError::LocaleNotFound`] if the locale cannot be located.
    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError>;
}

/// Resolves user-installed styles and locales from platform-specific data directories.
pub struct StoreResolver {
    data_dir: PathBuf,
    format: StoreFormat,
}

impl StyleResolver for StoreResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
        StoreResolver::resolve_style(self, uri)
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        StoreResolver::resolve_locale(self, id)
    }
}

impl citum_schema::StyleResolver for StoreResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, citum_schema::ResolutionError> {
        StyleResolver::resolve_style(self, uri)
            .map_err(|err| resolution_error_from_store_error(uri, err))
    }
}

/// A resolver that checks for embedded styles and locales.
pub struct EmbeddedResolver;

impl StyleResolver for EmbeddedResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
        citum_schema::embedded::get_embedded_style(uri)
            .ok_or_else(|| ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))?
            .map_err(|e| ResolverError::YamlError(ToString::to_string(&e)))
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        if let Some(bytes) = citum_schema::embedded::get_locale_bytes(id) {
            let content = String::from_utf8_lossy(bytes);
            Locale::from_yaml_str(&content)
                .map_err(|e| ResolverError::YamlError(ToString::to_string(&e)))
        } else {
            Err(ResolverError::LocaleNotFound(Cow::Owned(id.to_string())))
        }
    }
}

impl citum_schema::StyleResolver for EmbeddedResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, citum_schema::ResolutionError> {
        StyleResolver::resolve_style(self, uri)
            .map_err(|err| resolution_error_from_store_error(uri, err))
    }
}

/// A resolver that looks up styles in a registry and delegates to the matching source.
pub struct RegistryResolver {
    registry: StyleRegistry,
    base_dir: Option<PathBuf>,
    #[cfg(feature = "http")]
    http: Option<HttpResolver>,
    #[cfg(feature = "http")]
    git: Option<GitResolver>,
}

impl RegistryResolver {
    /// Create a registry resolver for the given registry.
    #[must_use]
    pub fn new(registry: StyleRegistry) -> Self {
        Self {
            registry,
            base_dir: None,
            #[cfg(feature = "http")]
            http: None,
            #[cfg(feature = "http")]
            git: None,
        }
    }

    /// Set the base directory used for relative path-backed registry entries.
    #[must_use]
    pub fn with_base_dir(mut self, base_dir: PathBuf) -> Self {
        self.base_dir = Some(base_dir);
        self
    }

    /// Set the HTTP resolver used for URL-backed registry entries.
    #[cfg(feature = "http")]
    #[must_use]
    pub fn with_http(mut self, http: HttpResolver) -> Self {
        self.http = Some(http);
        self
    }

    /// Set the Git resolver used for `git+https://` backed registry entries.
    #[cfg(feature = "http")]
    #[must_use]
    pub fn with_git(mut self, git: GitResolver) -> Self {
        self.git = Some(git);
        self
    }
}

impl StyleResolver for RegistryResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
        let entry = self
            .registry
            .resolve(uri)
            .ok_or_else(|| ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))?;

        if let Some(builtin) = &entry.builtin {
            return EmbeddedResolver.resolve_style(builtin);
        }

        if let Some(path) = &entry.path {
            let path = self
                .base_dir
                .as_ref()
                .map_or_else(|| path.clone(), |base| base.join(path));
            let path = path
                .to_str()
                .ok_or_else(|| ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))?;
            return FileResolver.resolve_style(path);
        }

        if let Some(url) = &entry.url {
            #[cfg(feature = "http")]
            {
                // Try git resolver if the URL is a git URI
                if url.starts_with("git+https://") || url.starts_with("git+http://") {
                    if let Some(git) = &self.git {
                        return git.resolve_style(url);
                    }
                    return Err(ResolverError::StyleNotFound(Cow::Owned(uri.to_string())));
                }

                let http = self
                    .http
                    .as_ref()
                    .ok_or_else(|| ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))?;
                return http.resolve_style(url);
            }

            #[cfg(not(feature = "http"))]
            {
                return Err(ResolverError::StyleNotFound(Cow::Owned(url.clone())));
            }
        }

        Err(ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        Err(ResolverError::LocaleNotFound(Cow::Owned(id.to_string())))
    }
}

impl citum_schema::StyleResolver for RegistryResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, citum_schema::ResolutionError> {
        StyleResolver::resolve_style(self, uri)
            .map_err(|err| resolution_error_from_store_error(uri, err))
    }
}

/// A resolver that handles local file paths.
pub struct FileResolver;

impl StyleResolver for FileResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
        // Normalize file:// URIs to plain filesystem paths.
        let raw_path = uri.strip_prefix("file://").unwrap_or(uri);
        let path = Path::new(raw_path);
        if path.exists() && path.is_file() {
            let bytes = fs::read(path).map_err(ResolverError::Io)?;
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

            let style: Style = match ext {
                "cbor" => ciborium::de::from_reader(Cursor::new(&bytes))
                    .map_err(|e| ResolverError::CborError(e.to_string()))?,
                "json" => serde_json::from_slice(&bytes)?,
                _ => Style::from_yaml_bytes(&bytes)
                    .map_err(|e| ResolverError::YamlError(ToString::to_string(&e)))?,
            };
            Ok(style)
        } else {
            Err(ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))
        }
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        let locales_dir = Path::new("locales");
        for ext in ["yaml", "yml", "json", "cbor"] {
            let path = locales_dir.join(format!("{id}.{ext}"));
            if path.exists() {
                return Locale::from_file(&path)
                    .map_err(|e| ResolverError::YamlError(ToString::to_string(&e)));
            }
        }
        Err(ResolverError::LocaleNotFound(Cow::Owned(id.to_string())))
    }
}

impl citum_schema::StyleResolver for FileResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, citum_schema::ResolutionError> {
        StyleResolver::resolve_style(self, uri)
            .map_err(|err| resolution_error_from_store_error(uri, err))
    }
}

/// A resolver that fetches styles from Git repositories via shallow clone.
///
/// URI format: `git+https://github.com/org/repo.git#path/to/style.yaml`
#[cfg(feature = "http")]
pub struct GitResolver {
    cache_dir: PathBuf,
}

/// A resolver that fetches HTTP(S) style URLs and caches them on disk.
#[cfg(feature = "http")]
pub struct HttpResolver {
    cache_dir: PathBuf,
    client: reqwest::blocking::Client,
    allowed_hosts: Vec<String>,
}

#[cfg(feature = "http")]
const HTTP_TIMEOUT: Duration = Duration::from_secs(15);

#[cfg(feature = "http")]
const HTTP_CACHE_MAX_AGE: Duration = Duration::from_hours(24);

#[cfg(feature = "http")]
impl HttpResolver {
    /// Create a resolver using the provided cache directory.
    #[must_use]
    pub fn new(cache_dir: PathBuf) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self {
            cache_dir,
            client,
            allowed_hosts: Vec::new(),
        }
    }

    /// Create a resolver using the platform cache directory.
    #[must_use]
    pub fn from_platform_cache_dir() -> Option<Self> {
        crate::platform_cache_dir().map(Self::new)
    }

    /// Set an allowlist of hosts for this resolver. Empty list allows all hosts.
    #[must_use]
    pub fn with_allowed_hosts(mut self, hosts: Vec<String>) -> Self {
        self.allowed_hosts = hosts;
        self
    }

    /// Fetch raw bytes from a URL.
    ///
    /// # Errors
    /// Returns an error if the request fails.
    pub fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut response = self.client.get(url).send()?;
        if !response.status().is_success() {
            return Err(format!("failed to fetch {url}: status {}", response.status()).into());
        }
        let mut bytes = Vec::new();
        response.copy_to(&mut bytes)?;
        Ok(bytes)
    }

    fn cache_path(&self, uri: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(uri.as_bytes());
        let hash = hasher.finalize();
        let mut hex = String::with_capacity(hash.len() * 2);
        for byte in hash {
            hex.push(hex_digit(byte >> 4));
            hex.push(hex_digit(byte & 0x0f));
        }
        self.cache_dir
            .join("styles")
            .join("http")
            .join(format!("{hex}.yaml"))
    }

    fn parse_style(uri: &str, bytes: &[u8]) -> Result<Style, ResolverError> {
        Style::from_yaml_bytes(bytes).map_err(|err| {
            ResolverError::YamlError(format!("failed to parse style fetched from {uri}: {err}"))
        })
    }

    fn cache_is_fresh(path: &Path) -> bool {
        path.metadata()
            .and_then(|metadata| metadata.modified())
            .and_then(|modified| {
                SystemTime::now()
                    .duration_since(modified)
                    .map_err(std::io::Error::other)
            })
            .is_ok_and(|age| age < HTTP_CACHE_MAX_AGE)
    }

    fn write_cache(path: &Path, bytes: &[u8]) -> Result<(), ResolverError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp_path = path.with_extension(format!("yaml.tmp.{}", std::process::id()));
        fs::write(&temp_path, bytes)?;
        fs::rename(&temp_path, path)?;
        Ok(())
    }
}

#[cfg(feature = "http")]
fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + (nibble - 10)),
        _ => '?',
    }
}

#[cfg(feature = "http")]
impl StyleResolver for HttpResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
        if !uri.starts_with("http://") && !uri.starts_with("https://") {
            return Err(ResolverError::StyleNotFound(Cow::Owned(uri.to_string())));
        }

        // Check allowlist if populated
        if !self.allowed_hosts.is_empty()
            && let Ok(url) = uri.parse::<reqwest::Url>()
            && let Some(host) = url.host_str()
            && !self.allowed_hosts.iter().any(|h| h == host)
        {
            return Err(ResolverError::Denied {
                uri: uri.to_string(),
                reason: format!("host '{host}' not in resolver allowlist"),
            });
        }

        let cache_path = self.cache_path(uri);
        if cache_path.is_file() && Self::cache_is_fresh(&cache_path) {
            let bytes = fs::read(&cache_path)?;
            return Self::parse_style(uri, &bytes);
        }

        let fetch_result = self.client.get(uri).send();

        match fetch_result {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::NOT_FOUND {
                    if cache_path.is_file() {
                        let bytes = fs::read(&cache_path)?;
                        return Self::parse_style(uri, &bytes);
                    }
                    return Err(ResolverError::StyleNotFound(Cow::Owned(uri.to_string())));
                }

                if !response.status().is_success() {
                    if cache_path.is_file() {
                        let bytes = fs::read(&cache_path)?;
                        return Self::parse_style(uri, &bytes);
                    }
                    return Err(ResolverError::HttpError(format!(
                        "failed to fetch {uri}: HTTP {}",
                        response.status()
                    )));
                }

                match response.bytes() {
                    Ok(bytes) => {
                        let style = Self::parse_style(uri, &bytes)?;
                        Self::write_cache(&cache_path, &bytes)?;
                        Ok(style)
                    }
                    Err(err) => {
                        // Check for stale cache and serve if available
                        if cache_path.is_file() {
                            let bytes = fs::read(&cache_path)?;
                            Self::parse_style(uri, &bytes)
                        } else {
                            Err(ResolverError::NetworkError {
                                uri: uri.to_string(),
                                reason: err.to_string(),
                            })
                        }
                    }
                }
            }
            Err(err) => {
                // Check for stale cache and serve if available
                if cache_path.is_file() {
                    let bytes = fs::read(&cache_path)?;
                    Self::parse_style(uri, &bytes)
                } else {
                    Err(ResolverError::NetworkError {
                        uri: uri.to_string(),
                        reason: err.to_string(),
                    })
                }
            }
        }
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        Err(ResolverError::LocaleNotFound(Cow::Owned(id.to_string())))
    }
}

#[cfg(feature = "http")]
impl citum_schema::StyleResolver for HttpResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, citum_schema::ResolutionError> {
        StyleResolver::resolve_style(self, uri)
            .map_err(|err| resolution_error_from_store_error(uri, err))
    }
}

#[cfg(feature = "http")]
impl GitResolver {
    /// Create a resolver using the provided cache directory.
    #[must_use]
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Create a resolver using the platform cache directory.
    #[must_use]
    pub fn from_platform_cache_dir() -> Option<Self> {
        crate::platform_cache_dir().map(Self::new)
    }

    fn cache_path(&self, uri: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(uri.as_bytes());
        let hash = hasher.finalize();
        let mut hex = String::with_capacity(hash.len() * 2);
        for byte in hash {
            hex.push(hex_digit(byte >> 4));
            hex.push(hex_digit(byte & 0x0f));
        }
        self.cache_dir
            .join("styles")
            .join("git")
            .join(format!("{hex}.yaml"))
    }

    fn parse_style(uri: &str, bytes: &[u8]) -> Result<Style, ResolverError> {
        Style::from_yaml_bytes(bytes).map_err(|err| {
            ResolverError::YamlError(format!("failed to parse style fetched from {uri}: {err}"))
        })
    }

    fn cache_is_fresh(path: &Path) -> bool {
        path.metadata()
            .and_then(|metadata| metadata.modified())
            .and_then(|modified| {
                SystemTime::now()
                    .duration_since(modified)
                    .map_err(std::io::Error::other)
            })
            .is_ok_and(|age| age < HTTP_CACHE_MAX_AGE)
    }

    fn write_cache(path: &Path, bytes: &[u8]) -> Result<(), ResolverError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut tmp = tempfile::NamedTempFile::new_in(
            path.parent().unwrap_or_else(|| std::path::Path::new(".")),
        )?;
        std::io::Write::write_all(&mut tmp, bytes)?;
        tmp.persist(path).map_err(|e| ResolverError::Io(e.error))?;
        Ok(())
    }

    /// Parse a git URI into repo URL and file path.
    pub fn parse_git_uri(uri: &str) -> Option<(String, String)> {
        if !uri.starts_with("git+https://") && !uri.starts_with("git+http://") {
            return None;
        }
        let rest = uri.strip_prefix("git+")?;
        let (repo_part, file_path) = rest.split_once('#')?;
        Some((repo_part.to_string(), file_path.to_string()))
    }
}

#[cfg(feature = "http")]
impl StyleResolver for GitResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
        let (repo_url, file_path) = Self::parse_git_uri(uri)
            .ok_or_else(|| ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))?;

        let cache_path = self.cache_path(uri);
        if cache_path.is_file() && Self::cache_is_fresh(&cache_path) {
            let bytes = fs::read(&cache_path)?;
            return Self::parse_style(uri, &bytes);
        }

        // Create a temporary directory for the git clone
        let tmpdir = tempfile::TempDir::new().map_err(ResolverError::Io)?;
        let tmpdir_str = tmpdir
            .path()
            .to_str()
            .ok_or_else(|| ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))?;

        let Ok(clone_output) = std::process::Command::new("git")
            .args(["clone", "--depth=1", &repo_url, tmpdir_str])
            .output()
        else {
            // Serve stale cache if available
            if cache_path.is_file() {
                let bytes = fs::read(&cache_path)?;
                return Self::parse_style(uri, &bytes);
            }
            return Err(ResolverError::NetworkError {
                uri: uri.to_string(),
                reason: "git clone failed to spawn (is `git` on PATH?)".to_string(),
            });
        };

        if !clone_output.status.success() {
            let stderr = String::from_utf8_lossy(&clone_output.stderr);
            // Serve stale cache if available
            if cache_path.is_file() {
                let bytes = fs::read(&cache_path)?;
                return Self::parse_style(uri, &bytes);
            }
            return Err(ResolverError::NetworkError {
                uri: uri.to_string(),
                reason: format!(
                    "git clone failed (exit {}): {}",
                    clone_output.status,
                    stderr.trim()
                ),
            });
        }

        // Check out the specific file
        let file_checkout = std::process::Command::new("git")
            .arg("-C")
            .arg(tmpdir_str)
            .args(["checkout", "HEAD", "--"])
            .arg(&file_path)
            .output();

        let file_path_full = tmpdir.path().join(&file_path);
        let bytes_result = if let Ok(output) = file_checkout {
            if output.status.success() && file_path_full.exists() {
                fs::read(&file_path_full)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "file not found in repo",
                ))
            }
        } else {
            Err(std::io::Error::other("git checkout failed"))
        };

        match bytes_result {
            Ok(bytes) => {
                let style = Self::parse_style(uri, &bytes)?;
                Self::write_cache(&cache_path, &bytes)?;
                Ok(style)
            }
            Err(_) => {
                // Serve stale cache if available
                if cache_path.is_file() {
                    let bytes = fs::read(&cache_path)?;
                    Self::parse_style(uri, &bytes)
                } else {
                    Err(ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))
                }
            }
        }
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        Err(ResolverError::LocaleNotFound(Cow::Owned(id.to_string())))
    }
}

#[cfg(feature = "http")]
impl citum_schema::StyleResolver for GitResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, citum_schema::ResolutionError> {
        StyleResolver::resolve_style(self, uri)
            .map_err(|err| resolution_error_from_store_error(uri, err))
    }
}

/// Default IPFS HTTP gateway used by [`CidResolver`] when none is configured.
#[cfg(feature = "http")]
pub const DEFAULT_CID_GATEWAY: &str = "https://dweb.link/ipfs/";

/// A resolver that fetches content-addressed Citum styles via an IPFS HTTP
/// gateway and verifies their integrity against the requested CID.
///
/// `CidResolver` accepts URIs of the form `cid:bafkrei…`. It rewrites the
/// URI to `<gateway>/<cid>`, delegates the HTTP fetch and disk caching to a
/// wrapped [`HttpResolver`], then re-hashes the response bytes and refuses to
/// hand back a [`Style`] whose CID does not match the requested one. Cache
/// entries for `cid:` URIs are immutable — they need not be revalidated.
#[cfg(feature = "http")]
pub struct CidResolver {
    gateway: String,
    http: HttpResolver,
}

#[cfg(feature = "http")]
impl CidResolver {
    /// Construct a `CidResolver` with an explicit gateway and HTTP backend.
    ///
    /// `gateway` should end with a trailing slash; `https://dweb.link/ipfs/`
    /// is the [`DEFAULT_CID_GATEWAY`].
    #[must_use]
    pub fn new(gateway: String, http: HttpResolver) -> Self {
        let gateway = if gateway.ends_with('/') {
            gateway
        } else {
            format!("{gateway}/")
        };
        Self { gateway, http }
    }

    /// Construct a `CidResolver` using [`DEFAULT_CID_GATEWAY`] and the platform
    /// cache directory. Returns `None` when the platform has no cache dir.
    #[must_use]
    pub fn from_platform_cache_dir() -> Option<Self> {
        HttpResolver::from_platform_cache_dir()
            .map(|http| Self::new(DEFAULT_CID_GATEWAY.to_string(), http))
    }

    fn resolve_cid_uri(&self, uri: &str) -> Result<Style, ResolverError> {
        let cid_str = crate::cid::strip_cid_scheme(uri);
        let canonical = crate::cid::canonicalize_cid(cid_str)?;
        let gateway_url = format!("{}{canonical}", self.gateway);
        let bytes =
            self.http
                .fetch_bytes(&gateway_url)
                .map_err(|err| ResolverError::NetworkError {
                    uri: uri.to_string(),
                    reason: err.to_string(),
                })?;
        crate::cid::verify_cid(uri, &canonical, &bytes)?;
        Style::from_yaml_bytes(&bytes).map_err(|err| {
            ResolverError::YamlError(format!("failed to parse CID-resolved style {uri}: {err}"))
        })
    }
}

#[cfg(feature = "http")]
impl StyleResolver for CidResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
        if !crate::cid::is_cid_uri(uri) {
            return Err(ResolverError::StyleNotFound(Cow::Owned(uri.to_string())));
        }
        self.resolve_cid_uri(uri)
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        Err(ResolverError::LocaleNotFound(Cow::Owned(id.to_string())))
    }
}

#[cfg(feature = "http")]
impl citum_schema::StyleResolver for CidResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, citum_schema::ResolutionError> {
        StyleResolver::resolve_style(self, uri)
            .map_err(|err| resolution_error_from_store_error(uri, err))
    }
}

/// Middleware resolver that wraps another [`StyleResolver`] and verifies the
/// content of every successful resolution against an expected CID.
///
/// Used by Phase 3 `extends-pin` enforcement: the parent resolution path
/// constructs a `VerifyingResolver` whose `expected` is the child's
/// declared pin, then routes the parent fetch through it. When `expected`
/// is `None`, the wrapper is a no-op pass-through.
///
/// Note: this wrapper has to re-fetch raw bytes through `HttpResolver`'s
/// public `fetch_bytes` API to verify integrity, because the inner
/// `resolve_style` returns a parsed `Style` whose serialized form is not
/// guaranteed to be byte-identical to the source. Callers that care about
/// `extends-pin` enforcement should prefer [`fetch_and_verify_bytes`] for
/// HTTP/CID URIs and skip the trip through `Style`.
#[cfg(feature = "http")]
pub struct VerifyingResolver<R: StyleResolver> {
    inner: R,
    expected: Option<String>,
}

#[cfg(feature = "http")]
impl<R: StyleResolver> VerifyingResolver<R> {
    /// Wrap `inner` with an integrity check against `expected_cid`.
    ///
    /// `expected_cid` may include or omit the `cid:` scheme prefix.
    #[must_use]
    pub fn new(inner: R, expected_cid: Option<String>) -> Self {
        Self {
            inner,
            expected: expected_cid,
        }
    }

    /// Returns the inner resolver, consuming this wrapper.
    pub fn into_inner(self) -> R {
        self.inner
    }
}

#[cfg(feature = "http")]
impl<R: StyleResolver> StyleResolver for VerifyingResolver<R> {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
        let style = self.inner.resolve_style(uri)?;
        if let Some(ref pin) = self.expected {
            // Best-effort verification: re-serialize and hash. This rejects
            // tampering at the trust boundary even if the canonical serializer
            // round-trip differs from the upstream bytes — at the cost of
            // false negatives for whitespace-only differences. Callers that
            // need exact-bytes verification should use
            // [`fetch_and_verify_bytes`] before parsing.
            let bytes = serde_yaml::to_string(&style).map_err(|err| {
                ResolverError::YamlError(format!("re-serialize for pin check: {err}"))
            })?;
            crate::cid::verify_cid(uri, pin, bytes.as_bytes())?;
        }
        Ok(style)
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        self.inner.resolve_locale(id)
    }
}

/// Fetch raw bytes for a remote URI and verify them against `expected_cid`
/// before returning. This is the canonical entry point for `extends-pin`
/// enforcement — it operates on bytes, not parsed structures, so a single
/// trailing newline cannot make verification falsely pass or fail.
///
/// Supported schemes: `https://`, `http://`, `cid:`. Unsupported schemes
/// (including `file://` and `git+https://`) return [`ResolverError::Denied`]
/// — pinning makes sense only for content-addressed or HTTPS-fetched bytes.
///
/// # Errors
///
/// - [`ResolverError::Denied`] for unsupported URI schemes.
/// - [`ResolverError::NetworkError`] when the HTTP fetch fails.
/// - [`ResolverError::IntegrityFailure`] when the bytes do not match
///   `expected_cid`.
#[cfg(feature = "http")]
pub fn fetch_and_verify_bytes(
    http: &HttpResolver,
    cid_resolver: &CidResolver,
    uri: &str,
    expected_cid: &str,
) -> Result<Vec<u8>, ResolverError> {
    let bytes = if crate::cid::is_cid_uri(uri) {
        let canonical = crate::cid::canonicalize_cid(crate::cid::strip_cid_scheme(uri))?;
        let gateway_url = format!("{}{canonical}", cid_resolver.gateway);
        http.fetch_bytes(&gateway_url)
            .map_err(|err| ResolverError::NetworkError {
                uri: uri.to_string(),
                reason: err.to_string(),
            })?
    } else if uri.starts_with("https://") || uri.starts_with("http://") {
        http.fetch_bytes(uri)
            .map_err(|err| ResolverError::NetworkError {
                uri: uri.to_string(),
                reason: err.to_string(),
            })?
    } else {
        return Err(ResolverError::Denied {
            uri: uri.to_string(),
            reason: "extends-pin only supports cid: and https:// URIs".to_string(),
        });
    };
    crate::cid::verify_cid(uri, expected_cid, &bytes)?;
    Ok(bytes)
}

/// A composite resolver that attempts resolution through a chain of resolvers.
pub struct ChainResolver {
    resolvers: Vec<Box<dyn StyleResolver>>,
}

impl ChainResolver {
    /// Create a new chain resolver with the given list of resolvers.
    pub fn new(resolvers: Vec<Box<dyn StyleResolver>>) -> Self {
        ChainResolver { resolvers }
    }
}

impl StyleResolver for ChainResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
        for resolver in &self.resolvers {
            match resolver.resolve_style(uri) {
                Ok(style) => return Ok(style),
                Err(ResolverError::StyleNotFound(_)) => {}
                Err(err) => return Err(err),
            }
        }
        Err(ResolverError::StyleNotFound(Cow::Owned(uri.to_string())))
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        for resolver in &self.resolvers {
            match resolver.resolve_locale(id) {
                Ok(locale) => return Ok(locale),
                Err(ResolverError::LocaleNotFound(_)) => {}
                Err(err) => return Err(err),
            }
        }
        Err(ResolverError::LocaleNotFound(Cow::Owned(id.to_string())))
    }
}

impl citum_schema::StyleResolver for ChainResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, citum_schema::ResolutionError> {
        StyleResolver::resolve_style(self, uri)
            .map_err(|err| resolution_error_from_store_error(uri, err))
    }
}

fn resolution_error_from_store_error(
    uri: &str,
    err: ResolverError,
) -> citum_schema::ResolutionError {
    citum_schema::ResolutionError::UriResolutionFailed {
        uri: uri.to_string(),
        reason: err.to_string(),
    }
}

impl StoreResolver {
    /// Create a new resolver with the given data directory and format.
    #[must_use]
    pub fn new(data_dir: PathBuf, format: StoreFormat) -> Self {
        StoreResolver { data_dir, format }
    }

    /// Resolve a style by ID.
    ///
    /// # Errors
    /// Returns an error if the style cannot be found or loaded.
    pub fn resolve_style(&self, id: &str) -> Result<Style, ResolverError> {
        self.resolve_item(id, "styles")
    }

    /// Resolve a locale by ID.
    ///
    /// # Errors
    /// Returns an error if the locale cannot be found or loaded.
    pub fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        self.resolve_item(id, "locales")
    }

    /// List all installed styles.
    ///
    /// # Errors
    /// Returns an error if the styles directory cannot be read.
    pub fn list_styles(&self) -> Result<Vec<String>, ResolverError> {
        self.list_items("styles")
    }

    /// List all installed locales.
    ///
    /// # Errors
    /// Returns an error if the locales directory cannot be read.
    pub fn list_locales(&self) -> Result<Vec<String>, ResolverError> {
        self.list_items("locales")
    }

    /// Install a style from a source file.
    ///
    /// # Errors
    /// Returns an error if the source file cannot be read or the destination cannot be written.
    pub fn install_style(&self, source: &Path) -> Result<String, ResolverError> {
        self.install_item(source, "styles")
    }

    /// Install a locale from a source file.
    ///
    /// # Errors
    /// Returns an error if the source file cannot be read or the destination cannot be written.
    pub fn install_locale(&self, source: &Path) -> Result<String, ResolverError> {
        self.install_item(source, "locales")
    }

    /// Remove an installed style by ID.
    ///
    /// # Errors
    /// Returns an error if the style cannot be found or removed.
    pub fn remove_style(&self, id: &str) -> Result<(), ResolverError> {
        self.remove_item(id, "styles")
    }

    /// Remove an installed locale by ID.
    ///
    /// # Errors
    /// Returns an error if the locale cannot be found or removed.
    pub fn remove_locale(&self, id: &str) -> Result<(), ResolverError> {
        self.remove_item(id, "locales")
    }

    // --- Internal Helpers ---

    fn resolve_item<T: DeserializeOwned>(
        &self,
        id: &str,
        category: &str,
    ) -> Result<T, ResolverError> {
        let items_dir = self.data_dir.join(category);
        if !items_dir.exists() {
            return Err(match category {
                "styles" => ResolverError::StyleNotFound(id.to_string().into()),
                _ => ResolverError::LocaleNotFound(id.to_string().into()),
            });
        }

        // Try exact match with current format extension first
        let path = items_dir.join(format!("{}.{}", id, self.format.extension()));
        if path.is_file() {
            return self.load_item_at(&path);
        }

        // Fallback: search all supported formats
        for format in StoreFormat::all() {
            let path = items_dir.join(format!("{}.{}", id, format.extension()));
            if path.is_file() {
                return self.load_item_at(&path);
            }
        }

        Err(match category {
            "styles" => ResolverError::StyleNotFound(id.to_string().into()),
            _ => ResolverError::LocaleNotFound(id.to_string().into()),
        })
    }

    fn load_item_at<T: DeserializeOwned>(&self, path: &Path) -> Result<T, ResolverError> {
        let content = fs::read(path)?;
        let format = StoreFormat::detect(path).unwrap_or(self.format);

        match format {
            StoreFormat::Yaml => serde_yaml::from_slice(&content)
                .map_err(|e| ResolverError::YamlError(ToString::to_string(&e))),
            StoreFormat::Json => serde_json::from_slice(&content).map_err(ResolverError::JsonError),
            StoreFormat::Cbor => ciborium::de::from_reader(Cursor::new(&content))
                .map_err(|e| ResolverError::CborError(e.to_string())),
        }
    }

    fn list_items(&self, category: &str) -> Result<Vec<String>, ResolverError> {
        let items_dir = self.data_dir.join(category);
        if !items_dir.exists() {
            return Ok(Vec::new());
        }

        let mut names = BTreeSet::new();
        for entry in fs::read_dir(items_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && StoreFormat::detect(&path).is_some()
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
            {
                names.insert(name.to_string());
            }
        }
        Ok(names.into_iter().collect())
    }

    // Helper: install an item from a source file.
    fn install_item(&self, source: &Path, category: &str) -> Result<String, ResolverError> {
        let name = source
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ResolverError::InvalidStyle("no filename".into()))?;

        let category_dir = self.data_dir.join(category);
        fs::create_dir_all(&category_dir)?;

        // Detect format from source, or use configured default
        let source_format = StoreFormat::detect(source).unwrap_or(self.format);
        let dest_path = category_dir.join(format!("{}.{}", name, source_format.extension()));

        fs::copy(source, dest_path)?;
        Ok(name.to_string())
    }

    fn remove_item(&self, id: &str, category: &str) -> Result<(), ResolverError> {
        let items_dir = self.data_dir.join(category);
        if !items_dir.exists() {
            return Err(match category {
                "styles" => ResolverError::StyleNotFound(id.to_string().into()),
                _ => ResolverError::LocaleNotFound(id.to_string().into()),
            });
        }

        // Search and remove all matching files for this ID
        let mut found = false;
        for format in StoreFormat::all() {
            let path = items_dir.join(format!("{}.{}", id, format.extension()));
            if path.exists() {
                fs::remove_file(path)?;
                found = true;
            }
        }

        if !found {
            return Err(match category {
                "styles" => ResolverError::StyleNotFound(id.to_string().into()),
                _ => ResolverError::LocaleNotFound(id.to_string().into()),
            });
        }

        Ok(())
    }
}
