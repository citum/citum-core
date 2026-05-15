---
# csl26-n3qp
title: 'Schema: add short-name to SimpleName + FlatName'
status: completed
type: task
priority: normal
created_at: 2026-05-15T11:09:23Z
updated_at: 2026-05-15T11:19:23Z
parent: csl26-ycyp
---

Add short_name: Option<MultilingualString> to SimpleName, short_name: Option<String> to FlatName, propagate in to_names_vec(). File: crates/citum-schema-data/src/reference/contributor.rs

## Summary of Changes\n\nAdded short_name: Option<MultilingualString> to SimpleName, short_name: Option<String> to FlatName, propagated in to_names_vec() SimpleName arm.
