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

/// Error type for store resolution operations.
#[derive(Error, Debug)]
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

/// A resolver that looks up styles in a registry and delegates to the matching source.
pub struct RegistryResolver {
    registry: StyleRegistry,
    base_dir: Option<PathBuf>,
    #[cfg(feature = "http")]
    http: Option<HttpResolver>,
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

/// A resolver that fetches HTTP(S) style URLs and caches them on disk.
#[cfg(feature = "http")]
pub struct HttpResolver {
    cache_dir: PathBuf,
    client: reqwest::blocking::Client,
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
        Self { cache_dir, client }
    }

    /// Create a resolver using the platform cache directory.
    #[must_use]
    pub fn from_platform_cache_dir() -> Option<Self> {
        dirs::cache_dir().map(|dir| Self::new(dir.join("citum")))
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

        let cache_path = self.cache_path(uri);
        if cache_path.is_file() && Self::cache_is_fresh(&cache_path) {
            let bytes = fs::read(&cache_path)?;
            return Self::parse_style(uri, &bytes);
        }

        let response = self
            .client
            .get(uri)
            .send()
            .map_err(|err| ResolverError::HttpError(format!("failed to fetch {uri}: {err}")))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ResolverError::StyleNotFound(Cow::Owned(uri.to_string())));
        }

        if !response.status().is_success() {
            return Err(ResolverError::HttpError(format!(
                "failed to fetch {uri}: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|err| ResolverError::HttpError(format!("failed to read {uri}: {err}")))?;
        let style = Self::parse_style(uri, &bytes)?;
        Self::write_cache(&cache_path, &bytes)?;
        Ok(style)
    }

    fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
        Err(ResolverError::LocaleNotFound(Cow::Owned(id.to_string())))
    }
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
