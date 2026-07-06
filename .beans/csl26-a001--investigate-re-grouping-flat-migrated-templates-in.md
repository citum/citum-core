---
# csl26-a001
title: Investigate re-grouping flat migrated templates into authored groups
status: todo
type: task
priority: normal
tags:
    - migrate
    - authorability
    - template
    - fidelity
created_at: 2026-06-16T12:55:29Z
updated_at: 2026-07-06T18:55:09Z
---

Occurrence-compiler emits a flat union template; component groups (imprint = publisher+place, locator = volume+issue+pages) are mostly gone in migrated output (see ACME). Groups are the primary mechanism for conditional formatting: a group renders its delimiter, affixes, and punctuation only when at least one member produces output. Without groups, components render individually — a delimiter or label before an absent variable becomes a stray artifact. Regrouping flat templates is therefore essential for correct output, not just visual tidiness.

GATE ON FIDELITY: confirm the engine renders identically before/after re-grouping. Flatness may be load-bearing for current pass rates. Draft until a safe grouping heuristic is validated.

Authorability follow-up from ACME review (PR #932). Pre-existing; not a regression.

## Design (2026-07-06)

Verified mechanism: `template_compiler/compilation.rs` (~line 230 and duplicated ~line 300) preserves a source group only when (2-3 components AND delimiter in a whitelist of five AND no wrap) OR (wrap or Term present); everything else is flattened into occurrence order. The original CSL groups are available at that point — **regrouping should be provenance-guided (keep what the XML declared), not heuristically re-inferred downstream.**

Plan, gated exactly as this bean requires:

1. **Measurement first:** add a debug counter to the compiler reporting groups-preserved vs groups-flattened per style; run the seeded random-100 corpus to size the change before touching behavior.
2. **Engine-equality gate, not oracle Jaccard:** for each style, render the full fixture corpus with the flat template and the group-preserving template through citum-engine and require **byte-identical** output to auto-accept. This is cheap (no citeproc), exact, and directly answers 'is flatness load-bearing'. Styles where output differs are the interesting cohort: diff them — if the grouped form drops stray delimiters before absent variables (the artifact this bean describes), that is a fidelity *improvement*, verify against citeproc via the existing synthesis scorer before accepting.
3. **Widen preservation incrementally:** relax one criterion at a time (first the delimiter whitelist, then the 2-3 size cap), re-running the gate each step. Never introduce groups the source XML did not declare — synthesized grouping (imprint/locator taxonomies) stays out of scope.
4. **Cleanup:** the two ~60-line group-collection blocks (citation vs bibliography) are near-verbatim duplicates — extract a shared helper in the same change.

Interaction: inferred-path templates (no XML provenance) are untouched. Relates to csl26-na19 (variant encoding) — land this first; better groups shrink the diffs na19 operates on.

Promote to todo: the investigation method is decided; step 1-2 are Sonnet-executable.
