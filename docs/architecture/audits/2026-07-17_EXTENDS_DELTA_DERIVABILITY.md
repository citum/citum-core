# Extends-Delta Derivability — Measurement

- **Date:** 2026-07-17
- **Bean:** `csl26-7iiu` (follow-up from
  [2026-07-17_MIGRATION_APPROACH_STRATEGIC_REVIEW.md](2026-07-17_MIGRATION_APPROACH_STRATEGIC_REVIEW.md))
- **Question:** What fraction of independent styles can be expressed as a
  small `extends` delta over an already-registered parent at oracle fidelity
  ≥ their current standalone synthesized result?
- **Instrument:** `scripts/find-alias-candidates.js` (behavioral
  fingerprinting, new `band` column and `--include-registered` flag) +
  `scripts/measure-delta-derivability.js` (new; per-pair standalone vs
  `--family-candidate <target> --minimize-wrapper` comparison, scored the
  same way as `report-migrate-sqi.js`). Artifacts under
  `scripts/report-data/*2026-07-17*`.

## Band scan (full corpus, threshold 0.80, registered candidates included)

| Metric | Value |
|---|---|
| Population (independent styles in `styles-legacy/`) | 2,844 |
| Candidates with a ≥0.80 behavioral neighbor among registered targets | 2,685 (94%) |
| Alias band (≥0.98) | 1,674 (59%) |
| Near-clone band (0.80–0.98) | 1,011 (36%) |
| Checked-in `styles/*.yaml` (141 total) in the alias band | **102 (72%)** |

The alias-band figures are the headline of the scan: the behavioral-clone
mass in the ecosystem is far larger than the registry's current 165 entries,
and nearly three-quarters of the checked-in long-tail renders ≥0.98-similar
to another registered style on the fixture set.

## Delta sweeps

Verdict rule: `delta-expressible` iff the minimal wrapper's combined strict
fidelity ≥ the standalone synthesized baseline. Only the single best_target
per candidate was tried.

| | random-100 corpus | checked-in `styles/` corpus |
|---|---|---|
| Near-clone pairs considered | 38 | 28 |
| Delta-expressible | 6 (15.8%; 21.4% excl. errors) | 1 (3.6%) |
| Not delta-expressible | 22 | 20 |
| Errors (standalone baseline broken) | 10 (26%) | 7 (25%) |
| Mean (wrapper − standalone) fidelity | −14.9 | −26.0 |
| Winners' mean fidelity gain | **+12.2 pts** | **+31.8 pts** (n=1) |
| Wrapper size (median) | 205 bytes, 1 extra key | 205 bytes, 1 extra key |
| Targets that are embedded parents | — | 4 of 28 |

Notable winners: `harvard-coventry-university` 0.69 → 0.92 over
`new-harts-rules-author-date-space-publisher`;
`mhra-author-date-publisher-place` 0.61 → 0.92 over the same parent;
`bulletin-de-correspondance-hellenique` 0.69 → 0.85 over
`karger-journals-author-date`.

## Findings

1. **Delta derivation is a real but selective lever.** Naive
   single-best-target forcing already improves ~1 in 6 near-clones, and when
   it wins the gains are large (+12 to +32 points) with genuinely minimal
   wrappers (~205 bytes, exactly one extra top-level key). But the mean
   delta is strongly negative: the behaviorally-closest target is often the
   *wrong* parent (worst case 0.88-similar note style forced onto
   `chicago-notes-classic-no-url` → 0.03 fidelity). Target selection must be
   measured, not assumed — trying the top-k targets instead of only the best
   is the obvious refinement.
2. **The checked-in corpus resists sibling-deltas — curation holds up.**
   Only 1/28 checked-in styles improves by re-basing on its nearest
   behavioral sibling, and only 4/28 best-targets are embedded parents.
   The per-style effort already invested in `styles/` beats naive
   re-derivation; the consolidation opportunity there is *aliasing*
   (102/141 in the ≥0.98 band), not re-derivation.
3. **A quarter of near-clone standalone migrations are broken outright.**
   17 of 66 pairs errored because the *standalone synthesized baseline*
   fails in the processor (`template variant operation in
   bibliography.type-variants[…] matched no component` /
   `TemplateVariantAnchorNotFound`) — the delta path could not even be
   compared. This is a distinct, actionable converter bug cluster in
   type-variant anchor emission, and it corrupts every fidelity measurement
   that touches these styles.
4. **Caveats.** Similarity is fixture-bounded: best_target near-ties within
   a family are unstable (the known
   `annals-of-the-association-of-american-geographers` →
   `taylor-and-francis-chicago-author-date` alias scores 0.9938 here but
   with `american-sociological-association` as best target). The instrument
   is memory-hungry (~6 GB peak; one sweep was OOM-killed at concurrency 2
   inside a 6 GB scope) — run it sandboxed
   (`systemd-run --user --scope -p MemoryMax=6G`) at `--concurrency 1..2`
   until per-pair engine cleanup is added.

## Addendum (same day): tier-0 auto-aliasing — negative result

Executing `csl26-qe4e` tier 0 (auto-register pairs with exact-match rates
1.0/1.0) failed its verification gate, and the failure is the finding:

- The instrument's exact-match columns are computed on **normalized text**
  (markup stripped). An independent raw-output check (citations +
  bibliography, markup included, full expanded fixture set) found only
  **1 of 90** candidate pairs byte-identical — the other 89 differ in
  formatting-class output (e.g. `<i>` on container titles).
- The single raw-identical pair, `oscola-journal-abbreviations → oscola`,
  is itself unsafe: the variant's one extra `form="short"` is the
  journal-abbreviation behavior, which the fixture set never exercises
  (no abbreviation data on any reference item). Fixture-identical ≠
  behaviorally identical.

**Consequence:** there are currently **zero safely auto-registrable
aliases**. Automated aliasing requires (a) markup-aware raw matching in the
instrument, (b) fixture items that exercise abbreviation/short-form data,
and (c) a declared-variant sniff (name/metadata) before any pair skips
human review. The 1,674-candidate alias band remains real as a *screen*;
the automation bar was simply set by normalized similarity, which is too
weak. Folded into `csl26-qe4e` (revised tiers) and `csl26-10lt`
(instrument work).

## Decision: expand the instrument; do not change migrate routing yet

The measured profile (selective, high-value wins; negative mean under naive
targeting) does **not** justify routing migrate's family-candidate path
through delta derivation by default — the conditional follow-up in
`csl26-7iiu` is resolved negatively. The productive next steps, in order:

1. Fix the type-variants anchor bug cluster (`csl26-b0ud`) — it blocks a
   quarter of the measurement surface and affects ordinary migrations.
2. Add top-k target trial and an embedded-only target mode to the
   instrument, plus per-pair memory cleanup (`csl26-10lt`); re-measure.
3. Evaluate alias-band consolidation of the checked-in long-tail
   (`csl26-8x90`) — 102 candidates, human-reviewed, highest-leverage
   maintenance reduction available; surfaces in the compat inheritance view
   (`csl26-zik7`).
