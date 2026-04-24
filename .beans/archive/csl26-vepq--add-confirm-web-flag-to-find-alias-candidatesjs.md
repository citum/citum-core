---
# csl26-vepq
title: Add --confirm-web flag to find-alias-candidates.js
status: completed
type: feature
priority: normal
created_at: 2026-04-19T13:11:05Z
updated_at: 2026-04-24T12:13:54Z
---

Enhance the alias discovery script with a --confirm-web flag that fires a web search per candidate above the similarity threshold (e.g. '"<journal name>" citation style author guidelines'). Attach the best evidence URL and a confidence note to the TSV output so reviewers have citable confirmation without manual lookup. Candidate search backend: Perplexity or DuckDuckGo API. See docs/guides/ALIAS_DISCOVERY.md#web-confirmation-planned-enhancement.
