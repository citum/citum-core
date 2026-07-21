/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Rendering logic for the `type-label` component: a localized description
//! of a reference's own type (e.g. "Dataset", "Classical work"), with a
//! `genre`/`medium` fallback before the locale term lookup.
//!
//! See `docs/specs/TYPE_CLASSIFICATION_CENTRALIZATION.md`.

use crate::reference::Reference;
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_schema::locale::TermForm;
use citum_schema::template::TemplateTypeLabel;

impl ComponentValues for TemplateTypeLabel {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        _hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        let effective_rendering = self.rendering.clone();

        let mut value = resolve_type_label_text(reference, options)?;

        if crate::values::should_strip_periods(&effective_rendering, options) {
            value = crate::values::strip_trailing_periods(&value);
        }

        if let Some(tc) = effective_rendering.text_case {
            value = crate::values::text_case::apply_text_case_with_language(
                &value,
                tc,
                Some(options.locale.locale.as_str()),
            );
        }

        if value.is_empty() {
            None
        } else {
            Some(ProcValues {
                value,
                pre_formatted: false,
                ..Default::default()
            })
        }
    }
}

/// Resolve the reference-type label text: `genre` (unless it merely
/// restates `ref_type`), else `medium`, else a locale term keyed by
/// `ref_type`.
fn resolve_type_label_text(reference: &Reference, options: &RenderOptions<'_>) -> Option<String> {
    let ref_type = reference.ref_type();

    if let Some(genre) = reference.genre().filter(|genre| *genre != ref_type) {
        return Some(options.locale.lookup_genre(&genre));
    }

    if let Some(medium) = reference.medium() {
        return Some(options.locale.lookup_medium(&medium));
    }

    options
        .locale
        .resolved_type_term(&ref_type, &TermForm::Long)
}
