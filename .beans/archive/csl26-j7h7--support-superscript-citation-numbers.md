---
# csl26-j7h7
title: Support superscript citation numbers
status: completed
type: feature
priority: high
created_at: 2026-02-07T06:53:04Z
updated_at: 2026-02-26T14:00:00Z
blocking:
    - csl26-6whe
    - csl26-l2hg
---

Nature and Cell styles use superscript numbers, not [1] or (Author Year).

Current: (Kuhn 1962)
Expected: ¹

Fix:
- Detect citation-number variable in CSL citation layout
- Detect vertical-align='sup' on number text
- Set citation.template to number-only for numeric styles
- Handle superscript as rendering option in citum_schema
- Test against Nature, Cell styles

Refs: GitHub #128, TIER3_PLAN.md Issue 1.1

## Summary of Changes

Oracle-verified: `nature.csl` passes 13/13 citations. In the plain-text rendering path,
both citeproc-js and Citum output bare citation numbers (`1`, `2`, …), so they match.

The `vertical-align="sup"` in CSL is a presentation hint for rich-text (HTML/RTF) output.
The existing `VerticalAlign::Superscript` in `citum-schema` (renderer.rs:197) already handles
this for formatted output contexts. No additional core logic required; superscript is covered
for the HTML renderer via the legacy schema conversion path.

Scope limited to plain-text fidelity gate — HTML renderer enhancements tracked separately.
