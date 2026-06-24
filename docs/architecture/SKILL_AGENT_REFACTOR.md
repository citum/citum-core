# Skill and Agent Refactor (2026-02-21)

## Objective
Improve execution speed and reduce instruction drift by separating routing,
planning, implementation, QA, and PR workflow concerns.

The repo also mirrors the public skill surface with repo-owned skill folders under
`.skills/` and a local installer into `$CODEX_HOME/skills`.

## Problems Addressed
- Overloaded style workflow mixed routing, planning, build, and QA.
- Agent contracts overlapped and conflicted (planner code expectations vs
  no-code policy, planner verification requirements without shell tools).
- PR workflow expectations were implicit instead of codified.

## New Skill Topology
- `style-evolve` (router): human-facing entrypoint that routes tasks.
- `styleauthor` (legacy alias): forwards to `style-evolve`.
- `style-maintain`: single-style maintenance and focused fixes.
- `style-migrate-enhance`: batch migration waves with before/after/rerun
  metrics.
- `style-qa`: standardized fidelity/SQI/format gate.
- `pr-workflow-fast`: branch/PR process with change-type validation gates.

## Skill Distribution
- Public agent skills live in `.skills/`.
- Legacy `.codex/skills/` entries remain as compatibility shims for existing
  Codex installs.
- Internal role contracts live in `.codex/agents/`.
- `./scripts/install-skills.sh` refreshes symlinks into `$CODEX_HOME/skills`.
- `./scripts/codex` remains a fallback for direct role execution.

## Agent Role Purity
- `@dstyleplan`: deep research and architecture only.
- `@styleplan`: implementation planning and escalation boundaries only.
- `@styleauthor`: execution and verification only.

## PR Efficiency Pattern
1. Create narrow `codex/*` branch.
2. Apply minimal diff needed to pass objective and gates.
3. Run checks based on touched change type.
4. Open PR with concise evidence: scope, validation, risks, follow-ups.

## Codex Invocation Fallback
- Repo-local `.codex/agents/` stores the role contracts.
- The runnable repo wrapper is `./scripts/codex`.
- Do not treat `.codex/commands/` as a supported host loader in this repo.

## Expected Outcomes
- Lower token and iteration cost for common style tasks.
- Fewer planner/implementer handoff ambiguities.
- Faster, more consistent PRs with explicit quality evidence.

---

## Topology Update (2026-06-24): tune mode + embedded-core tier

Following the recognition of deterministic migration limits (see
`docs/architecture/MIGRATION_STRATEGY_ANALYSIS.md` and
`docs/specs/MIGRATE_FULL_FIRST_ARCHITECTURE.md`), the skill topology was
extended:

### Changes
- **`style-evolve`**: added a 4th public mode — `tune`. Routes to `.claude/skills/style-tune/`.
- **`.claude/skills/style-tune/`**: new sub-skill. Owns the iterative LLM
  hand-tuning loop for embedded-core styles (seed from migrate → fidelity loop →
  SQI loop → QA). Model: sonnet.
- **`style-qa`**: made tier-aware. SQI is a **hard gate for embedded-core styles**;
  advisory for dependent styles.
- **`style-migrate-enhance`**: migrate output repositioned as a **seed/evidence**
  source for embedded-core targets (not a terminal deliverable). Long-tail batch
  behavior unchanged.
- **Shared docs** (`STYLE_WORKFLOW_DECISION_RULES.md`,
  `STYLE_WORKFLOW_EXECUTION.md`): added a third classification axis (portfolio
  tier: `embedded-core` vs `dependent`), tier-dependent quality bar, and the
  `tune` execution loop definition.

### Rationale
The embedded portfolio (16 styles, baked into the binary) sets the
maintainability standard for the whole portfolio. Achieving perfect fidelity and
clean SQI for those styles requires iterative LLM authoring — the converter
cannot reliably reach that bar autonomously. The `tune` mode and the
`embedded-core` tier make this a first-class workflow rather than an implicit
expectation.
