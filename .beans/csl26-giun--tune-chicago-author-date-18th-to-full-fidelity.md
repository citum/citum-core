---
# csl26-giun
title: Tune chicago-author-date-18th to full fidelity
status: in-progress
type: task
priority: high
created_at: 2026-06-30T18:46:08Z
updated_at: 2026-06-30T19:59:04Z
parent: csl26-h7oc
---

Tune `chicago-author-date-18th` to 100% fidelity + clean SQI via the
`style-tune` skill, against the shared Chicago corpus
(`chicago-18th-citations.json`, 15 items; `chicago-18th.json`, 402 refs).

## Baseline (measured 2026-06-30)
- citations: 11/15 (73%)
- bibliography: 298/402 (74%)
- gated via `chicago-shared-corpus`, `min_pass_rate: 0.73` (combined
  citation+bibliography rate; csl26-h7oc)

## Input contract (style-tune)
- Embedded style ID: `chicago-author-date-18th`
- Legacy CSL: `styles-legacy/chicago-author-date.csl`
- Citum YAML: `crates/citum-schema-style/embedded/styles/chicago-author-date-18th.yaml`
- Authority: CMOS 18 author-date system
- Extends: `chicago-18-base` (csl26-zs0f) — base options inherited, do not
  re-litigate base-level rules here

## Why first
Tuned first because `taylor-and-francis-chicago-author-date-core` extends
this style: fidelity gains here lift T&F's baseline before its Style-F deltas
are layered on.

## Todo
- [ ] Fidelity loop: oracle → classify failures → smallest correct YAML fix →
      re-run, until 100% on the shared corpus (citation + bibliography)
- [ ] SQI loop: `report-core` → hoist/preset/prune → re-check, until clean
- [ ] style-qa-reviewer handoff (tier: embedded-core)
- [ ] Confirm no regression on `references-expanded.json` / `core` fixture sets

- 2026-06-30 tune attempt: `webpage` citation variant added to render publisher/year for publisher-owned web pages. `node scripts/report-core.js --style chicago-author-date-18th` now reports headline fidelity 0.808, SQI 0.975, and `chicago-shared-corpus` 12/15 citations + 298/402 bibliography. Residual citation failures: `chi-article-magazine` (`Gourmet 2000b` vs `(2000)`), `chi-multi-source` classic book (`Augustine, De civitate Dei` vs `Augustine 1931`), and `chi-personal-communication` (`Johnson to O’Laughlin 1916` vs `Hiram Johnson`). Invalid/failed probes reverted: citation substitute keys `publisher`/`parent-serial` are schema-invalid; article-magazine `title: parent-serial` duplicates `Gourmet`; raw `container-title` is invalid in citation type-variants; personal_communication sender-recipient-year variant regresses `secondary-roles` recipient citation. Fidelity not green; SQI/QA/PR intentionally not run.
- 2026-06-30 recovery attempt on `codex/chicago-author-date-full-fidelity`: citation fidelity is now green for `chicago-shared-corpus` (`15/15`). Implemented bounded support for `parent-serial` substitutes, field-presence/absence `render-when` groups, classic conversion from CSL/Zotero `type: classic`, collection-editor substitution, custom contributor-role rendering, and audio-visual event/dimensions preservation. Bibliography improved from `298/402` to `331/400` in the shared oracle run; `node scripts/report-core.js --style chicago-author-date-18th` reports headline fidelity `0.868` and SQI `0.927`. Remaining bibliography failures are broad rich-reference clusters (book original/reprint/rich contributors, interviews, manuscripts, web pages, recordings/media, broadcasts, and manuscript/archive edge cases), so the original 100% + clean SQI + QA completion gate is not met. This draft PR is a substrate/progress PR, not a completed full-fidelity tune.
