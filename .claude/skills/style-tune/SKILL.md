---
name: style-tune
type: agent-invocable
description: Iterative LLM hand-tuning loop for embedded-core styles. Drives a style to 100% fidelity and clean SQI. Both are hard gates. Seeded from migrate evidence, not from the converter output as a terminal deliverable.
model: sonnet
---

# Style Tune

Authoritative shared process docs:
- `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md` — tier definition, quality bar,
  failure classification
- `docs/guides/STYLE_WORKFLOW_EXECUTION.md` — the `tune` loop definition (seed →
  fidelity loop → SQI loop → QA), stop conditions, and shared escalation rules

## Use This Skill When
- The target is an `embedded-core` style (verified via `citum style list --source embedded`
  or by checking `crates/citum-schema-style/src/embedded/styles.rs`).
- The goal is **both** 100% oracle fidelity **and** clean SQI.
- Migrate output is available (or can be generated) as the starting seed.

## What This Skill Is NOT
- Not for long-tail or dependent styles (use `style-maintain` or `style-migrate-enhance`).
- Not a batch wave tool — one embedded style per run.
- Not a converter fix tool — `citum-migrate` issues are escalated separately.

## Input Contract
- Embedded style ID (e.g. `apa-7th`, `ieee`).
- Legacy CSL path in `styles-legacy/` for oracle comparison.
- Citum YAML path in `crates/citum-schema-style/embedded/styles/`.
- Authority basis: publisher guide or style manual (primary authority first).

## Hard Gates
- Fidelity: 100% oracle pass rate (`node scripts/oracle.js <legacy> --json`).
- SQI: clean score (`node scripts/report-core.js --style <name>`).
- A `tune` pass is not complete until **both** gates are green.
- Never accept a fidelity regression as a tradeoff for SQI improvement.

## Execution Loop

Follow the full `tune` loop from `docs/guides/STYLE_WORKFLOW_EXECUTION.md`:

1. **Seed** — run `citum-migrate` or accept the existing YAML. Record baseline
   oracle fidelity and SQI.
2. **Fidelity loop** — oracle → classify failure → smallest correct YAML fix →
   re-run. Repeat until 100%.
3. **SQI loop** (only after fidelity is green) — `report-core` → hoist/preset/prune
   → oracle re-check → repeat until clean.
4. **QA gate** — hand off to `../style-qa/SKILL.md` with `tier: embedded-core`.

## Failure Classification
Use the shared decision rules for all mismatches:
- `style-defect` → fix in YAML.
- `migration-artifact` → note gap, do not cycle YAML to compensate; fix the seed
  if a converter improvement is available, otherwise hand-author around it.
- `processor-defect` → escalate to Rust workflow; stop YAML iteration on that cluster.
- `intentional divergence` → record ID, exclude from counts.

## Stop Conditions
- Two distinct approaches fail on the same cluster → reclassify.
- Residual explained by a registered divergence → record, do not count.
- Residual is `processor-defect` → escalate; move on.
- Migrate cannot produce a usable seed → switch to pure `create` path (hand-author
  from guide evidence directly).

## Output Contract
Every completed tune pass delivers:
- embedded style ID and authority basis
- tier: `embedded-core`
- seed baseline: oracle fidelity %, SQI score
- final: oracle fidelity %, SQI score
- fidelity changes made (per mismatch cluster)
- SQI changes made (hoisting, presets, type-variant compression)
- residuals reclassified (processor-defect / divergence IDs)
- QA verdict
- commit SHA and message

## Verification
- Oracle: `node scripts/oracle.js styles-legacy/<name>.csl --json`
- SQI: `node scripts/report-core.js --style <name>`
- Render smoke-check: `cargo run --bin citum -- render refs -b tests/fixtures/references-expanded.json -s crates/citum-schema-style/embedded/styles/<name>.yaml`
- QA handoff: `../style-qa/SKILL.md` with `tier: embedded-core`
