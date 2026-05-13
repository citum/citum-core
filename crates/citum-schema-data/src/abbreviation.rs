/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Document-level abbreviation map type.

use std::collections::HashMap;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Document-level map from full rendered strings to their abbreviations.
///
/// Keys are the full rendered string (exact, case-sensitive). Values are the
/// replacement abbreviation. Applied after value extraction, before output.
/// Accepts both `abbreviation-map` (YAML frontmatter) and `abbreviation_map` forms.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(transparent)]
pub struct AbbreviationMap(pub HashMap<String, String>);
