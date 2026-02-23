---
# csl26-ann1
title: Support annotated bibliography rendering
status: todo
type: feature
priority: normal
created_at: 2026-02-23T00:00:00Z
updated_at: 2026-02-23T00:00:00Z
---

Add support for rendering annotated bibliographies where each reference entry is followed by a descriptive annotation paragraph.

Current state: The `note` field exists on all reference types as `Option<String>`, but there is no template component or processor support to append it as an annotation block below a bibliography entry.

## Required changes

* Add `annotation` or `abstract` component to the bibliography template schema in `csln_core`
* Implement processor rendering for the annotation block (paragraph break + indented text)
* Decide: use existing `note` field or add a dedicated `abstract` field (note is for internal notes, abstract is for reader-facing summaries)
* Update style YAML schema to allow `- annotation: note` or similar template component
* Add oracle/integration test with an annotated bibliography style

## References

No CSL 1.0 equivalent (CSL 1.0 does not support annotated bibliographies natively).

Related feature: Similar to CSL-M's secondary note rendering, but specifically for user-facing annotations rather than internal notes.
