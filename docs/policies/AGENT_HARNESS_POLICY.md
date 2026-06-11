# Agent Harness Policy

**Status:** Active
**Version:** 1.0
**Date:** 2026-06-11
**Superseded by:**
**Related:** [../../CLAUDE.md](../../CLAUDE.md); [../specs/REPO_LOCAL_HARNESS.md](../specs/REPO_LOCAL_HARNESS.md); [../guides/AGENT_ORCHESTRATION.md](../guides/AGENT_ORCHESTRATION.md); [../guides/JJ_AI_CHANGE_STACK.md](../guides/JJ_AI_CHANGE_STACK.md)

## Rule

Planner, worker, and reviewer roles must operate from repo-tracked Citum truth:
plans and task packets define scope and acceptance, workers execute bounded
tasks, reviewers stay read-only by default, and repo verification is mandatory.

## Rationale

The Citum harness needs to support multiple hosts and personal toolchains
without letting hidden user config become project policy. Separating role
contracts from local execution choices keeps workflows inspectable,
reproducible, and reviewable in Git.

## Application

Use this role contract for non-trivial agent work:

| Role | Owns | Must not own |
|---|---|---|
| Planner-orchestrator | decomposition, durable task packet, acceptance criteria, worker/reviewer routing | unreviewed implementation drift |
| Worker | one bounded task packet, changed paths, evidence, status updates | stack shape, broad redesign, publish decisions |
| Reviewer | read-only findings, pass/reject, missing verification, policy conflicts | implementation edits unless explicitly reassigned |

Durable artifacts:

| Need | Artifact |
|---|---|
| Concrete tracked work unit | `.beans/*.md` |
| Non-trivial implementation contract | [../specs/](../specs/) |
| Binding recurring rule | [./](./) |
| Operational workflow | [../guides/](../guides/) |
| Dated evidence or audit record | [../architecture/audits/](../architecture/audits/) |

Verification must route through repo-owned commands and scripts. User-level
wrappers may reduce tokens or provide convenience, but they must not weaken or
replace the Citum gate for the touched change type.

Repo-local role contracts may declare `model_tier` and `reasoning_tier` to
describe required capability. They must not name exact model IDs, private tools,
or personal cost policy. `reasoning_tier` is limited to `low`, `medium`, and
`high`; anything more expensive or capable than the user's normal frontier tier
requires explicit user permission through the parent session.

## Exceptions

Small, obvious edits may skip a separate planner artifact when the existing
bean, spec, or user request already defines scope and acceptance. The final
response must still report verification and residual risk.

Temporary `.ai-intents/` files may be used during local `jj` drafting, but they
are not durable project artifacts and must be absent before Git export, push, or
PR creation unless the user explicitly requests durable prompt provenance.
Non-trivial AI-authored `jj` change stacks must follow
[../guides/JJ_AI_CHANGE_STACK.md](../guides/JJ_AI_CHANGE_STACK.md).

## Changelog

- v1.0 (2026-06-11): Established planner, worker, reviewer, artifact, and
  verification boundaries for the two-layer harness.
