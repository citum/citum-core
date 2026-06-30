---
# csl26-gzwj
title: Tune taylor-and-francis-chicago-author-date to full fidelity
status: todo
type: task
priority: high
created_at: 2026-06-30T18:46:09Z
updated_at: 2026-06-30T18:55:24Z
parent: csl26-h7oc
blocked_by:
    - csl26-giun
---

Tune `taylor-and-francis-chicago-author-date` to 100% fidelity + clean SQI via
the `style-tune` skill, against the shared Chicago corpus
(`chicago-18th-citations.json`, 15 items; `chicago-18th.json`, 402 refs).

## Baseline (measured 2026-06-30)
- citations: 11/15 (73%) — inherited from chicago-author-date-18th, untuned
  for Style F specifically
- bibliography: 298/402 (74%) — same inheritance
- gated via `chicago-shared-corpus`, `min_pass_rate: 0.73` (csl26-h7oc)

## Input contract (style-tune)
- Embedded style ID: `taylor-and-francis-chicago-author-date`
- Legacy CSL: `styles-legacy/taylor-and-francis-chicago-author-date.csl`
  (verify path; T&F styles trace to Style F in publisher guides)
- Citum YAML: `crates/citum-schema-style/embedded/styles/taylor-and-francis-chicago-author-date.yaml`
- Authority: Taylor & Francis Style F (Chicago author-date variant)
- Extends (via `-core`): `chicago-author-date-18th` — re-baseline only after
  that bean lands, then tune the Style-F-specific deltas on top

## Why second
`taylor-and-francis-chicago-author-date-core` extends
`chicago-author-date-18th`; doing this after that tune means the inherited
baseline numbers are already improved before T&F-specific work starts.

## Todo
- [ ] Re-run baseline once chicago-author-date-18th tune lands (inherited
      numbers will have moved)
- [ ] Fidelity loop on Style-F-specific deltas only (do not re-tune inherited
      rules — fix at the parent if a defect is shared)
- [ ] SQI loop
- [ ] style-qa-reviewer handoff (tier: embedded-core)
