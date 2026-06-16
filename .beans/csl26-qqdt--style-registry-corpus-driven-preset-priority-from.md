---
# csl26-qqdt
title: 'schema-style: corpus-driven preset discovery for config concerns'
status: todo
type: task
priority: normal
tags:
    - registry
    - presets
    - citum-analyze
created_at: 2026-06-16T15:49:15Z
updated_at: 2026-06-16T16:20:53Z
---

The `--config-presets` mode (csl26-t56t) discovers per-concern config
shapes (contributors, dates, titles, locators) across the CSL corpus that do
not match any existing named preset in citum-schema-style.

Run the report and promote the highest-frequency unnamed shapes as new named
presets in citum-schema-style/src/presets.rs (ContributorPreset, DatePreset,
TitlePreset) or options/locators.rs (LocatorPreset).

Priority order: rank by corpus_count within each concern. Run:

```bash
cargo run --bin citum-analyze -- styles-legacy --config-presets --json \
  | jq '.concerns[] | {concern, candidates: .candidates[:5]}'
```
