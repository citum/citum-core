---
# csl26-3fw0
title: 'migrate: note-class full-note template run-on'
status: completed
type: bug
priority: high
created_at: 2026-06-10T16:56:45Z
updated_at: 2026-06-10T21:17:19Z
parent: csl26-vmcr
---

Cluster C4 (worst class: note 16/19 below bar, share at threshold 15.8%) from docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md. Note-class full-note citation templates lose delimiters/affixes wholesale producing run-on output ('"Title"PublisherPlace'), and use wrong name form (full instead of initials). Evidence: early-medieval-europe (citations 9/20), zeitschrift-fur-medienwissenschaft (7/20). One bounded migrate-research pass.

## Pass evidence (2026-06-10)

Landed: (1) compile_citation_note — occurrence-based XML citation compilation for note styles (preserves groups/affixes, no author-date sort, poison-stripped); forced-XML control on early-medieval-europe: 9/20 -> 17/20 citations. (2) XML-extracted subsequent/ibid overrides now attach to inferred-source note citations (previously dropped whenever the inferrer won).

Note corpus (19 styles): +2 (china-information 58.6->72.4, vienna-legal 74.1->79.3) / -1 (bulletin -1.7) / 16 unchanged; sentinels unchanged (oscola 13/20 pre-existing, chicago-notes 20/20).

Residual: note-class gap dominated by bad inferred FIRST templates at falsely-high confidence (0.93 on 9/20 styles). Forcing XML-first regressed good inferred styles (6 up/11 down) — rejected. Requires measured inferred-vs-XML candidate selection; follow-up bean filed.

## Summary of Changes

The run-on mechanism is fixed: note-class citations no longer compile through the simplified author-date route. compile_citation_note preserves groups, affixes, and authored order via occurrence-based compilation, and XML-extracted subsequent/ibid repeat forms now attach to inferred-source note citations. Residual note-class gap (bad inferred first templates at falsely-high confidence) rerouted to csl26-jav1 (measured inferred-vs-XML selection).
