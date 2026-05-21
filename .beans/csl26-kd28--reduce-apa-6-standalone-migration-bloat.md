---
# csl26-kd28
title: Reduce APA 6 standalone migration bloat
status: todo
type: bug
priority: high
created_at: 2026-05-21T11:24:18Z
updated_at: 2026-05-21T11:24:18Z
parent: csl26-f1u7
---

APA 6 no-flag migration currently emits a standalone YAML file around 5,661 lines:

```bash
cargo run -q --bin citum-migrate -- styles-legacy/apa-6th-edition.csl | wc -l
```

Strict minimization evidence proves the 5-line `apa-7th` wrapper is unsafe: APA 6 and APA 7 differ in citation and bibliography behavior. The remaining size problem is therefore converter bloat, not a justification for wrapper inheritance.

## Scope

- Measure where the standalone output expands: citation templates, bibliography type variants, contributor/date/title options, conditionals, and XML fallback output.
- Remove duplicated or mechanically equivalent generated structures while preserving APA 6 semantics.
- Prefer converter-level improvements over style-specific hard-coding.
- Keep no-flag APA 6 standalone unless a future strict equivalence gate proves a safe parent candidate.

## Acceptance

- `citum-migrate styles-legacy/apa-6th-edition.csl` emits materially fewer lines than the current ~5,661 baseline.
- Strict minimization still rejects the unsafe APA 7 wrapper unless semantic equivalence actually changes.
- Existing oracle, SQI, clippy, and nextest gates pass.
