/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Output-driven template synthesis loop.
//!
//! Phase 2 of `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md`: a
//! deterministic propose/render/score/mutate loop that searches bounded
//! template mutations against citeproc-js reference output. The seed pool is
//! the inferrer output plus, during the transition, the XML-compiled
//! templates — the XML layout tree is never compiled as authority. Each round
//! proposes single named mutations of the incumbent, scores them with the
//! Phase 1 fitness functions, and replaces the incumbent only under the
//! Phase 1 acceptance rule. The final incumbent is validated on the held-out
//! fixture set; a held-out regression falls back to the best non-regressing
//! earlier incumbent.
//!
//! The section-agnostic loop lives in the `core` submodule; `citation` and
//! `bibliography` supply the section-specific seeds, scoring, and template
//! accessors; `operators` enumerates the bounded mutation families.

mod bibliography;
mod citation;
mod core;
mod operators;

pub use bibliography::synthesize_bibliography;
pub(crate) use bibliography::synthesize_bibliography_rounds;
pub use citation::synthesize_citation;
pub(crate) use citation::synthesize_citation_rounds;
pub use core::MAX_SYNTHESIS_ROUNDS;
