/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Rendering logic for localized term components.
//!
//! This module handles term component rendering with locale-aware lookup
//! and proper text handling for plural/singular forms.

use crate::reference::Reference;
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_schema::locale::TermForm;
use citum_schema::template::TemplateTerm;

impl ComponentValues for TemplateTerm {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        _reference: &Reference,
        _hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        let effective_rendering = self.rendering.clone();

        let form = self.form.unwrap_or(TermForm::Long);
        let mut value = options
            .locale
            .resolved_general_term(&self.term, form)
            .unwrap_or_default();

        // Apply strip-periods if configured
        if crate::values::should_strip_periods(&effective_rendering, options) {
            value = crate::values::strip_trailing_periods(&value);
        }

        // Apply text-case if configured
        if let Some(tc) = effective_rendering.text_case {
            value = crate::values::text_case::apply_text_case(&value, tc);
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
