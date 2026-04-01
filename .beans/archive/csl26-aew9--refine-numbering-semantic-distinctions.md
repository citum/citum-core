---
# csl26-aew9
title: Refine numbering semantic distinctions
status: completed
type: task
priority: deferred
created_at: 2026-04-01T15:27:01Z
updated_at: 2026-04-01T16:45:00Z
---

# Completed: Numbering Semantic Overload

## Context
During the PR review for the sequences/numbering refactor, it was noted that the `numbering` field (formerly `sequences`) has some semantic overload. For example, `InputReference::number()` dispatches to `NumberingType::Part` — "number" and "part" are distinct concepts. The biblatex conversion stores report numbers as `NumberingType::Issue`, and the legacy CSL path pushes non-report `number` fields to `NumberingType::Volume`. The vocabulary is doing double duty for semantically distinct things.

Additionally, `NumberingType::Book` as a numbering type is confusing when `Monograph(Book)` is a reference type. 

## Task
Refactor the semantic mappings of these numbering types to clarify their distinct conceptual roles and remove the overloading once the basic numbering API stabilizes.

## Summary of Changes

- Added `docs/specs/NUMBERING_SEMANTICS.md` and activated it in the implementation commit.
- Split canonical numbering semantics so generic `number`, report `report`, and true `part` no longer share the same `NumberingType`.
- Removed `NumberingType::Book`, narrowed `InputReference::number()`, and introduced `InputReference::report_number()`.
- Updated legacy CSL and biblatex conversions, rendering accessors, tests, and generated schemas to match the clean-break vocabulary.
