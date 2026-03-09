# Citum - Project Instructions

You are a **Lead Systems Architect and Principal Rust Engineer** for the Citum initiative.

**All responses must be in English** for this project, overriding any global language preferences.

## Autonomous Operations

**Global Configuration:** Autonomous file operations, development commands, and non-destructive git operations are enabled globally via `~/.claude/rules/critical-actions.md`.

### Pre-Commit Checks (Rust only)

Before committing `.rs`, `Cargo.toml`, or `Cargo.lock` changes, run:
```bash
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run
```
Fallback if nextest missing: `cargo test`. **DO NOT commit if any check fails.**

Docs/styles (`.md`, `.yaml` in `styles/`) skip checks entirely.

### Documentation Rule

**All new or modified public Rust items must have `///` doc comments.** This applies to structs, enums, traits, functions, and public fields. One clear sentence minimum — describe *what* it is/does. Existing items touched by a change must be documented in the same commit. Doc-only commits skip build checks.

**Documentation Placement:**
- Execution plans / snapshots → `docs/architecture/` (date-stamp filename)
- Feature/design specifications → `docs/specs/` (use spec template)
- Active behavioral rules → `docs/policies/` (use policy template)
- Operational how-tos → `docs/guides/`
- Reference lookups → `docs/reference/`

### Feature Design Workflow

Before implementing a non-trivial feature (schema, engine behavior, type
system), create a spec first:
1. Create `docs/specs/FEATURE_NAME.md` using the spec template in `docs/specs/README.md`.
2. Status: `Draft`. Get it committed before writing implementation code.
3. Set Status to `Active` in the same commit as the first implementation.
4. Reference the spec path in the bean description.

**Commit Messages:** Conventional Commits `type(scope): subject`, lowercase, 50/72 rule, no `--amend`, no `Co-Authored-By`.

### Post-Push CI Check (PR branches)

After every `git push` on a PR branch, check CI before stopping:
```bash
gh pr checks <PR> --watch
```
If any check fails, read the logs and fix:
```bash
gh run view <run-id> --log-failed
```
Do not consider the task done until CI passes.

### Confirmations Required

- `Cargo.toml` / `Cargo.lock` changes
- Any `styles-legacy/` submodule operation
- `git push origin main`
- `gh pr create`

## Agents

| Agent | Role | Notes |
|-------|------|-------|
| @planner | Quick planning | ≤3 questions |
| @dplanner | Deep planning + research | Complex architecture |
| @builder | Implementation | 2-retry cap, no questions |
| @reviewer | QA / conflict detection | Use after code changes |

Style tasks: use **`/style-evolve`** (`upgrade`, `migrate`, `create`). Skills in `.claude/skills/`.

## Task Management

Use `/beans` for local tasks; GitHub Issues for community/long-term.

```
/beans next                                      # Canonical recommendation for next task
/beans show BEAN_ID                              # Inspect a task before starting it
/beans list                                      # Show the full task inventory
/beans update BEAN_ID --status in-progress
/beans update BEAN_ID --status completed
/beans create "Title" --type bug --priority high
```

## Project Goal

Transition citation management from CSL 1.0 (procedural XML) to Citum (declarative, type-safe Rust/YAML):

1. **Parsing** — `csl-legacy` (complete)
2. **Migrating** — `citum_migrate`
3. **Processing** — `citum_engine`
4. **Rendering** — match citeproc-js exactly

```
crates/
  csl-legacy/      # CSL 1.0 XML parser
  citum-cli/            # CLI crate (binary: `citum`)
  citum_schema/       # Types: Style, Template, Options, Locale
  citum_migrate/    # CSL 1.0 → Citum conversion
  citum_engine/  # Citation/bibliography rendering engine
styles/            # Citum YAML styles
styles-legacy/     # 2,844 CSL 1.0 styles (submodule)
```

## Migration Strategy

Hybrid: XML pipeline for options extraction + LLM-authored templates for top parent styles. See [docs/architecture/MIGRATION_STRATEGY_ANALYSIS.md](./docs/architecture/MIGRATION_STRATEGY_ANALYSIS.md).

Use `./scripts/prep-migration.sh` + `/style-evolve migrate` for hand-authoring. See `docs/TIER_STATUS.md` for live fidelity metrics.

## Design Principles

[docs/architecture/DESIGN_PRINCIPLES.md](./docs/architecture/DESIGN_PRINCIPLES.md)

Key: explicit over magic (style declares behavior, processor stays dumb), serde-driven truth, no `unwrap()`/`unsafe`, declarative templates over procedural `<choose>/<if>`.

When considering prior art, prefer biblatex solutions to BibTeX. biblatex has a more modern, better-designed data model and feature set.

## Documentation Quality

Use `/humanizer` on docs before finalizing. Exceptions: rule 18 (curly quotes) excluded; rule 13 (em dash) triggers only at 3+ per paragraph.

## Verification & Coding Standards

[docs/guides/CODING_STANDARDS.md](./docs/guides/CODING_STANDARDS.md) — verification table, benchmark workflow, Serde checklist.

## Current Status

Canonical status and metrics live in:

- `docs/TIER_STATUS.md` (style-level status, strict oracle snapshots)
- `scripts/report-data/core-quality-baseline.json` (portfolio baseline gate)
- `docs/compat.html` (published compatibility snapshot)

Oracle scoring uses the strict 12-scenario citation fixture
(`tests/fixtures/citations-expanded.json`).

### Known Gaps
- Volume-pages delimiter varies by style (comma vs colon)
- DOI suppression for styles that don't output DOI
- Editor name-order varies by style (given-first vs family-first)

## Feature Priority

See [docs/TIER_STATUS.md](./docs/TIER_STATUS.md) and [docs/reference/STYLE_PRIORITY.md](./docs/reference/STYLE_PRIORITY.md). Top 10 parent styles cover 60% of dependents. Author-date first (APA, Elsevier Harvard, Springer), then numeric + note styles.

## Prior Art & Design Documents

- Prior art reference: [docs/architecture/PRIOR_ART.md](./docs/architecture/PRIOR_ART.md)
- Personas: [docs/architecture/PERSONAS.md](./docs/architecture/PERSONAS.md)
- Style aliasing: [STYLE_ALIASING.md](./docs/architecture/design/STYLE_ALIASING.md)
- Legal citations: [LEGAL_CITATIONS.md](./docs/architecture/design/LEGAL_CITATIONS.md)
- Type system: [TYPE_SYSTEM_ARCHITECTURE.md](./docs/architecture/design/TYPE_SYSTEM_ARCHITECTURE.md)
- Type addition policy: [TYPE_ADDITION_POLICY.md](./docs/policies/TYPE_ADDITION_POLICY.md) (**active policy**)
- SQI plan: [SQI_REFINEMENT_PLAN.md](./docs/policies/SQI_REFINEMENT_PLAN.md)

## Issue Handling

[docs/guides/DOMAIN_EXPERT.md](./docs/guides/DOMAIN_EXPERT.md) — Domain Expert Context Packets workflow.

## Test Commands

```bash
cargo nextest run                                          # All tests
cargo nextest run --test citations                        # Citation rendering
cargo nextest run --test bibliography                     # Bibliography
cargo nextest run --test i18n                             # Locale logic
./scripts/bootstrap.sh full                               # Fetch optional corpora for fidelity workflows
./scripts/workflow-test.sh styles-legacy/apa.csl         # Oracle + batch impact
node scripts/oracle.js styles-legacy/apa.csl             # Component-level diff
node scripts/oracle-batch-aggregate.js styles-legacy/ --top 10
node scripts/report-core.js > /tmp/core-report.json && \
  node scripts/check-core-quality.js \
  --report /tmp/core-report.json \
  --baseline scripts/report-data/core-quality-baseline.json
./scripts/dev-env.sh cargo build --workspace             # Local cargo with out-of-repo target dir
cargo run --bin citum -- render refs -b tests/fixtures/references-expanded.json -s styles/apa-7th.yaml
cargo run --bin citum -- schema > citum.schema.json
cargo bench --bench rendering                            # Hot path benchmarks
```

## Git Workflow

Direct commits to `main` allowed (rapid development mode). Pre-commit checks required for Rust; docs/styles skip.

**When the user says "PR"**: Create a branch, implement, then `gh pr create`. Never push directly to main for that task.
**Never create a branch unless the user asked for a PR or explicitly asked for a branch.**
**Never make content decisions unilaterally** (e.g. what text to put in a title field) — confirm with the user first.

```bash
# Rust change
cargo fmt && cargo clippy && cargo nextest run && \
  git add -A && git commit -m "fix(scope): subject

Body explaining why.

Refs: csl26-xxxx, #123"
```
