/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Data input models for Citum references, citations, and bibliographies.

use indexmap::IndexMap;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Citation input model.
#[allow(missing_docs, reason = "internal derives")]
pub mod citation;
/// Bibliographic reference data types.
#[allow(missing_docs, reason = "internal derives")]
pub mod reference;

pub use citation::{
    Citation, CitationItem, CitationMode, Citations, IntegralNameState, LocatorType, Position,
};

/// A collection of bibliographic references with optional metadata.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct InputBibliography {
    /// Bibliography metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<InputBibliographyInfo>,
    /// The list of references.
    pub references: Vec<reference::InputReference>,
    /// Optional compound entry sets keyed by set id.
    ///
    /// Each set id maps to an ordered list of reference ids that should be treated
    /// as one compound numeric group when `compound-numeric` is enabled by style.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<std::collections::BTreeMap<String, Vec<String>>>")
    )]
    pub sets: Option<IndexMap<String, Vec<String>>>,
    /// Custom user-defined fields for extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Metadata for an input bibliography.
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct InputBibliographyInfo {
    /// Human-readable title for the bibliography dataset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Creator or maintainer of the bibliography dataset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}
