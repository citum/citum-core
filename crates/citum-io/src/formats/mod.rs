/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Format-specific bibliography parsing and rendering helpers.

pub(crate) mod biblatex;
pub(crate) mod csl_json;
pub(crate) mod native;
pub(crate) mod output;

pub(crate) use csl_json::input_reference_to_csl_json;
pub(crate) use native::serialize_any;
pub(crate) use output::{render_biblatex, render_ris};

#[cfg(test)]
pub(crate) use native::{
    loaded_from_input_bibliography, parse_json_bibliography, parse_yaml_bibliography,
};
