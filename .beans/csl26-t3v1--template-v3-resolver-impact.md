---
# csl26-t3v1
title: TEMPLATE_V3 Implementation & Ecosystem Transition (jj Stack)
status: draft
type: task
priority: high
tags:
    - style
    - resolution
    - distributed-resolver
    - template
    - migrate
    - jj
created_at: 2026-05-05T00:00:00Z
updated_at: 2026-05-05T00:00:00Z
---

# csl26-t3v1

# Objective
Implement the TEMPLATE_V3 "Structural Diff" model in the Citum engine and transition the core style ecosystem (including authoring tools and `styles/`) to utilize it. This work will be delivered as a **Stacked PR sequence using `jj` (Jujutsu)** for seamless change management.

# Context
With the introduction of `DISTRIBUTED_RESOLVER.md` and `TEMPLATE_V3.md`, Citum is moving to a decentralized web of styles. The engine needs deep-merge capabilities to support surgical diffs (`modify`, `add`, `remove`) across network boundaries. 

Using `jj` allows us to maintain a live stack of these changes. We can refine the engine logic (base of the stack) and have those changes automatically propagate to the migration logic (top of the stack), ensuring consistent end-to-end testing throughout development.

# Proposal: Stacked Delivery with `jj`

### Revision 1: Engine Resolution Logic
*   **PR 1 (Target: `main`):** Implements the `DeepMerge` traits and updates `try_into_resolved_with` to apply diffs.
*   **Key Files:** `crates/citum-schema-style/src/lib.rs`, `crates/citum-schema-style/src/template.rs`.

### Revision 2: Ecosystem Transition
*   **PR 2 (Target: PR 1):** Updates `citum-migrate` to emit V3 diffs and performs the bulk refactor of `styles/`.
*   **Key Files:** `crates/citum-migrate/src/template_compiler/*`, `styles/*.yaml`.

# `jj` Workflow for Reviewers
1.  **Review PR 1:** This is the base of the stack.
2.  **Review PR 2:** This is the top of the stack. Because it's a `jj` stack, the PR in GitHub will only show the delta between the engine changes and the ecosystem refactor.
3.  **Refinements:** If review feedback requires changes to the engine, I will use `jj edit` on Revision 1. `jj` will automatically rebase Revision 2 on top of the fix, keeping the whole stack healthy.

# Goals
- Enable surgical style overrides that persist across upstream updates.
- Reduce the average line count of complex styles by 30-50%.
- Maintain bit-for-bit fidelity with existing oracle baselines.
