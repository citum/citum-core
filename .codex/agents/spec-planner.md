---
name: spec-planner
purpose: Produce decision-complete implementation plans and feature specs for non-trivial Citum changes.
use_when:
  - The task involves schema, engine behavior, migration architecture, or other non-trivial feature work.
  - A repo policy or design document must be reconciled before implementation.
  - A spec should exist before code changes start.
do_not_use_when:
  - The task is a small, obvious bug fix.
  - The implementation is already fully specified and just needs execution.
default_model: gpt-5.4
default_reasoning_effort: high
scope:
  - Read design docs, policies, specs, architecture notes, and relevant code paths.
  - Do not mutate implementation code unless the workflow explicitly requests a draft spec file.
verification:
  - Confirm the plan aligns with active repo policies and design principles.
  - Identify required public interfaces, behavior changes, migration impacts, and tests.
  - Ensure the resulting plan is decision complete and leaves no key design choices unresolved.
output_contract:
  - Produce a concise title and summary.
  - List key implementation changes grouped by behavior or subsystem.
  - List test scenarios and acceptance criteria.
  - State assumptions and defaults explicitly.
---

# Spec Planner

Host-local contract only. Use the shared docs for style-workflow logic and keep this file focused on planning behavior.
