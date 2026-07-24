---
# csl26-gc43
title: Fix GB/T 7714 numeric bibliography defects exposed by oracle harness fix
status: completed
type: bug
priority: normal
tags:
    - gb-t
    - fidelity
    - punctuation
    - rendering
created_at: 2026-07-24T13:19:22Z
updated_at: 2026-07-24T14:26:22Z
---

Fixing the oracle harness bugs tracked in csl26-7jib (normalizeText stripping
terminal punctuation before comparison; oracle-fast.js never applying the
STRICT_BIBLIOGRAPHY_STYLES exact-match gate) exposed real GB/T 7714 numeric
bibliography rendering defects that were previously invisible to
report-core.js's fidelity score. `node scripts/report-core.js --style
gb-t-7714-2025-numeric` now reports 238/250 bibliography matches (was
reporting 247/250 pre-fix -- the extra 9 were false passes masked by the
harness bugs, not real).

14 entries currently fail with genuine text differences (not structural
issues). Representative examples from `oracleDetail`:

- `[39]` missing bracketed access-date `[2019]` before the URL.
- `[45]` wrong medium code (`[CP/OL]` vs correct `[CP/Apparatus]`) and a
  missing period before the URL.
- `[22]` missing spaces and wrong bracket form around `[Z/film]` (renders
  `[Film]` with no surrounding spacing).
- `[44]`, `[24]` missing parenthetical publication dates entirely.
- `[74]`, `[75]`, `[78]`, `[98]`, `[167]`, `[169]` -- citum renders a GB/T
  era/date annotation (e.g. `（民国三十六年）`, `印刷`, `（清同治四年）`) that the
  citeproc-js oracle does not. This may be a legitimate div-011-class
  oracle/citum divergence (see `docs/architecture/audits/` and
  `scripts/lib/oracle-divergences.js`'s `GBT_DATE_ANNOTATION_PATTERNS`) rather
  than a citum bug -- needs adjudication against the GB/T 7714-2025 standard's
  own worked examples before deciding whether to fix the renderer or register
  a new oracle divergence.
- `[79]` bracket-form mismatch: oracle `1936.` vs citum `[1936].`.
- `[111]` citum renders an extra `[1990]` bracket the oracle doesn't.
- `[178]` citum duplicates the CSTR identifier after the URL (related to,
  possibly the same defect as, the already-tracked `csl26-ia43`).

Scope: investigate each, fix genuine Citum rendering bugs, and for the
date-annotation cluster specifically, determine whether it's a new div-011-
style oracle divergence or a real defect before touching the renderer.

Resolved — see Summary of Changes below. Follow-up: csl26-h3yr (exotic-class
type-variant authoring gap).

## Summary of Changes

Investigated all 14 originally-reported entries plus the wider set exposed once the oracle harness bugs (csl26-7jib) were fixed:

- **Date-annotation cluster ([74],[75],[78],[98],[167],[169] + [79],[111])**: already covered by the registered `div-011`/div-015 divergence (`scripts/lib/oracle-divergences.js` `GBT_DATE_ANNOTATION_PATTERNS`, `verification-policy.yaml`'s `div-011` key — the key name is stale relative to the register's div-015 entry, but the masking logic is correct and active). Confirmed via `node scripts/oracle.js ... --json`: these entries show `match: true` after divergence adjustment. No action needed.
- **[178] CSTR duplicate identifier**: already tracked by `div-009` and follow-up bean `csl26-ia43`, blocked on reviewer confirmation of the URL-containment interpretation. Left as-is; this is the sole remaining failure in the style's gated 203-item native corpus (`tests/fixtures/test-items-library/gb-t-7714-2025.json`), confirmed via the `gbt-7714-2025-upstream` benchmark_run (202/203 adjusted). `gb-t-7714-2025-numeric` is not in `scripts/report-data/core-quality-baseline.json`, so this does not block `just check-core-quality` (verified green on main both before and after this change).
- **[39] missing `[2019]` access-date bracket** (entry-dictionary, no `issued`): real style-defect. `chapter,entry-dictionary,entry-encyclopedia`'s `date: issued` component had `fallback: []`, unlike its `book,thesis,map` sibling which already falls back to a bracketed `accessed` year. Added the same `accessed`-year fallback in both the top-level and en-locale copies of this type-variant in `gb-t-7714-2025-base.yaml`. Fixed.
- **[45] wrong medium code `[CP/OL]` vs `[CP/Apparatus]`, missing period before URL** (software, `platform: Apparatus`): two distinct bugs.
  1. Processor-defect: `Reference::medium()` (`crates/citum-schema-data/src/reference/accessors.rs`) had no arm for `ClassExtension::Software`, so the carrier message arg always fell back to the URL-presence check (`OL`). Added a `Software` arm reading `platform` verbatim (no case-folding, unlike the other classes' `normalize_genre_medium` — GB/T's oracle expects the carrier code as authored, e.g. `Apparatus`).
  2. Style-defect: the `software` type-variant's `date: accessed` component carried the joining `suffix: '. '`, which vanished when `accessed` was absent (as here). Moved the period to `variable: url`'s `prefix` instead, matching the pattern already used in `chapter,entry-dictionary,entry-encyclopedia`. Fixed.
- **Remaining 9 entries ([22] film, [23] broadcast, [24] interview, [20] legal-case, and the treaty/hearing/bill/regulation/legislation entries)**: verified via the conversion-layer pre-flight (`cargo run --bin citum -- convert refs ... --from csl-json`) that these convert correctly to `motion-picture`/`broadcast`/`interview`/`legal-case`/`treaty`/`hearing`/`regulation`/`statute` — the defect is that `gb-t-7714-2025-base.yaml` has no `type-variants` entry for any of these ref types, so they fall through to the under-punctuated default template. None of these 9 items are in the style's gated native corpus. This is a substantial authoring gap (8 new type-variants), scoped out of this bug-fix PR into follow-up bean `csl26-h3yr`.

Net result for `node scripts/report-core.js --style gb-t-7714-2025-numeric`: 240/250 (was 238/250), and the gated native-corpus benchmark run is unchanged at 202/203 adjusted (the only remaining gap, [178]/CSTR, was already out of scope before this PR).
