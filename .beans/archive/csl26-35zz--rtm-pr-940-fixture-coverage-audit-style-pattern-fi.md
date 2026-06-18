---
# csl26-35zz
title: 'RTM: PR #940 — fixture coverage audit style pattern fix + squash'
status: completed
type: task
priority: high
created_at: 2026-06-18T14:28:34Z
updated_at: 2026-06-18T14:53:39Z
---

Fix common style authoring anti-pattern (raw affixes + hardcoded locale strings) across 5 files added in the PR, add missing locale terms, then squash both commits into a single passing feat(migrate) commit.

## Summary of Changes

- Fixed raw bracket/paren affixes and hardcoded locale strings in 5 PR files
  (hainan, nature-npg, nlm-superscript, taylor-francis-nlm, springer-vancouver-brackets-core)
- Extended fix to all styles with the same anti-pattern (~15 more style files)
- Added missing locale terms (term: in) to es-ES, eu-ES, ar-AR
- Fixed hardcoded role labels: elsevier-vancouver-core (editor suffix x2),
  chicago-notes-18th (translator short-label prefix x3),
  mhra-notes/mhra-author-date/new-harts-rules/mhra-notes-publisher-place (x6,
  translator verb-short prefix) 
- Squashed two commits into single feat(migrate) commit, force-pushed to PR #940
