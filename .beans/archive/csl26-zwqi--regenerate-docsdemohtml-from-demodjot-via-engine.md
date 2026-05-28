---
# csl26-zwqi
title: Regenerate docs/demo.html from demo.djot via engine
status: completed
type: task
priority: normal
created_at: 2026-05-28T11:25:06Z
updated_at: 2026-05-28T11:30:02Z
---

Replace the hand-authored (now mangled) demo.html content with an engine-generated version produced by a build script from docs/demo.djot. Drop bad commit d98ba846, write scripts/build-demo-page.js, adjust citum-interactive.css, verify interactivity.

## Summary of Changes\n\n- Reset branch (dropped bad commit d98ba846 via git reset --mixed main, preserved demo.djot)\n- Added scripts/build-demo-page.js: renders demo.djot via citum engine, extracts article + Bibliography sections, injects into clean HTML5 page template reproducing ad633d61 chrome\n- Updated docs/assets/citum-interactive.css: restructured .citum-bibliography rules for engine's nested #Bibliography section; updated sidebar CSS to target #Bibliography; added .demo-reproduce and #Bibliography h2 rules\n- Regenerated docs/demo.html: clean straight-quote HTML5, engine-rendered citations and bibliography, all interactive behaviors working (tooltips, highlight, sidebar toggle)
