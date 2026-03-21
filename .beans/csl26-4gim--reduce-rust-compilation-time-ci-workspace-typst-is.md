---
# csl26-4gim
title: Reduce Rust compilation time (CI + workspace + typst isolation)
status: in-progress
type: task
priority: high
created_at: 2026-03-21T01:14:45Z
updated_at: 2026-03-21T01:14:45Z
---

Implement all three mitigation layers from the /dplan analysis:

Layer A (CI pipeline — no code changes):
- A1: Drop redundant `cargo build` step (cargo test already builds)
- A2: Drop `--all-features` from clippy to exclude typst from CI graph
- A3: Replace 3 separate `cargo rustc -p <crate> --lib --all-features -- -Dmissing-docs` steps with a single `cargo doc --no-deps --workspace`
- A4: Add CARGO_PROFILE_DEV_DEBUG=0 and CARGO_PROFILE_DEV_CODEGEN_UNITS=256 CI env vars
- A5: Install mold linker + RUSTFLAGS on ubuntu-latest runner

Layer B (workspace config):
- B1: Add [profile.dev] debug=0, codegen-units=256 to Cargo.toml
- B2: Add [profile.test] inherits=dev, opt-level=1
- rust-toolchain.toml to stabilise cache keys

Layer C (typst isolation — separate commit):
- Extract crates/citum-pdf as thin workspace member owning typst/typst-pdf/typst-kit deps
- citum-cli typst-pdf feature becomes dep:citum-pdf
- Removes typst from citum-cli's direct dependency graph

Relevant files:
- .github/workflows/ci.yml
- Cargo.toml (workspace)
- .cargo/config.toml
- crates/citum-cli/Cargo.toml
- crates/citum-cli/src/typst_pdf.rs (moves to citum-pdf)
