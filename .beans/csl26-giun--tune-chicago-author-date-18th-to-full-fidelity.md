---
# csl26-giun
title: Tune chicago-author-date-18th to full fidelity
status: todo
type: task
priority: high
created_at: 2026-06-30T18:46:08Z
updated_at: 2026-06-30T18:46:08Z
parent: csl26-h7oc
---

Tune `chicago-author-date-18th` to 100% fidelity + clean SQI via the
`style-tune` skill, against the shared Chicago corpus
(`chicago-18th-citations.json`, 15 items; `chicago-18th.json`, 402 refs).

## Baseline (measured 2026-06-30)
- citations: 11/15 (73%)
- bibliography: 298/402 (74%) — currently the only gated number in the
  Chicago family, via `chicago-zotero-bibliography`, `min_pass_rate: 0.73`

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
