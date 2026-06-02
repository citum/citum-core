---
# csl26-4ada
title: Add givenname-disambiguation-rule field to Disambiguation
status: draft
type: feature
priority: normal
created_at: 2026-06-02T13:49:12Z
updated_at: 2026-06-02T13:49:44Z
---

Disambiguation struct lacks givenname_rule. Engine always expands all positions (all-names behavior). Should respect primary-name-with-initials (APA), primary-name (Chicago), by-cite (CSL 1.0.1 default). Spec: docs/specs/CROSS_ENTRY_FIDELITY.md
