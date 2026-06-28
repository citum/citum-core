---
# csl26-y68t
title: Evaluate test-that after 1.0 release
status: todo
type: task
priority: low
tags:
    - testing
    - research
created_at: 2026-06-28T12:11:01Z
updated_at: 2026-06-28T12:11:01Z
---

## Context

`test-that` is a Rust assertion and matcher library introduced in June 2026. It may eventually help Citum tests that currently use predicate-style assertions with weak failure diagnostics, especially enum/error shape checks and collection assertions.

Sources checked:

- https://hovinen.me/announcements/2026/06/24/introducing-test-that.html
- https://docs.rs/test-that/latest/test_that/
- `cargo info test-that` reported version `0.5.0`

## Defer until 1.0

Do not add `test-that` to the workspace yet. Re-evaluate only after the crate reaches `1.0`, when API churn risk is lower.

## Evaluation criteria

A future pilot is justified only if all of these are true:

- `test-that` has reached `1.0`.
- The API appears stable enough for Citum's test suite.
- Failure diagnostics are materially better than plain `assert_eq!`, `assert!`, or `matches!` in real Citum tests.
- The adoption does not weaken `docs/guides/CODING_STANDARDS.md` test-independence rules.

## Pilot constraints

If adopted later:

- Add it as a dev-dependency only.
- Trial it in one narrow crate with enum/error/container-heavy tests, likely `citum-io`, `citum_store`, or schema discriminator tests.
- Avoid rendered citation/bibliography output tests unless assertions still use full expected values rather than loose partial predicates.
- Do not use matcher syntax as a replacement for independent expected values.

## Verification for future pilot

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
```
