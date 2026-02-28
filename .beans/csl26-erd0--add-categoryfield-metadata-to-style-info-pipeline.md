---
# csl26-erd0
title: Add category/field metadata to style info pipeline
status: in-progress
type: feature
created_at: 2026-02-28T08:36:22Z
updated_at: 2026-02-28T08:36:22Z
---

Extend csl-legacy Info struct, citum-schema StyleInfo, and citum-migrate to capture CSL 1.0 category field metadata. Add CitationField enum (disciplines only), is_base flag, summary, links, rights. Bulk-update existing styles.
