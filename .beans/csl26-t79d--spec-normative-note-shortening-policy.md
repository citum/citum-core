---
# csl26-t79d
title: Spec normative note shortening policy
status: todo
type: feature
priority: medium
created_at: 2026-03-11T00:00:00Z
updated_at: 2026-03-11T00:00:00Z
---

Follow-up to the repeated-note rollout PR.

We need a normative spec for repeated-note and shortened-note behavior across
note styles, separate from the current shipped-style audit and rollout work.

Definition of done:

- Draft and activate a spec under `docs/specs/` for note-shortening policy.
- Paraphrase manual-derived rules without quoting copyrighted source text.
- Classify note-style families such as lexical relative markers, shortened-note
  preference, legal `id.` forms, and localized variants.
- Define what belongs in style data versus processor logic.
- Revisit the note-position audit manifest if the normative model differs from
  current shipped behavior.
- Tighten family-level tests only after the spec settles prose/integral
  repeated-cite semantics.
