---
# csl26-p0cl
title: 'Phase 0: Remove clap from csln_processor'
status: completed
type: task
priority: normal
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-02-22T00:00:00Z
blocking:
    - csl26-modz
---

Remove the unused `clap` dependency from `csln_processor/Cargo.toml`.

Library crates must not depend on CLI frameworks. Inspection of all `.rs`
files in `crates/csln_processor/src/` confirms clap is not imported anywhere.

## Implementation

Remove from csln_processor/Cargo.toml:
```
clap = { version = "4.4", features = ["derive"] }
```

## Verification

`cargo build` passes with no unused-dep warnings.

Refs: csl26-modz, docs/architecture/CITUM_MODULARIZATION.md
