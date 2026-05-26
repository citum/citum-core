/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! RIS bibliography parsing helpers.

use std::path::Path;

use citum_schema::InputBibliography;
use citum_schema::reference::InputReference;
use csl_legacy::csl_json::{DateVariable, Name, Reference as LegacyReference, StringOrNumber};

use crate::RefsError;

/// Load a RIS bibliography from a file path.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed as RIS.
pub fn load_ris(path: &Path) -> Result<InputBibliography, RefsError> {
    let src = std::fs::read_to_string(path)?;
    parse_ris(&src)
}

fn parse_ris(input: &str) -> Result<InputBibliography, RefsError> {
    let mut references = Vec::<InputReference>::new();
    let mut current = Vec::<(String, String)>::new();

    for line in input.lines() {
        let line = line.strip_prefix('\u{feff}').unwrap_or(line);
        let Some((tag, value)) = line.split_once("  - ") else {
            continue;
        };
        let tag = tag.trim();
        if tag.len() != 2 || !tag.is_ascii() {
            continue;
        }
        let tag = tag.to_string();
        let value = value.trim().to_string();
        if tag == "ER" {
            if !current.is_empty() {
                references.push(InputReference::from(ris_record_to_reference(&current)));
            }
            current.clear();
            continue;
        }
        current.push((tag, value));
    }

    if !current.is_empty() {
        references.push(InputReference::from(ris_record_to_reference(&current)));
    }

    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

fn ris_record_to_reference(fields: &[(String, String)]) -> LegacyReference {
    let get = |tag: &str| -> Option<String> {
        fields
            .iter()
            .find_map(|(k, v)| (k == tag).then(|| v.clone()))
    };
    let get_all = |tag: &str| -> Vec<String> {
        fields
            .iter()
            .filter(|(k, _)| k == tag)
            .map(|(_, v)| v.clone())
            .collect()
    };

    let id = get("ID")
        .or_else(|| get("L1"))
        .or_else(|| get("M1"))
        .unwrap_or_else(|| "item".to_string());
    let title = get("TI").or_else(|| get("T1"));
    let ty = get("TY").unwrap_or_else(|| "BOOK".to_string());
    let author = {
        let authors = get_all("AU")
            .into_iter()
            .map(|n| {
                let parts: Vec<_> = n.split(',').map(str::trim).collect();
                if parts.len() >= 2 {
                    #[allow(clippy::indexing_slicing, reason = "parts.len() >= 2")]
                    Name::new(parts[0], parts[1])
                } else {
                    Name::literal(parts.first().copied().unwrap_or(""))
                }
            })
            .collect::<Vec<_>>();
        (!authors.is_empty()).then_some(authors)
    };
    let issued = get("PY").or_else(|| get("Y1")).and_then(|s| {
        let year = s.chars().take(4).collect::<String>().parse::<i32>().ok()?;
        Some(DateVariable::year(year))
    });
    let doi = get("DO");
    let note = get("N1");
    let page = match (get("SP"), get("EP")) {
        (Some(sp), Some(ep)) => Some(format!("{sp}-{ep}")),
        (Some(sp), None) => Some(sp),
        _ => None,
    };
    let ref_type = if ty == "JOUR" || ty == "JFULL" {
        "article-journal".to_string()
    } else if ty == "CHAP" {
        "chapter".to_string()
    } else {
        "book".to_string()
    };

    LegacyReference {
        id,
        ref_type,
        author,
        title,
        container_title: get("JO").or_else(|| get("JF")),
        issued,
        volume: get("VL").map(StringOrNumber::String),
        issue: get("IS").map(StringOrNumber::String),
        page,
        doi,
        url: get("UR"),
        isbn: get("SN"),
        publisher: get("PB"),
        publisher_place: get("CY"),
        language: get("LA"),
        note,
        ..Default::default()
    }
}
