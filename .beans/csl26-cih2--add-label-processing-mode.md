---
# csl26-cih2
title: Add label processing mode
status: completed
type: feature
priority: high
created_at: 2026-02-20T02:06:39Z
updated_at: 2026-02-27T15:20:57Z
---

The style author guide listed label as a valid processing mode but it does not exist in the schema or processor. Implement label mode (alphabetic labels like 'a', 'b', 'c') as a fourth processing mode alongside author-date, numeric, and note. Add to Processing enum in schema, implement in citum_engine, and restore documentation in the style author guide.

## Summary of Changes

Added label processing mode row to the Processing Modes table in docs/guides/style-author-guide.html. Schema and engine were already fully implemented.
