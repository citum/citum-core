/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Embedded Citum style YAML files for priority citation styles.
//!
//! These are baked into the binary at compile time via `include_bytes!`,
//! allowing the CLI to load styles without a file path using `--builtin`.

use crate::Style;

/// Raw YAML bytes for an embedded style by name.
fn get_style_bytes(name: &str) -> Option<&'static [u8]> {
    match name {
        "elsevier-harvard-core" => Some(include_bytes!(
            "../../../../styles/embedded/elsevier-harvard-core.yaml"
        )),
        "elsevier-with-titles-core" => Some(include_bytes!(
            "../../../../styles/embedded/elsevier-with-titles-core.yaml"
        )),
        "elsevier-vancouver-core" => Some(include_bytes!(
            "../../../../styles/embedded/elsevier-vancouver-core.yaml"
        )),
        "springer-basic-author-date-core" => Some(include_bytes!(
            "../../../../styles/embedded/springer-basic-author-date-core.yaml"
        )),
        "springer-basic-brackets-core" => Some(include_bytes!(
            "../../../../styles/embedded/springer-basic-brackets-core.yaml"
        )),
        "springer-vancouver-brackets-core" => Some(include_bytes!(
            "../../../../styles/embedded/springer-vancouver-brackets-core.yaml"
        )),
        "taylor-and-francis-chicago-author-date-core" => Some(include_bytes!(
            "../../../../styles/embedded/taylor-and-francis-chicago-author-date-core.yaml"
        )),
        "taylor-and-francis-council-of-science-editors-author-date-core" => Some(include_bytes!(
            "../../../../styles/embedded/taylor-and-francis-council-of-science-editors-author-date-core.yaml"
        )),
        "taylor-and-francis-national-library-of-medicine-core" => Some(include_bytes!(
            "../../../../styles/embedded/taylor-and-francis-national-library-of-medicine-core.yaml"
        )),
        "chicago-shortened-notes-bibliography-core" => Some(include_bytes!(
            "../../../../styles/embedded/chicago-shortened-notes-bibliography-core.yaml"
        )),
        "preset-bases/apa-7th" => Some(include_bytes!("../../../../styles/embedded/apa-7th.yaml")),
        "preset-bases/chicago-author-date-18th" => Some(include_bytes!(
            "../../../../styles/embedded/chicago-author-date-18th.yaml"
        )),
        "preset-bases/chicago-notes-18th" => Some(include_bytes!(
            "../../../../styles/embedded/chicago-notes-18th.yaml"
        )),
        "chicago-notes-18th" => Some(include_bytes!(
            "../../../../styles/embedded/chicago-notes-18th.yaml"
        )),
        "chicago-author-date-18th" => Some(include_bytes!(
            "../../../../styles/embedded/chicago-author-date-18th.yaml"
        )),
        "apa-7th" => Some(include_bytes!("../../../../styles/embedded/apa-7th.yaml")),
        "elsevier-harvard" => Some(include_bytes!(
            "../../../../styles/embedded/elsevier-harvard.yaml"
        )),
        "elsevier-with-titles" => Some(include_bytes!(
            "../../../../styles/embedded/elsevier-with-titles.yaml"
        )),
        "elsevier-vancouver" => Some(include_bytes!(
            "../../../../styles/embedded/elsevier-vancouver.yaml"
        )),
        "springer-basic-author-date" => Some(include_bytes!(
            "../../../../styles/embedded/springer-basic-author-date.yaml"
        )),
        "springer-basic-brackets" => Some(include_bytes!(
            "../../../../styles/embedded/springer-basic-brackets.yaml"
        )),
        "springer-vancouver-brackets" => Some(include_bytes!(
            "../../../../styles/embedded/springer-vancouver-brackets.yaml"
        )),
        "american-medical-association" => Some(include_bytes!(
            "../../../../styles/embedded/american-medical-association.yaml"
        )),
        "ieee" => Some(include_bytes!("../../../../styles/embedded/ieee.yaml")),
        "taylor-and-francis-chicago-author-date" => Some(include_bytes!(
            "../../../../styles/embedded/taylor-and-francis-chicago-author-date.yaml"
        )),
        "taylor-and-francis-council-of-science-editors-author-date" => Some(include_bytes!(
            "../../../../styles/embedded/taylor-and-francis-council-of-science-editors-author-date.yaml"
        )),
        "taylor-and-francis-national-library-of-medicine" => Some(include_bytes!(
            "../../../../styles/embedded/taylor-and-francis-national-library-of-medicine.yaml"
        )),
        "chicago-shortened-notes-bibliography" => Some(include_bytes!(
            "../../../../styles/embedded/chicago-shortened-notes-bibliography.yaml"
        )),
        "modern-language-association" => Some(include_bytes!(
            "../../../../styles/embedded/modern-language-association.yaml"
        )),
        _ => None,
    }
}

/// A mapping of short aliases to full embedded style names.
pub const EMBEDDED_STYLE_ALIASES: &[(&str, &str)] = &[
    ("apa", "apa-7th"),
    ("mla", "modern-language-association"),
    ("ieee", "ieee"),
    ("ama", "american-medical-association"),
    ("chicago", "chicago-shortened-notes-bibliography"),
    ("chicago-notes", "chicago-notes-18th"),
    ("chicago-author-date", "chicago-author-date-18th"),
    ("vancouver", "elsevier-vancouver"),
    ("harvard", "elsevier-harvard"),
];

/// Resolve a style name or alias to the full embedded style name.
pub fn resolve_embedded_style_name(name: &str) -> Option<&'static str> {
    match name {
        "preset-bases/apa-7th" => return Some("preset-bases/apa-7th"),
        "preset-bases/chicago-author-date-18th" => {
            return Some("preset-bases/chicago-author-date-18th");
        }
        "preset-bases/chicago-notes-18th" => {
            return Some("preset-bases/chicago-notes-18th");
        }
        _ => {}
    }

    if let Some(n) = EMBEDDED_STYLE_NAMES.iter().find(|&&n| n == name) {
        return Some(*n);
    }
    EMBEDDED_STYLE_ALIASES
        .iter()
        .find(|(alias, _)| *alias == name)
        .map(|(_, full)| *full)
}

/// Parse an embedded style by name or alias.
///
/// Returns `None` if `name` is not a known builtin or alias.
/// Returns `Some(Err(_))` only if the embedded YAML is malformed (should not
/// happen for styles that passed CI).
pub fn get_embedded_style(name: &str) -> Option<Result<Style, serde_yaml::Error>> {
    resolve_embedded_style_name(name)
        .and_then(get_style_bytes)
        .map(Style::from_yaml_bytes)
}

/// All available embedded (builtin) style names, ordered by corpus impact
/// (dependent-style count descending).
pub const EMBEDDED_STYLE_NAMES: &[&str] = &[
    "elsevier-harvard-core",
    "elsevier-with-titles-core",
    "elsevier-vancouver-core",
    "springer-basic-author-date-core",
    "springer-vancouver-brackets-core",
    "springer-basic-brackets-core",
    "taylor-and-francis-chicago-author-date-core",
    "taylor-and-francis-council-of-science-editors-author-date-core",
    "taylor-and-francis-national-library-of-medicine-core",
    "chicago-shortened-notes-bibliography-core",
    "apa-7th",
    "elsevier-harvard",
    "elsevier-with-titles",
    "elsevier-vancouver",
    "springer-basic-author-date",
    "springer-vancouver-brackets",
    "springer-basic-brackets",
    "american-medical-association",
    "ieee",
    "taylor-and-francis-chicago-author-date",
    "taylor-and-francis-council-of-science-editors-author-date",
    "taylor-and-francis-national-library-of-medicine",
    "chicago-shortened-notes-bibliography",
    "chicago-notes-18th",
    "chicago-author-date-18th",
    "modern-language-association",
];
