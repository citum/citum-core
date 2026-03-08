---
# csl26-erd0
title: Add category/field metadata to style info pipeline
status: completed
type: feature
priority: normal
created_at: 2026-02-28T08:36:22Z
updated_at: 2026-02-28T08:47:20Z
---

Extend csl-legacy Info struct, citum-schema StyleInfo, and citum-migrate to capture CSL 1.0 category field metadata. Add CitationField enum (disciplines only), is_base flag, summary, links, rights. Bulk-update existing styles.

## Summary of Changes

- Extended csl-legacy Info with fields, is_base, summary, links, authors, contributors, rights
- Added parse_info_person() helper in csl-legacy parser
- Added CitationField enum (25 disciplines), StyleLink, StylePerson, StyleSource, extended StyleInfo in citum-schema
- Created InfoExtractor in citum-migrate mapping legacy Info -> StyleInfo with provenance
- Created update_style_info binary for bulk updates
- Updated 139/144 styles; 5 hand-authored styles skipped gracefully
- All 362 tests pass
