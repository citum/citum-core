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
        reference: &Reference,
        _hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        // Resolve effective rendering options (base merged with type-specific override)
        let mut effective_rendering = self.rendering.clone();
        if let Some(overrides) = &self.overrides {
            use citum_schema::template::ComponentOverride;
            let ref_type = reference.ref_type();
            let mut match_found = false;
            for (selector, ov) in overrides {
                if selector.matches(&ref_type)
                    && let ComponentOverride::Rendering(r) = ov
                {
                    effective_rendering.merge(r);
                    match_found = true;
                }
            }
            if !match_found {
                for (selector, ov) in overrides {
                    if selector.matches("default")
                        && let ComponentOverride::Rendering(r) = ov
                    {
                        effective_rendering.merge(r);
                    }
                }
            }
        }

        let form = self.form.unwrap_or(TermForm::Long);
        let mut value = options
            .locale
            .resolved_general_term(&self.term, form)
            .unwrap_or_default();

        // Apply strip-periods if configured
        if crate::values::should_strip_periods(&effective_rendering, options) {
            value = crate::values::strip_trailing_periods(&value);
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
