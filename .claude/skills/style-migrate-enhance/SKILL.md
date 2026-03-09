---
name: style-migrate-enhance
type: agent-invocable
description: High-throughput migration waves converting priority parent CSL 1.0 styles to Citum with repeatable before/after metrics and migration-engine gap recommendations. Fidelity is the hard gate.
model: sonnet
---

# Style Migrate+Enhance

## Use This Skill When
- The task is portfolio migration (for example: next 5 or 10 styles).
- You need repeatable before/after/rerun metrics.
- You want concrete recommendations for `citum_migrate` improvements from observed gaps.

## Input Contract
- Legacy style path(s) under `styles-legacy/`.
- Target Citum style path(s) under `styles/`.
- Batch size and priority source (`docs/reference/STYLE_PRIORITY.md`, `docs/TIER_STATUS.md`).
- Optional target metric (for example: bibliography >= 24/28).

## Output Contract
- Updated style YAML file(s).
- Metrics table per style:
  - baseline (seeded)
  - enhanced (edited)
  - rerun (fresh `citum-migrate` for comparison)
- List of migration-pattern gaps and recommended converter/preset follow-up.

## Autonomous Operation

Run the full wave — seed, fix, verify, QA, commit — without pausing between styles.
Only interrupt for `Cargo.toml`/`Cargo.lock` changes or `git push origin main` (per CLAUDE.md).

Commit after each successfully QA'd style rather than batching everything:
```bash
git add -A && git commit -m "feat(styles): migrate <style-name>"
```

## Workflow
0. Read `docs/adjudication/DIVERGENCE_REGISTER.md` before baseline capture or
   oracle review. Do not spend migration-wave time trying to erase a registered
   intentional divergence.
1. Select next priority wave.
2. Seed with migration baseline (`scripts/prep-migration.sh` or `citum-migrate`).
3. Capture baseline metrics (`node scripts/report-core.js`, `node scripts/oracle.js ... --json`).
4. Run iterative style fixes with fidelity-first ordering.
5. Re-run migration for apples-to-apples comparison.
6. Commit each passing style; produce final metrics + follow-up recommendations.

## Hard Gates
- Never accept a fidelity regression.
- Never classify a registered divergence as a migration or engine bug without
  first updating adjudication.
- SQI is tie-breaker and optimization only.
- If iteration 1 bibliography is below 50%: log it, attempt up to 3 more fix passes before
  flagging in the final summary — do not stop mid-wave and wait for user input.
- After bounded retries with no progress: note it in the wave summary and move to the next
  style rather than halting the entire wave.

## Required Artifacts
- Iteration log (what changed, what improved, what remains).
- Final wave summary table.
- Suggested migration-engine improvements only when repeated across styles.

## Verification
- Structured oracle: `node scripts/oracle.js <legacy-style> --json`
- Core quality report: `node scripts/report-core.js`
- Optional full workflow impact: `./scripts/workflow-test.sh <legacy-style>`

## Related
- Public router: `../style-evolve/SKILL.md`
- QA gate: `../style-qa/SKILL.md`
