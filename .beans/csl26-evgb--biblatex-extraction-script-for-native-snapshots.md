---
# csl26-evgb
title: biblatex extraction script for native snapshots
status: completed
type: task
priority: normal
created_at: 2026-03-06T22:21:09Z
updated_at: 2026-03-06T23:11:56Z
parent: csl26-anlu
---

Write scripts/gen-biblatex-snapshot.sh. LaTeX driver + biber pipeline that renders compound-numeric fixtures and extracts plain-text citations/bibliography into tests/snapshots/biblatex/<style>.json. Replaces oracle-native.js bootstrap for biblatex-sourced styles.

## Summary of Changes

Implemented `scripts/gen-biblatex-snapshot.js` — a general-purpose biblatex snapshot generator.

Key design decisions:
- Works with any biblatex style (not limited to compound-numeric)
- Auto-converts CSL JSON fixture to .bib via `cslJsonToBibtex()`
- 3-pass pdflatex+biber pipeline; pdftotext -layout for extraction
- Numbered ([1], [2]) and unnumbered (author-date) extraction paths
- Soft-hyphen de-hyphenation; en-dash/em-dash preserved at line ends
- Right-aligned numbering handled (allows up to 3 leading spaces for [N] detection)
- Staleness checked via source_hash of fixture file
- Snapshot: `tests/snapshots/biblatex/<citum-style>.json`

Smoke-tested against chem-angew (32/33 entries) and APA (33/33 entries).
