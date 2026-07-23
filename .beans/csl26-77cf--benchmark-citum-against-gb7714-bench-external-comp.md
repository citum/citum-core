---
# csl26-77cf
title: Benchmark citum against gb7714-bench external comparison
status: in-progress
type: task
priority: high
created_at: 2026-07-23T18:28:04Z
updated_at: 2026-07-23T20:52:09Z
---

Compare citum main vs the gb7714-bench external convergence benchmark (YDX-2147483647/gb7714-bench, citum PR #25 pinned to v0.77.0). Determine convergence deltas vs competitor engines, classify divergences as benchmark-limitation (2a, candidate upstream issue) vs citum-limitation (2b, candidate fix), and recommend actions to look competitive. Produces a dated audit doc via PR, optional citum fixes, and draft upstream issues (filed only after explicit go-ahead).

## Plan (see /home/bruce/.claude/plans/on-the-gb-t-7714-iterative-pumpkin.md)

- [x] Step 1: download aligned competitor outputs from gb7714-bench CI run 30021055407 (citum branch, target-out artifact); pin data/ submodule to matching revision
- [x] Step 2: build citum-core HEAD, reproduce processors/citum.nu pipeline for all 4 supported sources -> citum-main outputs
- [x] Step 3: reuse normalizeResult + StrDistance to compute raw+normalized convergence buckets, citum-0.77.0 and citum-main vs references (incl. period counterfactual)
- [x] Step 4: adjudicate divergences (2a benchmark-limitation vs 2b citum-limitation) against GB/T standard text, not majority cluster -- no 2a found, all gaps traced to citum or our own test infra
- [x] Write dated audit doc docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md via PR
- [ ] File any 2b fixes as citum PR(s) with native fixtures
- [ ] Draft upstream issue(s) for 2a items; file with gh only after explicit user go-ahead
