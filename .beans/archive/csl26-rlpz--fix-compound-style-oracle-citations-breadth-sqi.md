---
# csl26-rlpz
title: 'Fix compound style oracle: citations, breadth, SQI'
status: completed
type: bug
priority: high
created_at: 2026-03-07T01:02:43Z
updated_at: 2026-03-07T01:28:59Z
---

Compound styles show 100% fidelity but SQI < 65%. Root causes:
1. oracle-native.js hardcodes citations: {passed:0, total:0} and citationsByType: {}
2. compound-numeric-refs.yaml only has 'book' type (5 refs → 3 bib entries)
3. SQI typeCoverage (35% weight) = 0 → SQI capped at ~65%

Fixes:
- [ ] Expand compound-numeric-refs.yaml with article-journal, chapter, paper-conference types in compound sets
- [ ] Fix oracle-native.js to parse and compare citation sections; build citationsByType from fixture
- [ ] Delete old snapshots and regenerate from correct output
- [ ] Verify SQI improves to >= 85 for all 5 compound styles

## Summary of Changes

- Expanded compound-numeric-refs.yaml: 6 types, 10 refs, 4 compound sets + 2 standalones
- Fixed oracle-native.js: now tests citations (42/run) and builds citationsByType from fixture
- Fixed report-core.js: propagates native citation counts/citationsByType; renames oracle source to 'snapshot'
- Regenerated snapshots for all 5 compound styles
- Result: SQI 98-100 (was <65%), typeCoverage=100 (was 0), 42 citations tested (was 0)
- PR: https://github.com/citum/citum-core/pull/299

## Summary of Changes

- Expanded compound-numeric-refs.yaml: 6 types, 10 refs, 4 compound sets + 2 standalones
- Fixed oracle-native.js: now tests citations (42/run) and builds citationsByType from fixture
- Fixed report-core.js: propagates native citation counts/citationsByType; renames oracle source to 'snapshot'
- Regenerated snapshots for all 5 compound styles
- Result: SQI 98-100 (was <65%), typeCoverage=100 (was 0), 42 citations tested (was 0)
- PR: https://github.com/citum/citum-core/pull/299
