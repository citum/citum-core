---
# csl26-6wab
title: 'Fix note-style fidelity: bib 0/N deflates score'
status: in-progress
type: bug
created_at: 2026-04-12T16:44:33Z
updated_at: 2026-04-12T16:44:33Z
---

Note styles (hasBibliography: false) have oracle bibliography totals of 0/56 included in fidelity calculation, deflating scores (e.g. chicago-notes-18th shows 37.4% instead of 72.9%). Fix computeFidelityScore to skip bibliography when hasBibliography===false, and fix overall aggregation.
