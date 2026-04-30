#![allow(missing_docs, reason = "lib/crate")]

//! Platform-aware store for user-installed Citum styles and locales.
//!
//! Provides a `StoreResolver` that searches user data directories for custom styles
//! and locales, with fallback to embedded builtins. Supports YAML, JSON, and CBOR formats.

pub mod config;
pub mod format;
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

pub use config::StoreConfig;
pub use format::StoreFormat;
pub use resolver::StoreResolver;

use std::path::PathBuf;

/// Returns the platform-specific data directory for Citum user files.
///
/// This path is derived from [`dirs::data_dir()`] with a `citum` subdirectory appended.
///
/// Typical locations:
/// - Linux:   `~/.local/share/citum/`
/// - macOS:   `~/Library/Application Support/citum/`
/// - Windows: `%APPDATA%\citum\`
#[must_use]
pub fn platform_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("citum"))
}
