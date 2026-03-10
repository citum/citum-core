//! Compatibility facade crate for Citum schema models.
//!
//! This crate re-exports style-focused types from `citum-schema-style`
//! and data-focused accessors through `citum-schema-data`.

pub use citum_schema_style::*;

/// Canonical Citum style schema version for external consumers.
pub const SCHEMA_VERSION: &str = citum_schema_style::STYLE_SCHEMA_VERSION;

/// Data-oriented schema exports.
pub mod data {
    pub use citum_schema_data::*;
}
