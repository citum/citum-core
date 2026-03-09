---
name: style-maintain
type: agent-invocable
description: Fast targeted maintenance for an existing Citum style. Use for punctuation/layout bugs, missing type overrides, or syntax modernization. Not for migrations or batch waves.
model: haiku
---

# Style Maintain

## Use This Skill When
- Updating one style for punctuation/layout bugs.
- Adding a missing type override.
- Modernizing style syntax without changing rendered output intent.

## Input Contract
- Existing style path in `styles/`.
- One focused objective (formatting bug, missing type, or modernization).
- Optional reference oracle style in `styles-legacy/`.

## Autonomous Operation

Run the full fix loop without pausing for approval. Commit automatically when QA passes.
Only interrupt for `Cargo.toml`/`Cargo.lock` changes or `git push origin main` (per CLAUDE.md).

## Workflow

Token efficiency matters — diagnose everything before touching any files.

0. **Divergence preflight.**
   Read `docs/adjudication/DIVERGENCE_REGISTER.md` before the first oracle run.
   If any mismatch is already covered there, classify it under the registered
   divergence instead of treating it as a fresh defect.

1. **Single oracle call, all failures at once.**
   Run `node scripts/oracle.js styles-legacy/<name>.csl --verbose` (or the correct
   oracle per the routing table below). The `--verbose` flag prints every failure with
   oracle vs. CSLN side-by-side. Read all failures before writing a single line of YAML.
   Do not use `report-core.js` for upgrade tasks — it's a portfolio tool, not a diff tool.

2. **Classify all failures before fixing any.**
   For each failure decide: `style-defect`, `migration-artifact`, `processor-defect`, or
   `legacy-limitation`. First check whether the case is already adjudicated in the
   divergence register; if so, record the matching `div-XXX`. This shapes both what
   you fix and what you escalate (see Co-Evolution below).

3. **Apply all YAML fixes in one pass.**

4. **One confirming oracle run.** Verify fidelity improved, bibliography held.

5. **QA gate → commit.**
   `git add -A && git commit -m "fix(styles): <name> <change>"` — max 5 iterations
   before surfacing to user.

## Fix Ordering
1. Component overrides and punctuation/wrap controls.
2. Shared bibliography spine improvements.
3. `type-templates` only for true structural outliers.
4. Processor/schema changes only after planner escalation.

## Co-Evolution (Mandatory — not a checkbox)

Style work and engine work evolve together. When you classify a failure, don't stop at
"deferred." The point of this classification is to drive engine improvements.

**For each `processor-defect` or `missing-feature`:**

1. Use jCodeMunch to locate the relevant engine code — do not load full source files:
   ```
   search_symbols("<term or feature name>", repo: "local/citum-core")
   get_symbol("<SymbolName>", repo: "local/citum-core")
   ```
2. Assess tractability: can this be fixed in ~30 lines of Rust? If yes, fix it now.
3. If not tractable in this session: file a bean with the symbol name, file path, line
   number, oracle diff, and a concrete description of the fix needed.
   `beans create "engine: <description>" -t bug -d "..."`
4. Reference the bean ID in the Code Opportunities table. **"Deferred" without a bean
   ID is not acceptable** — it means the opportunity will be lost.

The Code Opportunities table is delivered as part of every task output (inherited from
style-evolve). Every row must be either `implemented` or `deferred: <bean-id>`.

## Hard Gates
- Preserve or improve fidelity.
- No unnecessary template explosion.
- Keep fallback behavior for non-explicit types reasonable.

## Oracle Routing (MANDATORY — check before running any oracle)

Read `originKey` from the style's `info.source.adapted-by` field or from `report-core` output.

**`oracle.js` and `oracle-yaml.js` both use citeproc-js as the reference — they are WRONG for biblatex-derived styles.**

| `originKey` | Correct oracle |
|---|---|
| `csl-derived` | `node scripts/oracle.js styles-legacy/<name>.csl` |
| `biblatex-derived` | `node scripts/report-core.js > /tmp/r.json` — failures are in `styles[name].bibliography.entries` where `match === false` |
| `citum-native` | `node scripts/oracle-yaml.js styles/<name>.yaml` only |

For `biblatex-derived`, the only oracle that uses the correct authority (biblatex snapshot in
`tests/snapshots/biblatex/<name>.json`) is `report-core.js`. Run it and parse the JSON output
to see per-entry failures. If the snapshot is missing:
```bash
node scripts/gen-biblatex-snapshot.js --style <biblatex-style-name> --citum-style <name>
```

## Verification
- Oracle per routing table above — **not** blindly `oracle.js <csl-path>`
- `cargo run --bin citum -- render refs -b tests/fixtures/references-expanded.json -s <style-path>`
- QA handoff to `../style-qa/SKILL.md`

## Related
- Public router: `../style-evolve/SKILL.md`
- QA gate: `../style-qa/SKILL.md`
