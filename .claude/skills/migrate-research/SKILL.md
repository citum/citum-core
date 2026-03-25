---
name: migrate-research
type: user-invocable, agent-invocable
description: >
  Autonomous research loop that improves citum-migrate converter fidelity by
  iterating on Rust code in crates/citum_migrate/src/. Measures fidelity across
  a corpus of priority styles, clusters failure patterns, attempts targeted Rust
  fixes, validates with cargo nextest, keeps improvements, reverts failures.
  Use whenever someone wants to improve bulk migration quality, fix converter
  bugs across multiple styles, or systematically close the gap between
  citum-migrate output and citeproc-js reference. Does NOT modify style YAML —
  that is style-migrate-enhance's domain.
model: sonnet
---

# Migrate Research

An autoresearch-inspired loop for improving the `citum-migrate` Rust binary.
Instead of fixing individual style YAML post-migration (what `style-evolve` does),
this skill iterates on the converter itself so that *all* future migrations
produce better output.

## Use This Skill When

- Systematic converter improvement is the goal (not one-off style fixes)
- Multiple styles share the same migration failure pattern
- You want to close the gap between `citum-migrate` output and reference fidelity
- After a `style-migrate-enhance` wave surfaced repeated `missing-feature` gaps
  that trace back to the converter

## Do Not Use When

- Fixing a single style's YAML → use `style-evolve upgrade`
- The failure is an engine rendering bug, not a converter bug → use Co-Evolution
- The failure requires new schema fields → file a bean, escalate to planner

## Entry Points

```
/migrate-research                         # Default: top-10 priority styles
/migrate-research --top 20                # Expand corpus
/migrate-research --styles "apa,nature"   # Specific styles
/migrate-research --resume                # Continue previous session
```

## Autonomous Operation

Run the full research loop without pausing for approval. The loop is:
**baseline → cluster → classify → hypothesize → fix → gate → measure → decide → log → repeat**.

Only interrupt for:
- `Cargo.toml` / `Cargo.lock` changes (confirm before touching)
- `git push origin main`

Everything else proceeds automatically. Report results at the end.

## Input Contract

- **Corpus**: style names or `--top N` (default: 10 from `docs/reference/STYLE_PRIORITY.md`)
- **Budget**: max iterations (default: 5 per session)
- **Skip list**: styles or failure patterns to ignore (optional)

## Session State

Each research session creates a `lab/` directory under the skill folder. This
directory is working scratch — do not commit it.

```
.claude/skills/migrate-research/lab/
  session-N/
    baseline.json       # Per-style oracle scores at session start
    clusters.md         # Failure patterns grouped by root cause
    attempts/
      attempt-1.md      # Hypothesis, changes, result, disposition
      attempt-2.md
      ...
    delta.json          # Final per-style before/after fidelity
    summary.md          # Session summary with recommendations
  INDEX.md              # Cross-session ledger
```

Add `lab/` to `.gitignore` if not already present.

## Workflow

### Step 0 — Resume check

If `--resume` or `lab/` exists with an incomplete session, read `INDEX.md` and
the latest session state. Present a summary and continue from the last attempt.

### Step 1 — Corpus selection

Select styles from priority list or user args. For each style, verify the legacy
CSL file exists in `styles-legacy/` (needed for oracle comparison).

```bash
# Verify submodules
ls styles-legacy/*.csl | head -5
```

### Step 2 — Baseline capture

Run the converter on each corpus style and measure fidelity:

```bash
# For each style in corpus:
cargo run --bin citum -- migrate styles-legacy/<name>.csl -o /tmp/migrate-test/<name>.yaml
node scripts/oracle.js styles-legacy/<name>.csl --json > lab/session-N/oracle-<name>.json
```

Aggregate into `baseline.json`: per-style citation and bibliography pass rates.

### Step 3 — Failure clustering

Read all oracle results. Group failures by **root cause**, not by style:

- "All styles missing `volume-pages` delimiter" → one cluster
- "Editor name-order wrong in 4 styles" → one cluster
- "DOI not suppressed in 3 styles" → one cluster

Write `clusters.md` with: cluster description, affected styles, example oracle
diff, and estimated scope (which converter module likely needs the fix).

### Step 4 — Classify each cluster

For each cluster, decide:

| Classification | Meaning | Action |
|---|---|---|
| `converter-gap` | The converter produces wrong/missing YAML for a CSL construct it should handle | Fix in this loop |
| `engine-gap` | The YAML would be correct but the renderer doesn't support it | File a bean, skip |
| `schema-gap` | The behavior requires schema fields that don't exist | File a bean, skip |

Read `docs/adjudication/DIVERGENCE_REGISTER.md` before classifying. If a cluster
matches a registered divergence, record `div-XXX` and exclude it from the fix queue.

Only `converter-gap` clusters are in scope. If zero converter-gaps remain, the
session terminates early — the converter is not the bottleneck.

### Step 5 — Pick the highest-impact cluster

Select the `converter-gap` that affects the most styles. Ties broken by priority
rank of affected styles.

### Step 6 — Hypothesize

State a one-line hypothesis before writing any Rust:

> "Hypothesis: the CSL `<text variable="volume"/>` inside `<group delimiter=", ">`
> is being migrated without preserving the parent group's delimiter, causing
> volume-pages to render as 'vol.pages' instead of 'vol., pages'."

This prevents blind trial-and-error. Record the hypothesis in the attempt file.

### Step 7 — Locate and fix

Use jCodeMunch to navigate the converter code:

```
get_repo_outline(repo: "local/citum-core")
search_symbols("migrate", repo: "local/citum-core")
get_symbol("<ConverterFunction>", repo: "local/citum-core")
```

Scope of changes: **only `crates/citum_migrate/src/`**. If the fix would require
engine or schema changes, re-classify the cluster and skip.

### Step 8 — Gate

Before measuring, the fix must pass the standard gate:

```bash
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run
```

If the gate fails, fix compilation/lint issues. If tests fail and the failure is
in the converter tests (expected — you changed behavior), update the test
expectations. If tests fail elsewhere, the fix has unintended side effects —
revert immediately.

### Step 9 — Measure

Re-run the converter + oracle on the full corpus:

```bash
# For each style in corpus:
cargo run --bin citum -- migrate styles-legacy/<name>.csl -o /tmp/migrate-test/<name>.yaml
node scripts/oracle.js styles-legacy/<name>.csl --json
```

Compare against baseline. Compute per-style and aggregate fidelity delta.

### Step 10 — Decide

| Outcome | Action |
|---|---|
| Corpus fidelity improved (any style gained, none regressed) | **Keep** — commit the fix |
| Net-zero (some gained, some regressed equally) | **Investigate** — isolate the regression before deciding |
| Corpus fidelity regressed | **Revert** — `git checkout -- crates/citum_migrate/` |

On **keep**: commit with conventional message:

```bash
git add crates/citum_migrate/ && git commit -m "fix(migrate): <description>

Corpus delta: +N scenarios across M styles.
Cluster: <cluster-name>"
```

On **revert**: record the attempt as `disposition: revert` in the attempt file.
Note what was learned — even failed experiments narrow the search space.

### Step 11 — Log and repeat

Update `lab/session-N/attempts/attempt-K.md` with:
- Hypothesis (from Step 6)
- Changes made (files, functions, lines)
- Gate result (pass/fail)
- Fidelity delta (per-style table)
- Disposition (keep/revert/investigate)
- Reflection: was the hypothesis confirmed? What was learned?

Return to Step 5 for the next cluster. Stop when:

## Convergence Rules

| Signal | Action |
|---|---|
| Corpus fidelity delta < 0.5% per iteration over last 2 attempts | Plateau — session complete |
| 3 consecutive reverts | Current approach exhausted — session complete |
| Budget exhausted (default: 5 iterations) | Session complete |
| Zero `converter-gap` clusters remain | Converter is not the bottleneck — session complete |
| Same converter module modified 3+ times without sustained gain | Try a different module or rethink the approach |

## Hard Gates

- **No fidelity regression on the corpus.** A fix that helps one style but
  breaks another is not acceptable without investigation.
- **No `Cargo.toml` changes without confirmation.** The converter fix should not
  require new dependencies.
- **No style YAML modifications.** This skill modifies the converter, not its
  output. Style cleanup is `style-migrate-enhance`'s job.
- **No `unwrap()` or `unsafe` in new Rust code.**
- **Divergence register is authoritative.** Do not try to "fix" a registered
  intentional divergence in the converter.

## Rust Fix Patterns

Common converter improvements and where to find them:

| Pattern | Likely location | jCodeMunch query |
|---|---|---|
| Missing YAML field from CSL attribute | `csl_to_citum.rs` or per-element handlers | `search_symbols("convert_text", repo: "local/citum-core")` |
| Wrong delimiter preservation | Group/layout conversion | `search_symbols("convert_group", repo: "local/citum-core")` |
| Missing type-variant generation | Type dispatch logic | `search_symbols("type_variant", repo: "local/citum-core")` |
| Options extraction gap | Global options converter | `search_symbols("convert_options", repo: "local/citum-core")` |
| Name-form handling | Name element conversion | `search_symbols("convert_name", repo: "local/citum-core")` |

## Output Contract

Every completed session delivers:

1. **Corpus delta table**: per-style citation and bibliography fidelity, before vs after
2. **Cluster table**: each failure cluster, classification, disposition (fixed/skipped/deferred)
3. **Attempt log**: per-iteration hypothesis, result, and reflection
4. **Beans filed**: IDs for any `engine-gap` or `schema-gap` items discovered
5. **Follow-up styles**: which styles should be re-run through `style-migrate-enhance`
   to benefit from the converter improvements
6. **Convergence reason**: why the session stopped

Format:

```
## Session Summary

Corpus: <N> styles | Budget: <M> iterations | Convergence: <reason>

### Corpus Delta
| Style | Citations (before) | Citations (after) | Bib (before) | Bib (after) |
|---|---|---|---|---|
| apa | 10/12 | 12/12 | 26/28 | 28/28 |
| ... | ... | ... | ... | ... |

### Clusters
| Cluster | Classification | Styles affected | Disposition | Commit |
|---|---|---|---|---|
| volume-pages delimiter | converter-gap | nature, cell, science | fixed | abc1234 |
| DOI suppression | engine-gap | elsevier-harvard, apa | bean: csl26-xxxx | — |

### Attempts
1. **volume-pages delimiter** — Hypothesis: group delimiter not preserved during
   `convert_group`. Result: confirmed, +4 scenarios. Disposition: keep.
2. ...

### Follow-up
Re-run `style-migrate-enhance` on: nature, cell, science (converter now handles
volume-pages correctly; fresh migration will produce better YAML baseline).
```

## Integration Points

| Skill | Relationship |
|---|---|
| `style-migrate-enhance` | Consumes converter improvements — re-run after a successful session |
| `style-maintain` | Handles post-migration YAML cleanup (different layer) |
| `style-evolve` | Router — can dispatch to this skill for `migrate` mode when converter gaps are the bottleneck |
| `migration-behavior-reporting` | Session results can feed into the migration behavior report |

## Oracle Routing

Same as `style-maintain` — only CSL-derived styles use `oracle.js`:

| `originKey` | Oracle |
|---|---|
| `csl-derived` | `node scripts/oracle.js styles-legacy/<name>.csl --json` |
| `biblatex-derived` | Not in scope for converter research (different origin) |
| `citum-native` | Not in scope (no CSL source to convert) |
