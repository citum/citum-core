//! Compatibility facade crate for Citum schema models.
//!
//! This crate re-exports style-focused types from `citum-schema-style`
//! and data-focused accessors through `citum-schema-data`.

pub use citum_schema_style::*;

/// Data-oriented schema exports.
pub mod data {
    pub use citum_schema_data::*;
}
