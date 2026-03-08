---
# csl26-380v
title: Enhance code documentation
status: completed
type: task
priority: normal
created_at: 2026-02-09T14:45:56Z
updated_at: 2026-03-01T14:29:26Z
---

There are many places in the codebase missing documentation.

## Summary of Changes

Added /// doc comments to all public API items across citum-schema, citum-engine, and citum-migrate crates.

### Files Updated

**citum-schema:**
- : Module-level documentation for public types and re-exports
- : Documentation for Rendering, TemplateComponent, TypeSelector, DelimiterPunctuation and their methods
- : Documentation for Citation::simple constructor
- : Documentation for GroupSortEntry::resolve method

**citum-engine:**
- : Documentation for Processor construction methods, config getters, and rendering methods

**citum-migrate:**
- : Module-level documentation and MacroInliner implementation methods
