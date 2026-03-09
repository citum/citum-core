---
# csl26-ann1
title: Support annotated bibliography rendering
status: completed
type: feature
priority: normal
created_at: 2026-02-23T00:00:00Z
updated_at: 2026-03-09T01:12:27Z
---

Add support for rendering annotated bibliographies where each reference entry is followed by a descriptive annotation paragraph.

Current state: The `note` field exists on all reference types as `Option<String>`, but there is no template component or processor support to append it as an annotation block below a bibliography entry.

## Required changes

* Add `annotation` or `abstract` component to the bibliography template schema in `citum_schema`
* Implement processor rendering for the annotation block (paragraph break + indented text)
* Decide: use existing `note` field or add a dedicated `abstract` field (note is for internal notes, abstract is for reader-facing summaries)
* Update style YAML schema to allow `- annotation: note` or similar template component
* Add oracle/integration test with an annotated bibliography style

But it's also possible, maybe even likely, that these notes don't belong in the bibliographic data, but instead reference them.
In that scenario, citum might perpahs have to consider paired entries for this case: (key, note).
A GUI app, then, might allow the user to select a few specific notes associated with their respective references, but omit others.
So then there might need to be a dedicate API for this?

## References

No CSL 1.0 equivalent (CSL 1.0 does not support annotated bibliographies natively).

Related feature: Similar to CSL-M's secondary note rendering, but specifically for user-facing annotations rather than internal notes.

## Summary of Changes

Implemented in commits 9367000 (feat(engine,cli): annotated bibliography support), fda0486 (tests), ec34b75 (non-HTML format fix), 121faf0, and 0329ee9 (djot inline rendering for annotations).

## Spec

Design specification migrated to [docs/specs/ANNOTATED_BIBLIOGRAPHY.md](../docs/specs/ANNOTATED_BIBLIOGRAPHY.md).
