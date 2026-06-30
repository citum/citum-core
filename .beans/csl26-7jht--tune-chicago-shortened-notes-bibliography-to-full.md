---
# csl26-7jht
title: Tune chicago-shortened-notes-bibliography to full fidelity
status: todo
type: task
priority: high
created_at: 2026-06-30T18:46:09Z
updated_at: 2026-06-30T18:46:09Z
parent: csl26-h7oc
blocked_by:
    - csl26-lxy3
---

Tune `chicago-shortened-notes-bibliography` to 100% fidelity + clean SQI via
the `style-tune` skill, against the shared Chicago corpus
(`chicago-18th-citations.json`, 15 items; `chicago-18th.json`, 402 refs).

## Baseline (measured 2026-06-30)
- citations: 6/15 (40%) — lowest of the four variants
- bibliography: 264/402 (66%)

## Input contract (style-tune)
- Embedded style ID: `chicago-shortened-notes-bibliography`
- Legacy CSL: `styles-legacy/chicago-fullnote-bibliography.csl` (verify exact
  source — shortened-note variant of the notes-bibliography family)
- Citum YAML: `crates/citum-schema-style/embedded/styles/chicago-shortened-notes-bibliography.yaml`
- Authority: CMOS 18 notes-bibliography system, shortened-note form
- Extends (via `-core`): `chicago-notes-18th` — re-baseline only after that
  bean lands, then tune the shortened-note + bibliography-specific deltas

## Why last
`chicago-shortened-notes-bibliography-core` extends `chicago-notes-18th`;
doing this after notes is tuned means the inherited citation baseline is
already improved, leaving this bean to focus on its own bibliography surface
(which notes-18th doesn't have) and shortened-note-specific deltas.

## Todo
- [ ] Re-run baseline once chicago-notes-18th tune lands (inherited citation
      numbers will have moved)
- [ ] Fidelity loop on the bibliography surface + shortened-note-specific
      citation deltas
- [ ] SQI loop
- [ ] style-qa-reviewer handoff (tier: embedded-core)
