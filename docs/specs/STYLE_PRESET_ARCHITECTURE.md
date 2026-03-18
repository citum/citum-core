# Style Preset Architecture

**Status:** Draft
**Version:** 1.0
**Date:** 2026-03-18
**Bean:** `csl26-fsjy`
**Related:** bean `csl26-zy07` (unblocked by this spec), `LOCALE_MESSAGES.md`

## Purpose

Citum styles have two levels of configuration reuse:

**Level 1 — Options presets (existing).** Named bundles of formatting
configuration (`ContributorPreset`, `DatePreset`, `TitlePreset`, etc.) applied
at the `options` key of a style file. Already implemented in
`crates/citum-schema-style/src/presets.rs`.

**Level 2 — Style presets (new).** A named, compiled-in `Style` struct
representing a complete well-known style. A YAML file (or wizard output) can
reference a style preset by name and express only the delta between itself and
the preset, rather than duplicating the full style definition.

The motivating case is **behavioral dependents** — styles that differ from a
parent by 2–5 rules (e.g. Turabian ≈ Chicago Notes + no ibid + footnote
punctuation tweaks). In CSL these are full standalone files. In Citum they
become a preset reference plus a compact `StyleVariantDelta`.

---

## §1 Problem Statement

The CSL corpus mixes two kinds of dependent styles:

1. **Locale/title-only dependents** — same formatting, different display name or
   default locale. Already handled by the Citum alias system (no YAML duplication).

2. **Behavioral dependents** — small formatting deviations from a parent. Today
   these become full standalone YAML files in `styles/`, creating maintenance
   burden and obscuring the relationship to the parent.

Additionally, the top well-known styles (`apa-7th`, `chicago-notes`, etc.) are
candidates for compiled-in embedding so that the wizard and CLI can reference
them without loading a YAML file from disk.

---

## §2 `StylePreset` Enum

A `StylePreset` identifies a compiled-in `Style` struct by a stable key.

```yaml
# YAML surface on a style document
preset: chicago-notes-18th
```

```yaml
# YAML surface with a variant delta
preset: chicago-notes-18th
variant:
  citation:
    ibid: ~       # null disables ibid — Turabian 9th ed.
  options:
    bibliography:
      entry-suffix: "."
```

### Naming convention

Style preset keys use `kebab-case` composed of `{short-name}-{edition}`:

| Preset key | `info.short_name` | `info.edition` |
|------------|-------------------|----------------|
| `chicago-notes-18th` | Chicago Notes | 18th |
| `chicago-author-date-18th` | Chicago Author-Date | 18th |
| `apa-7th` | APA | 7th |

The key is the canonical identity. `info.short_name` and `info.edition`
(csl26-zy07) are the human-readable metadata surface for UIs.

### Initial set

| Variant | Base |
|---------|------|
| `chicago-notes-18th` | compiled from `styles/chicago-shortened-notes-bibliography.yaml` |
| `chicago-author-date-18th` | compiled from `styles/chicago-author-date.yaml` |
| `apa-7th` | compiled from `styles/apa-7th.yaml` |

Additional presets are added as csl26-zy07 backfills `short_name`/`edition` on
the full style library.

---

## §3 `StyleVariantDelta` — Partial Style Overlay

A `StyleVariantDelta` is a partial `Style` where every field is optional. At
resolve time, fields present in the delta replace the corresponding fields in the
base preset. Fields absent from the delta are inherited unchanged.

**Merge semantics:** field-level (not deep). A delta `citation` replaces the
entire `citation` block of the base; it does not deep-merge individual citation
fields. Style authors who need surgical overrides should apply them within their
delta's `citation` block explicitly.

```rust
pub struct StyleVariantDelta {
    /// Overrides the base preset's top-level options.
    pub options: Option<Config>,
    /// Overrides the base preset's citation block.
    pub citation: Option<CitationSpec>,
    /// Overrides the base preset's bibliography block.
    pub bibliography: Option<BibliographySpec>,
    /// Forward-compatible escape hatch for variant concerns not yet in the
    /// schema (e.g. page-layout presets for student papers, tracked in a
    /// follow-up bean). Stored but ignored by the engine until the consuming
    /// feature is implemented.
    pub custom: Option<HashMap<String, serde_json::Value>>,
}
```

`custom` exists specifically so that a follow-up bean (Turabian student
title-page handling) can prototype its YAML surface without a schema break.

---

## §4 `StylePresetSpec` — Top-Level Style Field

A `StylePresetSpec` groups the preset key and optional variant delta into a
single YAML block, added as `preset:` at the top level of `Style`.

```rust
pub struct StylePresetSpec {
    pub preset: StylePreset,
    pub variant: Option<StyleVariantDelta>,
}
```

Full `Style` with the new field:

```rust
pub struct Style {
    // … existing fields …
    pub preset: Option<StylePresetSpec>,
}
```

When `preset` is present the engine produces the final `Style` via
`StylePreset::resolve(variant.as_ref())` before any further processing.
Explicit `options`, `citation`, and `bibliography` keys at the same level as
`preset` are applied **after** preset resolution (preset < file-level fields).

---

## §5 Registry Location

The `StylePreset` registry lives in `crates/citum-schema-style` as compiled-in
data (not in `citum-engine`). Rationale:

- The engine does not need to know about presets at render time — resolution
  happens before the `Style` struct reaches the engine.
- Keeping the registry in the schema crate preserves the declarative contract:
  a `Style` is a `Style` regardless of how it was produced.
- The wizard and CLI both depend on `citum-schema-style` already; no new
  dependency edges are introduced.

---

## §6 Locale Complement

Style presets interact cleanly with the locale override system (LOCALE_MESSAGES.md):

```yaml
preset: chicago-author-date-18th
options:
  locale-override: de-DE-chicago
```

The `de-DE-chicago` locale override encodes the locale-specific deviations from
`chicago-author-date-de.csl` (German conjunctions, date order, editor verb form)
without duplicating the style's structural template.

This replaces the CSL pattern of language-variant style files like
`chicago-author-date-de.csl`. German-speaking users select
`preset: chicago-author-date-18th` and set their locale to `de-DE` (or apply
`locale-override: de-DE-chicago` for style-specific refinements); no separate
German style file is needed.

See `locales/overrides/de-DE-chicago.yaml` and `LOCALE_MESSAGES.md §4` for the
worked example.

---

## §7 Wizard & CLI Implications

**Style Navigator** (citum-hub): The 'Closest match' banner names a preset key
(e.g. `chicago-notes-18th`), not a file path. "Use this" loads the compiled
preset directly into `WizardState`. A YAML file is only produced when the user
deviates from the preset — and even then the output contains `preset:` +
`variant:` only, not the full style.

**CLI**: `citum render` resolves `preset` before rendering. A future
`--preset <key>` flag would make the registry directly addressable without a
style file argument, but that is out of scope for this bean.

**File naming** (csl26-zy07): Once style presets exist, YAML filenames for
well-known styles should derive from `short_name` + `edition`. The rename wave
is tracked separately and blocks on this bean's preset key shape being stable.

---

## §9 Circular Dependency Prevention

To prevent infinite recursion during preset resolution, Citum implements two
levels of protection:

1. **Resolution Loop Protection:** The `into_resolved()` and `resolve()` methods
   internally track visited `StylePreset` variants using a `HashSet`. If a
   preset is encountered twice in the same resolution chain, the recursive call
   is aborted and the style is returned unresolved for that branch.

2. **Base Style Invariant:** A `Style` that serves as the base for a
   `StylePreset` (e.g. `styles/apa-7th.yaml`) **must not** itself contain a
   `preset` field. This is enforced by:
   - **Unit tests:** `all_presets_resolve_cleanly` verifies every variant in the
     registry resolves without loops.
   - **Validation policy:** Documentation and schema level warnings (planned)
     discourage circularity in user-authored styles.

If a circular dependency is detected at runtime, the engine will stop at the
first repetition and render using any fields present beyond the `preset` key,
effectively treating the circular reference as a no-op.

---

## §10 Acceptance Criteria

- [ ] `StylePreset::ChicagoNotes18th.base()` returns a `Style` that round-trips
      through serde without error.
- [ ] `StylePreset::ChicagoNotes18th.resolve(Some(&turabian_delta))` produces a
      `Style` where `citation.ibid` is `None`.
- [ ] A style YAML with `preset: chicago-notes-18th` and no other fields
      deserializes and produces the same effective `Style` as the base preset.
- [ ] A style YAML with `preset: chicago-notes-18th` + `variant.citation.ibid: ~`
      produces a `Style` where `citation.ibid` is `None`.
- [ ] `StyleVariantDelta.custom` round-trips arbitrary YAML under an unknown key
      without error or data loss.
- [ ] `options.locale-override: de-DE-chicago` paired with
      `preset: chicago-author-date-18th` renders `"und"` (not `"and"`) for a
      multi-author citation.
- [ ] `citum.schema.json` updated to include `StylePreset`, `StyleVariantDelta`,
      and `StylePresetSpec`.
- [ ] All existing oracle tests pass without regression.

---

## Changelog

- v1.2 (2026-03-18): Added §9 Circular Dependency Prevention.
- v1.1 (2026-03-18): Updated to include Apa7th and ChicagoAuthorDate18th.
- v1.0 (2026-03-18): Initial Draft.
