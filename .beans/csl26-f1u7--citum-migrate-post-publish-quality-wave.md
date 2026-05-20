---
# csl26-f1u7
title: citum-migrate post-publish quality wave
status: in-progress
type: epic
priority: high
created_at: 2026-05-20T17:04:36Z
updated_at: 2026-05-20T17:47:36Z
---

Post-publish converter quality wave. Drive citum-migrate output toward high SQI while preserving 100% fidelity on the established portfolio gate. Strategy in `~/.claude/plans/with-crates-now-published-reflective-snowglobe.md`:

1. Make SQI a measurable converter-output target (scorecard + baseline doc).
2. Close the diff-emission gap on the citation side (citation type-variants in diff form).
3. Family-aware preset emission (`extends: preset-bases/...`, `template-ref:`) for known APA / Chicago descendants.

Each phase is one PR; this epic tracks the wave end-to-end. Sentinels in the PR1 scorecard corpus (`apa`, `chicago-author-date`, `elsevier-harvard`, `elsevier-with-titles`, `elsevier-vancouver`, `springer-basic-author-date`, `ieee`, `american-medical-association`, `nature`, `cell`, `taylor-and-francis-chicago-author-date`) plus the migrate-research lab set (`karger-journals`, `institute-of-physics-numeric`, `thieme-german`, `multidisciplinary-digital-publishing-institute`) must hold 100% fidelity throughout. `chicago-notes` and `oscola` are *not* yet in the scorecard corpus and should be folded in by PR2 (they are the styles where citation type-variants matter most).

## Children
- [x] PR #1: scorecard + atomic-config diff fix + alias-wrapper routing (delivered as csl26-e7yw)
- [ ] PR #2: descendant manifest + family-aware extends rewrite
- [ ] PR #3: new preset base (Vancouver / numeric-journal), repeat rewrite
- [ ] PR #4: auto-derived family bases from cluster fingerprints


## Wave status

| PR | Bean | Status | Headline |
|---|---|---|---|
| PR1 | [[csl26-e7yw]] | completed | [#763](https://github.com/citum/citum-core/pull/763) — scorecard + alias-wrapper + atomic-config diff. Migrated mean SQI `93.57` -> `98.17`. |
| PR2 | [[csl26-kqji]] | draft | Descendant-of-preset-base wrapper rewrite for non-alias descendants. Blocked by PR1 landing. |
| PR3 | (not yet filed) | — | Author a Vancouver / numeric-journal preset base and repeat the rewrite pass for the numeric cluster (`american-medical-association`, `karger`, `iop`, `thieme`, `mdpi`). |
| PR4 | (not yet filed) | — | Auto-derive candidate family bases from cluster fingerprints. |

## Strategy doc

`~/.claude/plans/with-crates-now-published-reflective-snowglobe.md` (private to author, not checked in). The baseline doc `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` is the durable artifact that subsequent PRs refresh in place.
