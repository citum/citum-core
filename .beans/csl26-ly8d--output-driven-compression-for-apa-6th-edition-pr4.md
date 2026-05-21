---
# csl26-ly8d
title: Extend minimize-wrapper to all parent-link sources
status: todo
type: feature
priority: high
created_at: 2026-05-20T22:50:26Z
updated_at: 2026-05-20T23:36:23Z
parent: csl26-f1u7
blocked_by:
    - csl26-39tm
---

Follow-up to PR3 (csl26-39tm). PR3 shipped `--minimize-wrapper` for reverse-template-link candidates. The apparent apa-6th-edition win (5,661 → 5 LOC) was later rejected by strict APA 6/APA 7 evidence; that candidate must not be accepted without strict equivalence. The mdpi case (template-link candidate american-chemical-society) was scored but rejected because the minimize path doesn't currently fire for template-link parents — only reverse-template-link triggers `promote_family_candidate`.

Scope for this PR: extend the minimize path so template-link and independent-parent-link candidates also benefit. Concretely, allow `--minimize-wrapper` to operate whenever `parent_style_id` is set on the lineage (from any source: registry-alias, template-link, independent-parent-link, reverse-template-link, local-extends). Then the scorecard's A/B test will pick up mdpi and the numeric cluster as compression candidates.

Acceptance:
- APA 6 remains rejected unless strict citation and bibliography equivalence proves a candidate safe.
- Oracle citation and bibliography fidelity do not regress for existing standalone baselines.
- Same harness optionally applied to multidisciplinary-digital-publishing-institute (template-link candidate american-chemical-society) where fidelity allows.
- Baseline doc `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` refreshed in place.
- No regression on any existing sentinel.

Starting points:
- `MigrationEvidence` records discovered candidates and emitted form; a future minimization expansion should report per-scope equivalence results.
- Today's `apply_to_migrated_style` calls `diff_value` with `exclude_template_paths = !preserve_template_deltas`. For oracle-driven compression the equivalence check needs to run per template scope, not at the YAML-diff level — likely a new pass after `apply_to_migrated_style`.
