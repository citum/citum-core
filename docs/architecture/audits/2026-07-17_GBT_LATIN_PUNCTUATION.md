# GB/T 7714 Latin-Script Punctuation — Fix Record

- **Date:** 2026-07-17
- **Bean:** `csl26-fn9x`
- **Spec:** [docs/specs/MULTILINGUAL.md](../../specs/MULTILINGUAL.md) §3.2a (Active)

## Problem

`citum render refs --style gb-t-7714-2025-numeric -b tests/fixtures/test-items-library/gb-t-7714-2025.json`
rendered Latin-script references with full-width CJK punctuation, e.g.:

```
[2] Hawking S. A Brief History of Time[M]. New York：Bantam Dell Publishing Group，1988.
[3] LeCun Y，Bengio Y，Hinton G. Deep Learning[J/OL]. Nature，2015，521：436-444.
```

GB/T 7714 practice for a Latin-script reference uses Latin punctuation:
`New York: Bantam Dell Publishing Group, 1988.`

## Root cause

The CSL-M source style hardcodes full-width delimiters (`：，（）`) for every reference,
regardless of the individual item's own language. citeproc-js reproduces the same
hardcoded punctuation, so the fidelity gate — which uses citeproc-js as sole authority
(`verification-policy.yaml` → `gb-t-7714-2025-numeric`, `min_pass_rate: 1.0`) — was
satisfied by byte-parity while the output was non-conformant with the standard itself.

This is the same failure class as the tier-0 alias negative result (commit `558c67e2`,
"docs(migrate): record tier-0 alias negative result"): the verification proxy
(byte-parity against citeproc-js) is weaker than the real requirement (conformance to
the GB/T 7714 standard).

## Fix

An engine-level, opt-in script-aware punctuation remap:
`options.multilingual.scripts.latin.punctuation: latin` remaps a Latin-script item's
rendered full-width delimiters (`：，（）`) to Latin half-width equivalents (`: `, `, `,
`(`, `)`). CJK- and other-script items are unaffected. Full design, scope, and rationale:
[MULTILINGUAL.md §3.2a](../../specs/MULTILINGUAL.md#32a-script-aware-punctuation).

Enabled once, on the shared hidden family base
(`crates/citum-schema-style/embedded/styles/gb-t-7714-2025-base.yaml`), so all three
heads (numeric, author-date, note) inherit it — no per-head or per-type-variant
duplication.

Three insertion points in `citum-engine` were required, since full-width delimiters
enter rendered output from three independent places:

1. `render::component::render_component_with_format_and_renderer` — per-component value
   and literal `prefix`/`suffix`/`wrap`.
2. `render::citation::citation_to_string_with_format` — a citation section's own
   `wrap`/`prefix`/`suffix`/`delimiter` applied around its already-rendered components.
3. `processor::citation::apply_spec_wrap_and_affixes` — the outermost citation-spec
   `prefix`/`suffix` wrap for the whole citation cluster.

## Verification proxy: registered divergence

Latinizing Latin-script output makes Citum intentionally diverge from the citeproc-js
oracle snapshot (which still renders full-width, per the CSL-M source). Divergence
detectors in this repo are hand-written per-ID in `scripts/lib/oracle-divergences.js`
(div-004, div-005, div-008, div-009) — a `verification-policy.yaml` entry alone does not
mask anything. Added **div-010**: masks a citation/bibliography mismatch only when (a)
the item's effective language is positively Latin-script and (b) the delta is
punctuation-only after remapping full-width to Latin delimiters on the oracle side.
CJK items and non-punctuation deltas are never masked.

## Two pre-existing infrastructure bugs found and fixed

Verifying the fix surfaced two latent bugs in the divergence-adjustment machinery itself,
independent of GB/T. Both are fixed in this change because div-010 is otherwise inert:

1. **`buildAdjustedOracleResult` never recomputed the bibliography `passed`/`failed`
   aggregate.** It recomputed `citations.passed`/`.failed` from the adjusted entries, but
   for `bibliography` it only spread the raw `passed`/`failed` through unchanged — only the
   `entries` array reflected per-entry masking. This silently neutered every
   bibliography-scope divergence (div-009 already; now div-010 too) for any consumer
   reading the aggregate instead of walking entries by hand. Fixed in
   `scripts/lib/oracle-divergences.js` to mirror the citations recomputation exactly.
2. **`determineBenchmarkStatus` (the actual `min_pass_rate` gate) read raw counts, never
   `.adjusted`.** This meant *no* registered divergence (div-004 through div-010) had ever
   actually kept a style's gate green — they were purely reporting annotations. Fixed in
   `scripts/report-core.js` to prefer `oracleResult.adjusted?.bibliography`/`.citations`,
   falling back to raw when absent (e.g. `native-smoke` runs, which never set
   `min_pass_rate`). The fix is provably monotonic — adjusted match count is always ≥ raw
   match count — so it can only turn a `fail` into a `pass`, never the reverse; confirmed
   directly against all 5 styles currently using `min_pass_rate`
   (`gb-t-7714-2025-numeric`, `chicago-author-date-18th`, `chicago-notes-18th`,
   `chicago-shortened-notes-bibliography`, `taylor-and-francis-chicago-author-date`) with
   no unexpected status flips.

## Verification

- `citum render refs` output inspected directly for both the numeric bibliography and
  all three heads' citations (Latin items get Latin punctuation; CJK and the one
  Cyrillic (`ru-RU`) fixture item are unchanged).
- `cargo nextest run -p citum-engine -p citum-schema-style`: 1418 passed (includes 3 new
  behavior tests in `crates/citum-engine/tests/multilingual.rs`: Latin-script remap,
  CJK-script control, and the opt-in gate with the option absent).
- `node --test scripts/oracle.test.js scripts/report-core.test.js`: 85 passed (includes 5
  new div-010 unit tests — citation-scope mask, CJK-script non-mask, non-punctuation-delta
  non-mask, bibliography-scope mask, bibliography-aggregate recomputation — and 1 new
  `determineBenchmarkStatus`-prefers-adjusted test).
- `just pre-commit`: fmt, clippy (`-D warnings`), nextest all clean.
- **Direct before/after diff of `node scripts/oracle.js tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl`**
  against a clean baseline build (203 bibliography refs, 1 citation):

  | | Baseline | After fix |
  |---|---|---|
  | Raw bibliography match | 173/203 (85.2%) | 146/203 (71.9%) |
  | Unmasked (adjusted) failures | 29 | **18** |

  All 27 *new* raw deltas introduced by the punctuation remap are fully explained and
  masked by div-010/div-009 — **zero unmasked regressions**. The adjusted failure count
  strictly *improved* (29 → 18): several items that failed at baseline for a combination
  of a punctuation delta plus something else now pass entirely.

  **The `min_pass_rate: 1.0` gate was already failing before this change** (85.2% raw,
  85.7% adjusted — nowhere near 100%) and remains `fail` after it, for 18 pre-existing,
  unrelated reasons (e.g. `gbt7714.7.1.3:2`, an anonymous-author periodical entry missing
  its year and issue components entirely — a substitution/template gap, not a punctuation
  issue). Fixing those is out of scope here; filed as follow-up bean `csl26-d3hs`.

## Follow-ups (not in scope here)

- **18 pre-existing, unrelated raw fidelity failures** in `gb-t-7714-2025-numeric`
  (missing components, ordering differences on anonymous-author entries — see table
  above) keep the style's `min_pass_rate: 1.0` gate at `fail` independent of this change.
  Tracked in follow-up bean `csl26-d3hs`.
- Cyrillic (and other non-Latin, non-CJK script) items currently keep full-width
  punctuation; extending positive-evidence coverage to those scripts is a separate,
  narrower follow-up if it proves to matter in practice.
- Compound citations mixing Latin- and CJK-script items under one shared citation-spec
  wrap are not handled distinctly — the first item's language stands in for the whole
  cluster (see spec §3.2a). Not exercised by the current GB/T fixture set.
- `docs/adjudication/DIVERGENCE_REGISTER.md` maintains its own, unrelated `div-NNN`
  sequence (currently through div-013) for documented rendering-philosophy divergences,
  entirely independent of `scripts/report-data/verification-policy.yaml`'s `div-NNN`
  sequence (004, 005, 008, 009, now 010) for oracle-adjustment masking — the two already
  collide in number for unrelated entries (e.g. both have a "div-009"). Reconciling the
  two namespaces is a pre-existing documentation-debt item, out of scope here.
