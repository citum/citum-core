---
# csl26-77cf
title: Benchmark citum against gb7714-bench external comparison
status: completed
type: task
priority: high
created_at: 2026-07-23T18:28:04Z
updated_at: 2026-07-23T20:56:01Z
---

Compare citum main vs the gb7714-bench external convergence benchmark (YDX-2147483647/gb7714-bench, citum PR #25 pinned to v0.77.0). Determine convergence deltas vs competitor engines, classify divergences as benchmark-limitation (2a, candidate upstream issue) vs citum-limitation (2b, candidate fix), and recommend actions to look competitive. Produces a dated audit doc via PR, optional citum fixes, and draft upstream issues (filed only after explicit go-ahead).

## Plan (see /home/bruce/.claude/plans/on-the-gb-t-7714-iterative-pumpkin.md)

- [x] Step 1: download aligned competitor outputs from gb7714-bench CI run 30021055407 (citum branch, target-out artifact); pin data/ submodule to matching revision
- [x] Step 2: build citum-core HEAD, reproduce processors/citum.nu pipeline for all 4 supported sources -> citum-main outputs
- [x] Step 3: reuse normalizeResult + StrDistance to compute raw+normalized convergence buckets, citum-0.77.0 and citum-main vs references (incl. period counterfactual)
- [x] Step 4: adjudicate divergences (2a benchmark-limitation vs 2b citum-limitation) against GB/T standard text, not majority cluster -- no 2a found, all gaps traced to citum or our own test infra
- [x] Write dated audit doc docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md via PR
- [ ] File any 2b fixes as citum PR(s) with native fixtures
- [x] No 2a benchmark-side issues found -- every gap traced to citum or our own test infrastructure; nothing to draft/file

## Summary of Changes

Compared citum `main` (fb6ad60a) against the pinned `v0.77.0` used by gb7714-bench
PR #25, using the PR branch's own CI artifact (run 30021055407) for aligned
competitor outputs on one data revision, avoiding a local LaTeX/Typst/Pandoc/Zotero
toolchain run entirely.

Key findings (full detail + quantification in the audit doc):
- Q1: `main` already fixes a real garbling bug present in `v0.77.0`
  (`resolve_localized_type_variant` fallback gap, csl26-7hsx) -- badly-wrong
  entries drop ~23-25% -> ~7-9% vs Zotero.
- Q2: isolated a missing-terminal-period defect on ~87% of entries that our own
  internal fidelity oracle cannot see (local CSL-M fixture shares the defect).
  Counterfactual quantification: fixing it would raise exact-match-vs-Zotero from
  ~11-12% to ~76-88% depending on source/reference -- roughly 7x.
  Two smaller real (2b) findings also surfaced: `nocase` HTML leaking into
  plain-text output, and confirmation that `csl26-ia43` (CSTR/URL omission,
  already drafted) is real and worth prioritizing. The `.bib` source path
  (citum-migrate BibLaTeX conversion) lags the `.json` path badly -- separate,
  bigger-scope gap.
- Q3: no benchmark-side (2a) issue found -- gb7714-bench's methodology
  (positional matching, always-normalized leaderboard stats) held up under
  scrutiny. Recommended sequence: fix period -> fix nocase leak -> land
  csl26-ia43 -> cut a release -> ask PR #25 to bump the `CITUM_VERSION` pin.

No style/engine code changed this session (scope discipline -- the oracle-fixture
finding in particular touches shared test infrastructure, not just GB/T styles).
Filed PR #1088 for the audit doc; created 4 follow-up beans for the deferred work.
