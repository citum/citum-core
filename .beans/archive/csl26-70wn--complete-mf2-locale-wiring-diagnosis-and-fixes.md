---
# csl26-70wn
title: Complete MF2 locale wiring diagnosis and fixes
status: completed
type: task
priority: high
created_at: 2026-04-25T10:51:42Z
updated_at: 2026-04-25T10:59:15Z
---

Diagnostic-fix wave: complete es-ES to schema v2, publish locale authoring guide, refresh csl26-li63 body, and open follow-up beans. Phase A investigation confirmed engine MF2 wiring was already in place for count-based messages, but gender-aware MF2 role labels still need a separate follow-up.

## Background

MF2 locale-message support shipped in spec `docs/specs/LOCALE_MESSAGES.md` v1.3. Initial diagnosis suspected the engine was unwired, but Phase A exploration found the call sites already in place — `resolved_locator_term` and `resolved_role_term` in `crates/citum-engine/src/values/{locator,number,contributor/labels}.rs` consult the `messages:` map first and fall back to the legacy `terms:` / `roles:` / `locators:` maps. The visible symptoms came from elsewhere: four of five locale files have full v2 messages blocks, but `es-ES` declared `message-syntax: mf2` while shipping no messages block; no agent guidance referenced the v2 path. A later review clarified that `MaybeGendered<T>` is live for legacy term maps, while the MF2 path still lacks `$gender` arguments and multi-selector `.match` support.

## Todos

- [x] Phase B: add messages:, date-formats:, grammar-options:, legacy-term-aliases: blocks to locales/es-ES.yaml (excluding role.* messages until MF2 can express gender x plural role-label matrices)
- [x] Phase C: write docs/guides/AUTHORING_LOCALES.md
- [x] Phase C: add locale-authoring pointer to CLAUDE.md
- [x] Phase D: refresh csl26-li63 body with structured status snapshot
- [x] Phase A: verified MF2 is already wired (resolved_locator_term/resolved_role_term in engine; no work needed)
- [x] Commit and open PR

## Summary of Changes

- `locales/es-ES.yaml`: added `messages:`, `date-formats:`, `grammar-options:`, `legacy-term-aliases:` blocks. Plural-dispatched locator labels (page, chapter, volume, section) now route through MF2. Role labels intentionally omitted because `roles:` already supports `MaybeGendered<T>`, while MF2 role-label messages cannot yet receive `$gender` or match on both `$gender` and `$count`. Issue locator labels stay in the legacy `locators:` map because `Locale::locator_message_id` does not yet route `LocatorType::Issue` through MF2.
- `docs/guides/AUTHORING_LOCALES.md`: new authoring guide covering when to add MF2 entries, the supported subset, the interim gender limitation, required v2 blocks, and verification commands.
- `CLAUDE.md`: added "Authoring Locales" pointer with the interim gender limitation summary.
- `.beans/csl26-li63--production-readiness.md`: replaced one-line stub with structured acceptance-criteria list keyed to existing children.

Phase A (count-based engine wiring) was confirmed already complete during exploration — `resolved_locator_term` / `resolved_role_term` are wired in `crates/citum-engine/src/values/{locator,number,contributor/labels}.rs`. Gender-aware MF2 role-label migration remains a separate follow-up (`csl26-vm2g`).

## Verification

- `cargo fmt --check` clean
- `cargo clippy --all-targets --all-features -- -D warnings` clean
- `cargo nextest run`: 1090 / 1090 passing
- `scripts/check-core-quality.js`: gate passed (154 styles, fidelity 1.0 on the gated set; 5 pre-existing concision warnings unchanged)
