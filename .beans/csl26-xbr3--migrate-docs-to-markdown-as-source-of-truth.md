---
# csl26-xbr3
title: Migrate docs to markdown as source of truth
status: draft
type: feature
created_at: 2026-05-14T14:13:29Z
updated_at: 2026-05-14T14:13:29Z
---

Currently the docs site uses hand-authored HTML files in docs/. These should be replaced with markdown source files compiled to HTML via a static site generator (SSG) or build step, keeping HTML out of the repo.

Viable pages for markdown-first approach:
- developer.html (mostly prose + code blocks)
- reference.html (index-like, links out)
- operating.html (prose)
- reports.html (metrics/tables)

Pages needing SSG template layer (layout-heavy or data-driven):
- index.html (hero, feature cards)
- examples.html (sidebar + interactive IO panels)
- guides/style-authoring/*.html (sidebar nav + inline callouts)

Design decision: use a lightweight SSG (e.g. Eleventy or Vite + markdown-it) that supports:
- Shared nav/footer layouts (replace current build-layout.js)
- Callout/table shortcodes or remark plugins
- Sidebar generation from frontmatter
- Build-time schema badge injection (replaces deferred bean csl26-0y0z)
