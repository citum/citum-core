---
# csl26-1csf
title: 'engine: locale term override for no-date'
status: todo
type: bug
priority: high
created_at: 2026-03-08T13:39:50Z
updated_at: 2026-03-08T13:39:50Z
---

## Problem

harvard-cite-them-right citation renders (Forthcoming., n.d.) but oracle
expects (Forthcoming, no date). Two processor-defects:

1. **n.d. vs "no date"**: The en-US locale term for no-date is hardcoded as
   n.d. Harvard CTR requires "no date". No style-level term override exists.
   Fix: add locale term override support to style YAML schema, or allow
   styles to specify alternate locale terms.

2. **Trailing period on short-form family names**: With form:short, the
   engine appends a period to some names (Forthcoming. instead of Forthcoming).
   Likely from initialize-with bleeding into short form, or a name-parser
   treating single-token names as institutional with auto-period.

## Location (via jCodeMunch)

- `parse_general_term` — crates/citum-schema/src/locale/mod.rs:776
- `test_citation_locator_label_renders_term_with_loaded_locale`
  — crates/citum-engine/src/processor/tests.rs:317

## Acceptance Criteria

- (Forthcoming, no date) passes oracle for harvard-cite-them-right
- No regression on other styles using n.d. term
- Style YAML mechanism documented
