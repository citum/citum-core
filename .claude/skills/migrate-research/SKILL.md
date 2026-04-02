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
- A style-fidelity follow-up needs bounded rich-input evidence to determine
  whether the next pass belongs in migration, style, processor, or adjudication

## Do Not Use When

- Fixing a single style's YAML → use `style-evolve upgrade`
- The failure is an engine rendering bug, not a converter bug → use Co-Evolution
- The failure requires new schema fields → file a bean, escalate to planner
- The only new signal is a full supplemental corpus rerun with no bounded cluster,
  hypothesis, or classification → reduce the fixture first

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

## Operating Modes

### 1. Broad converter loop

Use the existing corpus-level autoresearch loop when the goal is to improve
`citum-migrate` across multiple styles at once.

### 2. Rich-input follow-up loop

Use this mode when a single style has an official supplemental benchmark and the
next pass needs tighter evidence selection before deciding whether the problem is
in migration, style YAML, processor behavior, or adjudication.

This mode is still process-first and migration-focused. It does not authorize
style editing inside `migrate-research`.

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
cargo run --bin citum-migrate -- styles-legacy/<name>.csl > /tmp/migrate-test/<name>.yaml
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
| `migration-artifact` | `citum-migrate` produced the wrong or incomplete style behavior | Fix in this loop |
| `style-defect` | The remaining mismatch belongs in hand-authored YAML, not migration | Route to `style-evolve upgrade` |
| `processor-defect` | The migrated YAML is reasonable, but engine rendering is wrong | File or update engine follow-up |
| `intentional divergence` | Citum should not chase citeproc parity for this case | Route to adjudication |

Read `docs/adjudication/DIVERGENCE_REGISTER.md` before classifying. If a cluster
matches a registered divergence, record `div-XXX` and exclude it from the fix queue.

Only `migration-artifact` clusters are implementation targets in this skill. If
zero `migration-artifact` clusters remain, the session terminates early — the
converter is not the bottleneck.

### Step 5 — Pick the highest-impact cluster

Select one bounded target cluster. In broad mode, choose the highest-impact
`migration-artifact`. In rich-input follow-up mode, choose exactly one
style-local cluster before any code edit.

Do not mix clusters in one pass.

### Step 5a — Evidence ladder for rich-input follow-up

When the style has official supplemental evidence, use this order:

1. Run the primary oracle hard gate.
2. Run the official style-scoped report with `report-core`.
3. Extract one reduced supplemental cluster before edits.
4. Use the full supplemental benchmark only as confirmation after the cluster rerun.

The official style report is the authority for whether a supplemental benchmark
is configured. The reduced cluster is the authority for fast iteration.

### Step 5b — Fixture minimization for rich-input follow-up

Before editing any migration code:

1. Select the cluster by explicit `--type` or `--ids`.
2. Run `scripts/extract-rich-benchmark-cluster.js`.
3. Confirm the reduced cluster reproduces the target mismatch.

If the extractor reports unresolved unmapped mismatch rows, keep the selected
cluster as the working fixture for that pass. Do not force further reduction.

Artifacts written by the extractor:

- `cluster-fixture.json`
- `cluster-before.json`
- `cluster-summary.json`

Preferred starting size: 1-5 bibliography rows.

Rich-input operator commands for this mode:

```bash
node scripts/oracle.js styles-legacy/<name>.csl --json > /tmp/<name>-primary.json
node scripts/report-core.js --style <name> > /tmp/<name>-report-before.json
cargo run --bin citum-migrate -- styles-legacy/<name>.csl > /tmp/<name>-migrated.yaml
node scripts/report-core.js --style <name> --style-file /tmp/<name>-migrated.yaml > /tmp/<name>-report-after.json
```

Important: do not use the legacy-style `oracle.js styles-legacy/<name>.csl ...` path
for a reduced-cluster "after" measurement when you need to validate the temporary
migrated style. That command can resolve to the checked-in YAML for known styles
and silently stop measuring the temporary migrated output.

Current limitation: `report-core --style-file` is trustworthy for temporary-style
loading, metadata, and quality metric evaluation, but the citeproc benchmark path
is not yet fully migrated-style-aware for every supplemental fixture shape. Use it
as the style-scoped evidence surface, but do not overclaim that every reduced-cluster
oracle command is automatically bound to the temporary migrated YAML.

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

If a migration-side change produces no delta in the reduced cluster and no delta
in `report-core --style-file`, stop classifying the cluster as migration-owned
and reroute the same bounded cluster to processor work.

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
cargo run --bin citum-migrate -- styles-legacy/<name>.csl > /tmp/migrate-test/<name>.yaml
node scripts/oracle.js styles-legacy/<name>.csl --json
```

Compare against baseline. Compute per-style and aggregate fidelity delta.

For rich-input follow-up mode, always record both:

- cluster before and after
- full supplemental benchmark before and after

Do not treat a full supplemental rerun as sufficient evidence if the reduced
cluster was not rechecked first.

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
| Zero `migration-artifact` clusters remain | Converter is not the bottleneck — session complete |
| Same converter module modified 3+ times without sustained gain | Try a different module or rethink the approach |

## Hard Gates

- **No fidelity regression on the corpus.** A fix that helps one style but
  breaks another is not acceptable without investigation.
- **No regression on the primary oracle hard gate.**
- **One bounded cluster per pass.**
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
4. **Beans filed**: IDs for any `processor-defect` or follow-up migration/style items discovered
5. **Follow-up styles**: which styles should be re-run through `style-migrate-enhance`
   to benefit from the converter improvements
6. **Convergence reason**: why the session stopped

For rich-input follow-up mode, also deliver:

7. **Target cluster**: one bounded cluster only
8. **Evidence ladder status**: primary oracle, official style report, reduced
   cluster rerun, full supplemental confirmation
9. **Per-pass accounting**: cluster before/after, full supplemental before/after,
   classification, and stop reason

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
| volume-pages delimiter | migration-artifact | nature, cell, science | fixed | abc1234 |
| DOI suppression | processor-defect | elsevier-harvard, apa | bean: csl26-xxxx | — |

### Attempts
1. **volume-pages delimiter** — Hypothesis: group delimiter not preserved during
   `convert_group`. Result: confirmed, +4 scenarios. Disposition: keep.
2. ...

### Follow-up
Re-run `style-migrate-enhance` on: nature, cell, science (converter now handles
volume-pages correctly; fresh migration will produce better YAML baseline).
```

### Rich-input follow-up template
```
Target style: <style>
Target cluster: <one cluster only>
Cluster selector: <type or ids>
Primary oracle before/after: <x/y -> x/y>
Official supplemental before/after: <m/n -> m/n>
Cluster before/after: <a/b -> c/d>
Classification: style-defect | migration-artifact | processor-defect | intentional divergence
Hypothesis: <one sentence>
Stop reason: landed | reroute | plateau | adjudication
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
