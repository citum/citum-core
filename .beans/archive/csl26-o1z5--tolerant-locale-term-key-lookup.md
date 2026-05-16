---
# csl26-o1z5
title: Tolerant locale term-key lookup
status: completed
type: feature
priority: normal
tags:
    - forward-compat
created_at: 2026-05-15T14:48:14Z
updated_at: 2026-05-16T16:31:38Z
---

Promote forward-compat row 08 from observed=HardFail to observed=SoftDegrade.

A locale term key not in the engine's current vocabulary should not abort parse. The lookup path should fall back to rendering the key itself (or empty + warning) and surface a CompatibilityWarning so users know the term wasn't honored.

Files affected:
- crates/citum-schema-style/src/locale/mod.rs (Locale::from_yaml_str and the term resolution path)

Spec: docs/specs/FORWARD_COMPATIBILITY.md
Snapshot row: 08-new-locale-term-key
