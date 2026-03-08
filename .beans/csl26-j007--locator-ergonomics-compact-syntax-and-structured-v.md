---
# csl26-j007
title: 'Locator ergonomics: compact syntax and structured values'
status: in-progress
type: feature
priority: normal
created_at: 2026-03-05T16:42:31Z
updated_at: 2026-03-05T18:05:47Z
---

Two related improvements to compound locator input ergonomics, building on csl26-z4t6.

## 1. Compact YAML syntax

Current verbose form:
```yaml
locators:
  - label: page
    value: "23"
  - label: line
    value: "13"
```

Compact map form (Citum prototype style):
```yaml
locators:
  page: "23"
  line: "13"
```

Implementation: `#[serde(untagged)]` enum on `locators` field accepting
either `Map(IndexMap<LocatorType, LocatorValue>)` or `List(Vec<LocatorSegment>)`.
Ordering preserved via IndexMap.

## 2. Structured LocatorValue for deterministic plurality

Current: `value: String` with heuristic plural detection (checks for `-`, `–`, `,`, `&`).
Problem: false positives like "figure A-3" trigger plural ("pp." instead of "p.").

Solution: hybrid `#[serde(untagged)]` enum:
```rust
enum LocatorValue {
    Text(String),                          // heuristic (95% case)
    Explicit { value: String, plural: bool }, // override
}
```

YAML:
```yaml
# Normal (heuristic works):
locators:
  page: "42-45"

# Explicit override:
locators:
  - label: page
    value:
      value: "figure A-3"
      plural: false
```

## Prior art
- Citum prototype: `enum Locator { KeyValue((LocatorTerm, String)), String(String) }`
- dplanner analysis recommended Option 3 (Hybrid) for zero-breakage + deterministic plurality

## Dependencies
- Requires csl26-z4t6 (compound locators) merged first
- Orthogonal to csl26-zafv (numeric compound citations)

Needs /dplan session to finalize exact serde strategy and test plan.
