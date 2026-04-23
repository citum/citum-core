# Style Workflow Decision Rules

**Status:** Active
**Version:** 1.0
**Date:** 2026-04-04
**Related:** `docs/guides/STYLE_WORKFLOW_EXECUTION.md`, `docs/architecture/SKILL_AGENT_REFACTOR.md`

## Rule
Shared style-workflow agents must classify each mismatch as `style-defect`, `migration-artifact`, `processor-defect`, or `intentional divergence`, and must stop iterating once a cluster is clearly outside the active workflow's scope.

Style-authority decisions for existing journals must follow this order:
1. current publisher or journal style guide
2. current publisher submission instructions or house rules
3. documented parent-family manual or base style reference
4. CSL implementation evidence
5. existing Citum YAML structure

Before editing YAML, shared style workflows must classify the target on two
axes:
1. semantic class: `base`, `profile`, `journal`, or `independent`
2. implementation form: `alias`, `config-wrapper`, `structural-wrapper`, or
   `standalone`

This classification is operational, not cosmetic. It determines whether the
style should be reduced to a thin wrapper, kept structural, or left
self-contained.

## Rationale
Style work in Citum repeatedly follows the same decision logic: determine whether the defect belongs in YAML, migration, engine behavior, or adjudication, then route the work accordingly. Putting that logic in one policy keeps the Claude and Codex wrappers thin and reduces drift between hosts.

## Application
- `style-defect` routes to style-local YAML repair.
- `migration-artifact` stays in migration-focused work.
- `processor-defect` routes to engine or processor follow-up.
- `intentional divergence` is recorded and excluded from fix counts.
- `profile` means both a semantic relationship and a config-wrapper contract.
  Profile work must preserve that contract: local metadata and scoped options
  are allowed, but local templates, local `type-variants`, and
  template-clearing `null` values are not.
- `journal` is semantic only. A journal descendant may legitimately be an
  `alias`, `config-wrapper`, or `structural-wrapper` depending on the evidence
  and the current infrastructure.
- If a publisher guide says a journal follows a known parent style with a few house tweaks, treat the parent preset as the baseline and keep only the documented deltas.
- Treat CSL XML and migrated standalone YAML as implementation evidence, not the canonical authority for wrapper thickness.
- If a guide-confirmed parent-plus-deltas relationship cannot yet be expressed compactly because preset merge semantics are too coarse, record that as an infrastructure constraint instead of preserving CSL duplication as the source of truth.
- If guide-backed parentage is real but the current merge mechanics still force
  a bulky child file, stop forcing compression. Record the infrastructure
  constraint and keep the style structural until the override model improves.
- If the same scenario fails with identical output after two distinct approaches, stop iterating on that scenario and reclassify it.
- If a registered divergence explains the failure, record the divergence ID instead of treating it as a fresh bug.
- A style wave is a bounded cohort executed through repeated `upgrade`,
  `migrate`, or `create` passes under these same rules. Keep waves scoped to one
  family or one clearly related cohort per PR.

## Exceptions
- Host-specific routing, model choice, and permission semantics stay in the wrapper files.
- Rich-input evidence ordering and per-skill output phrasing live in the execution guide.

## Changelog
- v1.2 (2026-04-23): Added the two-axis taxonomy as an operational workflow
  rule, made the config-wrapper profile contract explicit, clarified that
  journal descendants may remain structural, and added the infrastructure-stop
  rule for guide-backed wrappers blocked by current merge mechanics.
- v1.1 (2026-04-19): Added explicit source-of-truth ordering for preset-wrapper work and clarified that CSL artifacts are verification evidence, not authority.
- v1.0 (2026-04-04): Established shared style-workflow decision rules.
