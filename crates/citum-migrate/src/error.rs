/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Crate-level error type for the measured-selection pipeline (synthesis,
//! measured citation/bibliography selection, the embedded citeproc-js
//! runtime, and standalone-assembly bootstrap failures).

use std::fmt;

/// Errors raised while synthesizing, scoring, or assembling migrated
/// templates.
#[derive(Debug)]
pub enum MigrateError {
    /// Embedded citeproc-js runtime execution or call failure.
    Runtime(String),
    /// Fixture or locale file load failure.
    Fixture(String),
    /// Template-synthesis candidate selection/availability failure.
    Render(String),
    /// JSON/shape parsing failure of loaded fixture or reference data.
    Parse(String),
}

impl fmt::Display for MigrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrateError::Runtime(m)
            | MigrateError::Fixture(m)
            | MigrateError::Render(m)
            | MigrateError::Parse(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for MigrateError {}
