---
# csl26-a2nz
title: 'Fix site nav: sidebars, Capabilities nav, version badges, mobile'
status: completed
type: task
priority: normal
created_at: 2026-05-19T17:55:01Z
updated_at: 2026-05-19T18:20:37Z
---

1) Remove spurious version badges from 7 style-authoring pages; 2) Add advanced-examples.html to style-authoring sidebar; 3) Add sidebar to 7 integration pages; 4) Add Capabilities to top nav (all 29 pages); 5) Fix mobile sidebar (display:none → horizontal strip)

## Summary of Changes

Root cause: `scripts/build-layout.js` is the source of truth for nav — CI regenerates all HTML on every deploy, wiping any direct HTML edits. Fixed by updating NAV_TEMPLATE in build-layout.js (added Capabilities link, renamed Developer→Integrate, wired ACTIVE_CAPABILITIES placeholder), then regenerated all 27 HTML files locally with `npm run build:layout`.
