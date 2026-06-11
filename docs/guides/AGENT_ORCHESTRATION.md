# Agent Orchestration Guide

**Status:** Active
**Date:** 2026-06-11
**Related:** [../policies/AGENT_HARNESS_POLICY.md](../policies/AGENT_HARNESS_POLICY.md); [../specs/REPO_LOCAL_HARNESS.md](../specs/REPO_LOCAL_HARNESS.md); [JJ_AI_CHANGE_STACK.md](JJ_AI_CHANGE_STACK.md)

## Purpose

Define how Citum agents turn a request into bounded work, durable artifacts,
implementation handoffs, review, and verification without depending on a
specific user-level toolchain.

## Planner-Orchestrator Flow

Non-trivial asks must use this flow. A request is non-trivial when it touches
multiple files, changes public behavior, requires design reconciliation, affects
verification policy, or delegates work to another agent.

1. Ground in repo truth: read the relevant bean, spec, policy, guide, crate
   scoped instructions, and existing code or style evidence.
2. Decide whether the existing artifact is sufficient:
   - use an existing bean for a concrete task
   - write or update a spec for non-trivial behavior or interface changes
   - write an audit when the value is dated operational evidence
3. Decompose the work into one bounded task packet per worker.
4. Give each task packet explicit scope, allowed write surface, acceptance
   criteria, verification commands, and escalation triggers.
5. Route workers and reviewers by role, not by personal model name.
6. Close the loop by updating the durable artifact with what changed, what was
   verified, and what remains.

## Role Tiers

Repo role contracts may declare `model_tier` and `reasoning_tier`, but those
fields are capability requirements, not model choices. `reasoning_tier` values
are limited to `low`, `medium`, and `high`. User configuration maps those tiers
to host-specific model names, effort knobs, and cost controls.

Workers and reviewers may request a stronger tier only by returning an
escalation finding with the reason, risk, and expected benefit. The parent
session or orchestrator must get explicit user permission before using a
frontier-plus model, raising reasoning above the normal tier, or otherwise
materially increasing cost.

## Task Packet Shape

Every bounded worker handoff should include:

- goal in project terms
- source artifact: bean, spec, issue, PR, or user request
- required context files
- allowed write paths
- forbidden changes
- exact acceptance criteria
- verification commands for the touched change type
- stop/escalation triggers
- expected output: changed paths, evidence, verification, residual risk

The packet may live in the body of a bean, in a spec implementation section, or
in the parent session's handoff. Do not create a new permanent task directory
unless beans and specs prove insufficient for the workflow.

## Reviewer Handoff

Reviewers should receive:

- the source artifact and acceptance criteria
- changed paths or diff scope
- exact verification already run
- known risks or unresolved questions

Reviewers return pass/reject findings. They do not edit files unless the parent
session explicitly reassigns them as a worker with a new bounded task packet.

## Local Tools

User config decides which model, reasoning control, code-intelligence tool,
search tool, shell wrapper, or host-specific hook to use. Repo artifacts should
name capabilities and commands, not a contributor's private toolchain or exact
model IDs. A user tool may make work cheaper or faster; it must not change
Citum's acceptance criteria.

## jj and Intent Capture

When `.jj` exists, non-trivial AI-authored change stacks must follow
[JJ_AI_CHANGE_STACK.md](JJ_AI_CHANGE_STACK.md) for local stack isolation.
`.ai-intents/` files are temporary drafting provenance only. Remove them before
`jj git export`, Git push, or PR creation, and run
[../../scripts/check-ai-intents-clean.sh](../../scripts/check-ai-intents-clean.sh)
as part of publish hygiene.
