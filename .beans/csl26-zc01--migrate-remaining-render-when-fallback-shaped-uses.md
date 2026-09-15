---
# csl26-zc01
title: 'Migrate remaining render-when fallback-shaped uses to select: first (corpus sweep)'
status: todo
type: task
priority: normal
tags:
    - style
    - fidelity
    - render-when
created_at: 2026-09-10T11:16:04Z
updated_at: 2026-09-15T15:56:26Z
parent: csl26-ccdt
---

docs/architecture/audits/2026-09-06_RENDER_WHEN_DISPOSITION.md inventoried 49 'Fallback'-shaped render-when uses across the embedded style corpus (of 125 total; 123 in the Chicago family, 2 in gb-t-7714-2025-base) that are structurally expressible as select:first (docs/specs/GROUP_SELECT.md, shipped in csl26-tzs3/PR #1270). This is a full-corpus follow-on: after PR #1270/#1272 shipped the primitive and migrated the T&F-CSE publisher-place worked example (7 rows), a second stacked PR migrates ONE additional example style as a proof/demo (see that PR for the specific candidate and diff). The remaining ~48 fallback-shaped uses (nearly all Chicago-family) are NOT a mechanical find-and-replace: the audit found 108 of 125 uses are a MIX of fallback and policy-gate shapes per field, and at least one 'obvious' candidate (Chicago's volume-title, chicago-author-date-18th.yaml:416-425) turned out to have a hidden third interacting condition (part-number-non-numeric) that select:first cannot safely express (verified by hand-tracing its truth table -- one combination must render nothing, which select:first structurally can't do, since it always falls through to the next candidate). Each candidate needs the same careful truth-table verification before migrating, style by style. Policy-gate uses (25) and else-branches pairing with them are explicitly out of scope -- those need work-form routing design (csl26-zmxt), not select:first.

## Update (2026-09-10)

This PR ended up demonstrating two distinct outcomes, not one:

1. `chicago-notes-18th.yaml`'s broadcast identifier: a genuine `select: first` migration (the shared-tail trap, documented above already applies here too -- both original branches rendered `number`).
2. `chicago-shortened-notes-bibliography-core.yaml`'s author+title citation gate (`render-when: field-present/absent: author`): closes `csl26-x79y`. This one needed **no primitive at all** -- the field-absent branch's content (bare title) is a strict subset of the field-present branch's content (contributor + title), so deleting the render-when pair and merging into one unconditional group is correct on its own: the group's existing emptiness/delimiter-join semantics already degrade to bare title when `contributor: author`'s own substitution (editor, then translator) also comes up empty. This is a THIRD trap/non-trap category worth checking for on every remaining candidate before reaching for select:first: **is the absent-branch content already a strict subset of the present-branch content?** If so, the render-when pair can just be deleted, no select:first needed. Verified as a real bug fix (not just a refactor): chicago-shortened-notes-bibliography's exact-parity count moved 92->93/473 (chicago-18-base family aggregate 560->561/1629), zero regressions in the 35-style full corpus.

**New, NOT yet verified candidate found while investigating:** `chicago-author-date-18th.yaml:876-877,911-912` has a second `field-present/absent: author` pair, in the `interview:` type-variant. This looked superficially like the same bug but is NOT a strict-subset case -- the field-absent branch drops several fields the field-present branch has (archive-location, archive-collection, references, publisher, parent-monograph, event-place, raw-medium/dimensions detail) rather than being a subset of it. Needs its own truth-table verification before assuming either the select:first fix or the plain-merge fix applies; do not treat it as already covered by this bean's two examples above.

## Update (2026-09-15): genre-field candidates triaged

Checked every remaining `field-present: genre`/`field-absent: genre` pair before migrating any of them as a batch — genre was already flagged in the parent audit as a mixed field, and this confirms it:

- **Migrated** (`chicago-author-date-18th.yaml:636-652`, broadcast episode/genre): identical shape to the already-shipped `chicago-notes-18th` broadcast-episode demo. `select: first` between `variable: genre` and a new `message: term.episode` (mirroring the term this project already added for exactly this migration, per the comment at `embedded/locales/en-US.yaml:981-986`), with `number` hoisted out as a shared tail. Verified byte-identical against the real fixture (`ITEM-23`) and four synthetic genre×number combinations (present/present, present/absent, absent/present, absent/absent — the last confirming the engine's term-only-content suppression correctly drops the bare "Episode" term when nothing else in the group has data). Also checked the non-numeric-`number` edge case the real CSL macro (`identifier-number-bib`) gates on and this Citum implementation doesn't (`Episode Special` renders on both sides of the migration) — a pre-existing gap, unrelated to this refactor, not fixed here.
- **Rejected** (`chicago-notes-18th.yaml:661-685`, interview genre/by-line): tests `genre` but the present branch renders `raw-genre` (a different field) + "by INTERVIEWER"; the absent branch renders "interview by INTERVIEWER" with "interview" baked into the message text, no bare term available to hoist as a `select:first` fallback candidate the way `term.episode` was purpose-built for the broadcast case. Needs a `term.interview`-equivalent addition (a locale-authoring task, not a template restructure) before this can migrate safely.
- **Rejected** (`chicago-shortened-notes-bibliography-core.yaml:134-144`): tests `genre` but renders `title: primary` in both branches — the only difference is quote-wrapping. `genre` never appears rendered; this is a policy gate, not a fallback (per the parent audit's own classification test), squarely `csl26-zmxt` territory.

Both rejections left as-is; no new beans filed (tracked here and under `csl26-zmxt`'s policy-gate inventory).

## Update (2026-09-15): remaining single-field candidates triaged, none migrated

Continued the field-by-field triage after landing the broadcast/genre migration (see the update above). Checked every remaining candidate from the original safe-subset shortlist against its actual paired structure, not just its field name:

| Style | Lines | Field | Verdict | Why |
|---|---|---|---|---|
| chicago-author-date-18th | 566-569 | original-publisher | Reject — unpaired lone guard | `field-absent: original-title`, no sibling `field-present` branch; it's avoiding duplicate rendering with the big original-title block at 595-620, not a fallback pair. |
| chicago-author-date-18th | 570-573 | original-publisher-place | Reject — unpaired lone guard | Same shape, guards `field-absent: original-publisher`. |
| chicago-author-date-18th | 595-620 | original-title | Reject — entangled, unpaired | Single `field-present: original-title` gate on a whole multi-field composite block (CMOS 18 13.101 "originally published as" note); not select-first-shaped at all. |
| chicago-author-date-18th | 577-589 | issued (volume-title/part-number) | Reject — volume cluster | Two branches render *identical* content under non-overlapping conditions; belongs to the excluded volume-title/part-number-numeric compound-condition family (csl26-zmxt), not a distinct candidate. |
| chicago-author-date-18th | 321-329 | doi/url | **Reject — explicitly out of scope**, not just entangled | This is the real DOI-preferred-else-URL idiom (`csl26-zyic`'s exact pattern) and structurally clean (`select: first` between `variable: doi` and `variable: url`, each keeping its own prefix/suffix, no shared-tail issue). `csl26-zyic` explicitly excludes both Chicago styles from this idiom's corpus sweep "8 Chicago beans are in-flight, per the audit's own caution" — the same collision risk this bean's own scope decision already accepted for everything else. Do not migrate this one outside `csl26-zyic`'s own coordinated pass. |
| chicago-notes-18th | 441-490 | number-of-volumes / editor / volume-or-issue | Reject — entangled, 3-deep nesting | Compound work-form routing (number-of-volumes × volume-or-issue × editor), textbook `csl26-zmxt` territory. |
| chicago-notes-18th | 198-211 | title (article-magazine) | **Not attempted — plausible but unverified** | Present: article title alone. Absent: parent-serial + section + issued date (identify by periodical placement instead). No shared-tail issue, looks like a genuine substitute-fallback. Traced partway into the real CSL (`styles-legacy/chicago-notes.csl`'s `title-bib`/`title-note` macros) but didn't reach a full truth-table confirmation before time ran out. Worth a follow-up session with the same rigor as the broadcast migration (synthetic fixture renders across title-present/absent × section-present/absent). |
| chicago-notes-18th | 661-685, 702-734 | genre / title (interview) | Reject — entangled, asymmetric | Nested inside a much larger title-present/absent split for the `interview` type-variant; the two branches have genuinely different field sets (event-place/event-date/raw-medium appear only in one), not a simple substitute. |
| chicago-notes-18th | 848-886 | title (personal-communication) | Reject — not a strict subset, asymmetric | Absent branch has *more* detail fields (archive-location/collection/name, an extra year date) than the present branch's plain `archive` — same "looks like fallback, isn't" trap the parent audit already flagged for a near-identical interview pair. |

**Net result of this session's corpus-sweep work: one migration landed** (broadcast episode/genre, PR stacked on csl26-wt1u's fix), the rest of the shortlist rejected or deferred with reasons recorded here. This isn't a shortfall in effort — it's what the parent audit's own finding ("108 of 125 uses are a MIX of fallback and policy-gate shapes per field") predicts: once you actually open each site, the *shape* rarely matches the field-name grep that found it. The one clean win (article-magazine title, chicago-notes-18th:198-211) is worth a dedicated follow-up.
