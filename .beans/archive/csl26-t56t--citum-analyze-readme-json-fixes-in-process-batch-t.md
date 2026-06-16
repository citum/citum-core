---
# csl26-t56t
title: 'citum-analyze: README, JSON fixes, in-process batch test, coverage-gap mode'
status: completed
type: feature
priority: normal
created_at: 2026-06-16T14:14:54Z
updated_at: 2026-06-16T14:45:15Z
---

Four-phase enhancement of crates/citum-analyze:
1. Fix --json output routed via tracing::debug! (savings, profile_discovery, batch_test)
2. In-process batch test (kill cargo-run-per-style spawning)
3. README documenting both binaries and all modes
4. New --coverage-gap mode: corpus-wide set-diff of legacy CSL features vs migrate compiled output → prioritized converter gaps + auto-discovered preset families


## Summary of Changes

- **Phase 1** (`savings.rs`, `profile_discovery.rs`): Fixed `--json` output routed through `tracing::debug!` (silent without `RUST_LOG=debug`) — now writes to stdout.
- **Phase 2** (`batch_test.rs`, `Cargo.toml`): Replaced two `cargo run` spawns per style with in-process library calls (`csl_legacy` → `citum_migrate` → `citum_schema` → `citum_engine`). Full-corpus pass is now seconds, not hours.
- **Phase 3** (`README.md`): New README documenting both binaries, all five modes with exact invocations, and pipeline integration.
- **Phase 4** (`semantic.rs`, `coverage_gap.rs`, `main.rs`): Extracted `SemanticItem`/Jaccard helpers into `semantic.rs`. New `--coverage-gap` mode walks the full independent-style corpus in-process and emits two reports: prioritized converter gaps and preset-family clusters (Jaccard ≥ 0.65).
