---
name: style-evolve
type: user-invocable, agent-invocable
description: Single human-facing command for all Citum style work. Use whenever someone
  asks to fix, improve, convert, or create a citation style — even if they don't say
  "style-evolve". Routes to upgrade (fix existing Citum style), migrate (CSL 1.0 to
  Citum), or create (new style from scratch). Always use this rather than calling
  style-maintain or style-migrate-enhance directly.
model: sonnet
routes-to: style-maintain, style-migrate-enhance, style-qa
---

# Style Evolve

## Human UX (Public Entry Point)

```
/style-evolve upgrade <style-path>    # fix or improve an existing Citum style
/style-evolve migrate <csl-path>      # convert CSL 1.0 → Citum
/style-evolve create                  # new style from scratch or spec (see below)
```

Do not ask users to call internal skills directly.

## Autonomous Operation

Run the full pipeline without stopping to ask questions or seek approval between steps.
The user's goal is hands-off execution — they want a result, not a dialogue.

**Only interrupt for these explicit permission gates (per CLAUDE.md):**
- `Cargo.toml` / `Cargo.lock` changes (confirm before touching)
- `styles-legacy/` submodule operations
- `git push origin main`
- `gh pr create`

Everything else — reading files, running oracle/QA scripts, editing YAML, committing
the result — proceeds automatically. When in doubt, proceed and report at the end.

After QA passes, commit automatically:
```bash
git add -A && git commit -m "fix(styles): <style-name> <brief-description>"
```

Deliver the output contract as a final summary. Do not ask "what should I do next?"

## Mode Disambiguation

| Situation | Mode |
|---|---|
| Existing Citum style has a bug or formatting issue | `upgrade` |
| A CSL 1.0 `.csl` file exists and needs converting | `migrate` |
| No existing style; building from a spec or samples | `create` |

**Infer mode from context — do not ask if you can determine it:**
- Path ends in `.csl` or is under `styles-legacy/` → `migrate`
- Path is under `styles/` or the style already exists as Citum YAML → `upgrade`
- No path given, but a spec or sample is provided → `create`

Only ask if the mode is genuinely ambiguous after checking the filesystem.

## Modes

### 1. upgrade
Route to `../style-maintain/SKILL.md`.

### 2. migrate
Route to `../style-migrate-enhance/SKILL.md`.

### 3. create *(aspirational — not yet fully supported)*
Build a new Citum style from source evidence. Escalate to `@dplanner` for design.
Accepted source hints: `--source-url`, `--source-text`, `--source-issue`, `--source-file`.

## Co-Evolution Rule (Mandatory)

Every style iteration must also assess processor/preset opportunities — not just fix
the style. This is not a checkbox; it requires a delivered artifact.

At the end of every task, produce this table:

```
## Code Opportunities

| Description | Type | Action |
|---|---|---|
| <what was observed> | preset / missing-feature / processor-defect | implemented / deferred: <reason> |
```

**Types:**
- **preset** — a pattern repeated across styles that could become a shared preset
- **missing-feature** — behavior the processor can't express; requires engine work
- **processor-defect** — incorrect output from a valid template; requires an engine fix

**Rules:**
- Every task must include this table, even when empty (write "no opportunities
  observed this iteration" as the only row).
- `deferred` requires a **bean ID** — vague rationales like "only one style needs
  this" are not acceptable. File the bean, get the ID, reference it here.
- Before deferring a `processor-defect` or `missing-feature`: use jCodeMunch to
  locate the relevant engine code and assess if a fix is tractable this session.
  If tractable (~30 lines of Rust), implement it now. If not, file the bean first.
- A missing or empty table means the task is **incomplete**.

## Authority Hierarchy

Do not assume legacy CSL or citeproc-js behavior is always normatively correct.
Treat compatibility output as evidence, not law.

When outputs conflict, evaluate sources in this order:

1. Explicit publisher or style-guide rules
2. Citum design principles and schema intent
3. Stable bibliographic prior art, preferably `biblatex`
4. Legacy CSL and citeproc behavior
5. Local style convenience or migration shortcuts

If citeproc output appears bibliographically wrong, underspecified, or in tension
with project intent, do not blindly copy it. Classify the mismatch first.

## Normative vs Legacy Check

Before applying a fix for any non-trivial mismatch, explicitly decide which of
these buckets it belongs to:

- `style-defect` — the Citum style is wrong
- `migration-artifact` — migration preserved or introduced the wrong behavior
- `processor-defect` — the engine misrenders a valid style
- `legacy-limitation` — CSL/citeproc behavior is compatible legacy behavior but
  not the behavior Citum should preserve

Do this before optimizing for fidelity. If the answer is `legacy-limitation`,
an intentional divergence is allowed and should be preferred over copying the
legacy behavior.

## Intentional Divergence Rule

Intentional divergence from legacy CSL/citeproc is allowed when it is justified
by style-guide intent, bibliographic expectations, or Citum design goals.

When diverging intentionally:

- say explicitly that the divergence is intentional
- add or update regression coverage for the intended behavior
- explain verification impact if citeproc-based fidelity will remain lower
- avoid calling the result a processor defect unless the engine violates Citum's
  own declared semantics

## Shared Gates

- Compatibility fidelity regression is never allowed unless the task explicitly
  chooses a documented semantic divergence from legacy CSL behavior.
- SQI is optimization-only after fidelity is stable.
- All modes must pass `../style-qa/SKILL.md` before completion.
- If docs or beans are changed: `./scripts/check-docs-beans-hygiene.sh` must pass.

## Output Contract

Every completed task delivers:

1. Fidelity metrics: citations `N/M` and bibliography `N/M`
2. SQI delta: `+N`, `-N`, or `±0`
3. Authority basis: which source won (`style guide`, `Citum policy`, `biblatex`,
   `citeproc`, or mixed) and why
4. Divergences: `none` or a short statement of any intentional divergence from
   legacy CSL/citeproc behavior
5. Code Opportunities table (mandatory — see above)
6. QA verdict from `../style-qa/SKILL.md`

## Codebase Exploration (Engine / Schema Internals)

When assessing Code Opportunities or checking whether a processor feature exists,
use **jCodeMunch** instead of loading full source files. It returns symbol-level
slices at a fraction of the tokens.

```
# Map citum_engine's public API before assessing what's missing
get_repo_outline(repo: "local/citum-core")

# Check if a specific type or trait exists
get_symbol("StyleOptions", repo: "local/citum-core")

# Find all impls of a trait across crates
search_symbols("Render", repo: "local/citum-core")
```

Consult `~/.claude/skills/jcodemunch/SKILL.md` for the full tool reference.
Only fall back to `Read` for files where you need non-symbol content (YAML
styles, fixture JSON, shell scripts).

## Internal Skills (Pipeline Components)

- `style-maintain` — targeted fixes to existing Citum styles
- `style-migrate-enhance` — CSL 1.0 batch migration
- `style-qa` — QA gate
- `pr-workflow-fast` — PR packaging
- `jcodemunch` (`~/.claude/skills/jcodemunch/`) — symbol-level engine/schema lookup
