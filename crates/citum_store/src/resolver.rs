//! Store resolver for locating and managing user styles and locales.

use crate::format::StoreFormat;
use citum_schema::Style;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Error type for store resolution operations.
#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid style: {0}")]
    InvalidStyle(String),
    #[error("style not found: {0}")]
    StyleNotFound(String),
    #[error("locale not found: {0}")]
    LocaleNotFound(String),
    #[error("yaml error: {0}")]
    YamlError(String),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("cbor error: {0}")]
    CborError(String),
}

/// Resolves user-installed styles and locales from platform-specific data directories.
pub struct StoreResolver {
    data_dir: PathBuf,
    format: StoreFormat,
}

impl StoreResolver {
    /// Create a new resolver with the given data directory and format.
    pub fn new(data_dir: PathBuf, format: StoreFormat) -> Self {
        StoreResolver { data_dir, format }
    }

    /// Resolve a style by name from the store.
    ///
    /// Searches for `data_dir/styles/<name>.{yaml,json,cbor}` and deserializes it.
    pub fn resolve_style(&self, name: &str) -> Result<Style, ResolverError> {
        self.resolve_item("styles", name)
    }

    /// List all installed style names (stems, no extension).
    pub fn list_styles(&self) -> Result<Vec<String>, ResolverError> {
        self.list_items("styles")
    }

    /// List all installed locale names (stems, no extension).
    pub fn list_locales(&self) -> Result<Vec<String>, ResolverError> {
        self.list_items("locales")
    }

    /// Install a style from a source file.
    ///
    /// Copies the file into `data_dir/styles/` using its stem as the name.
    /// Returns the installed style name.
    pub fn install_style(&self, source: &Path) -> Result<String, ResolverError> {
        self.install_item(source, "styles")
    }

    /// Remove an installed style by name.
    pub fn remove_style(&self, name: &str) -> Result<(), ResolverError> {
        self.remove_item(name, "styles")
    }

    /// Install a locale from a source file.
    ///
    /// Copies the file into `data_dir/locales/` using its stem as the name.
    /// Returns the installed locale name.
    pub fn install_locale(&self, source: &Path) -> Result<String, ResolverError> {
        self.install_item(source, "locales")
    }

    /// Remove an installed locale by name.
    pub fn remove_locale(&self, name: &str) -> Result<(), ResolverError> {
        self.remove_item(name, "locales")
    }

    // Helper: resolve an item (style or locale) by name from a category.
    fn resolve_item<T: serde::de::DeserializeOwned>(
        &self,
        category: &str,
        name: &str,
    ) -> Result<T, ResolverError> {
        let category_dir = self.data_dir.join(category);

        // Try exact format first, then fallback to other formats
        let formats_to_try = [
            self.format,
            StoreFormat::Json,
            StoreFormat::Yaml,
            StoreFormat::Cbor,
        ];

        for fmt in &formats_to_try {
            let ext = fmt.extension();
            let path = category_dir.join(format!("{}.{}", name, ext));

            if path.exists() && path.is_file() {
                return self.deserialize_item(&path, *fmt);
            }
        }

        // If we got here, file wasn't found in any format
        let error = if category == "locales" {
            ResolverError::LocaleNotFound(name.to_string())
        } else {
            ResolverError::StyleNotFound(name.to_string())
        };
        Err(error)
    }

    // Helper: deserialize an item from file based on format.
    fn deserialize_item<T: serde::de::DeserializeOwned>(
        &self,
        path: &Path,
        format: StoreFormat,
    ) -> Result<T, ResolverError> {
        let content = fs::read(path)?;

        match format {
            StoreFormat::Yaml => serde_yaml::from_slice(&content)
                .map_err(|e| ResolverError::YamlError(e.to_string())),
            StoreFormat::Json => serde_json::from_slice(&content).map_err(ResolverError::JsonError),
            StoreFormat::Cbor => ciborium::de::from_reader(content.as_slice())
                .map_err(|e| ResolverError::CborError(e.to_string())),
        }
    }

    // Helper: list items in a category directory.
    fn list_items(&self, category: &str) -> Result<Vec<String>, ResolverError> {
        let category_dir = self.data_dir.join(category);

        if !category_dir.exists() {
            return Ok(Vec::new());
        }

        let mut names = Vec::new();

        for entry in fs::read_dir(&category_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                && let Some(ext) = path.extension().and_then(|e| e.to_str())
                && matches!(ext, "yaml" | "yml" | "json" | "cbor")
            {
                names.push(name.to_string());
            }
        }

        names.sort();
        Ok(names)
    }

    // Helper: install an item from a source file.
    fn install_item(&self, source: &Path, category: &str) -> Result<String, ResolverError> {
        let name = source
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ResolverError::InvalidStyle("no filename".to_string()))?
            .to_string();

        let category_dir = self.data_dir.join(category);
        fs::create_dir_all(&category_dir)?;

        // Detect format from source, or use configured default
        let format = StoreFormat::detect_from_extension(source).unwrap_or(self.format);
        let ext = format.extension();
        let dest = category_dir.join(format!("{}.{}", name, ext));

        fs::copy(source, &dest)?;
        Ok(name)
    }

    // Helper: remove an item by name from a category.
    fn remove_item(&self, name: &str, category: &str) -> Result<(), ResolverError> {
        let category_dir = self.data_dir.join(category);

        // Try all possible formats
        for fmt in &[StoreFormat::Yaml, StoreFormat::Json, StoreFormat::Cbor] {
            let path = category_dir.join(format!("{}.{}", name, fmt.extension()));
            if path.exists() {
                fs::remove_file(path)?;
                return Ok(());
            }
        }

        if category == "locales" {
            Err(ResolverError::LocaleNotFound(name.to_string()))
        } else {
            Err(ResolverError::StyleNotFound(name.to_string()))
        }
    }
}
