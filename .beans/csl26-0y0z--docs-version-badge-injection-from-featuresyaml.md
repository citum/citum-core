---
# csl26-0y0z
title: 'Docs: version badge injection from features.yaml'
status: todo
type: task
priority: low
created_at: 2026-05-14T13:23:56Z
updated_at: 2026-05-14T13:23:56Z
---

Inject per-feature version badges into guide pages at build time, sourced from docs/reference/features.yaml.

## Approach ideas

**Build-time injection (preferred)**
- Extend build-author-guide.js (or a new build-docs.js) to read features.yaml
- Match features by id to guide page sections (via frontmatter or heading convention)
- Inject a `.citum-version-since` badge at the relevant H2/section: 'since schema X.X / engine X.X'
- Status (active/preview/experimental) maps to a badge variant already in citum-theme.css

**Convention for mapping**
- Each markdown guide section declares `feature: <id>` in frontmatter or an HTML comment
- The build script looks up that id in features.yaml and injects the badge

**What NOT to do**
- Do not hardcode badges as static HTML in index.html or other pages (current broken approach)
- Do not add a standalone 'Know what a style depends on' homepage section

**Also consider**
- Markdown-as-source-of-truth for the top-level HTML pages (index, developer, reference, operating). Currently these are hand-authored HTML. A future step could use markdown+frontmatter source files, with CI building to HTML for GitHub Pages deployment. This pairs naturally with badge injection since the build script would process all pages uniformly.
