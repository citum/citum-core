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

pub mod csl_json;
pub mod model;
pub mod parser;
