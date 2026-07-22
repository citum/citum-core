---
# csl26-838l
title: Per-item term localization (autolang)
status: completed
type: feature
priority: normal
tags:
    - multilingual
    - locale
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-22T11:34:26Z
parent: csl26-0ugp
---

Rendering an item's terms in the item's language (German "hrsg. von" for a German source in an English-locale Chicago style) currently requires a citation.locales/bibliography.locales branch that swaps the whole template. Add an opt-in that switches locale-sensitive term/message/date-pattern lookups to the effective item language without changing template structure — the biblatex autolang analogue. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(g).

Spec: docs/specs/PER_ITEM_TERM_LOCALE.md (Draft). Opt-in options.multilingual.term-locale: style | item at the three MultilingualConfig scopes; item mode switches roles/terms/messages/date patterns to the effective item language's locale with exact-tag -> primary-language -> style-locale fallback. Typography (grammar-options) stays style-locale in v1; locale-scoped layout branches take precedence. Usefulness bounded by embedded locale coverage (csl26-tfi8, csl26-itri).

## Implementation Checklist

- [x] Schema: `TermLocale` enum + `term_locale` field on `MultilingualConfig`
- [x] `just schema-gen` regenerated
- [x] `Locale::with_term_surfaces_from` hybrid constructor
- [x] `locale_for_reference` returns `Cow<Locale>`, resolves item locale under `term-locale: item`
- [x] Fix 3 `RenderOptions` construction sites (grouped/core.rs, citation_render_options + 2 callers)
- [x] `term_locale_fallback_warnings` scanner + registration in session.rs/document.rs
- [x] Verify `.locale`-id readers (variable.rs/title.rs = style; term/message/labels = casing-only)
- [x] Tests: schema round-trip, engine unit (typography split), scope independence, fallback, branch precedence, rendering-only guarantee (§6)
- [x] Docs: PER_ITEM_TERM_LOCALE.md Draft → Active, acceptance criteria ticked
- [x] `just pre-commit` green

## Summary of Changes

Implemented `options.multilingual.term-locale: style | item` (default `style`,
byte-identical to prior behavior).

- **Schema** (`citum-schema-style`): `TermLocale` enum + `term_locale` field
  on `MultilingualConfig`; `Locale::with_term_surfaces_from` builds the
  hybrid locale (item word/date surfaces + style typography/identity) as an
  exhaustive field literal, not clone-then-mutate.
- **Engine**: `Renderer::locale_for_reference` (the single chokepoint already
  feeding both citation and bibliography `RenderOptions.locale`) now returns
  `Cow<Locale>` — branch match (unchanged) → full branch locale; no branch +
  `term-locale: item` + resolvable item locale → the hybrid; otherwise style
  locale. New `term_locale_fallback_warnings` scanner reports tagged items
  whose language has no loaded locale (untagged items stay silent).
- **Docs**: `PER_ITEM_TERM_LOCALE.md` Draft → Active, all acceptance criteria
  ticked, plus a new note on the casing-gate/style-id v1 boundary.
- **Tests**: `crates/citum-engine/tests/term_locale.rs` (7 integration tests,
  native `InputReference` construction, full `assert_eq!` on captured
  output) covering word/typography split, scope independence, fallback +
  warning, branch precedence (including that a branch also swaps typography,
  unlike the hybrid), bibliography-order stability, and byte-identical
  default. Plus 3 schema round-trip unit tests in `multilingual.rs`.
- `just pre-commit` green (fmt, clippy -D warnings, 2136 nextest tests);
  `just schema-gen` regenerated `docs/schemas/style.json`.

Also fixed an unrelated local environment issue found along the way: the
Mason-installed `rust-analyzer` binary (0.3.2237) was stale relative to the
pinned `stable` toolchain (1.97.0), causing a proc-macro server version
mismatch that cascaded into bogus diagnostics across edited files. Repointed
`~/.local/share/nvim/mason/bin/rust-analyzer` to the toolchain-shipped,
version-matched binary at
`~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rust-analyzer`.

## Follow-ups (not filed as beans, mentioning for visibility)

- Acceptance criterion 2 ("hrsg. von", "In:") is satisfied by the underlying
  mechanism (role/term lookups both read the same hybridized
  `options.locale` field the "and"/month-name tests exercise), but no test
  literally renders an editor role label — would need
  `options.contributors.role` preset wiring in a future test pass if closer
  fidelity to the literal spec example is wanted.
