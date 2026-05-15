---
# csl26-iaou
title: 'Fix abbreviation-map schema: restore flat map shape'
status: completed
type: task
priority: normal
created_at: 2026-05-15T11:09:22Z
updated_at: 2026-05-15T11:19:23Z
parent: csl26-ycyp
---

AbbreviationMap should remain a transparent HashMap<String, String>. Regenerate abbrev-map.json so the schema matches the flat map model. File: crates/citum-schema-data/src/abbreviation.rs

## Summary of Changes\n\nKept AbbreviationMap as a transparent HashMap newtype, regenerated the schema for the flat map shape, and added serde coverage proving reserved-looking keys are ordinary entries.
