#![allow(missing_docs, reason = "lib/crate")]

//! Platform-aware store for user-installed Citum styles and locales.
//!
//! Provides a `StoreResolver` that searches user data directories for custom styles
//! and locales, with fallback to embedded builtins. Supports YAML, JSON, and CBOR formats.

#[cfg(feature = "http")]
pub mod chain;
#[cfg(feature = "http")]
pub mod cid;
pub mod config;
pub mod format;
pub mod paths;
pub mod resolver;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod resolver_tests;

#[cfg(feature = "http")]
pub use chain::build_standard_chain;
pub use config::StoreConfig;
pub use format::StoreFormat;
#[cfg(feature = "http")]
pub use resolver::ChainResolver;
#[cfg(feature = "http")]
pub use resolver::{
    CidResolver, DEFAULT_CID_GATEWAY, GitResolver, HttpResolver, VerifyingResolver,
    fetch_and_verify_bytes,
};
pub use resolver::{
    EmbeddedResolver, FileResolver, RegistryResolver, ResolverError, StoreResolver,
};

use std::path::PathBuf;

/// Returns the platform-specific data directory for Citum user files.
///
/// This path is derived from [`paths::data_dir()`] with a `citum` subdirectory appended.
///
/// Typical locations:
/// - Unix (incl. macOS): `~/.local/share/citum/`
/// - Windows: `%APPDATA%\citum\`
#[must_use]
pub fn platform_data_dir() -> Option<PathBuf> {
    paths::data_dir().map(|d| d.join("citum"))
}

/// Returns the platform-specific configuration directory for Citum.
///
/// This path is derived from [`paths::config_dir()`] with a `citum` subdirectory appended.
///
/// Typical locations:
/// - Unix (incl. macOS): `~/.config/citum/`
/// - Windows: `%APPDATA%\citum\`
#[must_use]
pub fn platform_config_dir() -> Option<PathBuf> {
    paths::config_dir().map(|d| d.join("citum"))
}

/// Returns the platform-specific cache directory for Citum.
///
/// This path is derived from [`paths::cache_dir()`] with a `citum` subdirectory appended.
///
/// Typical locations:
/// - Unix (incl. macOS): `~/.cache/citum/`
/// - Windows: `%LOCALAPPDATA%\citum\`
#[must_use]
pub fn platform_cache_dir() -> Option<PathBuf> {
    paths::cache_dir().map(|d| d.join("citum"))
}
