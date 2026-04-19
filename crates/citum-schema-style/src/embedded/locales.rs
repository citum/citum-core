/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Embedded locale YAML files for common BCP 47 locales.
//!
//! These are baked into the binary at compile time via `include_bytes!`,
//! providing locale data when the CLI is invoked with `--builtin` and there
//! is no `locales/` directory on disk.

/// Raw YAML bytes for an embedded locale by BCP 47 ID.
///
/// Returns `None` for locales not bundled with the binary.
pub fn get_locale_bytes(id: &str) -> Option<&'static [u8]> {
    match id {
        "en-US" => Some(include_bytes!("../../../../locales/en-US.yaml")),
        "de-DE" => Some(include_bytes!("../../../../locales/de-DE.yaml")),
        "es-ES" => Some(include_bytes!("../../../../locales/es-ES.yaml")),
        "fr-FR" => Some(include_bytes!("../../../../locales/fr-FR.yaml")),
        "tr-TR" => Some(include_bytes!("../../../../locales/tr-TR.yaml")),
        _ => None,
    }
}

/// All available embedded locale IDs.
pub const EMBEDDED_LOCALE_IDS: &[&str] = &["en-US", "de-DE", "es-ES", "fr-FR", "tr-TR"];

/// Raw YAML bytes for an embedded locale override by ID.
///
/// Returns `None` for overrides not bundled with the binary.
pub fn get_locale_override_bytes(id: &str) -> Option<&'static [u8]> {
    match id {
        "en-US-chicago" => Some(include_bytes!(
            "../../../../locales/overrides/en-US-chicago.yaml"
        )),
        "de-DE-chicago" => Some(include_bytes!(
            "../../../../locales/overrides/de-DE-chicago.yaml"
        )),
        _ => None,
    }
}

/// All available embedded locale override IDs.
pub const EMBEDDED_LOCALE_OVERRIDE_IDS: &[&str] = &["en-US-chicago", "de-DE-chicago"];
