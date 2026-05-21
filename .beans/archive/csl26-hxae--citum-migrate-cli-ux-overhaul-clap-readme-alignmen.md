---
# csl26-hxae
title: 'citum-migrate CLI UX overhaul: clap + README alignment'
status: completed
type: task
priority: high
created_at: 2026-05-21T23:13:23Z
updated_at: 2026-05-21T23:19:11Z
parent: csl26-f1u7
---

Replace manual arg parsing with clap derive (fix invisible --help via tracing::debug!), add --version, colored output matching citum CLI. Update README: fix cargo run examples to use installed binary, add missing --emit-evidence/--family-candidate/--minimize-wrapper flags.

## Summary of Changes

- Replaced hand-rolled arg parser in `src/cli.rs` with clap 4.4 derive
- Copied `CLAP_STYLES` from citum-cli (green headers, cyan literals) for visual parity
- `--help` now prints to stdout and is visible without RUST_LOG; `--version` added for free
- Error messages now go to stderr via clap's built-in error handling
- `FamilyCandidateMode` stays internal; `Args` exposes `family_candidate: Option<String>` and `FamilyCandidateMode::from_arg()` resolves it
- `TemplateMode`/`LiveInferBackend` bridge to clap via `value_parser` — no clap dep added to the library crate
- README: all `cargo run --bin citum-migrate --` examples replaced with `citum-migrate`; added `--emit-evidence`, `--family-candidate`, `--minimize-wrapper` docs; corrected `--family-candidate` default description
- 135 tests passing, fmt+clippy clean
