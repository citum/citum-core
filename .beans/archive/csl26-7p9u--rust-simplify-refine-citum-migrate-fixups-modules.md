---
# csl26-7p9u
title: rust simplify refine citum-migrate fixups modules
status: completed
type: task
priority: high
created_at: 2026-03-15T14:07:43Z
updated_at: 2026-03-15T14:07:43Z
---

Bounded `rust-simplify` then `rust-refine` pass over
`crates/citum-migrate/src/fixups.rs`, aimed at splitting the current
policy-heavy helper bucket into clearer internal modules without changing
migration behavior.

## Checklist

- [x] Run `rust-simplify` on `crates/citum-migrate/src/fixups.rs`
- [x] Split locator normalization into a dedicated private module
- [x] Split inferred-template cleanup into a dedicated private module
- [x] Consolidate repeated recursive template/component walkers
- [x] Preserve migration behavior
- [x] Add focused migration regressions
- [x] Run `rust-refine` on the resulting owning module(s)
- [x] Append a dated progress note to `csl26-l13e`

## Constraints

- Structure-only change unless a concrete fidelity bug is discovered and split
  into a successor bean.
- Public behavior and migrate output must remain stable.

## Progress

- 2026-03-15: split `fixups.rs` into `fixups/{media,locator,template}.rs`
  behind a documented `fixups::mod` facade. Preserved the public API consumed by
  `main.rs`, moved recursive template walkers into the template-focused module,
  and verified the slice with `cargo test -p citum-migrate`.

## Summary of Changes

- Replaced the single `fixups.rs` file with `fixups/mod.rs` plus `media`,
  `locator`, and `template` submodules.
- Kept the public `fixups` surface stable through documented wrapper functions
  in `mod.rs`, so `main.rs` call sites did not need behavioral changes.
- Verified the refactor with `cargo fmt --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo nextest run`.
