---
# csl26-jt8t
title: Support non-standard numbering locators
status: todo
type: task
priority: deferred
created_at: 2026-04-01T15:27:01Z
updated_at: 2026-04-01T15:27:01Z
---

# Defer: Non-Standard Locators Support

## Context
The current `NumberingType` enum provides a controlled vocabulary for locators/numbering. However, this blocks truly non-standard data. For example, a film scholar who needs `reel: "3"` or a musicologist who needs `movement: "II"` currently has nowhere to put that in the model because the controlled vocabulary acts as the extensibility ceiling rather than the floor.

## Task
Design and implement a mechanism for arbitrary or non-standard locators/numberings that safely exceed the controlled `NumberingType` vocabulary, ensuring Citum remains fully extensible for edge-case scholarly requirements.
