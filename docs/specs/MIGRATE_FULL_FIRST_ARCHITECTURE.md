<!--
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
-->

# citum-migrate: Full-First Type-Variant Architecture

**Status:** Draft
**Date:** 2026-06-15
**Bean:** `csl26-e94m` (trigger), see also `csl26-fk0w` (co-evolution epic)
**Related:**
[`OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md`](OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md),
[`docs/architecture/audits/2026-06-14_MIGRATE_FIDELITY_LOCUS_CLASSIFICATION.md`](../architecture/audits/2026-06-14_MIGRATE_FIDELITY_LOCUS_CLASSIFICATION.md)

## Problem

`citum-migrate` currently conflates two distinct concerns in `build_final_style`
([`main.rs`](../../crates/citum-migrate/src/main.rs)):

1. **Semantic fixups** — correcting synthesis output to match citeproc-js
   behavior (e.g. gating `url`/`accessed` to web types, normalizing
   `legal_case`, ensuring media type templates exist).
2. **Diff encoding** — compressing the `TypeTemplateMap` of full per-type
   templates into a `TypeVariantMap` whose entries may be `Diff` (extends a
   parent) or `Full` (standalone), implemented in `build_type_variants`
   ([`template_diff.rs`](../../crates/citum-migrate/src/template_diff.rs)).

Because diff encoding is computed immediately after fixups run on the *same
pass* over the assembled style, any fixup that touches the base template or a
type template affects the diff weights computed against it. The ordering is
load-bearing in a way that makes it impossible to add or move fixups without
risking variant corruption.

### Concrete failure: url/accessed gating (csl26-e94m)

citeproc-js gates `url` and `accessed` at **render time** based on reference
type (`type="webpage post post-weblog"`). The CSL XML compiler drops this
`<if type="webpage">` gate when compiling the layout tree, so url/accessed leak
into the base template for all types.

The natural fix is a post-synthesis fixup that strips url/accessed from the
base template and all non-web type templates. But doing this before
`build_type_variants` changes which candidate parent wins for `legal_case` (the
base→`legal_case` diff weight shifts, `bill` wins as parent instead of the
base), corrupting the Brown v. Board of Education test case. Moving the fixup
after `build_type_variants` makes it inconsistent with the base stored in
`Diff.extends` references.

This is not a fixable ordering problem within the current design. It is a
symptom of conflating fixup application with diff encoding.

### Root cause in general terms

Diff encoding requires a stable, finalized base template. The current pipeline
finalizes the base and encodes diffs in the same pass, so any late-arriving
fixup that is correct at the semantic level is incorrect at the structural
level (corrupts existing diffs) or cannot be applied at all.

The same class of problem will recur for any fixup that:
- Removes a component from the base that is referenced by existing diffs
- Adds a component to the base that changes which variant is the cheapest diff
- Reorders or wraps base components in ways that shift LCS alignment

`gate_leaked_in_term` (`csl26-ivjp`) avoided this class because it only
operates on **type-variant** templates after they are flattened from the base,
never on the base itself. That exemption cannot be assumed for future fixups.

## Proposed Design: Full-First, Normalize-Later

Separate fixup application from diff encoding by ensuring `build_type_variants`
is called only after all fixups have run, operating on finalized Full templates.

### Two-phase assembly in `build_final_style`

**Phase 1 — Full templates:** assemble the base template and a
`TypeTemplateMap` where every entry is a complete, standalone
`Vec<TemplateComponent>`. Apply all semantic fixups at this stage. No
`Diff`/`extends` references exist yet; fixups see only concrete template
components and can modify them freely.

**Phase 2 — Compression:** pass the finalized base and `TypeTemplateMap` to
`build_type_variants` (or a renamed successor). This function computes diffs
purely as a serialization optimization. It emits `TemplateVariant::Diff` only
when the diff round-trips correctly through `engine_validate_variants`; all
others emit `TemplateVariant::Full`. The output is the `TypeVariantMap` stored
in the style.

```
Synthesis
  └─ select_and_process_bibliography_template
       → (new_bib: Full, type_templates: TypeTemplateMap of Full)

Phase 1 — Fixups (order-independent within this phase)
  ├─ postprocess_inferred_bibliography        (scrubs artifacts, relaxes suppression)
  ├─ normalize_legal_case_type_template       (semantic: legal-case formatting)
  ├─ ensure_inferred_media_type_templates     (semantic: media type completeness)
  ├─ ensure_inferred_patent_type_template     (semantic: patent template)
  ├─ gate_web_only_url_accessed          ← NEW: strips url/accessed from non-web types
  └─ [future fixups, freely ordered]

Phase 2 — Compression (pure serialization pass)
  └─ build_type_variants(finalized_base, finalized_type_templates)
       → TypeVariantMap (Diff where safe, Full otherwise)
```

### What changes

| Component | Current role | New role |
|---|---|---|
| `build_type_variants` | Called once in `build_final_style` immediately after fixups; result is the stored output | Called once at the END of `build_final_style`, after ALL fixups; semantics unchanged |
| `postprocess_inferred_bibliography` | Only called on the inferred path | Called on all paths (inferred and XML-seed winner), since it encodes semantic knowledge, not inference artifacts |
| `engine_validate_variants` | Disabled / commented out | Restored as the round-trip safety net for Phase 2. A `Diff` that fails engine round-trip is demoted to `Full`; this is the only correctness gate needed for compression. |
| Fixups that touch base template | Ordering is load-bearing | Ordering within Phase 1 is free; only constraint is Phase 1 before Phase 2 |
| `gate_web_only_url_accessed` | Could not be added without regression | Added in Phase 1; strips url/accessed from base and non-web type templates before compression |

### What does NOT change

- The synthesis loop (`src/synthesis/`), scoring, and candidate selection are
  unchanged. This refactor is downstream of synthesis selection.
- The XML seed candidate survives as Phase 1 input (as `OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md`
  specifies, until it wins ≈0 selections).
- `options_extractor/` (et-al thresholds, sort keys, etc.) is unaffected.
- The `TemplateVariant::Diff` / `TemplateVariant::Full` schema is unchanged.
- The style YAML serialization format is unchanged.

## Refactor vs. Rewrite

A full rewrite of `citum-migrate` around the Phase 2 synthesis loop would
address the same class of problem but at far greater scope and risk. The
diagnosis for the current tail fidelity is that failures are
**converter-dominated** (wrong or missing template data from the synthesis
candidates), not architecture-dominated. The Full-first ordering fix is
targeted and verifiable: it unblocks a class of fixups (render-time gating)
that are currently impossible to apply without regression.

The synthesis loop itself is already the correct architecture per
`OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md` Phase 2. The problem is in the assembly
code that handles the synthesis loop's *output*, not the loop itself.

A rewrite should be evaluated separately, after the fidelity locus
classification matures and the XML seed wins ≈0 selections. At that point the
XML compiler can be removed and the fixup layer audited for which functions
compensate for XML-seed artifacts (can be deleted) vs. encode genuine semantic
knowledge (must survive in any rewrite). See [Fixup Audit](#fixup-audit) below.

## Fixup Audit

Classifying the current fixup surface by purpose, to inform future rewrite
scope:

| Fixup | File | Class | Notes |
|---|---|---|---|
| `gate_leaked_in_term` | `fixups/gating.rs` | XML-seed artifact | `<if>` group flattening drops the enclosing group; type-variant specialization leaves a bare `in` term. Needed only because the XML compiler flattens conditionals. |
| `scrub_inferred_literal_artifacts` | `fixups/template.rs` | Inference artifact | Removes literal strings injected by the inferrer that have no schema equivalent. |
| `relax_inferred_bibliography_date_suppression` | `bib_postprocess.rs` | Inference artifact | Synthesis can over-suppress dates; this relaxes the suppression. |
| `repair_inferred_bibliography_type_templates` | `bib_postprocess.rs` | Synthesis gap | Injects missing `primary-title` / `publisher` into type templates that should inherit them. Compensates for synthesis not yet reaching those components. |
| `normalize_legal_case_type_template` | `fixups/media.rs` | Semantic knowledge | Legal-case formatting rules (no quotes, no parent-monograph, strip certain terms, etc.). Independent of XML compilation; survives any rewrite. |
| `ensure_inferred_media_type_templates` | `fixups/media.rs` | Semantic knowledge | Creates audio-recording / video templates when the legacy CSL signals them. |
| `ensure_inferred_patent_type_template` | `fixups/media.rs` | Semantic knowledge | Creates patent template when expected. |
| `ensure_personal_communication_omitted` | `fixups/media.rs` | Semantic knowledge | Omits personal-comm types per legacy behavior. |
| `should_merge_inferred_type_template` | `fixups/template.rs` | Synthesis integration | Decides which XML-compiled type templates survive into the inferred path. Becomes vestigial when the XML seed wins ≈0 selections. |
| `gate_web_only_url_accessed` (proposed) | `fixups/gating.rs` | Semantic knowledge | Mirrors citeproc-js render-time type= gate; independent of XML compilation. |

**Observation:** the fixup layer is roughly half XML-seed-artifact compensators
(deletable when the XML seed is removed) and half genuine semantic knowledge
(must survive). The semantic fixups are the stable surface to preserve in a
future rewrite.

## CLI Flag Consideration

The normalization/compression pass (Phase 2) is a pure serialization
optimization. It does not affect correctness — a style with all
`TemplateVariant::Full` entries is semantically identical to one with Diff
entries, just larger. This suggests an optional `--no-compress-variants` flag
for debugging, where Phase 2 is skipped and all type-variant entries are emitted
as `Full`. This is low-priority but useful for diagnosing diff-encoding bugs.

Conversely, a `--compress-variants` flag applied to already-migrated styles
(analogous to SQI) would allow retroactive compression of styles authored with
explicit Full entries. This is a natural follow-on, not in scope for this
refactor.

## Acceptance Criteria

- [ ] `build_type_variants` is called only once in `build_final_style`, after
  all fixups have run.
- [ ] `postprocess_inferred_bibliography` runs on both the inferred and
  XML-seed-winner paths (currently inferred-only).
- [ ] `engine_validate_variants` is restored and active (demotes unsafe Diffs
  to Full).
- [ ] `gate_web_only_url_accessed` is implemented as a Phase 1 fixup, gating
  url/accessed on web types only.
- [ ] Brown v. Board of Education (`legal_case` in jar) passes after the gate
  is applied.
- [ ] `url`/`accessed` no longer appear in non-web type renders (jar interview,
  early-medieval-europe article-journal).
- [ ] Full migrate batch (`oracle-migrate-batch.js`) shows no regression vs.
  the pre-branch baseline.
- [ ] `build_final_style` and its assembly helpers are extracted from
  `main.rs` into a focused module, with the Phase 1 / Phase 2 boundary as a
  function boundary; `main.rs` is back under the house size limit.
- [ ] All tests pass (`just pre-commit`).

## Implementation Notes

The refactor is localized to `main.rs::build_final_style` and the call site for
`build_type_variants`. The fixup functions themselves do not change; only their
call order relative to `build_type_variants` changes.

`postprocess_inferred_bibliography` currently holds some fixups that reference
`inferred_bib_source` to decide whether to run. Extracting those into a
separate function (or removing the guard) is the main structural change needed.

`engine_validate_variants` was previously removed with a `// TEMP: skip`
comment. Restoring it is independent of the ordering refactor and can be done
in the same commit.

### Decompose `main.rs`

`crates/citum-migrate/src/main.rs` is ~1970 LOC — far over the 300-line house
limit and the single reason the two-phase ordering is hard to see and hard to
change safely. The refactor is the natural point to split it:

- **`build_final_style`** and its assembly helpers (the Phase 1 / Phase 2 split)
  belong in a dedicated `assembly.rs` (or `final_style/` module), so the
  phase boundary is a module boundary, not a comment in a 2000-line file.
- The CLI entry (`main`, arg parsing, I/O) stays in `main.rs`; everything that
  manipulates templates moves out.
- `select_and_process_bibliography_template` and the `inferred_bib_source`
  plumbing move alongside `build_final_style` so the path-selection logic sits
  next to the code that consumes it.

This decomposition is a precondition, not a side effect: the current size is
why a load-bearing ordering constraint could hide in `build_final_style`
undetected. Splitting Phase 1 (fixups) and Phase 2 (compression) into separate
functions in a focused module makes the "all fixups before any compression"
invariant structurally enforceable rather than convention.
