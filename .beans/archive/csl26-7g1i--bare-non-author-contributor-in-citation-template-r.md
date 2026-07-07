---
# csl26-7g1i
title: Bare non-author contributor in citation template renders names twice
status: completed
type: bug
priority: high
created_at: 2026-07-07T16:11:28Z
updated_at: 2026-07-07T16:19:42Z
---

A citation template containing a non-author contributor component (e.g. editor) under default author-date processing renders the name list twice, comma-joined: 'John Smith, John Smith'. Pre-dates the role-label defaults change (HEAD rendered 'John Smith, John Smith (ed.)'). Found while testing csl26-xve4 / PR #1028.

## Root Cause

Asymmetry in the grouped-citation fallback (crates/citum-engine/src/processor/rendering/grouped/core.rs): render_author_for_grouping_with_format renders the 'author part' from the first grouping component of the template — a contributor of ANY role (find_grouping_component) — but filter_author_from_template stripped only ContributorRole::Author from the item-part template. A template leading with translator/editor rendered those names in both halves, comma-joined.

## Summary of Changes

- filter_author_from_template now detects when the grouping component is a non-author contributor and strips that exact leading contributor from the item parts (new helper remove_first_contributor_with_role in processor/rendering/helpers.rs), keeping it symmetric with the author part.
- Title-grouping semantics deliberately untouched (separate class, no observed bug).
- Regression test leading_non_author_contributor_renders_once_in_grouped_citation in crates/citum-engine/tests/citations.rs.
- report-core diff vs PR #1028 tip: zero change across the corpus (no embedded style leads a citation template with a non-author contributor).
