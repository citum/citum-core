---
# csl26-1vq0
title: 'Redistribute examples.html: capability showcase + recipe pages'
status: completed
type: task
priority: normal
created_at: 2026-05-19T17:30:38Z
updated_at: 2026-05-28T23:53:28Z
---

Slim examples.html to a capability showcase (~300 lines). Add 3 new pages: typst.html, lualatex.html (corrected to two-pass, experimental), advanced-examples.html for style authors. Update 5 inbound-link pages. Fixes inaccuracies in Typst (no embedded crates, external CLI) and LuaLaTeX (two-pass, citum-labs, CITUM_LIB_PATH).

## Summary of Changes

Work landed in commit `39be5733 docs: redistribute examples.html to targeted pages` on main. examples.html slimmed to a capability showcase; three new pages created: guides/integrations/typst.html, guides/integrations/lualatex.html, guides/style-authoring/advanced-examples.html. developer.html updated with Typst card and corrected LuaLaTeX info.
