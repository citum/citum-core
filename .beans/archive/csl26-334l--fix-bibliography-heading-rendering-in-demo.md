---
# csl26-334l
title: Fix bibliography heading rendering in demo
status: completed
type: bug
priority: normal
created_at: 2026-05-28T12:20:32Z
updated_at: 2026-05-28T12:20:36Z
---

Bibliography h1 and h2 headings in demo.html were unstyled/oversized. Add explicit font-size/weight rules for #Bibliography > h1 (1.1rem) and compact uppercase styling for #Bibliography h2 (0.75rem).

## Summary of Changes\n\nAdded  rule (1.1rem, weight 600) and updated  to compact uppercase treatment (0.75rem, uppercase, weight 700, border-bottom separator). Verified via computed styles in browser preview.
