---
# csl26-cdjx
title: Wire conversion contract into style workflow docs and skills
status: completed
type: task
priority: normal
created_at: 2026-07-02T18:52:44Z
updated_at: 2026-07-02T18:56:28Z
parent: csl26-cvfy
---

Follow-on to the conversion-layer contract (docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md, merged in PR #993). The style-workflow classification vocabulary (style-defect / migration-artifact / processor-defect / intentional divergence) in docs/policies/STYLE_WORKFLOW_DECISION_RULES.md has no operational test for attributing a mismatch to the conversion layer. Add a conversion-layer pre-flight rule to the policy (citum convert refs + canonicalization-table check), thin pointers from the style-tune and style-qa skills, and a note in style-migrate-enhance and migrate-research that the newly-routed CSL types (collection, review, review-book, performance, figure, graphic, musical_score, pamphlet, periodical, entry, post-weblog) now reach styles as real ref_types, so migration recommendations should cover their type-variants. Interim manual procedure until csl26-3r34 mechanizes tagging in oracle.js.

## Summary of Changes

- `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md` (v1.4): new
  "Conversion-layer pre-flight" section — verify the reference converts
  truthfully against the CSL_TYPE_CONVERSION_CONTRACT canonicalization
  table before classifying; processor-defect (conversion) claims need
  pre-flight evidence and a contract-test reproduction. Reading guidance
  for newly-routed types and validated note-field overrides.
- `docs/guides/STYLE_WORKFLOW_EXECUTION.md`: fidelity-loop classification
  step references the pre-flight.
- Skills (host-neutral pointers, logic stays in the policy):
  `style-tune`, `style-qa` (reject unevidenced conversion claims),
  `style-migrate-enhance` (missing type-variants for newly-routed types
  are reportable migration gaps), `.skills/migrate-research`.
- `.codex/agents/migration-researcher.md`: same pointer for the Codex
  host.
- Interim manual procedure until csl26-3r34 mechanizes tagging.
