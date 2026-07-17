---
# csl26-zmod
title: 'GB/T long-tail: periodicals, archives, conference containers, graphic'
status: completed
type: task
priority: normal
tags:
    - fidelity
    - migrate
    - style
created_at: 2026-07-16T15:52:20Z
updated_at: 2026-07-17T10:29:29Z
parent: csl26-8uxa
---

Remaining structural gaps for gb-t-7714-2025-numeric upstream corpus after wave 2 (~20 entries): whole-periodical entries (8.4.2:1-4) render via the flat default template; archival entries (8.12.3:1-4) need archive-location as title suffix and 收藏地：收藏者 imprint; conference proceedings container-title lost in conversion when only event-title present (8.6.3:1-3); graphic/audiovisual (8.11.3.2:2, 7.2.1:5) need [Z] marker and （create-date）[access-date] pattern; patent number source field mismatch (8.10.2:1-4, oracle uses call-number-ish application number); name particles (van der) dropped and Jr. period (8.5.3:10, 8.3.2:4); container-title-short strip-periods (7.1.3:2 Br Med J); serial issued full-date for online-first (8.5.1.1:7, 8.5.3:4); accessed-date conditional on missing issued (8.11.3.2:5, 8.13.3:3); CSTR tail dedupe (8.14.3:1).


## Session summary (2026-07-16)

Oracle score: **164/203 → 187/203** for `gb-t-7714-2025-numeric`. All fixes live in the shared hidden base (`crates/citum-schema-style/embedded/styles/gb-t-7714-2025-base.yaml`) plus supporting engine/conversion changes, so `-note` and `-author-date` inherit automatically. `gb-t-7714-2025-author-date` also improved as a side effect (7/51 → 18/48 on the same check-core-quality run).

### Fixed clusters (24/26 targeted entries)

- **Cluster 1 — periodicals**: added a dedicated `periodical:` type-variant (was falling through to the broken flat default). 3/4 fixed; `gbt7714.8.4.2:2` deferred (see below).
- **Cluster 2 — archives**: rewrote `manuscript,personal_communication` → `manuscript,personal_communication,pamphlet` (the `pamphlet` CSL-type override wasn't covered), moved `archive-location` into the title's `：`-suffix position, and built a proper `archive-place：archive，issued` imprint with accessed-year fallback. Also fixed a real bug in `csl-legacy`'s `handle_string_variable` that conflated `archive-place` and `archive-location` into the same field (dropping `archive-place` whenever `archive-location` was already set). All 4/4 fixed.
- **Cluster 3 — conference proceedings**: found and fixed a genuine engine gap — `event_title()`/`event_place()` in `citum-engine/src/values/variable.rs` had no match arm for `ClassExtension::CollectionComponent`, so paper-conference items with only `event-title` (no legacy `container-title`) always rendered empty. Added the missing arms plus a container/event coalesce group in the style. All 3/3 fixed.
- **Cluster 4 — patents**: `application_number` was hardcoded `None` in `legal.rs`; wired it from CSL `call-number`, and made the `patent-number` component prefer it over the granted number (falls back to granted number if absent). Also found `Patent` had no `pages` field at all (needed for `8.10.2:1`'s trailing `：8-9`) — added it end-to-end (struct, conversion, accessor). All 4/4 fixed.
- **Cluster 5 — name particles/suffix**: fixed a real deserialization bug — `csl_legacy::csl_json::Name`'s `dropping_particle`/`non_dropping_particle` fields had no `rename`/`alias` for the CSL-JSON-standard hyphenated spelling (`dropping-particle`), so any hyphenated particle silently failed to parse. Also added an opt-in `strip_periods` to name-suffix rendering (threaded through `NamesOverrides`/`NameFormatContext`) so GB/T's `container-author` doesn't carry a literal "Jr." period into a following list delimiter. Both fixed.
- **Cluster 6 — container-title-short strip-periods**: `strip-periods` was a dead/no-op field for `title:` components (never wired to any actual stripping). Rather than reuse the existing shared `Rendering::strip_periods` (which other, unrelated styles already had set on title components without it ever doing anything — activating it broke those styles by stripping legitimate periods from proper nouns like "Merriam-Webster.com"), added a dedicated `strip-periods-all` field scoped to `TemplateTitle` only. Both entries fixed with zero cross-style impact.
- **Cluster 7 — graphic/AV + conditional accessed-date**: added a `graphic:` type-variant; split `book,thesis,map,software` into `book,thesis,map` (accessed-year-only fallback via `date.fallback`, matching the CSL `date` macro) and a separate `software:` variant (full accessed-date, matching the CSL `creation-accessed-date` catch-all) — these two families need different accessed-date precision and were wrongly sharing one shape. Also fixed a real engine bug: `TemplateDate`'s `fallback` mechanism called a nested component's `.values()` directly, bypassing the generic wrap/prefix/suffix application layer, so a fallback date's own `wrap: brackets` was silently dropped. All 4/4 fixed.
- **Cluster 9 — serial full-date for online-first articles**: added a new `TemplateConditionField::VolumeOrIssue` and gated the `article-journal,article-magazine` container date between `form: year` (volume/issue present) and `form: year-month-day` (online-first, no volume/issue yet), matching the CSL source's own conditional. Both fixed.

### Deferred (documented, not fixed)

- **`gbt7714.8.4.2:2`** (periodical): oracle wants `1957—[1990]` for an open-ended serial date range where the closing year is stored in a repurposed `season` field. Narrow, single-item edge case; not chased further.
- **`gbt7714.8.14.3:1`** (CSTR tail dedupe, cluster 8): investigated within the 15-minute budget. This is a genuine oracle/Citum semantic mismatch, not a bug: the item only has Zotero's `tex.cstr` note convention (no real top-level `CSTR` field), which Citum's csl-legacy layer promotes to a first-class CSTR value (existing, deliberate feature) but citeproc-js (the oracle) has no concept of `tex.cstr` at all, so it never surfaces a competing value — even though the same string happens to appear embedded in the URL for this one item. Not fixed.

### Remaining ~13 failures — NOT in scope

All belong to the separate, concurrently in-progress bean `csl26-49sj` (conditional number-labels/edition/版/volume): ordinal editions ("5 editors" vs "5th editors"), missing volume/subtitle suffixes ("：第35卷", "：第1册"), map scale/dimensions, and arXiv version labels. Left untouched.

### Regression control

- Ran `just check-core-quality` (the project's official gate): **passed cleanly — 157 styles, fidelity=1.0 for all, warnings=0.**
- Additionally did a manual before/after diff across all 157 embedded styles via an isolated worktree (own `CARGO_TARGET_DIR`, to avoid a cache-corruption false-positive I hit and had to clean-rebuild past). One residual "regression" surfaced in `chicago-author-date-18th`/`taylor-and-francis-chicago-author-date`: the `archive-place` bug fix (see Cluster 2) now correctly renders previously-dropped archive-place text there too, but this interacts with a pre-existing, unrelated capitalization bug in that style ("interview" vs "Interview") to flip the fuzzy match-scorer's verdict on one entry. The underlying content is objectively *more* correct, not less; the official check-core-quality gate does not flag it. Left as-is — fixing the unrelated capitalization bug is out of scope.
- `just pre-commit` (fmt + clippy -D warnings + full `cargo nextest run`): **clean — 2040/2040 tests pass.**
- `just schema-gen`: ran; `docs/schemas/{bib,server,style}.json` updated with the new `Patent.pages`, `TemplateTitle.strip-periods-all`, and `TemplateConditionField::VolumeOrIssue` fields (all purely additive).

### Files touched (style)

- `crates/citum-schema-style/embedded/styles/gb-t-7714-2025-base.yaml` (all style fixes — public `-numeric`/`-note`/`-author-date` files untouched)

### Files touched (engine/schema, in support of the above)

- `crates/csl-legacy/src/csl_json.rs` — hyphenated particle serde rename/alias; archive-place vs archive-location split
- `crates/citum-schema-data/src/reference/conversion/legal.rs` — patent `application_number`/`pages` wiring
- `crates/citum-schema-data/src/reference/types/specialized.rs` — `Patent.pages` field
- `crates/citum-schema-data/src/reference/accessors.rs` — `pages()` arm for `Patent`
- `crates/citum-engine/src/values/variable.rs` — `event_title`/`event_place` arms for `CollectionComponent`
- `crates/citum-engine/src/values/number.rs` — patent-number prefers application number
- `crates/citum-engine/src/values/title.rs` — `strip_periods_all` wiring
- `crates/citum-engine/src/values/date.rs` — fallback components now apply their own wrap/prefix/suffix
- `crates/citum-engine/src/values/contributor/{names,mod,merged,substitute}.rs` — opt-in suffix period stripping
- `crates/citum-engine/src/processor/rendering/grouped/core.rs` — `VolumeOrIssue` condition field
- `crates/citum-engine/src/render/bibliography.rs` — reverted an over-broad full-width dangling-punctuation pattern that regressed other styles (kept the ASCII-only table as-is)
- `crates/citum-schema-style/src/template.rs` — `TemplateTitle.strip_periods_all`, `TemplateConditionField::VolumeOrIssue`
- `crates/citum-schema-style/src/style/diagnostics.rs` — allowlist entry for `strip-periods-all`
- `docs/schemas/{bib,server,style}.json` — regenerated

## Summary of Changes

Landed (see csl26-8uxa wave-3 notes for full per-cluster breakdown): dedicated periodical/graphic type-variants, archive imprint rebuild, conference event-title, patent application-number, name particles/suffixes, container-title-short punctuation, conditional accessed/full-date handling (164→187/203); one more conversion fix (map/document edition+pages wiring in from_document_ref) closed the rest (187→190/203). Remaining structural gaps filed as csl26-ra71.
