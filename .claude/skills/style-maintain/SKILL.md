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
1. Reproduce mismatch with one oracle snapshot.
2. Apply smallest YAML-first fix.
3. Recheck oracle metrics.
4. Run QA gate — if rejected, iterate (max 5 attempts) before surfacing to user.
5. On QA pass: `git add -A && git commit -m "fix(styles): <name> <change>"` and report results.

## Fix Ordering
1. Component overrides and punctuation/wrap controls.
2. Shared bibliography spine improvements.
3. `type-templates` only for true structural outliers.
4. Processor/schema changes only after planner escalation.

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
