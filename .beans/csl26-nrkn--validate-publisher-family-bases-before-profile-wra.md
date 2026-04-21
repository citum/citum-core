---
# csl26-nrkn
title: Validate publisher-family bases before profile-wrapper conversion
status: todo
type: task
priority: normal
tags:
    - styles
    - taxonomy
    - verification
created_at: 2026-04-21T12:14:46Z
updated_at: 2026-04-21T12:14:46Z
---

Evaluate the embedded publisher/profile styles that were tentatively remapped to
existing bases during the style-taxonomy rename work, and decide which of those
styles are actually thin parent-plus-deltas wrappers versus independent authored
styles.

This bean exists because the uncommitted follow-on on `refactor/style-taxonomy-rename`
showed that broad family grouping is plausible, but the specific `extends:`
assignments to today's Tier-1 bases are not evidence-stable enough to land as
registry/spec truth.

## Why this is a follow-up instead of part of the current PR
- The branch-local `extends:` experiments changed rendered output materially on
  the shared `tests/fixtures/references-expanded.json` surface; this is not a
  metadata-only taxonomy edit.
- Publisher guidance supports family grouping, but not the exact current base
  mappings in every case.
- Some candidate mappings conflict with the style's own source metadata:
  - `taylor-and-francis-council-of-science-editors-author-date` points to
    `cse-name-year`, not APA.
  - `springer-basic-brackets` points back to `springer-basic-author-date`, not IEEE.
  - `elsevier-harvard` points to `ecology-letters`, not APA.

## Evidence to carry forward
- Representative profile/base comparisons on the shared fixture changed about
  70-152 rendered lines.
- Working-tree `extends:` additions alone changed about 56-93 rendered lines vs
  the committed branch state in sampled styles.
- Authority order for style decisions remains:
  1. publisher or journal guide
  2. publisher house rules
  3. named parent-style manual or base reference
  4. CSL/template-link evidence
  5. current Citum YAML structure

## Candidate styles to evaluate
- `elsevier-harvard`
- `elsevier-vancouver`
- `springer-basic-author-date`
- `springer-basic-brackets`
- `springer-vancouver-brackets`
- `taylor-and-francis-council-of-science-editors-author-date`
- `taylor-and-francis-national-library-of-medicine`
- `chicago-shortened-notes-bibliography` as the control case for a proven profile

## Required outcome for each style
Classify each style as exactly one of:
- thin profile on an existing base
- thin profile on a new publisher/standards base
- independent/self-contained style

## Tasks
- [ ] Build a per-style evidence table using the authority order above.
- [ ] Record the current publisher-guide relationship, if any, for each candidate profile.
- [ ] Record the CSL template-link or named-parent evidence for each candidate profile.
- [ ] Re-run reduced citation/bibliography fixture comparisons for each style against its candidate base or family base.
- [ ] Decide the classification for each candidate style.
- [ ] Update the taxonomy wording so `profile` means guide-backed parent-plus-deltas, not output-similar-to-an-existing-base.
- [ ] Split any required implementation into narrower beans if new bases or converter work are needed.

## Acceptance
- there is a per-style evidence table covering all candidate profiles
- each profile has a written classification decision and rationale
- no style is converted to a thin wrapper unless reduced YAML reproduces current
  accepted behavior on the chosen verification surface
- taxonomy language is tightened to prevent future output-similarity shortcuts

## Stop-Loss Rule
- do not reland the reverted branch-local `extends:` mappings just because they
  seem directionally correct
- if a style needs a new publisher-family or standards-specific base, record
  that explicitly instead of forcing it onto an existing Tier-1 base

## Related
- csl26-v961
- csl26-ocdt
- csl26-wp6y
