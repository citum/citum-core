# Migrate Random-Sample Improvement Wave — Outcome

- **Date:** 2026-06-11
- **Epic:** `csl26-vmcr` (promote citum-migrate with random-sample fidelity metrics)
- **Baseline:** [2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md](2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md) — 43/100 styles at ≥90% combined strict fidelity (seed 20260610, strict `--force-migrate` oracle)
- **Quality bar:** 80/100 at ≥90%, no style class below 60% — **not met; publication deferred**

## What the wave shipped

| Change | PR | Effect |
|---|---|---|
| note-class citation repeat forms | merged pre-wave-end | part of 43 → 52 |
| strip suppressed variable poison | merged pre-wave-end | part of 43 → 52 |
| full variants in wrapper emission | merged pre-wave-end | part of 43 → 52 |
| measured inferred-vs-XML citation selection (`csl26-jav1`) | #907 | 52 → 53; note-class at-threshold 15.8% → 21.1%; removes the wrong-template failure mode permanently |
| C3: default-branch bibliography order + conditional leakage (`csl26-cxi9`) | #908 | probe styles flipped to ≥90% (`zeitschrift-fur-allgemeinmedizin` 52/58 → 58/58, `brazilian-journal-of-psychiatry` 47/58 → 53/58, estonian proceedings → 56/58, `scientia-iranica` → 54/58); sentinels hold |

The last measured full-corpus headline (before #908) was **53/100**. No full re-measure
was run after #908; based on the 24-style close-miss band composition, the
post-#908 headline is estimated near 60/100. Anyone needing the exact figure can run:

```bash
node scripts/report-migrate-sqi.js --corpus random --sample 100 --seed 20260610
```

## Why the wave stopped

1. **Arithmetic.** Flipping the *entire* 80–90% close-miss band (24 styles) only
   reaches 77/100. The bar additionally requires recovery from the 70–80% band
   and note-class lifting from ~21% to 60%+ at-threshold — multiple further waves.
2. **Economics.** Cost per point rose sharply across the wave: the early
   deterministic fixes delivered 43 → 52 cheaply; `csl26-jav1` (the most
   expensive item) delivered +1 overall. C3 was cheap *because* it was a single
   root-cause bug; the remaining clusters are not.
3. **Locus of the gaps.** The remaining failure clusters are increasingly
   engine-level (e.g. once-only variable consumption semantics, `csl26-y4o7`),
   not converter-level — consistent with the converter-plateau note in
   `crates/citum-migrate/CLAUDE.md`.

## Decision

- Stop LLM-driven improvement waves on the migrate fidelity number.
- Defer the public Migrate page and weekly scorecard (`csl26-rksq`,
  priority `deferred`) until the number is earned by ordinary engineering.
- Remaining levers stay in the backlog as normal bugs: `csl26-y4o7`
  (engine consumption semantics; fixing it also repairs checked-in styles),
  `csl26-21ep` (C5 physics compact form, low priority).
- The random-corpus scorecard mode and the seeded baseline remain the
  measurement instrument of record for any future attempt.

## Recorded idea: output-driven template synthesis

Surfaced while closing this wave: retire XML layout compilation entirely and
synthesize Citum templates by searching candidates against citeproc-js
reference output, using the in-process machinery this wave built (deno_core
reference rendering, citum-engine candidate rendering, oracle-mirroring
similarity scoring). Full write-up, prerequisites, and acceptance criteria:
standalone draft bean `csl26-aynr`.
