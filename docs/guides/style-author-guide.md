# Style Author Guide

This guide is for people who write and maintain Citum styles.

## What Success Looks Like

Use two quality signals, with clear priority:

1. Compatibility fidelity: output matches the chosen authority oracle.
2. SQI: style quality, maintainability, and fallback robustness.

Compatibility fidelity is the default gate. SQI helps choose between equally
correct solutions.

Do not assume the citeproc-js oracle is always normatively correct. Sometimes
it captures legacy CSL behavior that Citum should intentionally improve on.

## Authority Hierarchy

When behavior is ambiguous or conflicting, use this order of precedence:

1. Explicit publisher or style-guide rules
2. Citum design principles and schema intent
3. Stable bibliographic prior art, preferably `biblatex`
4. Legacy CSL and citeproc behavior
5. Existing local style shortcuts

Treat citeproc as a compatibility authority, not as an unquestionable source of
bibliographic truth.

## Normative vs Legacy Decisions

Before fixing a non-trivial mismatch, decide what kind of mismatch it is:

- `style defect`: the Citum style is wrong
- `migration artifact`: the migration preserved or introduced the wrong behavior
- `processor defect`: the engine misrenders a valid style
- `legacy limitation`: legacy CSL/citeproc behavior is real, but not behavior
  Citum should preserve

This classification should drive the fix:

- fix the style for `style defect`
- improve migration logic for `migration artifact`
- change engine behavior for `processor defect`
- prefer an intentional divergence for `legacy limitation`

## Intentional Divergence

Intentional divergence from legacy CSL/citeproc is allowed when it better matches
style-guide intent, bibliographic expectations, or Citum's design goals.

When you choose divergence:

- state that it is intentional
- explain which authority basis won
- add regression coverage for the intended behavior
- record any expected impact on citeproc-based compatibility metrics

## Core Principles

- Keep behavior explicit in style YAML.
- Prefer declarative templates; use `type-variants` only for genuine structural differences.
- Avoid hidden logic in processor code for style-specific formatting.
- Keep contributor names structured (`family`/`given` or `literal`).
- Preserve multilingual fallback behavior (original -> transliterated -> translated).
- Prefer readable, reusable style definitions over one-off hacks.

## Deterministic Structure Rules

Production styles are checked by `node scripts/style-structure-lint.js`.

- Anonymous generated YAML anchors such as `&id001` and `*id001` are not accepted in committed styles.
- Legacy `items:` group blocks must be authored as `group:`.
- Do not keep inert `substitute.overrides` under `template: []`; that shape is explicit dead config.
- Do not repeat the same component-level `shorten` block when a safe higher-scope `contributors.shorten` setting can express the same behavior.
- Do not keep `type-variants` that are byte-for-byte identical to the section base template.

SQI still rewards maintainability improvements, but these structure rules are enforced separately from SQI scoring.

If a repeated pattern cannot be expressed cleanly with current option scope or presets, treat that as a preset or tooling gap to fix. Do not leave duplication behind as the final style shape.

## `number:` vs `variable:`

Some fields exist in both template enums, especially `volume` and `number`.

Use `number:` when the style wants number-aware behavior:

- numeric formatting such as `ordinal` or `roman`
- number-specific labels
- numeric punctuation/layout conventions

Use `variable:` when the field should pass through as plain text with no number
formatting semantics.

Rule of thumb:

- `number: volume` means "treat this as a number component"
- `variable: volume` means "emit the value as a plain string"

When in doubt, prefer `number:` for canonical numeric bibliographic fields and
`variable:` for text-like identifiers or already-formatted strings.

## Role-Substitute Chains

Use `options.substitute.role-substitute` when one contributor role should stand
in for another or suppress an explicit fallback contributor component.

Example:

```yaml
options:
  substitute:
    role-substitute:
      container-author:
        - editor
        - editorial-director
```

How this works:

- the map key is the primary role to prefer
- the list is the ordered fallback chain
- the same chain is used for both fallback resolution and suppression of
  explicit fallback contributors

In practice, the APA chapter pattern above means:

- render `container-author` when it exists
- if `container-author` is absent, a `container-author` component may fall back
  to `editor`, then `editorial-director`
- if a separate `editor` component is also present, it is suppressed when
  `container-author` exists so the names do not render twice

Authoring rules:

- role names normalize to canonical kebab-case, so `container_author` and
  `container-author` resolve to the same role
- built-in template roles and custom contributor-role strings are both valid
- custom roles still participate in fallback and suppression even if they do
  not have a dedicated template enum variant
- locale-driven role labels are only shown when the resolved role has a known
  locale term; fallback itself does not depend on label availability

## Field-Scoped Language Metadata

`language` on a reference means "the item is generally in this language."

`field-languages` means "this specific field is in a different language than the rest of the item."

This matters for mixed-language works such as:

- a German edited volume containing an English-language chapter
- a Japanese article published in an English-language journal
- a bilingual record where the short title is English but the full title is German

Example:

```yaml
references:
  - id: chapter-1
    class: collection-component
    type: chapter
    title: English Article
    language: de
    field-languages:
      title: en
      parent-monograph.title: de
    issued: "2024"
    parent:
      type: edited-book
      title: Deutscher Sammelband
      issued: "2024"
```

How to read that example:

- `language: de` says the item is generally treated as German.
- `field-languages.title: en` says the chapter title itself should use English-sensitive formatting rules.
- `field-languages.parent-monograph.title: de` says the container book title should use German-sensitive formatting rules.

In practice, this lets a style apply different title formatting to the chapter title and the book title inside the same bibliography entry.

### When to use `field-languages`

Use `field-languages` only when entry-level `language` is not precise enough.

Do use it when:

- the chapter/article title and the container title are in different languages
- a `title-short` is in a different language than `title`
- the record is intentionally mixed-language and formatting must follow the field's own language

Do not use it when:

- the whole item is in one language
- the multilingual value already carries its own `lang` and you do not need to override it

### Supported scopes in this pass

Current engine support recognizes these keys:

- `title`
- `title-short`
- `parent-monograph.title`
- `parent-serial.title`

Unknown keys are accepted in data, but ignored by the engine for now.

### Relationship to localized templates

`field-languages` affects which language the engine uses for a specific title field.

`citation.locales[]` and `bibliography.locales[]` affect which template branch the engine picks for the item as a whole.

Example:

```yaml
citation:
  template:
    - variable: note
  locales:
    - locale: [de]
      template:
        - variable: publisher
    - default: true
      template:
        - variable: note
```

That means:

- template selection is per item
- title formatting can still vary per field inside that item

Bibliography branches work the same way and can change the full entry layout:

```yaml
bibliography:
  template:
    - contributor: author
    - title: primary
      prefix: ". "
  locales:
    - locale: [ja, zh, ko]
      template:
        - contributor: author
        - variable: publisher
          prefix: ". "
        - date: issued
          form: year
          prefix: ", "
        - title: primary
          prefix: ". "
```

See `styles/experimental/locale-specific-bibliography-layouts.yaml` for a
complete end-to-end example that uses `tests/fixtures/multilingual/multilingual-cjk.json`.

### Nested option scopes

Think of style options as three layers:

- `options` sets shared defaults for the whole style.
- `citation.options` overrides only citation-relevant fields.
- `bibliography.options` overrides only bibliography-relevant fields.

Global `options` is now limited to shared settings that make sense across
citation and bibliography rendering. Bibliography-entry controls such as
`separator`, `entry-suffix`, `hanging-indent`, and similar bibliography-only
behavior must live under `bibliography.options`.

Shared settings can still live at the top level:

```yaml
options:
  contributors:
    shorten: {min: 3, use-first: 1}
```

That same kind of shared setting can also be overridden locally where it makes
sense:

```yaml
citation:
  options:
    contributors:
      shorten: {min: 4, use-first: 2}
```

Bibliography-entry controls are different. They are valid only inside
`bibliography.options`, and they are never valid globally or inside
`citation.options`:

```yaml
bibliography:
  options:
    separator: ", "
```

```yaml
options:
  bibliography:
    separator: ", "
```

```yaml
citation:
  options:
    separator: ", "
```

Use `citation.options` for citation-local overrides such as:

```yaml
citation:
  options:
    contributors:
      shorten: {min: 3, use-first: 1}
    locators:
      form: short
```

Use `bibliography.options` for bibliography-local overrides such as:

```yaml
bibliography:
  options:
    entry-suffix: "."
    separator: ", "
    suppress-period-after-url: true
```

Do not put bibliography-entry controls under `citation.options`, and do not put
citation-only fields such as `locators`, `notes`, or `integral-names` under
`bibliography.options`.

## Practical Workflow

1. Start from a nearby style in `/styles`.
2. Identify the authority basis for the behavior you are implementing.
3. Classify major mismatches as style defect, migration artifact, processor defect, or legacy limitation.
4. Implement the target style-guide or Citum-intended rules in YAML (`options`, `citation`, `bibliography`).
5. Run oracle checks to confirm rendered output and understand compatibility impact.
6. Fix compatibility mismatches first unless the task calls for a documented semantic divergence.
7. Improve SQI only when output semantics stay unchanged.
8. Re-run checks before finishing.

## Preset Catalog

Use presets first, then override only what is style-specific.

### Contributor presets (`options.contributors`)

Each preset encodes name formatting conventions for a style family. Pick the closest match; add explicit overrides for fields that differ.

| Preset | Format | Initials | Conjunction | et al. | Example |
|--------|--------|----------|-------------|--------|---------|
| `apa` | First family-first | `. ` (period-space) | `&` symbol | 21/19 | `Smith, J. D., & Jones, M. K.` |
| `chicago` | First family-first | none (full names) | `and` text | none | `Smith, John D., and Mary K. Jones` |
| `vancouver` | All family-first | none | none | 7/6 | `Smith JD, Jones MK` |
| `ieee` | All given-first | `. ` (period-space) | `and` text | none | `J. D. Smith, M. K. Jones` |
| `harvard` | All family-first | `.` (period only) | `and` text | none | `Smith, J.D., Jones, M.K.` |
| `springer` | All family-first | none | none | 5/3 | `Smith JD, Jones MK` |
| `numeric-compact` | All family-first | none | none | 7/6 | `Smith J, Jones M` |
| `numeric-medium` | All family-first | none | none | 4/3 | `Smith J, Jones M` |
| `numeric-tight` | All family-first | none | none | 7/3 | `Smith J, Jones M, Brown A, et al.` |
| `numeric-large` | All family-first | none | none | 11/10 | `Smith J, … [10 authors], et al.` |
| `numeric-all-authors` | All family-first | none | none | none | `Smith JD, Jones MK, Brown AB` |
| `numeric-given-dot` | All given-first | `.` (period only) | none | none | `J.D. Smith, M.K. Jones, A.B. Brown` |
| `annual-reviews` | All family-first | none | none | 7/5, demote-never | `van der Berg J, Smith M, Jones A, Brown B, White C, et al.` |
| `math-phys` | All family-first | `.` (period only) | none | none (set separately) | `Smith, J., Jones, M., Brown, A.` |
| `soc-sci-first` | First family-first, rest given-first | `. ` (period-space) | none | none (set separately) | `Smith, J. D., M. K. Jones` |
| `physics-numeric` | All given-first | `. ` (period-space) | none | none (set separately) | `J. Smith, M. Jones, A. Brown` |

**Choosing between similar presets:**

- Compact initials (`""`): `vancouver`, `numeric-compact`, `numeric-medium`, `numeric-tight`, `numeric-large`, `numeric-all-authors`, `annual-reviews`, `springer`
- Period-only initials (`"."`): `harvard`, `math-phys`, `numeric-given-dot`
- Period-space initials (`". "`): `apa`, `ieee`, `soc-sci-first`, `physics-numeric`
- Given-first (no inversion): `ieee`, `physics-numeric`, `numeric-given-dot`
- First-author-only inversion: `apa`, `chicago`, `soc-sci-first`
- All inverted: everything except `ieee`, `physics-numeric`, `numeric-given-dot`

When you need a different et al. threshold than the preset provides, use the preset and add a `shorten:` override at the context level:

```yaml
options:
  contributors: math-phys       # all family-first, period initial, comma sort-sep
bibliography:
  options:
    contributors:
      shorten: { min: 11, use-first: 10 }   # override et al. threshold
```

### Date presets (`options.dates`)

| Preset | Month format | EDTF markers | Example |
|--------|-------------|--------------|---------|
| `long` | Full names | Yes (`ca.`, `?`, en-dash ranges) | `January 15, 2024` |
| `short` | Abbreviated | Yes | `Jan 15, 2024` |
| `numeric` | Numbers | Yes | `1/15/2024` |
| `iso` | Numbers | No | `2024-01-15` |

### Title presets (`options.titles`)

| Preset | Article/component | Book/monograph | Journal/periodical |
|--------|------------------|---------------|--------------------|
| `apa` | plain | *italic* | *italic* |
| `chicago` | "quoted" | *italic* | *italic* |
| `ieee` | "quoted" | *italic* | *italic* |
| `humanities` | plain | *italic* | *italic* |
| `journal-emphasis` | plain | plain | *italic* |
| `scientific` | plain | plain | plain |

### Substitute presets (`options.substitute`)

Controls what replaces the author when it is missing:

- `standard`: Editor → Title → Translator (most styles)
- `editor-first`: Editor → Translator → Title
- `title-first`: Title → Editor → Translator
- `editor-short` / `editor-long`: Editor only, with short or long role label
- `editor-translator-short` / `editor-translator-long`: Editor then Translator
- `editor-title-short` / `editor-title-long`: Editor then Title
- `editor-translator-title-short` / `editor-translator-title-long`: Full chain

### Template presets

- `citation.use-preset: numeric-citation` for numeric styles that render citation numbers via style-level wrapping (`[1]`, `(1)`, or superscript contexts).

### Processing defaults

`options.processing` sets bibliography-family defaults, not citation-list sorting:

- `author-date`: bibliography defaults to `author-date-title`
- `note`: bibliography defaults to `author-title-date` when a bibliography is present
- `label`: bibliography defaults to `author-date-title`
- `numeric`: no bibliography sort is implied; insertion order is preserved unless `bibliography.sort` is set

`citation.sort` remains explicit-only. If you omit it, multi-cite clusters keep the citation input order.

### Example combining presets

```yaml
options:
  processing: author-date
  contributors: math-phys         # Springer math/physics family-first with period initial
  dates: short
  titles: apa
  substitute: standard
bibliography:
  options:
    contributors:
      shorten: { min: 3, use-first: 1 }   # override et al. threshold only
```

```yaml
options:
  contributors: numeric-compact
  dates: long
  titles: humanities
  substitute: editor-translator-title-short

citation:
  use-preset: numeric-citation
  wrap: brackets
```

## Style-Level Presets
 
To avoid duplicating complete styles for the most common formatting families, Citum provides **Style-Level Presets**. These are named, compiled-in `Style` structs that you can reference at the top of your style YAML.
 
### Base Presets
 
Use the `preset` key to load a base style definition:
 
```yaml
preset: chicago-notes-18th
```
 
This produces a complete Chicago Notes 18th edition style without any other fields in your file.
 
### Overriding Preset Fields

Declare `preset:` and then add any top-level fields you want to change.
All fields — `citation`, `bibliography`, `options`, `info` — merge over the
preset base, with your fields taking ultimate precedence.

Behavioral variant (e.g. Turabian = Chicago Notes without ibid):

```yaml
preset: chicago-notes-18th
citation:
  ibid: ~    # null disables ibid (Turabian 9th ed.)
```

Locale variant (e.g. Chicago for German users):

```yaml
preset: chicago-author-date-18th
options:
  locale-override: de-DE-chicago
```

Both patterns use the same mechanism: top-level fields merged onto the preset.
There is no separate `variant` layer.

Preset-backed styles currently show up in two places on disk:

- `styles/preset-bases/<name>.yaml` is the canonical embedded base loaded by
  `preset: <name>`.
- `styles/<name>.yaml` is the public style entrypoint that users can load
  directly, publish, or override further.

That is intentional, not two independent implementations. The preset base is
the internal source of truth for inheritance, while the top-level style file is
the user-facing wrapper layer. For `apa-7th`, the wrapper currently also carries
materialized overrides, so changes that affect the public style still need to be
kept in sync with the preset base until the authoring/export pipeline collapses
that duplication automatically.
 
Refer to [STYLE_PRESET_ARCHITECTURE.md](../specs/STYLE_PRESET_ARCHITECTURE.md) for the full list of available presets and technical details.
 
## Repeated Note Citation Recipes
 
Use citation position entries when note styles need distinct rendering for:
 
- first cite (`citation.template`)
- subsequent cites (`citation.subsequent`)
- ibid cites (`citation.ibid`)
- non-immediate repeat (`citation.subsequent.template`)
- immediate repeat (`citation.ibid.template`)

Position resolution order is:

1. `citation.ibid` for `Ibid` and `IbidWithLocator`
2. `citation.subsequent` when `citation.ibid` is absent
3. base `citation.template`

### Chicago or Turabian short repeats (no lexical ibid)

Define only `citation.subsequent` and omit `citation.ibid`.
Immediate repeats then reuse the same short form through fallback.

```yaml
citation:
  template:
    - contributor: author
      form: long
    - title: primary
      prefix: ", "
  subsequent:
    template:
      - contributor: author
        form: short
      - title: primary
        form: short
        prefix: ", "
```

### OSCOLA or MHRA ibid patterns

Define `citation.ibid` explicitly and include locator rendering in that template.

```yaml
citation:
  template:
    - contributor: author
      form: long
    - title: primary
      prefix: ", "
  subsequent:
    template:
      - contributor: author
        form: short
      - title: primary
        form: short
        prefix: ", "
  ibid:
    template:
      - term: ibid
      - variable: locator
        prefix: ", "
```

### Bluebook-style immediate repeat (`Id.`)

Use locale term data (or a term override) for the lexical token, then keep the
same `citation.ibid` structure.

```yaml
citation:
  ibid:
    template:
      - term: ibid
      - variable: locator
        prefix: " at "
```

## Verification Commands

Run from repository root:

```bash
# Compare a style against oracle output
node scripts/oracle.js styles-legacy/apa.csl

# Check core fidelity + SQI drift
node scripts/report-core.js > /tmp/core-report.json
node scripts/check-core-quality.js \
  --report /tmp/core-report.json \
  --baseline scripts/report-data/core-quality-baseline.json
```

If your style work includes Rust code changes, run:

```bash
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run
```

Use `cargo test` if `cargo nextest` is unavailable.

## How to Use SQI Well

SQI is most useful for improving style quality after correctness is established.

Target improvements such as:

- Better type coverage.
- Stronger fallback behavior.
- Less duplication across templates and `type-variants`.
- Cleaner use of shared options and presets.

**Preset-first approach for high SQI without `type-variants`:**
1. Use option presets (`options.contributors`, `options.dates`, `options.titles`,
   `options.substitute`) and template presets (`citation.use-preset`) to share
   configuration across styles without per-type repetition.
2. Design the generic template to handle the common case cleanly — avoid
   special-casing types that differ only in a label or one optional field.
3. Add `type-variants` only when a reference type needs a structurally different
   component set (different fields, different order).

Do not trade fidelity for a higher SQI score.

## Common Mistakes

- Putting style-specific punctuation rules into processor code.
- Solving one style with hardcoded exceptions instead of using presets or `type-variants`.
- Duplicating variable rendering when substitution/fallback can do it cleanly.
- Accepting small oracle regressions for “cleaner” YAML.

## Definition of Done

A style update is complete when:

- Oracle fidelity target is met.
- No fidelity regressions are introduced in affected core styles.
- SQI is stable or improved.
- Style YAML remains explicit, readable, and maintainable.

## Related Reading

- [Rendering Workflow](./RENDERING_WORKFLOW.md)
- [SQI Refinement Plan](../policies/SQI_REFINEMENT_PLAN.md)
- [Type Addition Policy](../policies/TYPE_ADDITION_POLICY.md)
- [Citum Personas](../architecture/PERSONAS.md)
