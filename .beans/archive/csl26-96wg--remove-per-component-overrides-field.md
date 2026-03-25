---
# csl26-96wg
title: Remove ComponentOverride from template schema
status: completed
type: feature
priority: normal
created_at: 2026-03-23T20:17:41Z
updated_at: 2026-03-25T20:59:20Z
---

Completed earlier than this bean reflected. `ComponentOverride` and the
template-component `overrides` field are already absent from `crates/`.

The confusion came from two different override mechanisms:

- removed template-component overrides in the template schema
- still-live `options.substitute.overrides` entries in some styles

This bean is retained only as a corrected historical record. Follow-on work, if
any, should target the remaining live mechanisms directly rather than reusing
this bean.
