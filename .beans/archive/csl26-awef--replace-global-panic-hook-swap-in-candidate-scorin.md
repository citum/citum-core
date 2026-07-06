---
# csl26-awef
title: Replace global panic-hook swap in candidate scoring
status: completed
type: task
priority: normal
created_at: 2026-07-06T18:42:20Z
updated_at: 2026-07-06T22:35:42Z
parent: csl26-al39
---

Audit F3 (2026-07-06 migrate review): measured_citation.rs::catch_candidate_unwind swaps the process-global panic hook around every bibliography-candidate render. Latent race under any future parallelism, and the panic payload is discarded so engine bugs are indistinguishable from bad candidates (scored 0). Fix: install a silencing hook once via std::sync::Once, or keep catch_unwind and tracing::debug! the captured payload with the candidate name.

## Summary of Changes

Replaced the per-call `take_hook`/`set_hook`/restore dance in `catch_candidate_unwind` (crates/citum-migrate/src/measured_citation.rs) with a `std::sync::Once`-guarded single installation of a custom panic hook. The hook logs the panic payload and location via `tracing::debug!` instead of silently discarding it (visible with `RUST_LOG=citum_migrate=debug`), so an engine panic during bibliography-candidate rendering is now distinguishable from a merely bad candidate. The hook is installed once for the process and never restored — scoped acceptable since this is the only `catch_unwind` use in the `citum-migrate` binary. Added a unit test asserting repeated calls (including panicking ones) behave correctly under the one-time init. Full `just pre-commit` gate passes (1819 tests).
