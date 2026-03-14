//! CSL 1.0 XML parser for the legacy `.csl` style format.
//!
//! This crate reads a Citation Style Language 1.0 XML file and deserialises it
//! into a typed in-memory representation ([`model`]).  The resulting IR is
//! consumed by the `citum-migrate` crate to produce equivalent Citum styles.
//!
//! # Modules
//! - [`model`] – data structures that mirror the CSL 1.0 XML schema.
//! - [`parser`] – XML → model parser built on top of [`roxmltree`].
//! - [`csl_json`] – companion CSL-JSON reference model (legacy input format).

/// CSL-JSON reference types used alongside legacy CSL style processing.
pub mod csl_json;
/// Rust data structures that mirror the CSL 1.0 XML schema.
pub mod model;
/// XML parsing entry points for reading legacy CSL styles into the model.
pub mod parser;
