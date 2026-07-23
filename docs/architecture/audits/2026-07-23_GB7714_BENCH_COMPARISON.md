# gb7714-bench External Comparison — Findings & Recommendations

- **Date:** 2026-07-23
- **Bean:** `csl26-77cf`
- **Related:** [csl26-d3hs](../../../.beans/archive/csl26-d3hs--gbt-7714-numeric-18-pre-existing-unrelated-raw-fid.md) (numeric fidelity), [2026-07-22_GBT_DATE_ANNOTATION_FIDELITY.md](2026-07-22_GBT_DATE_ANNOTATION_FIDELITY.md)
- **External:** [YDX-2147483647/gb7714-bench](https://github.com/YDX-2147483647/gb7714-bench), citum processor PR [#25](https://github.com/YDX-2147483647/gb7714-bench/pull/25) (pinned `CITUM_VERSION=v0.77.0`)

## Purpose

`gb7714-bench` cross-compares ten GB/T 7714 reference engines (Zotero/citeproc-js,
BibTeX/BibLaTeX/Lua LaTeX packages, several Typst packages, Pandoc, and now Citum) on
a shared corpus, and publishes results at gb7714.zhtyp.art. This audit answers three
questions: (1) how does Citum `main` compare on that benchmark, given the PR is pinned
to the released `v0.77.0`; (2) how does that benchmark's methodology compare to our
own internal fidelity harness, and where any gap is a limitation of their benchmark
(2a) vs. ours (2b); (3) what should we do to look competitive.

## Methodology

`gb7714-bench`'s `/converge/` view has no single ground truth — it computes per-entry
Levenshtein-distance buckets (`exact`, `letterCaseOnly`, `<3`, `<10`, `<max`,
`disaster`) between a chosen reference combo and every other combo, matched
**positionally by array index**. Its `calcDistances` (`website/app/components/
StrDistance.tsx`) unconditionally runs entries through `normalizeResult`
(`website/app/lib/result_normalize.ts`) first — folding full-width `，：；`→ASCII,
CJK/Latin spacing, ZWSP/NBSP — so **the published leaderboard numbers are always
normalized**; the separate raw/normalize toggle only affects the line-by-line
`/compare/` diff view, not the headline stats.

To keep every engine's output on one aligned data revision (required — matching is
positional, not entry-ID-based) without running the LaTeX/Typst/Pandoc/Zotero
toolchain locally:

- Pulled the `target-out` artifact from the `citum` branch's own CI run
  ([30021055407](https://github.com/YDX-2147483647/gb7714-bench/actions/runs/30021055407),
  2026-07-23, success) — Citum `v0.77.0` + every competitor engine, one data revision.
- Checked out the `data/` submodule to that exact pinned revision
  (`42e5c083…`) locally.
- Built `citum` at `main` (`fb6ad60a`, 25 commits ahead of the `v0.77.0` tag) and
  reproduced `processors/citum.nu`'s pipeline by hand for all 4 sources Citum
  supports (`builtin.{bib,json}`, `better.{bib,json}`).
- Reused the benchmark's own `normalizeResult`/`calcDistances`/`countDistances`
  (transcribed faithfully into a standalone script — the website's Node 22+
  `vitest` toolchain wasn't runnable on this machine's Node 20) rather than
  reimplementing comparison logic. No entry-count mismatches were found across any
  candidate/reference pair (positional alignment is safe).

## Q1 — `main` vs. the pinned `v0.77.0`

Dramatic improvement, not incremental. On `builtin.json`/`better.json` vs. the
Zotero reference (normalized, matching the leaderboard's own metric):

| | exact | badly-wrong (`<max`+`disaster`) |
|---|---|---|
| `citum-0.77.0` (builtin.json) | 38/344 (11.0%) | 79/344 (23.0%) |
| `citum-main` (builtin.json) | 39/344 (11.3%) | **25/344 (7.3%)** |
| `citum-0.77.0` (better.json) | 38/344 (11.0%) | 85/344 (24.7%) |
| `citum-main` (better.json) | 39/344 (11.3%) | **31/344 (9.0%)** |

`v0.77.0` emits catastrophically garbled entries for a real subset of references —
fields concatenated with no separators or punctuation (e.g. `[85] 8Dunbar K
L，Mitchell D ARevealing nature's synthetic potential…2013[2013]473-487http://…`)
alongside consistent **ASCII** punctuation. `main` renders those same entries
cleanly (`[85] Dunbar K L，Mitchell D A. Revealing nature's synthetic
potential…[J/OL]. ACS Chemical Biology，2013，8（3）：473-487. http://…DOI:…`)
with correct **full-width** GB/T punctuation, tracking Zotero closely. Root cause:
`resolve_localized_type_variant` was missing a fallback tier for English-language
items whose type wasn't redefined in the style's `en` locale override — fixed by
`csl26-7hsx`, one of the 25 commits since the tag. The `<max`/`disaster` collapse
(23–25% → 7–9%) is entirely this fix landing.

**The pinned `v0.77.0` binary is not representative of what Citum can do today.**

## Q2 — how their benchmark compares to ours, and where gaps sit

Theirs: relative convergence across engines on a shared corpus, no gold answer,
already punctuation-width-tolerant at the leaderboard level. Ours
(`report-core.js`/`oracle.js`): absolute fidelity against a citeproc-js gold string
per style, with an explicit divergence-adjudication mechanism (`oracle-divergences.js`
+ `verification-policy.yaml`) for cases where citeproc-js itself is wrong relative to
the published standard (e.g. `div-011`, era/EDTF — Citum already confirmed correct
there against `original.toml`'s own worked examples).

**The gap that matters is (2b), but it splits into two separate findings — don't
conflate them:**

`report-core.js --style gb-t-7714-2025-numeric` currently reports `fidelityScore:
0.989` and the GB/T-corpus-scoped benchmark run at `193/203` (95.1%) — not the
"100% adjusted" recorded as the outcome of `csl26-d3hs`. That 10-entry gap remains
**unattributed**; it is a separate, still-open discrepancy from the finding below
(root-causing it is out of scope for this pass — flagged as a follow-up, not solved
here). What this pass *did* isolate is a defect our internal harness cannot see at
all, which the external benchmark surfaced instead:

### Finding: missing terminal period on most bibliography entries

The GB/T 7714—2025 standard's own worked examples (`data/GB-T_7714—2025.original.toml`,
extracted from the official PDF) end every entry in a period regardless of what the
last field is — including after a bare page-locator (`…2023:35.`) and after a URL/DOI
(`…029983.`). Real Zotero output in the CI artifact agrees (missing in 1/344 entries).
Citum — both `v0.77.0` and `main` — omits it whenever the entry has no `dimensions`,
`url`, `cstr`, or `doi` field to trigger a subsequent field's `prefix: '. '`:
**301/344 entries (87%) on `builtin.json`**. Confirmed on our own fixture
(`ITEM-2`, the Hawking book, `tests/fixtures/references-expanded.json`) rendered
directly via `citum render -s gb-t-7714-2025-numeric`: `…1988` with no period.

This single character costs an entry the `exact`/`letterCaseOnly` buckets the
leaderboard sorts on (`normalizeResult` doesn't touch trailing punctuation). The
counterfactual — append `.` to any `citum-main` entry lacking one, recompute vs.
Zotero. Checked against both style variants gb7714-bench tests; `extended` (CSL-M,
bilingual) is the fairer comparison since it's `gb-t-7714-2025-numeric.yaml`'s own
migration source (`adapted-by: citum-migrate`, `source.links[rel=template]`):

| source | reference | `citum-main` exact | `citum-main`+period exact |
|---|---|---|---|
| builtin.json | zotero-compliant | 39/344 (11.3%) | 272/344 (79.1%) |
| builtin.json | zotero-**extended** | 42/344 (12.2%) | **303/344 (88.1%)** |
| better.json | zotero-compliant | 39/344 (11.3%) | 263/344 (76.5%) |
| better.json | zotero-**extended** | 42/344 (12.2%) | **294/344 (85.5%)** |

**A one-line style fix is worth roughly 7× the leaderboard's exact-match rate**
against either reference.

Root cause is a config gap, not a template rewrite: `citum-schema-style`'s
`bibliography.options` already has a general, type-independent
`entry_suffix`/`entry_suffix_after_url`/`entry_suffix_after_doi` mechanism
(`crates/citum-schema-style/src/options/bibliography.rs:34-57`), used by other
embedded styles — it is simply unset on `gb-t-7714-2025-base.yaml`. Fix is
`bibliography.options.entry-suffix: '.'` with `entry-suffix-after-url: true` and
`entry-suffix-after-doi: true` (confirmed required by the URL/DOI examples above),
inherited by `-numeric`/`-author-date`/`-note`. ~43/344 entries already end
correctly; the existing `TerminalLink`-aware logic in
`crates/citum-engine/src/render/bibliography.rs:182-189` is designed to guard
against double-punctuating those — worth a direct check in the fix PR.

### Why our own fidelity number is blind to this, not why it's low

This is not what causes the 193/203 gap above — it's a defect our harness scores as
**passing**, which is worse in a different way. Checked the exact same Hawking-shaped
entry (`tests/fixtures/references-expanded.json` `ITEM-2`) directly in
`report-core.js`'s live output, cache cleared to rule out staleness
(`.oracle-cache/report-core` — the same class of bug `csl26-d3hs` found and partially
fixed for `oracle.js`'s dependency hashing, so ruling it out here mattered):

```
oracle: "…New York：Bantam Dell Publishing Group，1988"   (no period)
citum:  "…New York：Bantam Dell Publishing Group，1988"   (no period)
"match": true
```

Our local `tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl` oracle **shares** the
missing-period defect — it differs from whatever CSL upstream zotero-chinese/styles
currently ships (the one real Zotero/citeproc-js renders in the CI artifact, which
has the period). Citum matches a flawed gold string and scores as passing. **The
external benchmark surfaced a defect our own gold standard cannot see.**

This is fixture staleness, not a cache issue, and not a case for registering a new
oracle divergence — unlike `div-011`, here the oracle and the standard **agree**
(both want the period); only Citum's native rendering and our local oracle copy are
wrong. Needs its own investigation to confirm scope and refresh the fixture; flagged
as a separate bean rather than bundled into the period fix.

### Two smaller, real (2b) findings from the same corpus

- **`nocase` HTML markup leaking into plain-text bibliography output.** Titles
  authored with Djot `[text]{.nocase}` case-protection (`crates/citum-engine/src/
  render/rich_text.rs`) surface as literal `<span class="nocase">…</span>` /
  `<i>…</i>` in the `--json` `.text` field gb7714-bench's processor consumes —
  e.g. `[161] <span class="nocase">Library of Congress</span>[EB/OL]…` where
  Zotero renders `Library of Congress[EB/OL]…`. Confirmed present in the
  `v0.77.0` CI artifact itself (7 occurrences on `builtin.json`, not a local-build
  artifact) — longstanding, not a regression from the 25 commits since the tag.
  No existing bean; new finding.
- **CSTR identifier not omitted when redundant with the URL already shown** (e.g.
  `[144]` main appends `CSTR:35001.37.01.33142.20…` after a URL Zotero considers
  sufficient). Already tracked, unimplemented: `csl26-ia43` (draft).

### Not a defect — informs messaging, not code

Full-width vs. ASCII punctuation and CJK/Latin spacing differences are absorbed by
`normalizeResult` in the leaderboard's own headline stats (see Methodology) — Citum
being full-width-correct per GB/T costs nothing there. No upstream issue needed;
this is exactly what the benchmark's own normalization already handles correctly.

### Scope note: the `.bib` source path lags the `.json` path badly

Even with the period counterfactual applied, `builtin.bib`/`better.bib` exact-match
against Zotero stayed at 42–48/344 (12–14%), vs. 76–79% on the `.json` sources — the
`citum convert refs` BibLaTeX→YAML conversion step introduces its own, much larger
divergence unrelated to bibliography-template formatting. This is a distinct,
bigger-scope area (citum-migrate BibLaTeX field mapping) that deserves its own
investigation rather than folding into this pass.

## Q3 — Recommendation

1. **Fix the entry-suffix gap first** (`gb-t-7714-2025-base.yaml`) — highest
   leverage of anything found, ~7× the exact-match rate on the benchmark's own
   headline metric, and a two-line style change using an existing engine
   mechanism. Needs a fixture covering a "bare" entry (no url/cstr/doi) per this
   repo's native-fixture test rule.
2. **Fix the `nocase` plain-text leak** — real, visible, easy to spot-check.
3. **Land `csl26-ia43`** (CSTR/URL omission) — already scoped, just needs doing.
4. **Cut a release once (1)–(3) land, then ask PR #25 to bump
   `CITUM_VERSION`** — the pin must move *after* the period fix ships, or the
   public benchmark shows garble-fixed-but-still-no-period and undersells the
   actual improvement. Sequence matters here.
5. **Investigate the CSL-M oracle fixture staleness** and the **BibLaTeX
   conversion gap** as separate, follow-up beans — both real, both bigger than a
   single-session fix, neither blocking (1)–(4).

No 2a upstream `gb7714-bench` issue came out of this pass — see the explicit
call-out below rather than treating it as a settled recommendation.

## Verification

- `gh run download 30021055407 -n target-out` (gb7714-bench, `citum` branch) +
  `data/` submodule pinned to `42e5c083…` — competitor outputs, one revision.
- `cargo build --release --bin citum` at `fb6ad60a`; pipeline reproduced by hand
  per `processors/citum.nu`; no stderr, no line-count mismatches vs. any reference.
- Convergence buckets computed via a faithful standalone transcription of
  `website/app/lib/result_normalize.ts` + `website/app/components/StrDistance.tsx`
  (their `vitest` toolchain needs Node ≥21.7 for `util.styleText`; this machine
  runs Node 20.4.0 — transcription was necessary, not a shortcut).
- `citum render -b tests/fixtures/references-expanded.json -s
  gb-t-7714-2025-numeric -m bib --json -k ITEM-2`: reproduces the missing-period
  defect directly against our own fixture, independent of the gb7714-bench data.
- `node scripts/report-core.js --style gb-t-7714-2025-numeric` (cache cleared,
  ruling out the `csl26-d3hs`-class staleness bug): `fidelityScore: 0.989`,
  GB/T-corpus benchmark run `193/203`; the Hawking entry (`ITEM-2`) matches at
  `"match": true` with the period missing on both sides, confirming the harness
  is blind to the defect rather than reporting it as the cause of the 193/203 gap.

## Open follow-ups (not actioned this session)

Per session scope discipline, no style/engine code was changed — findings only,
pending direction on fix-PR scope (the oracle-fixture question in particular
touches shared test infrastructure, not just the GB/T styles). Beans to create:
entry-suffix fix (blocking release), `nocase` plain-text leak fix, CSL-M oracle
fixture staleness investigation, BibLaTeX conversion fidelity investigation.
