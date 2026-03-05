# Numeric Compound Citations — Architecture Record

**Bean:** csl26-zafv  
**Status:** Implemented in revised form (2026-03-05)

## Final Decision

Compound numeric grouping is modeled as an **optional top-level bibliography relationship**:

```yaml
sets:
  abc: [doe, smith, jones]
```

Per-reference `group-key` is not part of this design.

This aligns with biblatex entry-set design cues: grouping is relationship data, not a field replicated on every reference variant.

## Why This Design

1. Keeps reference items clean for the common case.
2. Represents grouping as first-class relationship data.
3. Makes compound mode opt-in via style configuration.
4. Avoids making uncommon chemistry behavior mandatory in baseline reference input.

## Data Model

### InputBibliography

`InputBibliography` now supports:

- `references: Vec<InputReference>`
- `sets: Option<IndexMap<String, Vec<String>>>`

Each set ID maps to an ordered list of reference IDs.

### Style Opt-In

Compound behavior only activates when style enables `options.bibliography.compound-numeric`.

`CompoundNumericConfig` controls citation addressing behavior and sub-label formatting:

- `subentry: true` enables member addressing (`[2a]`, `[2b]`).
- `subentry: false` keeps whole-group addressing (`[2]`).
- `sub-label`, `sub-label-suffix`, and `sub-delimiter` configure bibliography sub-item labels.

## Validation Rules

During bibliography loading:

1. Unknown set member IDs are rejected.
2. Duplicate membership across sets is rejected.
3. Empty sets are allowed (inert).
4. Singleton sets are allowed (no merge effect).

## Processor Behavior

1. Numeric numbering assigns one number slot per set.
2. Set-member ordering is preserved from `sets[set_id]`.
3. Compound group metadata is tracked internally for rendering.
4. Citation output mode is style-controlled:
   - `subentry: true` → sub-item form (`[2a]`)
   - `subentry: false` → whole-group form (`[2]`)

## Public API Surface

### Engine

- Added: `Processor::with_compound_sets(style, bibliography, sets)`
- Added: `Processor::with_locale_and_compound_sets(style, bibliography, locale, sets)`
- Added: `Processor::try_with_compound_sets(style, bibliography, sets)`
- Added: `Processor::try_with_locale_and_compound_sets(style, bibliography, locale, sets)`
- Existing constructors remain and default to no sets.

### I/O

- Added: `load_bibliography_with_sets(...) -> LoadedBibliography`
- `LoadedBibliography` carries both references and optional sets.
- Existing `load_bibliography(...)` remains as a compatibility wrapper returning references only.

## Non-Goals (This PR)

1. No compatibility shim for per-reference `group-key` in this branch.
2. No cite-site override syntax (mciteplus-style) in this PR.

## Next Steps

1. Keep cite-site grouping overrides as a separate follow-up design task.
2. Expand style fixtures that demonstrate compound-numeric with realistic chemistry templates.
3. Add additional integration snapshots for HTML/Djot/Typst compound bibliography rendering.

## Files Updated By This Design

- `crates/citum-schema/src/lib.rs`
- `crates/citum-schema/src/reference/*`
- `crates/citum-engine/src/io.rs`
- `crates/citum-engine/src/processor/*`
- `crates/citum-engine/src/values/*`
- `crates/citum-cli/src/main.rs`
- `tests/fixtures/compound-numeric-refs.{yaml,json}`
