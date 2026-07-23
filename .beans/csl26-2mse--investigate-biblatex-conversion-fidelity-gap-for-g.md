---
# csl26-2mse
title: Investigate BibLaTeX conversion fidelity gap for GB/T corpus
status: todo
type: task
priority: normal
created_at: 2026-07-23T20:55:21Z
updated_at: 2026-07-23T20:55:21Z
---

On the gb7714-bench benchmark, citum's .bib source path (via citum convert refs, BibLaTeX->Citum-YAML) diverges from Zotero far more than the .json path: even with the terminal-period fix applied as a counterfactual, exact-match-vs-Zotero stayed at 12-14% on builtin.bib/better.bib vs 76-88% on builtin.json/better.json (same underlying references, same rendering style). This points to a real gap in citum-migrate's BibLaTeX field mapping/conversion, separate from bibliography-template formatting. See docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md 'Scope note' section. Needs its own investigation to characterize which fields/entry types are mismapped.
