---
# csl26-hqy5
title: Custom locale file invocation for builtin styles
status: completed
type: feature
priority: normal
tags:
    - multilingual
created_at: 2026-05-16T11:28:28Z
updated_at: 2026-05-23T14:37:38Z
---

## Goal

Make it possible to render with a custom (non-embedded) locale file when the style is a builtin alias. Right now the CLI's `--locale <id>` flag has two paths:

- **File-path style** (`-s styles/embedded/apa-7th.yaml`): the resolver searches a sibling `locales/` directory on disk and reads `<id>.yaml`. Custom locales work.
- **Builtin-alias style** (`-s apa`): the resolver consults `citum_schema::embedded::get_locale_bytes()` only. The locale must be baked into the binary at compile time. Custom locales **don't work**.

This is awkward and was surfaced during the `csl26-v6ok` smoke test: a provisional `eu-ES` locale can be exercised end-to-end only via the path-style form, which is non-obvious.

## Design options (decide before coding)

1. **`--locale-file <path>` flag** — explicit override; bypasses the alias resolver entirely. Smallest blast radius.
2. **Disk-first lookup for builtin styles** — always check `./locales/<id>.yaml` first, then fall back to embedded. Friendlier UX inside a citum checkout; risk: pollutes user-namespace from working directory.
3. **User locale store** — already partly scoped under `csl26-erwz` (User style & locale store). May subsume this bean.

## Todo

- [x] Decided: route locale lookup through `citum_store::ChainResolver` (option 3, subsuming-via-erwz). No new flag needed; existing `citum locale add` + `-L <id>` Just Works.
- [x] Implemented in `crates/citum-cli/src/style_resolver.rs` (`create_processor`); `load_locale_builtin` deleted in favor of `citum_store::load_locale_or_default`. New `FileLocaleResolver` carries the file-style sibling-`locales/` semantics into the chain.
- [x] Updated `docs/guides/AUTHORING_LOCALES.md` §Verification with both file-path and builtin-alias smoke recipes.
- [x] Unit tests in `crates/citum_store/src/resolver_tests.rs`; existing `create_processor` tests in `crates/citum-cli` still pass.

## Summary of Changes

- New `FileLocaleResolver` in `citum_store::resolver` resolves `<base_dir>/<id>.{yaml,yml,json,cbor}` for the file-style branch.
- New `build_chain_with_file_locale_dir(dir)` and `load_locale_or_default(chain, id)` helpers in `citum_store::chain` give hosts a single chain-based locale entry point.
- `create_processor` in `citum-cli` now builds the resolver chain once and calls `load_locale_or_default`, deleting the ad-hoc `load_locale_builtin` path. Builtin-alias styles consult the user store before falling back to embedded.
- Error semantics unchanged: explicit `-L <id>` still errors if the chain produces a fallback ("locale not found: '<id>'").
- Docs: `AUTHORING_LOCALES.md` smoke recipe covers both style input shapes; `CITUM_STORE_PLAN.md` and `DISTRIBUTED_RESOLVER.md` reflect the new resolver.

## Related

- Parent feature: `csl26-v6ok` (surfaced the gap)
- Possibly subsumes / subsumed by: `csl26-erwz` (User style & locale store)
- Relevant code: `crates/citum-cli/src/style_resolver.rs::load_locale_builtin` (line ~229)
