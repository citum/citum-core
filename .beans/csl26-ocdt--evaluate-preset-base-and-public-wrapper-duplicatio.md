---
# csl26-ocdt
title: evaluate preset-base and public-wrapper duplication across embedded styles
status: todo
type: task
priority: normal
created_at: 2026-04-10T17:36:49Z
updated_at: 2026-04-10T17:37:00Z
---

Evaluate whether Citum's current split between:
- internal preset bases under `styles/preset-bases/`
- public loadable styles under `styles/`

is architecturally necessary across the embedded-style set, or whether the
current near-mirroring is transitional duplication that should be collapsed.

This is not APA-specific. Use APA as the clearest example, but review the
entire embedded-style registry and the preset loader / embedded-style loader
contract together.

Expected owning subsystem:
- primary: `citum_schema_style`
- secondary: `styles/`
- consult: CLI / server / docs consumers that load public style paths directly

## Questions To Answer
- Which use cases require a non-recursive preset base distinct from the public
  style handle?
- Which embedded styles currently have both a preset base and a public wrapper,
  and how much real delta exists between them?
- Can public wrappers be reduced to thin `preset:` entrypoints with minimal
  metadata or true overrides, rather than full mirrored style definitions?
- Should the embedded-style registry expose the public wrapper, the preset
  base, or both?
- What migration or authoring-pipeline change would let us eliminate redundant
  duplication without breaking CLI, docs, tests, or builtins?

## Tasks
- [ ] Inventory every style in the embedded-style registry and map whether it
  has a paired preset base, a public wrapper, or both.
- [ ] Trace how `StylePreset`, the embedded-style registry, and direct style
  file loading are each consumed by the CLI, tests, docs, and server.
- [ ] Classify the duplication:
  required architectural split, thin-wrapper opportunity, or accidental mirror.
- [ ] Propose the target contract for preset bases vs public wrappers.
- [ ] If cleanup is warranted, break the implementation work into follow-up
  beans by subsystem.

## Acceptance
- the repo has a concrete answer for why both layers exist
- the answer covers the full embedded-style set, not just APA
- any unjustified duplication is called out explicitly with a recommended
  cleanup path

## Stop-Loss Rule
- do not collapse files opportunistically during the investigation
- stop at architectural analysis plus a concrete follow-up plan unless a
  separate implementation bean is created
