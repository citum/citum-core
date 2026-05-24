/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Format-specific bibliography parsing and rendering helpers.

pub(crate) mod biblatex;
pub(crate) mod csl_json;
pub(crate) mod native;
pub(crate) mod output;
pub(crate) mod ris;

pub(crate) use biblatex::load_biblatex_bibliography;
pub(crate) use csl_json::{input_reference_to_csl_json, load_csl_json_bibliography};
#[cfg(test)]
pub(crate) use native::loaded_from_input_bibliography;
pub(crate) use native::{
    deserialize_any, load_citum_json_bibliography, parse_cbor_bibliography,
    parse_json_bibliography, parse_yaml_bibliography, serialize_any,
};
pub(crate) use output::{render_biblatex, render_ris};
pub(crate) use ris::load_ris_bibliography;
