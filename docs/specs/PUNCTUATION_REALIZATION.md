# Punctuation Realization Layer Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-07-18
**Related:** [`MULTILINGUAL.md`](./MULTILINGUAL.md) §3.2a,
[`PUNCTUATION_NORMALIZATION.md`](./PUNCTUATION_NORMALIZATION.md),
[`CALENDAR_DATE_ANNOTATIONS.md`](./CALENDAR_DATE_ANNOTATIONS.md),
[2026-07-18 multilingual architecture audit](../architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md)
§5; beans `csl26-k2kp` (this feature), `csl26-kneq` (increment 1),
`csl26-30ga` (script resolution prerequisite), `csl26-fn9x` (the original
remap), `csl26-p05x` (mixed-script cluster follow-up)

## Purpose

Define a single realization layer that maps semantic punctuation marks to
glyphs per **(mark, effective item script)**, so that bilingual styles author
punctuation intent once and each item's script selects the appropriate form
(full-width `（），：` for CJK-script items, half-width `(), : ` for
Latin-script items). This replaces the current architecture, in which styles
author literal glyphs for one script and a one-directional string-rewrite
pass (`remap_to_latin_punctuation`) repairs items of the other script at
three separately maintained insertion points.

## Scope

**In scope:** the semantic mark vocabulary; the token form for template
`delimiter`/`prefix`/`suffix` fields; script-aware realization of the
existing `wrap` vocabulary (`WrapPunctuation`); the engine default
realization table and its per-script style override; the realization default
for items without script evidence; pipeline ordering relative to
punctuation normalization and output-format escaping; the compatibility
contract for literal-authored punctuation and the existing remap.

**Explicitly out of scope:** punctuation *dynamics* — collision resolution,
quote movement, and delimiter suppression remain governed by
[`PUNCTUATION_NORMALIZATION.md`](./PUNCTUATION_NORMALIZATION.md); quote
*character* selection, which already realizes through locale
`grammar-options` (`open-quote`/`close-quote`) and is unchanged; bidi
handling; locale-file participation in realization tables (future work
gated on per-item locale loading, see
[`PER_ITEM_TERM_LOCALE.md`](./PER_ITEM_TERM_LOCALE.md)); and per-segment
realization inside mixed-script compound citations (`csl26-p05x`).

## Background: the shape of the problem

Full-width delimiters currently enter rendered output from literal
characters in style YAML (`delimiter: ，`, `prefix: （`), and the only
script-aware mechanism is `remap_to_latin_punctuation`
(`crates/citum-engine/src/render/component.rs`), an opt-in
full-width → half-width rewrite of already-rendered strings. Three
properties make this a dead end as multilingual coverage grows:

1. **It is one-directional.** A style authored with half-width glyphs (or
   the semantic `wrap: parentheses`, which every output format renders
   half-width unconditionally) has no path to full-width output for CJK
   items. This is the gap tracked by `csl26-kneq` and blocking
   `csl26-0kqf`.
2. **It runs at three insertion points** — component rendering,
   citation-cluster wrap, citation-spec wrap — which must be discovered and
   kept in sync by hand (`MULTILINGUAL.md` §3.2a documents this as a known
   burden).
3. **It rewrites strings, not intent.** The four-character mapping table is
   a renderer-held policy decision, and every new script or mark extends a
   string-surgery pass rather than a data table.

There is already a working precedent for the right shape in the codebase:
`wrap: quotes` names an intent, and the glyphs come from locale
`grammar-options`. This spec extends the same move — *the style names the
mark, a table supplies the glyph* — with the item's script as the selector.

## Design

### 1. Principle

Punctuation that a style expresses semantically is carried as a **typed
mark** through component assembly and realized into glyphs **late**, by a
table keyed on the mark and the item's effective script. The width of
delimiters is not authored; it follows the item's script.
([`CALENDAR_DATE_ANNOTATIONS.md`](./CALENDAR_DATE_ANNOTATIONS.md) already
states this rule for calendar-note wraps; this spec makes it the general
mechanism rather than a per-feature carve-out.)

Literal punctuation remains fully supported and is **never rewritten** by
this layer. Realization applies only to marks the style expressed
semantically.

### 2. Mark vocabulary

The v1 vocabulary covers the marks bilingual bibliographic styles actually
vary by script, named by mark rather than by role (matching the existing
`WrapPunctuation` naming convention):

| Mark | Latin realization | CJK realization |
|---|---|---|
| `comma` | `, ` | `，` |
| `colon` | `: ` | `：` |
| `semicolon` | `; ` | `；` |
| `period` | `. ` | `。` |
| `parentheses` (open/close) | `(` / `)` | `（` / `）` |
| `brackets` (open/close) | `[` / `]` | `【` / `】` |

Notes:

- Latin realizations include their conventional trailing space; CJK
  full-width forms carry their own visual spacing and take none. This is
  why realization must produce the *whole* separator string, not just a
  character substitution.
- `quotes` is deliberately absent: quote glyphs realize through locale
  `grammar-options` today and continue to. A future revision may unify the
  two tables; v1 does not.
- The table is closed. New marks require a spec revision, mirroring the
  deliberate narrowness of the collision-policy fields in
  [`PUNCTUATION_NORMALIZATION.md`](./PUNCTUATION_NORMALIZATION.md).

### 3. Style schema: the token form

`delimiter`, `prefix`, and `suffix` accept either a literal string (today's
behavior, unchanged) or an explicit mark reference:

```yaml
# GB/T 7714 issue-number component, today (literal, CJK-only):
- number: issue
  prefix: （
  suffix: ）

# Under this spec (semantic, realized per item script):
- number: issue
  wrap: parentheses

# Field separator, today vs. semantic:
delimiter: ，
delimiter: { mark: comma }
```

The mapping form `{ mark: <name> }` is used rather than a bare string so
that literal strings can never be misread as tokens (`delimiter: comma`
stays the literal text "comma"). This follows the explicit-over-magic
principle; a shorthand can be considered later without breaking the
mapping form.

`wrap` already *is* the token form — `parentheses`/`brackets` simply gain
script-aware realization, with no schema change. Styles that need a
non-realized wrap keep using literal `prefix`/`suffix`.

### 4. Realization tables and overrides

Resolution order for a mark rendered for an item with effective script
class `S`:

1. **Style override** — `options.multilingual.scripts.<S>.realization`, a
   small map from mark name to glyph string, living in the same per-script
   config block that already holds `delimiter`, `sort-separator`,
   `use-native-ordering`, and `punctuation`:

   ```yaml
   options:
     multilingual:
       scripts:
         cjk:
           realization:
             brackets: [ "〔", "〕" ]   # style prefers hollow brackets
   ```

2. **Engine default table** — the table in §2, keyed by script class.

Script classes for v1 are `latin` and `cjk`, matching the existing
`scripts.<script>` key set; the design extends to `cyrillic`, `arabic`,
etc. by adding table rows, not mechanism.

Locale files do **not** participate in v1: realization is selected by the
*item's* script, while locale files are loaded per *style locale*. Once
per-item locale loading exists
([`PER_ITEM_TERM_LOCALE.md`](./PER_ITEM_TERM_LOCALE.md)), a locale-supplied
realization layer between steps 1 and 2 becomes possible and is noted as a
future extension.

### 5. Effective script and the realization default

The script selector is the effective item script from the unified ISO
15924 resolver (`csl26-30ga`): explicit BCP 47 script subtag first, then
primary-language lookup. Until that bean lands, the existing boolean
`is_latin_script_language` serves as an interim two-class adapter.

**The positive-evidence rule is preserved, with a style-declared default.**
Items with no usable script evidence realize through the style's declared
default context:

```yaml
options:
  multilingual:
    realization-default: cjk   # GB/T 7714; unset defaults to latin
```

- For GB/T, an untagged item realizes full-width — byte-identical to
  today's literal-authored output and to citeproc-js.
- For every style that does not set `realization-default`, untagged items
  realize Latin half-width — byte-identical to today's `wrap: parentheses`
  output.

Evidence moves an item *away* from the declared default; absence of
evidence never does. This is the same governing rule as `MULTILINGUAL.md`
§3.2a, generalized from "never remap without evidence" to "never leave the
declared default without evidence."

### 6. Pipeline position

Target ordering of the rendering tail, per the phase-3 plan in
[`PUNCTUATION_NORMALIZATION.md`](./PUNCTUATION_NORMALIZATION.md):

1. **Assemble** components; semantic marks stay typed, literals stay
   strings.
2. **Normalize** — collision policy, quote movement, delimiter
   suppression. Typed marks make this *simpler*: a `comma` token is
   `CommaLike` by construction instead of by character sniffing.
3. **Realize** — marks become glyph strings via §4.
4. **Format** — output-format escaping (HTML, LaTeX, Typst, …), which must
   see final glyphs.

Realization strictly precedes output-format escaping and strictly follows
normalization. The current implementation normalizes on strings; the
incremental path is to realize `WrapPunctuation` at the existing wrap
rendering sites first (increment 1 below), then migrate delimiter joins to
carry marks as the normalization phase is extracted.

### 7. Compatibility contract

- **Literal punctuation is never rewritten** by this layer, in either
  direction. The only rewrite of literals remains the existing opt-in
  `scripts.latin.punctuation: latin` remap, which this spec reframes as a
  **compatibility shim** for literal-authored bilingual styles. Once the
  embedded GB/T styles are migrated to semantic marks, the shim is
  deprecated (kept functional for external styles, no longer extended to
  new scripts or marks).
- **Hard gate:** byte-for-byte output parity for every existing style that
  does not opt in. `wrap: parentheses`/`brackets` on Latin-script and
  untagged items in styles without `realization-default` must render
  exactly as today.
- The remap and realization compose without conflict: realization produces
  script-correct glyphs before the remap pass would run, and the remap's
  own gate (`wants_latin_punctuation`) already restricts it to
  positively-Latin items.

### 8. Phasing

1. **Increment 1 — `csl26-kneq` (independent, first).** Script-aware
   realization of `WrapPunctuation` only: `parentheses`/`brackets` render
   full-width for CJK-script items. No new schema surface beyond
   `realization-default`. Unblocks `csl26-0kqf` (calendar-note wraps).
2. **Increment 2.** The `{ mark: … }` token form for
   `delimiter`/`prefix`/`suffix`, the engine default table, and the
   per-script `realization` override. Schema regeneration.
3. **Increment 3.** Migrate embedded bilingual styles (GB/T 7714 first)
   from literal full-width punctuation to semantic marks +
   `realization-default: cjk`; demote the remap to shim status in docs and
   `MULTILINGUAL.md` §3.2a.
4. **Future.** Locale-supplied realization (after per-item locale
   loading); additional script classes (`cyrillic`, `arabic`) with their
   own evidence rules; per-segment realization in mixed-script compound
   citations (`csl26-p05x`).

## Implementation Notes

Non-normative pointers:

- `WrapPunctuation` / `WrapConfig`:
  `crates/citum-schema-style/src/template.rs`. Every output format
  (`crates/citum-engine/src/render/{html,latex,markdown,typst,plain,djot,org}.rs`)
  currently hardcodes half-width `(`/`[`; increment 1 threads the resolved
  script into these sites, following the pattern by which quote marks are
  already passed in.
- Remap shim and its gate: `crates/citum-engine/src/render/component.rs`
  (`remap_to_latin_punctuation`, `wants_latin_punctuation`).
- Per-script style config: `ScriptConfig` in
  `crates/citum-schema-style/src/options/multilingual.rs` gains
  `realization`; `MultilingualConfig` gains `realization_default`.
- Script resolution: `is_latin_script_language`
  (`crates/citum-engine/src/values/mod.rs`) until `csl26-30ga` supplies the
  ISO 15924 resolver.
- Regenerate schemas (`just schema-gen`) in the same commit as any
  `citum-schema*` change.

## Acceptance Criteria

- [ ] `wrap: parentheses` and `wrap: brackets` render full-width for
  CJK-script items and half-width for Latin-script items in one bilingual
  style (increment 1).
- [ ] Byte-for-byte parity for all existing styles that set neither
  `realization-default` nor a `realization` override, including untagged
  items.
- [ ] `realization-default: cjk` makes untagged items realize full-width;
  positive Latin evidence still realizes half-width in the same style.
- [ ] `delimiter: { mark: comma }` renders `，` for CJK items and `, ` for
  Latin items; `delimiter: "comma"` renders the literal text "comma".
- [ ] A per-script `realization` override replaces the engine default for
  exactly the overridden marks.
- [ ] Literal punctuation in `prefix`/`suffix`/`delimiter` is never
  rewritten by the realization layer.
- [ ] Realization output passes through output-format escaping (HTML,
  LaTeX, Typst, plain, Djot) unchanged in meaning.
- [ ] The GB/T embedded style migrated to semantic marks matches its
  standard-derived expectations, with citeproc-js divergences registered
  where the standard and the oracle disagree (increment 3).
- [ ] Generated schemas include the token form, `realization-default`, and
  per-script `realization`; all new public Rust items are documented.

## Changelog

- v1.0 (2026-07-18): Initial draft, from the 2026-07-18 multilingual
  architecture audit §5. Defines mark vocabulary, token form, realization
  tables, style-declared default, pipeline position, compatibility
  contract, and phasing with `csl26-kneq` as increment 1.
