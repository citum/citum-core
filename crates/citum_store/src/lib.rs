//! Platform-aware store for user-installed Citum styles and locales.
//!
//! Provides a `StoreResolver` that searches user data directories for custom styles
//! and locales, with fallback to embedded builtins. Supports YAML, JSON, and CBOR formats.

pub mod config;
pub mod format;
pub mod resolver;

pub use config::StoreConfig;
pub use format::StoreFormat;
pub use resolver::StoreResolver;

use std::path::PathBuf;

/// Returns the platform-specific data directory for Citum user files.
///
/// On Linux/macOS: `~/.local/share/citum/`
/// On Windows: `%APPDATA%\Citum\`
pub fn platform_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("citum"))
}
