/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Store resolver for locating and managing user styles and locales.

use crate::format::StoreFormat;
use citum_schema::{Locale, Style};
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use thiserror::Error;

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
}

/// Resolves user-installed styles and locales from platform-specific data directories.
pub struct StoreResolver {
    data_dir: PathBuf,
    format: StoreFormat,
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
                .map_err(|e| ResolverError::YamlError(e.to_string())),
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
