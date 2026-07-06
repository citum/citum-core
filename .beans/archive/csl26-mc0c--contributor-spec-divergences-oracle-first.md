---
# csl26-mc0c
title: Contributor spec divergences (oracle-first)
status: completed
type: task
priority: normal
tags:
    - contributors
    - types
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-06T13:27:30Z
parent: csl26-8m2p
---

Batch of silent spec divergences in values/contributor that override declared options; each needs oracle fixtures BEFORE changing behavior: (a) delimiter-precedes-last hardcoded branches (two-name citation 'never', GivenFirst bibliography 'never', Contextual true for two names); (b) substitute-title always quoted in citation context, bypassing per-type title formatting; (c) long-form roles auto-append (ed.)-style labels for seven hardcoded roles with no declarative off-switch, and resolve_explicit_label silently substitutes the role term for unknown keys; (d) inverted-name suffix joins with space instead of sort-separator (Smith, J. Jr. vs Smith, J., Jr.). docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md findings 9, 16, 19.

## Implementation checklist

- [x] Finding 9: tests + fix (scope narrowed after discovering 2 of 3 hardcodes are load-bearing for real APA output; only the bibliography-contextual case was fixed; other two documented as div-013)
- [x] Finding 19: test + fix (assemble_inverted_long_name now uses sort_separator before suffix)
- [x] Finding 16(a): schema + tests + wiring (SubstituteTitleQuoteMode, resolve_category_quote) done
- [x] Finding 16(b): pre-flight sweep confirmed real dependency (elsevier-with-titles-core.yaml); tests + doc-comment added, no code change
- [x] Finding 16(c): tests + warnings.rs fix + shared RECOGNIZED_LABEL_TERMS helper
- [x] Divergence register: div-011, div-012, div-013
- [x] just schema-gen, just pre-commit (1809 tests), workflow-test.sh apa.csl (byte-identical baseline), full 154-style corpus report (0.955 quality score)
- [ ] Open PR

## Summary of Changes

Closed audit findings 9, 16, and 19 (contributor-spec divergences), oracle-first:
regression tests were added for every finding before or alongside each fix, and no
fix landed without confirming it against real embedded-style output (APA in
particular, since it's the highest-risk consumer of every affected code path).

- **Finding 9** (`delimiter-precedes-last` partially ignored): only the
  bibliography/non-given-first/`Contextual` branch was a genuine bug and got fixed
  (CSL's "contextual" = delimiter only for 3+ names). The other two hardcoded
  branches (citation context; given-first bibliography) turned out to be
  load-bearing for `apa-7th.yaml`'s real, oracle-verified output — naively "fixing"
  them broke two existing tests. Kept as-is, now documented as `div-013`.
- **Finding 19** (inverted-name suffix): fixed directly.
  `assemble_inverted_long_name` now joins a generational suffix (e.g. "Jr.") with
  the sort-separator ("Smith, J., Jr.") instead of a plain space
  ("Smith, J. Jr."), matching citeproc-js. No regressions.
- **Finding 16(a)** (substitute-title always quoted in citation context): added a
  new gated schema option, `contributors.substitute.title-quote`
  (`always` | `by-category`), defaulting to `always` (today's behavior, unchanged).
  `by-category` defers to the style's normal `titles:` category rendering.
- **Finding 16(b)** (hardcoded 7-role Long-form auto-labels): a pre-flight sweep of
  every embedded style confirmed at least one (`elsevier-with-titles-core.yaml`)
  genuinely depends on the raw hardcoded default with zero configuration. No engine
  change made; added regression tests locking in the default and confirming the
  existing (previously undocumented) `contributors.role.roles.<role>.preset: none`
  escape hatch actually works, plus a doc-comment pointing at it.
- **Finding 16(c)** (`resolve_explicit_label` silently substitutes the role's own
  term for an unrecognized `label.term` key): added a new style-load-time warning
  (`unknown_role_label_term`) reusing the existing `unknown_enum_warnings` scan
  mechanism. Render behavior is unchanged — diagnostic only.
- Added `div-011`, `div-012`, `div-013` to `docs/adjudication/DIVERGENCE_REGISTER.md`.

Verification: `cargo nextest run` (1809 workspace tests), `cargo clippy -D warnings`,
`cargo fmt --check` all pass. `workflow-test.sh styles-legacy/apa.csl` byte-identical
citation/bibliography output before and after (checked at every step). Full
154-style corpus report (`node scripts/report-core.js`): 0.955 quality score.
`just schema-gen` regenerated `docs/schemas/style.json`.
