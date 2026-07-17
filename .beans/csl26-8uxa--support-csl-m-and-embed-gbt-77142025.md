---
# csl26-8uxa
title: Support CSL-M and embed GB/T 7714—2025
status: in-progress
type: feature
priority: high
tags:
    - migrate
    - multilingual
    - schema
    - style
created_at: 2026-07-15T12:24:38Z
updated_at: 2026-07-17T10:29:16Z
---

Implement the CSL-M migration and embedded GB/T 7714—2025 family approved from GitHub Discussion #828.

Specs:
- docs/specs/MULTILINGUAL.md
- docs/specs/REFERENCE_IDENTIFIERS.md
- docs/specs/TEMPLATE_V3.md

## Design Record

- The three public styles inherit from a hidden embedded family root; the
  numeric style is the canonical target for the short GB/T aliases.
- Standard-specific type/carrier labels are selected by one inherited,
  style-owned MF2 message with typed arguments. This does not introduce a
  generic literal component or general CSL-style condition language.
- Numeric and note bibliographies omit an unavailable publication date, as the
  source CSL-M styles do. Author-date citations retain the source style's
  explicit Chinese `无日期` and English `n.d.` terms.
- The pinned upstream fixture revision is the fidelity oracle. A draft PR is
  appropriate before the bibliography hard gate is met so schema choices and
  remaining data-preservation gaps can be reviewed early.

## Acceptance Criteria

- [x] Localized layouts select both structure and rendering locale.
- [x] Supplementary CSTR identifiers migrate and render through a typed map.
- [x] csl-legacy preserves ordered CSL-M layouts and citum-migrate emits them.
- [x] Three embedded GB/T styles share a hidden base.
- [x] Upstream fixtures retain source, revision, and license attribution.
- [ ] The numeric style reaches 100% fidelity (default corpus + upstream corpus) and clean SQI, and joins core-quality-baseline.json. Note and author-date are embedded but diagnostic-only (count_toward_fidelity: false), tracked in follow-up beans.
- [x] Schema generation, docs/beans hygiene, and just pre-commit pass.
- [x] PR checks pass.

## Re-scope (2026-07-16)

Gate narrowed to the numeric style: the gb7714-bench project tests numeric only, and its bench PR waits on this work. Baseline measured with the now-wired report-core gate: fidelity 0.494 (citations 21/21, bibliography 113/250 across default + upstream corpora), SQI 0.985. The 133 upstream failures cluster into ~12 root causes; tuning proceeds cluster-first (triage table below as waves land).

### Wave-1 triage (numeric, upstream corpus, 133 failures)

| Cluster | ~Entries | Kind | Fix |
|---|---|---|---|
| carrier-marker spurious `. ` before `[M]` | 25 | style | base.yaml: attach carrier marker to title, no delimiter |
| access/update date formats (`[2024-01-15]`, `（2023）`) | 17 | style/engine | numeric full-date for accessed/updated |
| cited pages missing (`：35`, also before URLs) | 19 | conversion | csl-legacy: parse note-field cheater syntax (`note: "page: 35"`) |
| et-al term language (`et al` vs `等`) | 9 | engine/config | localized layout term selection by item language |
| edition rendering (`版` duplication, ordinals, 7.4:*) | 8 | style/engine | edition suffix suppression |
| publisher place / dangling `，` when date missing (7.5.*) | 12 | style | suppression + place fallback |
| container editors + title sub-parts (8.3.2:*, 8.1:2) | 10 | data+style | book-chapter path, title suffixes |
| long tail (standards numbers, periodical runs, maps) | ~30 | mixed | re-triage after wave 1 |

Tool: `scripts/analyze-oracle-clusters.py` on `oracle.js --json` output.

### Wave-1 progress (2026-07-16)

Numeric upstream corpus: 70/203 → 101/203 so far (+ pending: cited-pages via Monograph.pages, translator 译 label).

- fixed: substitute chain pulled title into author slot (base.yaml, editor-only substitute) — +14
- fixed (oracle-side): scripts/locales-zh-CN.xml was missing; citeproc silently fell back to en-US terms (等→et al, 译→trans., ordinal 2nd). Vendored from CSL locales @17ee1a93; GB snapshots force-regenerated (staleness gap: csl26-fvpo) — +1 direct, unblocks term clusters
- fixed: engine now honors options.dates month: numeric → ISO-hyphenated dates for month-bearing forms — +13
- in progress: Monograph.pages for cited pages (GB 引文页码; biblatex pages-on-book prior art) — conversion + schema, delegated
- in progress: translator role label via zh-CN locale roles (译) replacing hardcoded trans. suffix in base.yaml
- divergence candidates: citeproc renders en ordinals for zh editions (2nd 版; standard writes 2版) — verify against upstream metadata.json before registering
- deferred (design needed): title sub-parts 第{n}卷 volume circumfix (CSL-M %s terms — likely locale-owned MF2 message with $number arg); edition 版-only-when-numeric conditional

### Wave-1 result (2026-07-16)

Landed: gate wiring + numeric-only re-scope, cluster analyzer, substitute fix, zh-CN oracle locale, engine numeric-month ISO dates, Monograph.pages (cited pages), zh role labels (译), delimiter-group restructure of journal/imprint tails.

Official gate after wave 1: fidelity 0.494 → 0.716; after wave 2: **0.808** (citations 21/21; combined bib 198/250; upstream corpus 154/203), SQI 0.985.

Wave-2 triage (74 remaining, by root cause):
- conversion gaps (~25): patent/standard/report number dropped; publisher-place lost when publisher absent (place lives inside Publisher struct); conference event→container-title; preprint (PP) structure; periodical whole-run entries
- container editors before container title in //-entries (~8)
- edition semantics (~7): 版 label only when numeric; citeproc en-ordinal quirks (2nd 版 / 5th editors) are divergence candidates — verify against upstream metadata.json first
- title sub-parts (~7): 第{n}卷 volume circumfix, volume-title, map scale — needs number-label design (CSL-M %s terms)
- misc: serial full-date issued, name particles (van der), CSTR dedupe, accessed-date conditionals

### Wave-2 result (2026-07-16, afternoon)

Numeric upstream corpus: 129/203 → **154/203**; all gates green (2031 Rust tests, 52 report tests).

- fixed: patent/standard/report variants now read typed number variables (patent-number, standard-number, report-number) — +8
- fixed: container-author falls back to editor/editorial-director/collection-editor via role-substitute; chapter head groups translator before // — +6
- fixed: SerialComponent.number mapped from legacy (eids render as ：147370) — +3
- fixed: place-only imprints preserved (publisher_from_parts helper, all conversion sites; Sonnet-delegated) — +6
- fixed: preprint class routed through the article,dataset variant + {PP} message arm — +2
- fixed (CI): report-core no longer processes hidden -base family roots (Fidelity Checks was erroring on gb-t-7714-2025-base)
- reverted: unconditional accessed date in book variant (GB shows accessed only when issued is missing — needs conditional design, folded into csl26-zmod)

Wave-3 is beaned: csl26-49sj (conditional number labels / divergences), csl26-zmod (structural long tail, ~20 entries).

### Wave-3 result (2026-07-17)

Numeric upstream corpus: 154/203 → **190/203**. Both wave-3 beans landed:

- **csl26-49sj** (conditional number labels): typed `when-numeric: <label-form>` gate on TemplateNumber, resolving locale-owned terms with CSL-M `%s`-circumfix support (see TEMPLATE_V3.md §2.4 for the design and the MF2-vs-typed-field boundary). Caught and corrected a mid-implementation mistake (hardcoded zh glyphs in the shared bilingual base) before it landed.
- **csl26-zmod** (structural long tail): dedicated periodical/graphic type-variants, rebuilt archive imprint, conference event-title wiring, patent application-number preference, name-particle/suffix fixes, container-title-short punctuation, conditional accessed-date/full-date handling (164→187/203); map/document edition+pages wiring closed the rest (187→190/203).

Official gate (report-core): numeric fidelity 0.845 (bib 208/250), citations 21/21. `just check-core-quality` clean across all 157 styles (no regressions); `just pre-commit` clean (2040/2040 tests).

Not yet 100%: 13 entries remain, split into a filed ordinal-number-form gap (csl26-g49a — `NumberForm::Ordinal` is schema-only, never implemented in the engine; verified this is NOT a citeproc/oracle divergence, both sides agree on term text) and 9 genuine structural gaps (csl26-ra71, wave-3 follow-up: container-volume-in-chapter, volume-title note append, circa dates, map scale/dimensions, CSTR dedupe, preprint version prefix).

Numeric stays tracked (`count_toward_fidelity: true, min_pass_rate: 1.0` in verification-policy.yaml, unchanged) but not yet baseline-gated in core-quality-baseline.json — matches the acceptance criterion's actual bar, not fudged to the achieved rate. The acceptance-criteria checkbox for 100% fidelity remains unchecked pending csl26-g49a + csl26-ra71.
