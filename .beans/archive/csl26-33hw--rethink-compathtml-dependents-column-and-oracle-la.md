---
# csl26-33hw
title: 'Rethink compat.html: dependents column and oracle labeling for compound styles'
status: completed
type: task
priority: normal
created_at: 2026-03-07T01:22:55Z
updated_at: 2026-03-08T02:02:15Z
---

User feedback on compat.html report:

1. **'Dependents' column has no meaning for Citum-native styles** - the concept of CSL ecosystem dependents only applies to migrated styles. For compound/native styles it should either be hidden or renamed to something Citum-specific (e.g. 'Usage' or removed).

2. **Oracle source label needs rethink** - 'citum-native' was misleading (these came from biblatex specs); changed to 'snapshot' as interim fix. But the column may need renaming or restructuring to make the provenance/benchmark clear.

Possible directions:
- Rename 'Dependents' column to 'CSL Dependents' with N/A shown for non-CSL-origin styles
- Add separate 'Origin' or 'Source' column showing biblatex/CSL/scratch
- Replace 'snapshot' oracle label with something describing the benchmark type (e.g. 'biblatex-ref', 'hand-authored')

See also: compound style YAML source fields (adapted-by: citum-create needs rethinking too)
