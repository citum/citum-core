---
# csl26-f1u7
title: citum-migrate post-publish quality wave
status: completed
type: epic
priority: high
created_at: 2026-05-20T17:04:36Z
updated_at: 2026-05-23T12:50:08Z
---

Post-publish converter quality wave. Drive citum-migrate output toward high SQI while preserving 100% fidelity on the established portfolio gate. Strategy in `~/.claude/plans/with-crates-now-published-reflective-snowglobe.md`:

1. Make SQI a measurable converter-output target (scorecard + baseline doc).
2. Close the diff-emission gap on the citation side (citation type-variants in diff form).
3. Family-aware preset emission (`extends: preset-bases/...`, `template-ref:`) for known APA / Chicago descendants.

Each phase is one PR; this epic tracks the wave end-to-end. Sentinels in the PR1 scorecard corpus (`apa`, `chicago-author-date`, `elsevier-harvard`, `elsevier-with-titles`, `elsevier-vancouver`, `springer-basic-author-date`, `ieee`, `american-medical-association`, `nature`, `cell`, `taylor-and-francis-chicago-author-date`) plus the migrate-research lab set (`karger-journals`, `institute-of-physics-numeric`, `thieme-german`, `multidisciplinary-digital-publishing-institute`) must hold 100% fidelity throughout. `chicago-notes` and `oscola` are *not* yet in the scorecard corpus and should be folded in by PR2 (they are the styles where citation type-variants matter most).

## Children
- [x] PR #1: scorecard + atomic-config diff fix + alias-wrapper routing (delivered as csl26-e7yw)
- [x] PR #2: descendant manifest + family-aware extends rewrite (csl26-kqji, #766)
- [x] PR #3: output-driven compression + evidence emission, apa-6th first (csl26-39tm)
- [ ] PR #4: new preset base (Vancouver / numeric-journal), repeat rewrite
- [ ] PR #5: auto-derived family bases from cluster fingerprints


## Wave status

| PR | Bean | Status | Headline |
|---|---|---|---|
| PR1 | [[csl26-e7yw]] | completed | [#763](https://github.com/citum/citum-core/pull/763) — scorecard + alias-wrapper + atomic-config diff. Migrated mean SQI `93.57` -> `98.17`. |
| PR2 | [[csl26-kqji]] | completed | [#766](https://github.com/citum/citum-core/pull/766) — descendant-of-preset-base wrapper rewrite. Adds template-parent routing in `lineage.rs` and `chicago-notes`/`oscola` sentinels. |
| PR3 | [[csl26-39tm]] | completed | [#767](https://github.com/citum/citum-core/pull/767) — output-driven compression + evidence emission. `--minimize-wrapper`, `--family-candidate`, `--emit-evidence` CLI surface; `MigrationEvidence` sidecar JSON. |
| PR4-pre | [[csl26-dqtx]] | completed | [#768](https://github.com/citum/citum-core/pull/768) — strict normalized-output equivalence gate + clustered citation coverage. APA 6 minimize correctly rejected; standalone form preserved. |
| PR4 | [[csl26-ly8d]] | scrapped | Obsoleted by PR4-pre strict gate (mdpi template-link candidate already exercised and rejected per baseline `237 → 237 ✗`). Same logic as PR4b — no sentinel candidate would change emitted output today. |
| PR4b | [[csl26-tjqn]] | scrapped | Obsoleted by csl26-kd28 (standalone bloat reduction) and PR #768 strict gate. Per `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md`, no sentinel candidate would change emitted output today. |
| PR5 | (not yet filed) | — | Author a Vancouver / numeric-journal preset base and repeat the rewrite pass for the numeric cluster (`american-medical-association`, `karger`, `iop`, `thieme`, `mdpi`). |
| PR6 | (not yet filed) | — | Auto-derive candidate family bases from cluster fingerprints. |

## Strategy doc

`~/.claude/plans/with-crates-now-published-reflective-snowglobe.md` (private to author, not checked in). The baseline doc `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` is the durable artifact that subsequent PRs refresh in place.

## Summary of Changes

Wave delivered as four merged PRs plus follow-up fixes; portfolio fidelity preserved throughout.

### Headline metrics

Source: `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` (generated commit `7753f82d`, 18-style corpus).

| Aggregate | n | mean | p10 | p50 | p90 |
|---|---:|---:|---:|---:|---:|
| Migrated YAML SQI | 18 | 96.30 | 89.10 | 99.07 | 100 |
| Public YAML SQI | 17 | 92.46 | 83.93 | 96.27 | 100 |
| Migrated − Public | 17 | +3.87 | -0.27 | +2.40 | +10.87 |

Sentinel fidelity unchanged or improved against pre-wave baseline; every style remains at or above its prior oracle pass rate.

### Mechanisms shipped

| PR | Mechanism |
|---|---|
| [#763](https://github.com/citum/citum-core/pull/763) (PR1, csl26-e7yw) | SQI scorecard + alias-wrapper diff emission + atomic-config `diff_value` fix. |
| [#766](https://github.com/citum/citum-core/pull/766) (PR2, csl26-kqji) | Descendant-of-preset-base wrapper rewrite via `<info><link rel="independent-parent">` discovery; `chicago-notes` / `oscola` folded into corpus. |
| [#767](https://github.com/citum/citum-core/pull/767) (PR3, csl26-39tm) | `--minimize-wrapper`, `--family-candidate`, `--emit-evidence` CLI surface; `MigrationEvidence` sidecar JSON. |
| [#768](https://github.com/citum/citum-core/pull/768) (PR4-pre, csl26-dqtx) | Strict normalized-output equivalence gate; clustered citation coverage extension. APA 6 minimize correctly rejected. |
| `d0f3a80a` | `fix(migrate): reduce output bloat` — standalone-form Rust XML-fallback cleanup. |
| `6cb34f3f` | `fix(migrate): reject unsafe wrapper minimization` — gate tightening follow-up. |

### Wave-level invariants held

- Sentinel corpus locked at 18 styles (15 original + `chicago-notes`, `oscola`, `apa-6th-edition`).
- 100% fidelity preserved on every sentinel across all four PRs.
- Baseline doc `docs/architecture/2026-05-20_MIGRATE_SQI_BASELINE.md` is the durable artifact; refreshed in place per PR.
- No version bump triggered by the wave's chore-level close-out.

### Scrapped under this epic

- `csl26-tjqn` (PR4b) — auto-minimize-by-default; obsoleted by PR4-pre strict gate.
- `csl26-ly8d` (PR4) — extend minimize to template-link / independent-parent-link; obsoleted by PR4-pre strict gate (template-link candidate `mdpi → ACS` already exercised end-to-end and correctly rejected).

### Deferred to a future wave

These were aspirational items in the original strategy doc. They are *not* required for the wave's success criteria and are intentionally left as seeds for a successor epic when corpus / cluster evidence justifies the work:

- **PR5** — author a Vancouver / numeric-journal preset base and repeat the rewrite pass for the numeric cluster (`american-medical-association`, `karger-journals`, `institute-of-physics-numeric`, `thieme-german`, `multidisciplinary-digital-publishing-institute`). This is the productive next move because the current corpus has no safe template-link minimize candidates — a real preset base would change that.
- **PR6** — auto-derive candidate family bases from cluster fingerprints across the broader portfolio.
