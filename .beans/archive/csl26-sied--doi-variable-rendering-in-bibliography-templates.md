---
# csl26-sied
title: DOI variable rendering in bibliography templates
status: scrapped
type: feature
priority: normal
created_at: 2026-03-16T11:48:18Z
updated_at: 2026-03-16T12:16:44Z
---

RSC and other styles that output DOIs in bibliography cannot currently do so via `variable: doi` in the template. ITEM-1 in the RSC oracle expects `DOI:10.1234/example` appended to the entry but Citum has no mechanism for DOI field rendering with prefix. jCodeMunch symbol path: likely in citum-schema-style TemplateVariable or variable rendering in citum-engine. Oracle scenario: RSC ITEM-1 article-journal 1/33 failure. Unlocks: any style that needs DOI in bibliography output.

## Reasons for Scrapping
Duplicate of csl26-aa23 (same title, created in same session).
