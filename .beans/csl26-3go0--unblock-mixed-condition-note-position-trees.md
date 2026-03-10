---
# csl26-3go0
title: Unblock mixed-condition note position trees
status: in-progress
type: task
priority: normal
tags:
    - migration
    - styles
    - citations
created_at: 2026-03-10T22:20:45Z
updated_at: 2026-03-10T22:54:30Z
---

Follow-on from csl26-qfa3 and archive bean csl26-494i.

Problem:
XML-mode migration still falls back to the base citation template when note
styles mix position tests with other conditions in the same choose tree.

In scope:
- chicago-notes
- chicago-notes-bibliography-17th-edition
- mhra-notes
- mhra-notes-publisher-place
- mhra-notes-publisher-place-no-url
- new-harts-rules-notes
- new-harts-rules-notes-label-page
- new-harts-rules-notes-label-page-no-url

Deliverables:
- extend migrate support for mixed-condition note position trees without
  flattening sibling content
- add regression coverage for the blocked note-tree shapes
- re-run the note/legal batch from csl26-qfa3 after the migrate fix lands
- only open style-local cleanup follow-ons if specific styles still diverge
  after the shared migrate fix

2026-03-10 implementation notes:
- Shared migrate fix implemented in `citum-migrate`; XML-mode output now emits
  `citation.subsequent` / `citation.ibid` sections for all eight scoped note
  parents.
- Regression coverage added for mixed `position + type`, `position + variable`,
  `position + locator`, and one intentionally unsupported ambiguous-fallback
  tree.
- Verification completed: `cargo fmt`, `cargo clippy --all-targets --all-features
  -- -D warnings`, `cargo nextest run`, scoped oracle batch, core quality gate,
  and bean hygiene checks.
- Raw XML-mode regeneration was checked, but the existing shipped note-style
  YAML files were not replaced automatically in this PR. Those files already
  include hand-tuned cleanup beyond what the shared XML migrator can infer.
- Actual style upgrades should happen style by style: regenerate the parent YAML,
  compare it against the currently shipped style and oracle output, then keep the
  regenerated version only if it improves or preserves fidelity. If a specific
  style still needs manual cleanup after the shared migrate fix, split that into
  a narrow follow-on bean rather than widening this shared migration task.
