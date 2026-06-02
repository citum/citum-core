# Locator Input Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-06-02
**Related:** bean csl26-j007, bean csl26-8sty,
`crates/citum-schema-data/src/citation.rs`,
`docs/specs/LOCATOR_RENDERING.md`,
`docs/specs/NON_STANDARD_NUMBERING_AND_LOCATOR_KINDS.md`

## Purpose

Authoritatively document how a citation locator is **expressed as input** —
what YAML/JSON forms are accepted, how the canonical in-memory model is
constructed from them, and how plurality is determined. This is the input
counterpart to `LOCATOR_RENDERING.md`, which documents *how a locator is
printed* once in memory.

This spec also records a design review of locator-input **ergonomics** first
raised in bean csl26-j007, with explicit options (including do-nothing) for
each open question.

## Scope

**In scope:**
- Three locator-input surfaces and how each produces `CitationLocator`.
- The `LocatorType`, `LocatorValue`, `LocatorSegment`, and `CitationLocator`
  data model.
- Plurality detection and its failure modes.
- Prose/Djot locator normalisation (`normalize_locator_text`).
- The compact-map syntax that was prototyped and reverted (historical record +
  future option).
- Ergonomic open questions with named options.

**Out of scope:**
- Locator rendering (labels, range-format, patterns) → see `LOCATOR_RENDERING.md`.
- Locator kind inventory and custom kinds → see
  `NON_STANDARD_NUMBERING_AND_LOCATOR_KINDS.md`.
- Style-level `LocatorConfig` → see `LOCATOR_RENDERING.md`.

## Design

### Canonical data model

All three input surfaces collapse to the same in-memory type:

```
CitationLocator = Single(LocatorSegment)
               | Compound { segments: Vec<LocatorSegment> }   // ≥2 segments
```

(`crates/citum-schema-data/src/citation.rs`, `CitationLocator`, line ~460)

```
LocatorSegment = { label: LocatorType, value: LocatorValue }
```

```
LocatorValue = Text(String)                    // plurality by heuristic
             | Explicit { value: String, plural: bool }  // explicit override
```

(`citation.rs` ~376)

```
LocatorType = Page | Chapter | Section | … | Custom(String)
```

~35 known kinds plus an open `Custom(String)` variant. See
`NON_STANDARD_NUMBERING_AND_LOCATOR_KINDS.md` for the full inventory.

### Input surface 1 — structured YAML/JSON

The primary machine-facing input. `CitationItem.locator` accepts either form
directly:

```yaml
# Single locator (shorthand — no wrapper needed)
locator:
  label: page
  value: "23"

# Single locator with explicit plural override
locator:
  label: figure
  value:
    value: "A-3"
    plural: false   # override: heuristic would see '-' and say plural

# Compound locator (≥2 segments, explicit 'segments' wrapper required)
locator:
  segments:
    - label: chapter
      value: "3"
    - label: page
      value: "42-45"
```

The deserialiser (`CitationLocatorRepr`, `citation.rs` ~470) is `untagged`:
it tries `Single(LocatorSegment)` first, then `Compound { segments }`. A
one-element `segments` list is rejected at construction time (`compound()`
enforces ≥2).

### Input surface 2 — Djot/Pandoc prose citations

Prose citations embed a locator as free text inside the Djot citation span
and are normalised by `normalize_locator_text` (`citation.rs` ~616):

```djot
[@smith2020, p. 23]
[@smith2020, ch. 3, p. 42]   # compound — comma before a recognised label
[@smith2020, 42]              # bare number → LocatorType::Page
```

**Parsing rules:**

1. Trim whitespace; empty → `None`.
2. Split on `,` only when the token *after* the comma begins with a recognised
   locator label or abbreviation (from the style's `aliases` table). This
   avoids splitting `"1, 3, 5"` into multiple locators.
3. For each segment: strip the leading label (longest alias wins), remainder
   is the value. A bare number with no label defaults to `Page`.
4. 0 segments → `None`. 1 segment → `Single`. ≥2 → `Compound`.

**Important limitation:** the prose path always produces `LocatorValue::Text`.
`Explicit { plural }` is unreachable from prose input — there is no syntax to
override the plurality heuristic.

### Input surface 3 — forward-compat / CSL-JSON bridge

`crates/citum-engine/src/api/forward_compat.rs` translates legacy CSL-JSON
`locator: String` + `label: String` fields into `CitationLocator::Single`. This
path also produces `LocatorValue::Text`.

### Plurality detection and the heuristic

`LocatorValue::is_plural` (`citation.rs` ~401):

```rust
match self {
    Text(s) => s.contains('–') || s.contains('-') || s.contains(',') || s.contains('&'),
    Explicit { plural, .. } => *plural,
}
```

The heuristic answers "does this value reference more than one unit?" which
controls term selection (`"p."` vs `"pp."`). The characters it watches for are:

| Character | Intended meaning |
|-----------|------------------|
| `-` or `–` | Range (`42–45` → plural) |
| `,` | Discontinuous list (`1, 3, 5`) |
| `&` | Paired items (`A & B`) |

**Known false positive:** a hyphen in a non-numeric identifier (`"figure A-3"`,
`"sec. 3-2"`) triggers plural even though the value denotes a single item. The
`Explicit` variant exists to override this for structured input, but has no
equivalent on the prose path.

**Two separate checks:** the engine's `apply_range_format`
(`crates/citum-engine/src/values/locator.rs`) detects ranges for formatting
purposes by scanning for `-`/`–`/`—`. This is *not* the same code as
`is_plural`. The two checks can agree or disagree independently.

---

## Historical note: compact map syntax (reverted)

Bean csl26-j007 originally specified a compact YAML map form:

```yaml
# Compact map — NOT currently implemented
locator:
  page: "23"
  line: "13"
```

This was implemented (`IndexMap`-backed `LocatorsInput`) but subsequently
reverted during the cite-site compound-grouping refactor (commits `8e38a8ca`,
`74920ed0`). There are no remaining references to `LocatorsInput` or the map
form in the codebase as of 2026-06-02.

See **Option B** below for the tradeoffs of reviving it.

---

## Open questions / ergonomic options

Each option is written as a named choice to facilitate a decision without
requiring another investigation pass.

### Option group A — Plural-detection heuristic

**A0 — Do nothing (status quo)**

Keep `contains(-|–|,|&)` as-is. Simple, ≈95% correct for typical page
references. The `Explicit` escape hatch is available for structured input.

*Cost:* Identifiers like `"figure A-3"` or `"sec. 3-2"` will silently select
the plural term on the structured path if the author forgets to use `Explicit`.
On the prose path there is no escape — such a locator always gets the wrong
term.

**A1 — Tighten the heuristic (recommended)**

Only treat a hyphen/en-dash as a range indicator when it sits between
digit-or-roman-numeral sequences, e.g.:

```
is_plural ≡ value matches /\d\s*[–-]\s*\d/
          ∨ value contains ','
          ∨ value contains '&'
```

This keeps `"42–45"`, `"1, 3, 5"`, and `"A & B"` plural while making
`"figure A-3"` and `"sec. 3-2"` singular without any author override.

*Scope:* one-line change in `LocatorValue::is_plural`; no schema changes; add
a test case for `"A-3"` and `"sec. 3-2"`.

*Risk:* roman-numeral ranges (`"xii–xv"`) would regress to singular unless the
regex also covers `[a-z]+\s*[–-]\s*[a-z]+` runs (arguably over-broad).

**A2 — Unify plurality and range detection**

Move the range-detection logic from `apply_range_format` into `LocatorValue`
so `is_plural` and "is a range" are the same computation. Eliminates the
classes of cases where they disagree.

*Scope:* crosses a crate boundary (`citum-schema-data` → `citum-engine`);
either move `PageRangeFormat` down or add a crate dependency. More invasive
than A1. Best revisited after a concrete disagreement is found in practice.

**A3 — Prose-path override syntax**

Add author-facing syntax to override plurality inside a Djot/Pandoc locator
string, since `Explicit { plural }` is unreachable from prose today. Possible
forms:

```djot
[@key, fig. A-3!s]   # '!s' = force singular
[@key, pp. 3-2!p]   # '!p' = force plural (redundant here, illustrative)
```

*Scope:* requires changes to `normalize_locator_text` + the Djot parser. Large
enough to warrant its own bean. Not recommended for this spec cycle.

---

### Option group B — Compact map syntax

**B0 — Leave reverted (recommended)**

Keep the current `{label, value}` + `{segments: […]}` forms as the only
structured input. The compound form requires the explicit `segments` wrapper,
which is slightly verbose for hand-authoring but unambiguous and schema-clean.

**B1 — Revive compact map**

Re-implement `LocatorsInput` accepting:

```yaml
locator:
  page: "23"
  line: "13"
```

alongside the existing verbose forms (untagged enum).

*Advantages:* marginally shorter for hand-authored compound locators.

*Costs:*

1. **Ambiguity at the schema level.** An object could be either a `Single`
   (`{label, value}`) or a compact map (`{page: "23"}`). Serde's `#[serde(untagged)]`
   distinguishes them by field names, but this is fragile — a future field
   added to `LocatorSegment` with the same name as a locator type would cause a
   silent mismatch.
2. **No place for `Explicit`** in the map form. `{page: "A-3"}` can only produce
   `LocatorValue::Text`. Authors needing a plural override would have to fall back
   to the verbose list form anyway.
3. **Ordering.** `IndexMap` preserves insertion order for serialisation, but
   the map form still cannot express segment-level ordering as clearly as the
   explicit `segments` list.

*Verdict:* B1's ergonomic gain is modest. Recommend B0 unless a concrete
user-workflow need surfaces.

---

### Option group C — Single vs compound structural asymmetry

The single form omits the wrapper; the compound form requires `segments`. This
is intentional (the single form is the ergonomic shorthand; compound is
deliberately explicit). No open action.

One future option (out of scope here) is a unified form that normalises
`segments: [{…}]` (one element) to `Single` at deserialisation, removing the
enforcement that `compound()` ≥ 2. This would be a non-breaking change since
the schema currently rejects single-element `segments`.

---

## Implementation notes

- `LocatorValue` uses `#[serde(untagged)]`. The `Text` arm must come first so a
  bare string is never mistakenly interpreted as `Explicit`.
- `normalize_locator_text` delegates alias lookup to the caller-supplied
  `aliases: &[(String, LocatorType)]` slice (populated from the style's locale
  term table). It does not hard-code any alias strings.
- `split_locator_segments` splits on `,` only before a recognised label, so
  discontinuous page lists (`"1, 3, 5"`) are preserved as single values.

## Acceptance criteria

*This spec documents shipped behaviour. These criteria track spec accuracy, not
new implementation.*

- [x] Three input surfaces are documented with examples.
- [x] `CitationLocator` / `LocatorSegment` / `LocatorValue` model documented.
- [x] Heuristic documented with known false-positive example.
- [x] Prose-path limitation (no `Explicit` override) documented.
- [x] Compact-map history recorded.
- [x] Options A0–A3, B0–B1, C documented with tradeoffs.
- [ ] User selects preferred option(s) from each group; beans filed for any
      non-A0/B0 choices.

## Changelog

- v1.0 (2026-06-02): Initial version. Spec covers shipped behaviour as of
  commit `2757aa3c` and subsequent refactors through `74920ed0`.
