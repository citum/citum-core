---
# csl26-zmxt
title: Design work-form routing to replace render-when's structural-policy uses
status: todo
type: task
priority: normal
tags:
    - schema
    - engine
    - style
    - chicago
    - fidelity
created_at: 2026-09-06T21:58:21Z
updated_at: 2026-09-15T20:46:13Z
parent: csl26-40n4
---

Under csl26-40n4 (Chicago family substrate). The render-when disposition audit (docs/architecture/audits/2026-09-06_RENDER_WHEN_DISPOSITION.md) found 25 of render-when's 125 uses are structural policy gates -- the tested field (volume-or-issue, part-number-numeric, part-number-non-numeric, genre, title) never appears in the branch it guards, meaning it routes an unrelated component (editor form, container title, page prefix) based on a property of the reference. No declarative primitive covers this today; it's why render-when can't be removed, only frozen.

Forcing-case inventory (file:line in the audit's appendix): chicago-author-date-18th.yaml:457/465 (editor form by volume-or-issue), chicago-author-date-18th.yaml:426/489 (title routing by part-number-non-numeric), and the genre/title B-shape uses across chicago-notes-18th.yaml and chicago-shortened-notes-bibliography-core.yaml.

Likely home: docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md, or a new work-form concept alongside it. Related to csl26-x61x (Chicago volume/issue/series grammar).

## Todo
- [x] Enumerate the full 25-use B-shape set with rendered-content diffs (what actually differs between branches) -- done 2026-09-15, see update below
- [ ] Propose a declarative primitive (option, discriminator, or type-variant axis)
- [ ] Spec in docs/specs/ before implementation
- [ ] Once shipped, migrate render-when's structural-policy uses and deprecate render-when

## Update (2026-09-15): todo #1 done — full policy-gate enumeration with rendered diffs

Systematically re-derived the render-when inventory with a script (PyYAML + a line-tracking loader) instead of re-reading files by hand, classifying by the audit's own test: does the tested field render as a component inside its own `field-present` branch? Script + raw JSON lived in a scratchpad, not preserved — rewrite rather than look for it in a future session.

**Reconciliation with the 2026-09-06 audit's manual count:** 119 render-when blocks found (125 in the audit; -2 explained by this session's broadcast-episode migration, PR #1277, which removed one fallback pair from chicago-author-date-18th — the rest is unaccounted-for and small). Of those, 28 initially classified as policy-gate, but 2 (`chicago-author-date-18th.yaml:1083,1238`) turned out to be `modify.match` selectors on the `legal-case` type-variant — they reuse an *already-counted* render-when condition from the extended `book` template purely to locate which `title: primary` component gets `emph: true`, not an independent new gate. Excluding those: **26 genuine policy-gate uses**, against the audit's 25 — a difference of one, within the tolerance a manual audit vs. a script will produce. Two implementation bugs found and fixed along the way (both in the classification script, not the templates): `title:` components were checked by *value* (the `TitleType`, e.g. `primary`) instead of by *key presence*, and the injected line-tracking metadata was leaking into condition counts, both of which the spot-checks against the audit's own worked examples caught before trusting the output.

**The 26 reduce to 13 distinct shapes**, not 26 independent problems — most repeat near-identically across the Chicago-family sibling styles (`chicago-author-date-18th`, `chicago-notes-18th`, `chicago-shortened-notes-bibliography-core`, `taylor-and-francis-chicago-author-date-core` share large template blocks). Grouped below by field and shape, with a rendered-content diff for a representative of each group — real `citum render` output, not inferred from the YAML (synthetic references in the scratchpad; a couple of real fixtures used where available, noted per group).

| # | Field(s) | Shape | Instances (file:line) | Diff (representative) |
|---|---|---|---|---|
| A | `part-number-non-numeric` | Renders `title: primary` (plain, no volume-title branch) | author-date:426, notes:327, shortened:278, tf-core:116 | present → `... pt. The Sequel, 2020` includes `The Complete Works` as title; see group B for the paired volume-title interaction |
| B | `part-number-numeric` | Renders `title: primary`, nested inside its own volume-title-present/absent sub-branch (2 near-identical sub-shapes across sibling files) | author-date:430, tf-core:120 (simple); notes:333, notes:390, shortened:284, shortened:352 (nested) | `MV-PARTNUM-NUM` → `Smith, A. 2020. The Complete Works. pt. 2, 2020.`; `MV-PARTNUM-NUM-VOLTITLE` → `Smith, A. 2020. The Sequel. pt. 2, 2020.` |
| C | `volume-or-issue` | Editor's contributor **form** changes (`verb` vs. default) — the audit's own canonical example | author-date:465, tf-core:131 | `MV-VOLORISSUE` (bibliography) → `Smith, A. 2020. Book Title. Edited by B. Jones. Vol. 3.`; `MV-NOVOLORISSUE` → `Smith, A. 2020. Book Title. Edited by B. Jones.` (editor form is identical in this pair at the bibliography level — the routing is more visible at the citation level, see group E) |
| D | `volume-or-issue` | `pages` gets a `": "` prefix vs. none | author-date:1027, author-date:1366 | not independently re-rendered (identical block text, two type-variants in the same file) |
| E | `volume-or-issue` | Editor form change, visible at the **citation** level | notes: (paired with C's editor-form block, same condition, citation-side rendering) — `MV-VOLORISSUE` (citation) → `A. Smith (, vol. 3, Book Title (2020))`; `MV-NOVOLORISSUE` → `A. Smith (Book Title (2020))` | |
| F | `volume-or-issue` | Descriptive `genre`/`publisher`/`date` group wrapped in parens appears/disappears | notes:478 | `MV-VOLORISSUE` (citation) → `..., vol. 3, ...`; `MV-NOVOLORISSUE` → no such clause |
| G | `volume-or-issue` | `date: issued` form switches (`year` vs. `year-month-day`) — the "online-first article" case; comment at the site explains: no volume/issue yet means GB/T cites the full date instead of a bare year | gb-t-7714-2025-base:543 | `MV-VOLORISSUE` → `Smith A，2020. Book Title：第3卷[M].`; `MV-NOVOLORISSUE` → `Smith A，2020. Book Title[M].` (synthetic `book` type doesn't exercise the online-first date-form branch directly; the YAML's own comment documents the intent — a real `article-journal` fixture with/without volume would show the date-form switch itself) |
| H | `volume-title` | Big nested part-number/volume/collection-title descriptive block (publisher-place-style detail) | author-date:490, shortened:406, tf-core:156 | `MV-VOLTITLE` → `Smith, A. 2020. The Sequel. The Complete Works, 2020.` (this group's block did not fire visibly in this synthetic case — no `part-number`/`volume` set alongside `volume-title`; needs a follow-up ref with both) |
| I | `volume-title` | `date: issued` gets a `", "` prefix vs. not, paired with `part-number present + volume-title absent` (compound) | author-date:577 (present), author-date:583 (compound absent) | not independently re-rendered — complementary halves of one placement decision |
| J | `original-title` | Whole "originally published as ..." note clause appears | author-date:595 | `MV-ORIGTITLE` → `Smith, A. 2020. Translated Title. Originally published as Titre Original (Editions Originales).` |
| K | `collection-title` | Big nested volume/part-number/title descriptive block | notes:300, shortened:247 | `MV-COLLTITLE` (citation) → `A. Smith (Great Books Series, vol. 3, Great Books Series, Book Title (2020))` — note the duplicated `Great Books Series`, a pre-existing rendering artifact unrelated to this investigation, not something this session fixed |
| L | `collection-title` | Simpler `collection-title` + `issued` date group (different shape from K, same field, same file) | notes:428 | not independently re-rendered — same field, structurally distinct from K, confirming the field-level "mix of shapes" pattern extends within a single file, not just across styles |
| M | `genre` | `title: primary` quote-wrapping toggles (originally flagged rejecting from PR #1277) | shortened:135 | `[Cited...]`-unrelated; genre-present → title unquoted, genre-absent → title quoted; no other content differs |

**Excluded as not independent (`modify.match` selectors, not new gates):** `chicago-author-date-18th.yaml:1083` (references group A's condition), `:1238` (references group B's condition) — both are `legal-case`'s `extends: book` / `modify:` block picking which inherited `title: primary` variant gets `emph: true`.

**What this confirms for todo #2 (propose a declarative primitive):** the routing target varies per shape — contributor form (C), a number's prefix (D), a whole descriptive clause's presence (E/F/K), a date's form (G) or prefix (I), a title's emphasis (via the modify-selector cases) or quote-wrapping (M). A single new primitive analogous to `select: first` (an ordered candidate list) won't fit this — these are genuinely **routing decisions over unrelated components**, not fallback content choices. The `INPUT_REFERENCE_CLASS_DISCRIMINATOR.md`-adjacent "work-form concept" this bean already floats — an explicit, named property on the reference (e.g. "is this a titled volume, a numbered volume, or an unnumbered whole") that templates branch on directly, instead of re-deriving it from raw field presence at every site — is the shape the evidence points toward, not a rendering-primitive extension. That's a design question for whoever picks up todo #2, not decided here.
