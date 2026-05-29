# Intentional Divergence Register

This document records intentional behavioral departures from legacy CSL/citeproc where Citum has chosen a different rendering strategy based on publisher rules, biblatex prior art, or design principles.

## Purpose

The Divergence Register serves as a **durable audit trail** of known structural and formatting differences between Citum output and citeproc-js for a given style. It allows style-wave authors and maintainers to:

1. Distinguish intentional divergences from defects during oracle review
2. Track the authority basis for each choice (publisher rules, biblatex precedent, design principle, or documentary evidence)
3. Communicate expectations to downstream users and tooling integrations
4. Version divergence decisions as styles evolve across phases

## Extension Protocol

When adding a new intentional divergence:

1. **Assign the next sequential ID**: Use `div-NNN` where NNN is the next integer after the highest existing ID.
2. **Classify the authority basis**: Select one of four values:
   - `Publisher rules` — The behavior matches how the authoritative publisher (Nature, ACS, IEEE) handles that element
   - `Biblatex prior art` — The behavior mirrors biblatex's well-established convention
   - `Citum design principle` — The behavior follows an explicit Citum architectural decision (e.g., explicit-only policies)
   - `Documentary evidence` — The behavior is grounded in a published standard, specification, or style guide
3. **Note oracle/compat impact**: State whether existing oracle/compatibility snapshots should be updated or left as known mismatches
4. **Reference context**: Optionally link to the relevant bean, commit, or discussion that motivated the divergence

## Divergence Table

| ID | Styles | Behavior | Legacy CSL/citeproc | Citum | Authority | Oracle/Compat Impact |
|----|--------|----------|----------------------|--------|-----------|----------------------|
| div-001 | `nature`, `cell`, `plos`, `acs-nano`, and other numeric-compound styles | Bibliography entries grouped as publisher-level sets (e.g. "1. AuthorA … 2. AuthorB …") rather than rendered as individual references with per-reference group-key logic | Renders each reference independently using `group-by` keys; compound grouping not a native CSL feature | Engine renders compound sets as first-class bibliography units via `compound: true` option | Publisher rules | Causes structural mismatches in oracle output for all compound-numeric styles; expected and accepted |
| div-002 | `angewandte-chemie`, `american-chemical-society`, `rsc`, `numeric-comp` | Format decisions (punctuation, name order, volume/pages delimiter) follow biblatex defaults rather than citeproc output where they conflict | Treated citeproc output as normative reference | Biblatex prior art takes precedence when CSL output appears bibliographically wrong or underspecified | Biblatex prior art | Minor per-field mismatches in oracle comparisons; classified as `legacy-limitation`, not defects |
| div-003 | All author-date, note, and numeric styles | Processing families supply explicit bibliography sort defaults: `author-date` → `author-date-title`; `note` → `author-title-date`; `label` → `author-date-title`; `numeric` → none (insertion order preserved). Explicit style-level `bibliography.sort` always wins. | Applies uniform global bibliography sort; numeric styles may sort alphabetically by default | Each processing family has an explicit sort default; numeric styles preserve insertion order | Biblatex prior art | Minimal impact; most styles set explicit sorts. Insertion-order stability for numeric styles is intentional. |
| div-004 | All styles using author-based sorting | Works with no author/editor/translator sort by title rather than being treated as empty/equal | Behavior underspecified; often sorts missing-name works together at top or bottom inconsistently | When author sort key is absent, falls back unconditionally to title as sort key (removed `author_fallback_to_title` flag) | Biblatex prior art | May cause ordering differences for reference sets with anonymous works; consistent with bibliographic expectation |
| div-005 | All styles | Citation-list sorting never implied by processing family; only explicit `citation.sort` triggers sorting | Some styles rely on implicit citation-list sorting derived from processing class | Phase 1 policy: `citation.sort` explicit-only; absent → preserve input order. Family-level citation sort deferred to later phase. | Citum design principle | Possible ordering differences in citation lists for styles expecting family-implied sort; acceptable Phase 1 limitation |
| div-006 | ~~`new-harts-rules-notes`, `new-harts-rules-notes-label-page`, `new-harts-rules-notes-no-url`~~ | ~~Patent bibliography author name-order: first author inverted (family-first), subsequent authors given-first~~ | ~~All authors inverted when `name-order: family-first` is used in contributor config~~ | **Resolved** — added `NameOrder::FamilyFirstOnly` variant; affected styles updated to `name-order: family-first-only` | Engine fix | Fixed in csl26-zzun |
| div-007 | All styles using `interviewer` contributor with `form: verb` | Verb form renders as `"interviewed by"` | `"interview by"` in upstream `locales-en-US.xml` (missing past participle — grammatically incorrect passive construction) | `"interviewed by"` — standard English past-participle passive for the role | Citum design principle | No oracle impact; no current style uses `form: verb` on interviewer contributor blocks |
| div-008 | All styles using author-based sort with multiple authors sharing a family name | Secondary sort key after author family name: citeproc-js uses given name (family → given → suffix → title); Citum uses title (family → title) | citeproc-js sorts within same-family-name groups by given name ascending | Citum uses title as the secondary sort key after family name — consistent with the title-fallback philosophy of div-003/div-004 and with biblatex `sortname` field semantics | Citum design principle | Causes reordering within same-family-name groups in any author-sorted bibliography; numeric citation labels shift accordingly. Divergence-aware oracle adjustment fires independently alongside div-004 when both conditions are present in the same fixture. |
| div-009 | All author-date styles using `year-suffix` disambiguation with reprinted works | Year-suffix letter keys on the `issued` (reprint) year only; original-publication date is rendered but never enters the collision key. Three reprints (orig 1926/1926/1927, all issued 1967) all receive suffixes: `(1926/1967a) (1926/1967b) (1927/1967c)`. | citeproc-js keys suffix assignment on the full rendered date string; in this scenario the work with a distinct original date does not receive a suffix: `(1926/1967a) (1926/1967b) (1927/1967)`. | Citum keys on `issued` year exclusively (`build_group_key` in `disambiguation.rs`). Working assumption: major author–date systems (APA, Chicago) treat the year of the edition consulted as the operative year for disambiguation; no major style is known to require a different rule. The citeproc-js behavior is undocumented and has no evidence of user dependence. | Documentary evidence | Oracle will show `(1927/1967c)` for Citum vs `(1927/1967)` for citeproc-js in affected reprint scenarios; expected and accepted. |

## Revision History

- **2026-03-07**: Initial register created with five foundational divergences (div-001 through div-005)
- **2026-03-28**: Added div-006 for Hart's Rules patent author name-order gap
- **2026-04-11**: Resolved div-006 — added `NameOrder::FamilyFirstOnly`; updated 7 chicago/hart's styles
- **2026-03-28**: Added div-007 for `interviewer` verb-form upstream typo (`"interview by"` → `"interviewed by"`)
- **2026-04-12**: Added div-008 for secondary sort key divergence (citeproc-js: given name; Citum: title) after shared family-name match
