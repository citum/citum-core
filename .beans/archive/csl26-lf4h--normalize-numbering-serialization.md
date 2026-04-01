---
# csl26-lf4h
title: Normalize numbering serialization
status: completed
type: task
priority: deferred
created_at: 2026-04-01T15:27:01Z
updated_at: 2026-04-01T16:30:00Z
---

# Completed: Numbering Serialization Normalization

## Context
There is currently a dual representation of numbering data with no normalization. The biblatex/legacy CSL conversion paths write directly into `numbering` (no shorthand set), while the YAML authoring path uses shorthands (no `numbering` set). Both are valid inputs and the getter unifies them. But the serialized output format differs based on origin, and nothing prevents both from being populated simultaneously with conflicting values.

## Task
Implement a post-deserialization normalization step that migrates shorthand fields into the `numbering` array on ingest. This will ensure the serialized form is always canonical and resolves the dual-representation ambiguity without breaking YAML ergonomics (shorthands will still deserialize fine, but won't serialize back).

## Summary of Changes

- Added shared shorthand normalization for monographs, collections, collection components, serial components, and classic works.
- Preserved existing non-conflicting `numbering` entries while giving shorthand fields precedence for volume, issue, edition, and part.
- Canonicalized serialization so normalized references emit `numbering` and keep `sequences` as an input-only compatibility alias.
