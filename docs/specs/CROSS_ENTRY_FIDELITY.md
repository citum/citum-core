# Cross-Entry Fidelity: Disambiguation and Subsequent-Author Substitution

**Status:** Active
**Version:** 1.0
**Date:** 2026-06-02
**Related:** `csl26-hxqq`

## Purpose

Two citation behaviors are inherently *cross-entry* — they depend on the
relationship between adjacent bibliography entries or between multiple
citations in a cluster, not just on a single entry rendered in isolation:

1. **Author-date disambiguation** — When two or more references produce
   identical rendered citation keys (same author group and same year), the
   processor must differentiate them by expanding names, adding initials, or
   appending a year suffix (`2023a`, `2023b`).

2. **Subsequent-author substitution** — In bibliography output, when a
   reference has the same author group as the preceding entry, many styles
   replace the repeated author string with a substitute symbol (typically
   three em-dashes: `———`).

Both behaviors are implemented in the engine. This spec records Phase 0
investigation results, the config-model decision, and the acceptance criteria.

---

## Phase 0 Findings

### Config Model

The engine's disambiguation is **per-style config gated**. The gate is the
`disambiguation_flags()` method in
`crates/citum-engine/src/processor/disambiguation.rs`:
it reads `config.processing`, and when `processing: author-date` (i.e.
`Processing::AuthorDate`) is present, calls `Processing::AuthorDate.config()`
which returns all three disambiguation strategies enabled:

```rust
disambiguate: Some(Disambiguation {
    names: true,
    add_givenname: true,
    year_suffix: true,
}),
```

**Decision:** No change to the config model is needed for disambiguation.
Any style with `processing: author-date` already gets full disambiguation.
Styles should not need to repeat these flags explicitly.

Subsequent-author substitution is **per-style, opt-in** via
`bibliography.options.subsequent_author_substitute` (string) and
`bibliography.options.subsequent_author_substitute_rule` (enum). Absence =
no substitution; this is the correct behavior for styles like APA that do
not use em-dashes.

### Confirmed Bug

`styles/experimental/chicago-author-date.yaml` is missing
`subsequent-author-substitute: "———"` and
`subsequent-author-substitute-rule: complete-all` in its `bibliography.options`.

The reference implementation (`styles/chicago-author-date-classic.yaml`) and
the schema preset (`RepeatedAuthorRendering::Dash` in `scoped.rs`) both confirm
the correct values. The Zotero `chicago-author-date.csl` in `styles-legacy/`
does not implement em-dashes (it was never added to the reference CSL), so
the oracle cannot catch this gap by comparison.

### Infra Gaps

1. **Fixture coverage**: `tests/fixtures/references-expanded.json` has one
   same-author-same-year pair (Garcia 2019, ITEM-31/ITEM-32) and one
   same-author disambiguation citation (`disambiguate-year-suffix`). Missing:
   a same-family-different-given pair (e.g., "Smith, John" and "Smith, Jane"
   with the same year) to exercise givenname expansion.

2. **Oracle blind spot for native styles**: The oracle compares against a
   citeproc-js run of the source CSL. For native/experimental Citum styles
   whose reference CSL omits a feature (like Chicago's em-dash), oracle
   passes even when the feature is absent from the YAML.

3. **No portfolio-wide parity audit**: There is no automated check that
   styles with `subsequent-author-substitute` in their source CSL also have
   it in the migrated YAML. 322 source CSL files use the attribute; 13 of
   141 migrated YAML styles have it. Many of the 322 map to styles outside
   `styles/`, but a targeted audit is needed.

4. **`report-core.js` does not score `subsequent_author_substitute`** as a
   bibliography component for applicable styles.

---

## CSL 1.0.2 Reference (Perplexity-verified)

### Disambiguation Cascade

Applied in this order, each strategy skipped once the previous resolves the
collision:

1. **`disambiguate-add-names`** — Expand the author list beyond et-al
   truncation until citations differ. Only effective when et-al is triggered.
   The processor increases the number of rendered names in the variable(s)
   that participate in disambiguation; if expanding to the full list resolves
   the collision, subsequent strategies are skipped.

2. **`disambiguate-add-givenname`** — Add given names or initials to
   differentiate same-family-name authors. Controlled by
   `givenname-disambiguation-rule`:
   - `all-names` — expand all names in the list; expansion is global (all
     occurrences of the affected name show given names, not just ambiguous ones)
   - `all-names-with-initials` — same scope but constrained to initials form;
     never escalates to full given names solely for disambiguation
   - `primary-name` — expand only the first author's given name (full form);
     co-authors are left unchanged unless add-names requires them
   - `primary-name-with-initials` — same as primary-name but initials only
   - `by-cite` *(default since CSL 1.0.1)* — per-cite, minimal subset; one
     cite may get initials while another collision gets a different subset;
     no globalization requirement

3. **`disambiguate-add-year-suffix`** — Append `a`, `b`, `c`, … to the year
   when name strategies are exhausted. Suffix ordering is deterministic:
   sorted by the per-group sort when configured, otherwise by title order.
   The suffix must remain stable across the document.

**Note on APA's divergence from the canonical cascade:** APA 7th prioritizes
year-suffix *before* givenname expansion for same-author-same-year collisions.
CSL's canonical cascade is add-names → add-givenname → year-suffix. For
same-author (identical authors) same-year, givenname expansion produces no
change anyway (same given name), so the engine falls through to year-suffix
regardless. For different-family-same-year collisions APA uses initials. In
practice our engine produces correct output, but the APA-specific ordering is
not explicitly encoded.

### Schema Gap: `givenname_rule` — Resolved (csl26-4ada)

**Resolved in csl26-4ada.** `Disambiguation` now has:

```rust
pub struct Disambiguation {
    pub names: bool,
    pub add_givenname: bool,
    pub year_suffix: bool,
    pub givenname_rule: GivennameRule,  // added in csl26-4ada
}
```

`GivennameRule` models all five CSL values (`by-cite` default, `all-names`,
`all-names-with-initials`, `primary-name`, `primary-name-with-initials`). The
engine collapses them to two scopes: `primary-name` and
`primary-name-with-initials` expand only the first author; all other values
expand all positions (current behavior). Initials vs full given name continues
to be driven by the contributor config's `initialize-with` / `name-form`
settings. The `by-cite` per-cite minimal-subset algorithm is a documented
divergence deferred to a future follow-up.

See `docs/specs/DISAMBIGUATION.md` §2.1 for the full rule table.

### `subsequent-author-substitute` Semantics

### `subsequent-author-substitute` Semantics

Attribute on `<bibliography>` (CSL) / `bibliography.options` (Citum YAML).
Applied **after** formatting, when comparing the current bibliography entry's
rendered name list to the immediately preceding entry. Has no effect on
citations. Distinct from `<substitute>` (field fallback for missing variables,
which operates *before* formatting per item).

- **Value**: the substitute string (e.g., `———`, `---`, `——`, `&#8212;&#8212;&#8212;`).
- **Rule** (`subsequent-author-substitute-rule`):
  - `complete-all` *(default)* — substitute only when the entire author group
    (all names, same count) matches the previous entry's group. Reflects CSL
    1.0 original behavior.
  - `complete-each` — same complete-match requirement, but renders each
    substituted name slot individually rather than the list as a whole.
    Relevant when a style renders a name list via multiple `cs:name` calls.
  - `partial-each` — substitute per rendered name that also appears in the
    preceding entry's list at the same position; stop at first mismatch.
    Enables cascading patterns: `Doe+Roe 2010` → `———, and Poe 2012`.
  - `partial-first` — only the first rendered name is considered; if it
    matches the first name of the previous entry, substitute it, regardless
    of co-authors. Same cascade form but without per-name matching depth.

**For Chicago author-date**, `complete-all` is correct: the dash replaces the
full author group only when all authors are identical. The partial rules were
introduced specifically for Chicago-like cascades with varying co-authors, but
CMoS 18 §14.67 specifies the dash for identical groups only.

---

## Style-by-Style Expectations

### APA 7th Edition (`styles/apa-7th.yaml` / embedded)

- **Em-dash**: ❌ not used. APA 7 always repeats the full author group.
- **Disambiguation**: year-suffix for same-author-same-year (APA prioritizes
  this over givenname expansion); initials for same-family-different-given
  (givenname rule: `primary-name-with-initials`). The engine's `author-date`
  default produces correct output because givenname expansion on identical
  authors has no effect, so year-suffix is reached anyway. Initials are
  rendered via the contributor config's `initialize-with: ". "`.
- **Schema gap**: resolved in csl26-4ada — `GivennameRule::PrimaryNameWithInitials`
  restricts expansion to the first author; initials vs full still driven by
  contributor config `initialize-with`.

### Chicago Manual of Style 18th ed., author-date (`styles/experimental/chicago-author-date.yaml`)

- **Em-dash**: ✅ fixed in this PR — `bibliography.options`:
  ```yaml
  subsequent-author-substitute: "———"
  subsequent-author-substitute-rule: complete-all
  ```
- **Disambiguation**: year-suffix for same-author-same-year; full given name
  of first (primary) author for same-family collisions (givenname rule:
  `primary-name`). The contributor config `initialize-with` must NOT be set
  (or be empty) to render full given names, unlike APA.
- **Schema gap**: resolved in csl26-4ada — `GivennameRule::PrimaryName`
  restricts expansion to the first author; initials vs full still driven by
  contributor config.

---

## Acceptance Criteria

- [ ] `styles/experimental/chicago-author-date.yaml` renders `———` for
  consecutive same-author bibliography entries, matching CMoS 18 spec.
- [ ] APA 7th bibliography does **not** render em-dashes for repeated authors.
- [ ] `tests/fixtures/` includes same-family-different-given pairs that
  trigger givenname disambiguation; oracle confirms matching output.
- [ ] `scripts/audit-cross-entry-parity.js` runs without errors and reports
  zero unexplained offenders across `styles/`.
- [ ] `report-core.js` scores `subsequent_author_substitute` as a component
  for applicable author-date and note styles.
- [ ] `scripts/report-data/core-quality-baseline.json` updated to reflect new
  scoring.
- [ ] Full Rust gate passes: `cargo fmt --check && cargo clippy --all-targets
  --all-features -- -D warnings && cargo nextest run`.
