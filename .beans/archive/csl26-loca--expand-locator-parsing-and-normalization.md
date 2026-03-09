---
# csl26-loca
title: Expand locator parsing and normalization
status: completed
type: feature
priority: normal
created_at: 2026-02-14T00:00:00Z
updated_at: 2026-03-09T14:21:49Z
parent: csl26-q8zt
---

The prototype WinnowCitationParser for Djot supports basic locator labels (p, ch, vol, etc.), but needs to be more robust.

Next steps:
- Expand the list of supported locator labels (see CSL 1.0 locator list).
- Implement normalization of locator forms (e.g., "page 10" -> page: 10).
- Support range parsing in locators (e.g., "pp. 10-12").
- Support multiple items with locators in single citation groups.
- Investigate locale-aware locator label detection.

Impact: Essential for scholarly citation fidelity.
Effort: 1 week



## Summary of Changes

- Unified the citation locator model in the schema and processor.
- Landed Djot locator parsing and rendering updates on `main` via `feat(citation): unify locator model`.
- Updated citation fixtures, oracle helpers, and snapshots for the new locator representation.
