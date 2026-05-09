---
# csl26-mrg3
title: Optimize merge_style_overlay to avoid serialization round-trips
status: todo
type: task
priority: low
created_at: 2026-05-09T00:00:00Z
updated_at: 2026-05-09T00:00:00Z
---

The current implementation of `merge_style_overlay` in `citum-schema-style` relies 
on re-serializing data structures (JSON/YAML) to perform deep merges. While 
functional and flexible, this adds significant overhead to the style resolution 
process.

## Scope

- Evaluate and implement a strongly-typed deep merge strategy for Citum styles 
  that avoids intermediate serialization.
- The solution must account for the project's multi-format support, which 
  includes JSON, YAML, and CBOR.
- Consider utilizing a custom macro or a specialized `Merge` trait to handle 
  the recursive merging of templates and variants.
- Baseline the performance of the current serialization-based approach against 
  the new implementation to verify gains.

## Rationale

Eliminating the serialization round-trip during style merging will reduce 
latency, particularly in the `citum-server` interactive mode where styles 
may be resolved or overlaid frequently.
