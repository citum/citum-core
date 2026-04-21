---
# csl26-rwgi
title: Implement config-only profile overrides
status: completed
type: feature
priority: normal
tags:
    - styles
    - taxonomy
    - schema
created_at: 2026-04-21T15:53:54Z
updated_at: 2026-04-21T16:09:04Z
---

Implement docs/specs/CONFIG_ONLY_PROFILE_OVERRIDES.md end to end.

## Tasks
- [ ] Add typed options.profile schema, profile capabilities, and resolution errors
- [ ] Make profile resolution config-only and reject template-bearing local overrides
- [ ] Convert embedded profile styles to config-only wrappers over hidden compiled roots
- [ ] Update taxonomy/spec references and regenerate docs/schemas
- [x] Run frontmatter, bean hygiene, fmt, clippy, and nextest

## Acceptance
- profiles resolve without structural template merging
- profile validation rejects template-bearing local fields and null-clears
- embedded public profiles are config-only wrappers
- docs/specs/CONFIG_ONLY_PROFILE_OVERRIDES.md is Active and references this bean

## Summary of Changes

Implemented typed `options.profile` axes and fallible profile resolution in `citum-schema-style`, enforced config-only profile wrappers for builtin registry profiles, moved embedded profile bodies into hidden compiled `*-core` styles, updated taxonomy/spec docs, regenerated `docs/schemas/style.json`, and passed frontmatter, bean hygiene, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo nextest run`.
