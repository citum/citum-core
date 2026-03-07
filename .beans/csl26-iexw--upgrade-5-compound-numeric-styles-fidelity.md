---
# csl26-iexw
title: Upgrade 5 compound-numeric styles fidelity
status: in-progress
type: task
priority: normal
created_at: 2026-03-07T19:53:28Z
updated_at: 2026-03-07T20:36:56Z
---

Upgrade all 5 compound-numeric styles to improve biblatex fidelity: numeric-comp, chem-acs, angewandte-chemie, chem-rsc, chem-biochem. Final step: PR.

## WIP Notes

### Oracle fix (report-core.js)
- Added `expandCompoundBibEntries()` to expand compound snapshot blocks before comparing
- chem-acs/chem-rsc/chem-biochem snapshots have 25 entries (entry 0 = items 1-9 compound block)
- After fix: chem-acs bib=3/33, angewandte-chemie bib=12/33 (was all 0 before)

### Styles progress
- angewandte-chemie: cit=26/26 bib=12/33 fidelity=0.644 (complete round of edits done)
- chem-acs: cit=26/26 bib=3/33 fidelity=0.492 (partial edits; oracle now unblocked)
- chem-rsc: cit=26/26 bib=0/33 fidelity=0.441 (not yet edited)
- chem-biochem: cit=25/26 bib=0/33 fidelity=0.424 (not yet edited)
- numeric-comp: cit=0/0 bib=0/33 fidelity=0.000 (not yet edited)

### Remaining chem-acs issues (from oracle diff)
- parent-serial emph: true → remove (journal names should NOT be italic)
- parent-monograph emph: true → remove (book container names should NOT be italic)
- primary title suffix '.' → suppress for book/report/thesis types
- chapter/paper-conference/entry-encyclopedia: suppress primary title
- DOI suppression when pages present (engine gap — deferred)
- entry-encyclopedia: year prefix ': ', volume 'Vol.' prefix
- report publisher prefix '; ' → ', ' override needed
- motion_picture/broadcast year prefix: ',' not ' '
- webpage/misc year prefix: ',' not ' '

### Remaining work (not started)
- chem-rsc YAML edits (given-first names, quoted article titles, different separators)
- chem-biochem YAML edits (year in parens after author for articles)
- numeric-comp YAML edits (full given names, In: prefix, year in parens, doi: prefix)
- Run full oracle on all 5 after edits
- Update quality baseline if fidelity improves
