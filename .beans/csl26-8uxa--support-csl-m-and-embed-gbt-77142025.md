---
# csl26-8uxa
title: Support CSL-M and embed GB/T 7714—2025
status: in-progress
type: feature
priority: high
tags:
    - migrate
    - multilingual
    - schema
    - style
created_at: 2026-07-15T12:24:38Z
updated_at: 2026-07-15T12:49:15Z
---

Implement the CSL-M migration and embedded GB/T 7714—2025 family approved from GitHub Discussion #828.

Specs:
- docs/specs/MULTILINGUAL.md
- docs/specs/REFERENCE_IDENTIFIERS.md

## Acceptance Criteria

- [x] Localized layouts select both structure and rendering locale.
- [x] Supplementary CSTR identifiers migrate and render through a typed map.
- [ ] csl-legacy preserves ordered CSL-M layouts and citum-migrate emits them.
- [ ] Three embedded GB/T styles share a hidden base.
- [ ] Upstream fixtures retain source, revision, and license attribution.
- [ ] All three styles reach 100% fidelity and clean SQI.
- [ ] Schema generation, docs/beans hygiene, and just pre-commit pass.
- [ ] PR checks pass.
