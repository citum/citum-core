---
# csl26-aew9
title: Refine numbering semantic distinctions
status: todo
type: task
priority: deferred
created_at: 2026-04-01T15:27:01Z
updated_at: 2026-04-01T15:27:01Z
---

# Defer: Numbering Semantic Overload

## Context
During the PR review for the sequences/numbering refactor, it was noted that the `numbering` field (formerly `sequences`) has some semantic overload. For example, `InputReference::number()` dispatches to `NumberingType::Part` — "number" and "part" are distinct concepts. The biblatex conversion stores report numbers as `NumberingType::Issue`, and the legacy CSL path pushes non-report `number` fields to `NumberingType::Volume`. The vocabulary is doing double duty for semantically distinct things.

Additionally, `NumberingType::Book` as a numbering type is confusing when `Monograph(Book)` is a reference type. 

## Task
Refactor the semantic mappings of these numbering types to clarify their distinct conceptual roles and remove the overloading once the basic numbering API stabilizes.
