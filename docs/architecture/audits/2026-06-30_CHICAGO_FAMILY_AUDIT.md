# Chicago Family Audit — Shared Components, Order Layers, Missing Facts

- **Date:** 2026-06-30
- **Bean:** `csl26-fr6f` (child of epic `csl26-40n4`)
- **Scope:** `chicago-author-date-18th`, `chicago-notes-18th`,
  `chicago-shortened-notes-bibliography(-core)`,
  `taylor-and-francis-chicago-author-date(-core)` — the four embedded Chicago
  18 variants in `crates/citum-schema-style/embedded/styles/`.
- **Trigger:** [PR #984](https://github.com/citum/citum-core/pull/984) tuned
  `chicago-author-date-18th` periodicals in isolation. Question raised: stop
  tuning one style at a time and instead build a shared Chicago substrate.
  This audit is the prerequisite classification pass before any substrate
  code change (epic `csl26-40n4`).

## Why this audit exists

CMOS 18 author-date and notes share rules for names, titles, source
components, publication facts, and source-type distinctions — but the
*rendered order* genuinely differs (author-date puts year immediately after
author; notes bibliography places date later; notes citations are a separate
note-citation grammar). Before introducing any shared YAML base, we need to
know precisely which rules are duplicated-but-identical (safe to share),
which are duplicated-but-order-different (must stay separate), and which
facts are simply absent from Citum's conversion/accessor layer (an engine/Rust
problem, not a YAML problem). This audit performs that classification.

Note on fidelity numbers: this audit does not re-run the oracle. Treat any
prior fidelity percentages in memory or chat history as stale; get current
numbers from `node scripts/report-core.js --styles
chicago-author-date-18th,chicago-notes-18th,chicago-shortened-notes-bibliography,taylor-and-francis-chicago-author-date`
before starting Children 2–5 of the epic.

## Current inheritance (verified against embedded YAML, 2026-06-30)

```
chicago-author-date-18th          extends: book
chicago-notes-18th                extends: dataset
chicago-shortened-notes-bibliography-core  extends: chicago-notes-18th
chicago-shortened-notes-bibliography       extends: chicago-shortened-notes-bibliography-core
taylor-and-francis-chicago-author-date-core extends: chicago-author-date-18th
taylor-and-francis-chicago-author-date      extends: taylor-and-francis-chicago-author-date-core
```

`chicago-author-date-18th` and `chicago-notes-18th` are **siblings with no
shared base** — both extend generic presets (`book`, `dataset`), not each
other or a common Chicago ancestor. T&F and shortened-notes are already
family-shaped (each is two layers deep off one of the two heads).

## Per-source-type comparison

Source types present as explicit `type-variants` keys, by variant. `core`
columns inherit the parent unless they override (`—` = inherits unchanged,
`(rm)` = explicit `remove`, `(ext)` = `extends` + modify):

| Type | author-date-18th | notes-18th | shortened-notes-core | T&F-core |
|---|---|---|---|---|
| article-journal | ✓ (bib only) | ✓ (full + subsequent) | ✓ | ✓ |
| article-magazine | ✓ | ✓ | — | — |
| article-newspaper | ✓ | ✓ | — | — |
| bill-proceeding | ✓ | — | — | — |
| bill-record | ✓ | — | — | — |
| book | ✓ | ✓ (full + subsequent) | ✓ | ✓ |
| broadcast | ✓ | ✓ | — | — |
| chapter | ✓ | ✓ (full + subsequent) | ✓ | ✓ |
| dataset | — (uses default template) | ✓ | — | — |
| entry-dictionary | ✓ | — | ✓ | — |
| entry-encyclopedia | — | ✓ | — | — |
| interview | — | ✓ | — | ✓ |
| legal-case | ✓ (ext: book) | ✓ | — | — |
| manuscript | ✓ | ✓ | ✓ | — |
| motion-picture | ✓ | ✓ | — | ✓ |
| paper-conference | — | ✓ (rm: genre/publisher/year group) | — | — |
| patent | ✓ | ✓ | ✓ | — |
| personal-communication / `_` | ✓ (empty) | ✓ | ✓ (empty ×2, both spellings) | — |
| report | ✓ (ext: book, rm translator) | ✓ (ext: dataset, rm pub/url) | — | — |
| standard | ✓ | — | ✓ | — |
| thesis | ✓ (ext: book, rm translator) | ✓ (ext: dataset, rm pub/url) | — | — |
| webpage | ✓ | ✓ (ext: dataset, rm publisher) | — | — |
| default / interview / motion-picture (T&F-only) | — | — | — | ✓ (own templates, sentence-case) |

Observations:
- 12 of ~20 source types appear in **both** author-date-18th and notes-18th
  with materially different component lists — not just reordering. E.g.
  `manuscript` in author-date-18th has no `archive-collection`; notes-18th's
  `manuscript` does. `personal-communication` differs: author-date suppresses
  it entirely (bibliography, by design — private communications aren't
  listed); notes-18th renders a full citation template with sender/recipient/
  dates.
- `report`/`thesis`/`webpage` use the `extends` + `remove` pattern in **both**
  heads independently, against *different* base types (`book` vs `dataset`)
  — same pattern, can't share the literal removal list because the bases
  differ.
- T&F overrides only 8 of author-date-18th's ~20 types (the high-volume ones:
  article-journal, book, chapter, default, interview, motion-picture) and
  leaves the rest inherited — meaning T&F currently has **no own answer** for
  bill-proceeding, bill-record, broadcast, legal-case, manuscript, patent,
  standard. Those silently fall through to `chicago-author-date-18th`'s
  CMOS-style (not T&F Style F) rendering. Worth flagging to Child 5 (final
  tuning) even though out of scope for the substrate itself.

## Classification: shared component vs order-layer vs missing fact

### A. Shared component candidates (safe for a hidden common base)

These are genuinely the same rule, expressed independently in two-plus
files today:

1. **Page-range format.** `page-range-format: chicago16` is set identically
   in `chicago-author-date-18th`, `chicago-notes-18th`, and
   `chicago-shortened-notes-bibliography-core`. T&F overrides to `expanded`
   (an intentional T&F Style F divergence) — confirms the other three should
   share a value, T&F should override it.
2. **`punctuation-in-quote: true`.** Identical across all three CMOS heads.
3. **`demote-non-dropping-particle: display-and-sort`.** Identical in
   author-date-18th and notes-18th and shortened-notes-core.
4. **`multilingual: romanized-translated`.** Identical across all four heads
   (T&F repeats it verbatim despite inheriting from author-date-18th already
   — currently redundant, a tell that it *should* live one level up only).
5. **DOI prefix convention** (`https://doi.org/` as a literal prefix string)
   — repeated verbatim in 6+ places across all four files (book, chapter,
   article-journal, standard, patent message patterns). A single shared
   `doi` component definition would remove this duplication and let T&F's
   variant (`". https://doi.org/" ... suffix: "."`) be the one deliberate
   override.
6. **`pattern.issued-date` / `pattern.patent-number` / `pattern.in-container`
   / `pattern.accessed-date` message templates** — these `message:` blocks
   are copy-pasted near-identically between author-date-18th, notes-18th,
   and shortened-notes-core (e.g. patent's issued-date group appears 3×
   with only minor wrap/prefix differences). Strong shared-component
   candidate.
7. **`personal-communication` bibliography suppression policy.** Both
   author-date-18th and shortened-notes-core render `personal-communication:
   []` (and shortened-notes-core duplicates it under both `personal-
   communication` and `personal_communication` spellings — itself a
   pre-existing inconsistency worth a one-line fix regardless of the
   substrate work). This is a deliberate Chicago-wide policy: private
   communications are cited in-text/in-notes only, never bibliographized.
   Worth centralizing as the suppression policy lives in 2 of 4 files only
   (notes-18th's citation-grammar `personal-communication` is the in-text
   *citation* form, a different concern — not a candidate for merge).

### B. Order-layer (style-specific, must NOT be merged)

1. **Date position relative to author.** Author-date: `date: issued, form:
   year` immediately follows the author group in nearly every type-variant
   (book, chapter, article-*, broadcast, standard, motion-picture). Notes:
   date is folded into a parenthetical publication-facts group that comes
   *after* title and container (`(Publisher, Year)` or `(Year)`), sometimes
   after a translator clause. This is the single biggest reason a literal
   shared bibliography template cannot work — confirmed structurally across
   every shared type (book, chapter, article-journal).
2. **Title wrap/case conventions.** Author-date: most primary titles are
   unwrapped/sentence content (book/chapter use bare `title: primary`);
   notes: many primary titles get `wrap: punctuation: quotes` even for
   monographs in `manuscript`. These follow each grammar's own house style,
   not a shared rule.
3. **Editor/translator clause placement and prefix wording.** Author-date
   uses `form: verb` + group constructs (`contributor: editor, form: verb`);
   notes uses `form: long` + explicit `", ed. "` / `", interview by "` /
   `", to "` literal-prefix phrasing characteristic of note prose. Same
   underlying fact (editor exists), different sentence grammar.
4. **citation grammar entirely.** Author-date's `citation:` block (parenthetical
   author-year) and notes'/shortened-notes' `citation:` block (footnote
   prose with `ibid:`/`subsequent:` state machines) share no structure at
   all and must stay fully separate — this was already correctly identified
   in the original proposal and is confirmed by reading both files in full.

### C. Missing facts (conversion/accessor gaps — Rust, not YAML)

Facts referenced by **at least one** variant's template but with no general
Citum schema/accessor support visible in the type-variants above, or present
in raw form only (e.g. via `variable:` with no semantic accessor):

1. **Archival correspondence facts** — `archive`, `archive-collection`,
   `archive-location`, `archive-name`, `archive-place` appear in
   `chicago-notes-18th`'s `manuscript`/`personal-communication` variants but
   *not* in `chicago-author-date-18th`'s `manuscript` (which has
   `archive-location` + a `group: [archive-name, archive-place]` only — no
   `archive-collection`). Bringing author-date's manuscript handling up to
   notes' richer set is a direct, low-risk win for Child 4.
2. **Recordings / broadcasts.** `broadcast` in both heads uses raw
   `variable: dimensions`, `variable: medium`, `variable: number` (episode)
   with no semantic distinction between a TV episode number and a journal
   issue number — both render through generic `number:`/`variable:`
   components. A typed "episode" or "broadcast facts" accessor would let
   both heads share one component definition (currently diverge only in
   prefix wording, which is itself evidence the underlying data model is
   identical).
3. **Original publication dates.** Only `chicago-notes-18th`'s `book`
   type-variant has `date: original-published` + `variable: original-
   publisher` + `variable: original-publisher-place`. `chicago-author-date-
   18th`'s `book` has no equivalent — CMOS 18 author-date *does* call for
   "orig. pub. 1950" style annotations for reprints; this is a real gap,
   not a deliberate omission. Tag: unblocks author-date-18th, T&F (inherits
   author-date's gap).
4. **Event dates** — not found as a distinct accessor in any of the four
   files; `interview`/`motion-picture`/`broadcast` all reuse `date: issued`
   for what CMOS calls the interview/broadcast/release date. No variant
   currently distinguishes "event date" from "issued date" as a named
   accessor. Affects notes-18th's `interview` (uses two `date: issued`
   entries back to back — for interview-date and publication-date — which
   is a workaround, not a real two-field model) and would benefit all four
   once a true event-date field exists.
5. **Note-derived roles** — `recipient` (used in notes-18th's
   `personal-communication`) and `interviewer` (notes-18th `interview`, T&F
   `interview`) are contributor roles only notes-18th and T&F currently
   reference; author-date-18th's own `personal-communication` is suppressed
   entirely so the role never renders there, but if a user requests an
   author-date personal-communication citation surface in the future, the
   role needs to already exist in the shared accessor layer. Low priority,
   noted for completeness.
6. **Patent number / issued-date message patterns** (`pattern.patent-number`,
   `pattern.issued-date`) are duplicated as inline `message:` blocks per
   file rather than backed by a shared accessor — listed here as much a
   conversion-layer concern as a YAML-duplication concern (see A.6); the
   *data* (patent number, dual month-day/year issued date) is present, only
   the rendering pattern is copy-pasted.

## Recommendation: where a hidden common base is safe

**Safe to introduce now** (Child 3, blocked by nothing structural): a
`chicago-18-base` hidden style carrying only Section A items — page-range
format, punctuation-in-quote, demote-non-dropping-particle, multilingual,
shared DOI/message-pattern components, and the personal-communication
bibliography-suppression policy. `chicago-author-date-18th` and
`chicago-notes-18th` would both `extends: chicago-18-base` (replacing their
current `book`/`dataset` extends — verify those presets aren't relied on for
anything beyond what's already explicit in each file; a quick `report-
core.js` diff before/after will catch any regression). T&F and
shortened-notes inherit the base transitively and keep their existing
override layers untouched.

**Not safe to merge:** any bibliography ordering, title-wrap convention, or
editor/translator clause — see Section B. Two order layers stay as they are
today (author-date head vs notes head); do not attempt a third "merged"
bibliography template.

**Biggest fidelity lever:** Section C (conversion/accessor facts),
specifically items 1 and 3 (archival fields, original-publication dates) —
both are small, well-scoped Rust additions that benefit author-date-18th
immediately and proportionally lift T&F (which inherits author-date's
gaps) and the rich Zotero bibliography benchmark
(`tests/fixtures/test-items-library/chicago-18th.json`) that already
exercises archival and reprint-style sources.

## Feeds into

- Child `csl26-8br0` (shared fixture): use the type list in the comparison
  table above to ensure the new fixture's reference set covers every type
  with a materially different component list, not just every type name.
- Child `csl26-zs0f` (component base): implement exactly Section A as
  `chicago-18-base`.
- Child `csl26-ifhx` (conversion facts): implement Section C items 1–6 in
  priority order as listed.
- Child `csl26-h7oc` (final tuning): use the T&F coverage gap noted under
  "Per-source-type comparison" (7 untouched types silently inheriting CMOS
  rendering instead of Style F) as a tuning checklist item.
