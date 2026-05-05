---
# csl26-t3v1
title: TEMPLATE_V3 Merging Logic for Distributed Resolvers
status: draft
type: task
priority: high
tags:
    - style
    - resolution
    - distributed-resolver
    - template
created_at: 2026-05-05T00:00:00Z
updated_at: 2026-05-05T00:00:00Z
---

# csl26-t3v1

# Objective
Update Citum's style resolution logic to support the TEMPLATE_V3 deep-merge model, ensuring that templates can inherit and surgically modify components across distributed network boundaries.

# Context
With the introduction of `DISTRIBUTED_RESOLVER.md` and `TEMPLATE_V3.md`, Citum is moving from a monolithic repository to a decentralized web of styles. The current "First Match Wins" field replacement in the engine's `try_into_resolved_with` function is insufficient for V3 templates. 

If a style inherits from a remote URI, it needs to be able to apply "diffs" (modify/add/remove) to the components of the parent's templates rather than just replacing them entirely. This prevents the "cascading hard-fork" problem where downstream styles stop receiving upstream fixes.

# Proposal
Evolve the `citum-schema-style` resolution logic:

1.  **Deep Merge Trait:** Implement a merging trait for template-bearing structures (`CitationSpec`, `BibliographySpec`, `TemplateGroup`).
2.  **Recursive Diff Application:** Update `try_into_resolved_with` to apply local `type-variants` diffs to the fully resolved templates of the parent style.
3.  **Expanded Options Configuration:** Move contributor and date formatting policies from positional template attributes into the `options` hierarchy, ensuring consistent formatting without macros.

# Key Considerations
- **Order of Operations:** Resolving the parent URI must happen before applying the child's diffs.
- **Cycle Detection:** Maintain existing loop protection while traversing complex macro/extends chains.
- **Performance:** Ensure deep-merging doesn't significantly slow down style resolution, especially in the browser (WASM).

# Goals
- Enable surgical style overrides that persist across upstream updates.
- Support institutional style branding (e.g., "Add this logo to all Harvard-derived styles").
- Simplify GUI editing by maintaining a "Link" to parent styles.
