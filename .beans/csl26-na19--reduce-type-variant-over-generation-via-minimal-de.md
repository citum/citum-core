---
# csl26-na19
title: Reduce type-variant over-generation via minimal default template
status: todo
type: task
priority: normal
tags:
    - migrate
    - authorability
    - template
created_at: 2026-06-16T12:55:29Z
updated_at: 2026-07-06T18:55:09Z
---

Migration emits many type-variants that only 'remove' components (e.g. speech extends article-newspaper removing edition/section). Driven by maximal-default + diff-everything model.

Note: term: literals (edition/section labels) render regardless of data presence, so THOSE removes are semantically real. The deeper issue is over-generation: derive a minimal type-agnostic default and have types add what they need, rather than subtract from a maximal union.

Large; converter-dominated tail (see crates/citum-migrate/CLAUDE.md). Draft.

Authorability follow-up from ACME review (PR #932). Pre-existing; not a regression.

## Design (2026-07-06)

Confirmed model: assembly materializes per-type Full templates; `sqi_refinement.rs::encode_bibliography_type_variants` re-encodes them as diffs against the maximal default via `template_diff::build_type_variants`, so subtractive-heavy variants are a direct artifact of the maximal-union default.

Recommended approach — **choose the default by minimizing total diff cost, not by inverting the union:**

1. **Measurement:** script the per-corpus distribution of diff ops (adds vs removes vs modifies per type-variant) from migrated output; this quantifies over-generation and gives the acceptance metric (total ops, remove share).
2. **Candidate defaults:** (a) current maximal union (baseline); (b) intersection of per-type templates (types only add); (c) the median/most-common per-type template. Compute total encoded diff cost for each candidate per style and pick the minimum — this is a pure re-encoding choice, **render-neutral by construction** (each type's resolved Full template is unchanged), so it needs no fidelity gate, only the existing `template_diff` engine round-trip validation.
3. **Semantics guard:** term: literals render regardless of data presence, so removes of term-bearing components are semantically real (this bean's own note) — the encoder must keep those as explicit removes and never silently absorb them into a smaller default.
4. **Follow-through:** re-baseline SQI after the encoding change; csl26-2uq2 (sprawl double-count) and csl26-7auw (diff readability) both interact with the same encoder and should ride the same wave.

Sequencing: after csl26-a001 (provenance-preserved groups shrink diffs first). Promote to todo: approach decided, step 1-2 Sonnet-executable against template_diff.rs.
