# `select: first` and `online-access`: Cost/Benefit Review Before Release

- **Date:** 2026-09-18
- **Question:** Two declarative primitives — `select: first` (`docs/specs/GROUP_SELECT.md`) and `online-access` (`docs/specs/MEDIUM_DESIGNATOR.md`) — merged shortly before this review, together adding ~2,380 lines across the engine and schema. Before cutting a release: is that a good bet, or does the small number of real applications mean the implementation is flawed, or simply incomplete?

## Bottom line

**Neither.** `select: first` is not flawed and not incomplete relative to its own job — every real style it has now touched (five, three of them new since this review started) confirms it does exactly what it claims, and it found the same real, previously-undetected bug in three unrelated styles the moment anyone actually looked. `online-access` has reached its entire intended scope — three styles, confirmed exhaustively against the whole embedded corpus, not three out of some larger unexplored set. The low usage count on `select: first` reflects that nobody has pointed `citum-migrate` at it yet (tracked separately, `csl26-zyic`), not that the primitive itself has limited reach.

## What was actually found, in plain terms

`select: first` was applied to five real styles. Two were the original worked examples (T&F-CSE's publisher-place fallback, Chicago's broadcast-episode fallback — both byte-identical refactors, no bug). The other three — **Chicago's genre fallback, APA, Harvard (Cite Them Right), and IEEE** — were all new. Three of those four turned up the *same* real, previously undetected bug:

> A reference with **both** a DOI and a URL rendered **both**, concatenated — e.g. `https://doi.org/10.1234/example https://example.com/article` — where every one of these styles' real citation rules says DOI should win and the URL should be suppressed entirely.

This is not a rare edge case. Any modern journal article commonly carries both fields; APA and IEEE are two of the most widely used citation styles there are. Confirmed against a live citeproc-js engine for each style (not the fuzzy compatibility gate `oracle.js` normally uses), and against a real existing fixture entry (`TLIB-SEL-STANDARD-1`) that was *already* exhibiting the bug in APA's and Harvard's rendered output before the fix — this wasn't invented to justify the primitive, it was sitting there.

`select: first` fixed all three the same way, with the same three-line shape, and did not touch anything else. Full embedded-corpus exact-parity diffs (isolated before/after) show **zero regressions** across all runs.

## What `select: first` genuinely cannot do — and why that's not a gap

A separate, larger catalog of `render-when` uses (`csl26-zmxt`, completed 2026-09-15) found 26 places, across the Chicago family, where a field's mere *presence* routes an unrelated decision — an editor's grammatical form, a date's format, a number's punctuation — rather than choosing displayable content. `select: first` structurally cannot touch these; it chooses between candidate *content*, and none of these cases offer a content choice. That catalog is evidence of a different, harder, **not-yet-designed** problem (a possible future "work-form" concept), not a report card on `select: first`. Nobody asked for that feature before this review; it isn't blocking anything here.

## The cost, honestly

| Primitive | Total lines (commit) | Engine behavior | Schema/validation | Tests | Docs/beans | Style usage at merge |
|---|---|---|---|---|---|---|
| `select: first` | 1,293 (`706a6c0ca`) | ~560 (`sorting.rs`, `rendering/grouped/core.rs`, `values/mod.rs`) | ~130 | ~390 | ~250 | 1 style, 7 rows |
| `online-access` | 1,087 (`d383f95b1`) | ~235 (`template_policy.rs`, `values/date.rs`, `component_predicates.rs`) | ~290 | ~265 | ~320 | 3 styles |

Most of `select: first`'s engine cost is not the core idea (an ordered candidate list, first non-empty wins) — that part is small. It's making a *losing* candidate's side effects (disambiguation tracking, sort-key resolution, term-only-content suppression) not leak into the winner or the rest of the document. That cost is paid once, by the primitive, so every future usage — including the three found this review — gets it for free. The alternative was either extending `render-when` (rejected on principle by the project's own 2026-09-06 audit: procedural conditionals don't belong in the declarative template layer) or one-off hand-written Rust special cases per style, which don't generalize and give `citum-migrate` nothing to target when converting future styles automatically.

## Remaining known opportunity, not acted on here

`csl26-zyic` already lists other embedded/exemplar styles with grep-signature matches for the same DOI-preferred-else-URL idiom (`american-medical-association`, `gb-t-7714-2025-*`, `modern-language-association`, `royal-society-of-chemistry`, and others) that have **not** been individually confirmed against their real CSL source the way IEEE/APA/Harvard were in this review. Some fraction will be genuine matches; some may have their own secondary defects (title-case, quote-routing) the way ASME's original worked example did. Not claimed as fixed; flagged as the next place to look, at the pace of one style at a time with real verification, not a bulk sweep.

## Recommendation

Ship. Both primitives behave correctly everywhere they're used, `select: first` has now been proven on styles it had never touched at merge time (not just its own worked examples), and the two open follow-ons (`citum-migrate` emitting `select: first` automatically, and the `csl26-zmxt` work-form design) are already tracked as separate, unblocked work — holding the release on either would be delaying on unrelated, larger scope rather than on any defect found in what's shipping.

## Rejected alternatives

- **Hold the release until `citum-migrate` auto-emits `select: first`.** Rejected: that's `csl26-zyic`'s own, larger, separately-scoped project; nothing found in this review depends on it, and the primitive's correctness doesn't depend on how many styles use it yet.
- **Treat the 26-use `render-when` catalog as a blocker.** Rejected: it documents a different feature that doesn't exist yet and was never part of what's shipping in this release.
- **Judge the primitive by lines-of-code-per-usage at time of merge.** Rejected as the wrong metric on its own: usage count was gated by migrate-side automation not existing yet (a known, tracked gap), not by any limit in the primitive's applicable surface — which is exactly what widened, cleanly, the moment this review looked.

## Evidence appendix

- `select: first` spec: `docs/specs/GROUP_SELECT.md`; `online-access` spec: `docs/specs/MEDIUM_DESIGNATOR.md` v1.2 (its own "Known limitation" section, added this review cycle, documents why it cannot serve a fourth style).
- Real usages, in order: T&F-CSE publisher-place (`706a6c0ca`), Chicago broadcast-episode (`ee64866b7`), NLM/springer/T&F-CSE `online-access` (`d383f95b1`), Chicago genre fallback (`refactor(styles): broadcast episode select:first`, this session), APA DOI/URL (`fix(styles): apa doi-preferred-else-url fallback`), springer cited-date bracket fix-in-place (`fix(styles): springer cited-date bracket fixes` — investigated switching to `online-access`, rejected, fixed in place instead), Harvard and IEEE DOI/URL (this doc's own change).
- `csl26-zmxt`: full 26-use/13-shape policy-gate catalog, completed 2026-09-15.
- `csl26-zyic`: unconfirmed candidate list for further `select: first` DOI/URL migrations.
- Verification method for every style change referenced here: real citeproc-js ground truth (not `oracle.js`'s fuzzy compatibility gate) plus an isolated pre-change-vs-post-change `report-core.js --all-features` corpus diff, checked for zero exact-parity regressions.
