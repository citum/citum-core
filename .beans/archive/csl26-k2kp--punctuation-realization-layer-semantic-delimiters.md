---
# csl26-k2kp
title: 'Punctuation realization layer: semantic delimiters'
status: completed
type: feature
priority: normal
tags:
    - multilingual
    - punctuation
    - architecture
created_at: 2026-07-18T20:31:58Z
updated_at: 2026-07-19T16:30:21Z
parent: csl26-0ugp
blocking:
    - csl26-0kqf
---

Introduce a realization layer that maps semantic punctuation roles (list separator, field separator, subfield delimiter, wrap open/close, sort separator) to glyphs per (role, effective script, locale), so styles author intent once and each item's script selects half-width or full-width forms. Subsumes the one-directional remap_to_latin_punctuation pass and its three insertion points; csl26-kneq (script-aware wrap rendering) is the first implementation increment. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §5.

## Increment 1 (absorbs the retired draft bean csl26-kneq)

- [x] Thread item script/language into wrap rendering
- [x] Render full-width CJK delimiters (（ ）, 【 】) for CJK-script items via wrap: parentheses/brackets
- [x] Byte-for-byte parity preserved for Latin-script styles (oracle gate)
- [x] Tests covering CJK + Latin items in one bilingual style

Unblocks csl26-0kqf (calendar-note wraps), which depends on this increment alone. Add the formal blocking link to csl26-0kqf once both bean files are on main; csl26-kneq itself is retired on the codex/calendar-date-annotations branch.

Spec: docs/specs/PUNCTUATION_REALIZATION.md (Draft). Increment 1 is csl26-kneq (script-aware WrapPunctuation rendering); increments 2-3 add the { mark: ... } token form, realization tables with per-script style overrides, realization-default, and the GB/T migration that demotes remap_to_latin_punctuation to a compatibility shim.

## Summary of Changes

Increment 1 implemented: `wrap: parentheses`/`brackets` now realizes
full-width CJK delimiters (（ ）, 【 】) for CJK-script items and half-width
Latin delimiters for Latin-script items, at all three insertion points that
mirror `remap_to_latin_punctuation`'s reach (`render/component.rs`,
`processor/citation.rs` citation-spec wrap, `processor/rendering/grouped/core.rs`
grouped-year wrap), plus two additional call sites discovered during
implementation (`values/date.rs` fallback-date wrap, `values/contributor/mod.rs`
role-label wrap).

Also included per user decision: `options.multilingual.realization-default`
(`latin` | `cjk`) schema field, since all embedded styles will eventually
migrate to this system and the "no-evidence default" branch had to be written
regardless — cheaper to do once now than reopen this code for a second PR.

**Design correction found during implementation (see spec v1.2 changelog):**
naive per-item script evidence (resolved from the reference's raw `language`
field) broke Chicago/APA styles that romanize non-Latin sources — their
citations render as Latin-script prose (`Hua Linfu 华林甫, ... (1999): 168–79`)
despite a `zh`/`ja`/`ko` `language` field, since raw item language is not the
same as the item's *rendered* script. Fixed by gating per-item evidence
override on style opt-in: evidence only overrides the default in styles that
set `realization-default: cjk`; styles that haven't opted in (every existing
style today) realize Latin unconditionally, regardless of item language. This
narrows increment 1's acceptance criteria slightly from the original spec
wording — documented in `docs/specs/PUNCTUATION_REALIZATION.md` §5 and its
v1.2 changelog entry.

Trait method `OutputFormat::wrap_punctuation` gained a `ScriptClass` parameter
across all 9 output-format implementations; glyph selection centralized in a
single `realize_wrap` table (`render/format.rs`) rather than duplicated per
format, since all 9 prior implementations were byte-identical.

Deferred to increment 2 (per plan, confirmed with user): the `{ mark: ... }`
token form for `delimiter`/`prefix`/`suffix` and the per-script `realization`
override map. Deferred to increment 3: migrating embedded GB/T styles to
semantic marks + `realization-default: cjk`, and demoting
`remap_to_latin_punctuation` to a compatibility shim.

Verification: `just pre-commit` green (fmt, clippy -D warnings, 2067 tests,
zero failures — including zero changes to any pre-existing test assertion,
confirming byte-for-byte parity). Schemas regenerated (`just schema-gen`);
diff is additive-only (`realization-default` field + enum, 20 lines).
