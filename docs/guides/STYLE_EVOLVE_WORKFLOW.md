# Style Evolve Workflow

## Who This Is For
New contributors who want one command path for style work.

## One Command Rule
Use `/style-evolve` for all style tasks.

Do not choose internal skills directly unless you are maintaining the workflow.

For Codex CLI use, install the repo-owned skills with
`./scripts/install-codex-skills.sh` and invoke the named skills directly.
`./scripts/codex <role> <target...>` remains available as a fallback for direct role
targeting.

## Modes
1. `upgrade`
- Improve existing Citum style(s).
- Typical goal: increase SQI while preserving or improving fidelity.

2. `migrate`
- Convert one or more CSL 1.0 styles into high-quality Citum styles.
- Typical goal: high oracle parity plus maintainable templates.

3. `create`
- Create a style from reference evidence.
- Accepts mixed source inputs:
  - `--source-url`
  - `--source-text`
  - `--source-issue`
  - `--source-file`

## Examples
```bash
/style-evolve upgrade --styles styles/elsevier-harvard.yaml --target-sqi 0.90
/style-evolve migrate --legacy styles-legacy/apa.csl --count 1
/style-evolve create --source-url https://example-style-guide.org --source-text "example citations and bibliography"
```

## Quality Policy
- Fidelity is the hard gate.
- SQI is a secondary optimization metric.
- For styles with configured `benchmark_runs`, official rich-input evidence is auto-run as supplemental advisory output.
- Every iteration must assess both:
  - style-level edits
  - processor/preset/feature opportunities
- Before editing YAML, classify the target using the shared two-axis taxonomy:
  semantic class (`base`, `profile`, `journal`, `independent`) and
  implementation form (`alias`, `config-wrapper`, `structural-wrapper`,
  `standalone`).
- Profile work must preserve the config-wrapper contract: scoped options and
  metadata only, with no local templates, no local `type-variants`, and no
  template-clearing `null`.
- Journal descendants may legitimately remain structural wrappers when
  guide-backed deltas or current merge mechanics prevent a meaningful thin
  reduction.
- Choose parent styles from current guide-backed authority first, not nearest
  CSL/template similarity.
- If guide-backed parentage is real but the current merge model still forces a
  bulky child file, record the infrastructure constraint and stop forcing
  compression.

## Wave Guidance

A style wave is a bounded cohort executed through repeated `upgrade`,
`migrate`, or `create` passes under the shared workflow docs.

- Keep one wave to one family or one clearly related cohort per PR.
- Profile-family work may require a `create` pass for a hidden family root,
  followed by `upgrade` passes to reduce the public handles.
- `style-evolve` remains a three-mode public entrypoint. A wave is execution
  scope, not a fourth mode.

## Internal Pipeline (For Maintainers)
`/style-evolve` routes internally to:
- `style-maintain` (upgrade path)
- `style-migrate-enhance` (migrate path)
- `style-qa` (required gate)
- `pr-workflow-fast` (branch/PR efficiency)

`/styleauthor` remains available as a legacy alias and forwards to `/style-evolve`.

For Codex CLI users outside the host slash-command UI, use the named Codex skills
first:

```bash
./scripts/install-codex-skills.sh
# Then invoke $style-evolve, $migrate-research, or $rust-simplify in Codex.
```

Use `./scripts/codex <role> <target...>` only when you need direct access to an
internal role contract.
