---
# csl26-zafv
title: Support numeric compound citation styles
status: draft
type: feature
created_at: 2026-03-05T15:58:00Z
updated_at: 2026-03-05T15:58:00Z
---

CSL schema issue #437: numeric compound styles group multiple references under a single citation number with sub-labels (a, b, c...).

Example: "2. a) Zwart KB, et al. (1983) ..., b) van der Klei IJ, et al. (1991) ..."

This is prevalent in chemistry journals and was never supported in CSL 1.0.

Design questions to resolve:
- How does input signal compound grouping? Style-level `processing: numeric-compound`? Explicit field on `Citation`?
- Sub-label rendering: alphabetic (a, b, c) per locale? Configurable?
- Delimiter between sub-items within a compound citation
- Interaction with existing numeric citation numbering

Orthogonal to compound locators (csl26-z4t6, completed).
Needs a /dplan session before implementation.
