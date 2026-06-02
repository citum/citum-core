---
# csl26-ha0e
title: format_document resolver-trait entry point
status: completed
type: feature
priority: low
created_at: 2026-05-09T12:39:40Z
updated_at: 2026-06-02T10:44:45Z
---

Add a third engine entry point so callers can inject any
`citum_schema::StyleResolver` impl rather than relying on
adapter-side pre-resolution for `StyleInput::Id`/`Uri`/`Path`.

## Background

`csl26-isrv` Tier 1 ships two entry points:

- `format_document(request)` — handles `StyleInput::Path` and `Yaml`
  locally; errors on `Id`/`Uri`.
- `format_document_with_style(style, request)` — adapters pre-resolve
  the style and pass it in.

The current shape forces the server's `format_document` arm to do
per-variant match boilerplate in `crates/citum-server/src/rpc.rs`. A
reviewer of PR #646 noted this asymmetry: WASM and FFI users with
their own resolvers (e.g., embedded-only registry) cannot inject them
into the engine.

## Trait already exists

`citum-schema-style/src/lib.rs:103` defines a minimal trait that
`citum-engine` can accept directly via the existing `citum_schema`
umbrella dep, with zero new crate-level dependencies:

```
pub trait StyleResolver {
    fn resolve_style(&self, uri: &str) -> Result<Style, ResolutionError>;
}
```

`citum_store::resolver::ChainResolver` and the other concrete resolvers
already implement it (per csl26-r8d2 Phase 1).

## Scope

- [x] Add `format_document_with_resolver(request, resolver)` to
  `crates/citum-engine/src/api/document.rs`. `Yaml` parses inline;
  `Id`/`Uri`/`Path` go through `resolver.resolve_style(value)`.
  Delegates to the existing `format_document_with_style` once a
  `Style` is in hand.
- [x] Re-export from `crates/citum-engine/src/lib.rs`.
- [x] Unit test using a mock `StyleResolver` returning a known `Style`
  for any input.
- [ ] Optional: refactor `citum-server/src/rpc.rs` `format_document`
  arm to call the new entry point with `ChainResolver`, removing the
  per-variant match.

## Acceptance

- New entry point compiles plus tests pass.
- Existing `format_document` and `format_document_with_style` paths
  unchanged — purely additive.
- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run` clean.

## Refs

- Parent: csl26-isrv (archived)
- Related: csl26-r8d2 (Phase 1 resolver trait, completed)
- Reviewer note source: PR #646 second-pass review, observation 1

## Summary of Changes

Work completed in 8e2ce23b. Added format_document_with_resolver to crates/citum-engine/src/api/document.rs, re-exported from crate root, with unit test using a mock StyleResolver. The optional rpc.rs refactor was deferred (not required for the entry point itself).
