/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Rendering for supplementary standardized identifiers.

use crate::reference::Reference;
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_schema::template::TemplateIdentifier;

impl ComponentValues for TemplateIdentifier {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        _hints: &ProcHints,
        _options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        reference
            .identifier(self.identifier.as_str())
            .filter(|value| !value.trim().is_empty())
            .map(|value| ProcValues {
                value: value.to_string(),
                pre_formatted: false,
                ..Default::default()
            })
    }
}
