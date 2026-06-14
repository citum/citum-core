---
# csl26-dc1d
title: 'migrate: numeric style migrated as author-date (bio-protocol)'
status: todo
type: bug
priority: high
tags:
    - migrate
    - fidelity
created_at: 2026-06-14T11:20:14Z
updated_at: 2026-06-14T11:49:29Z
parent: csl26-vmcr
---

bio-protocol (CSL numeric, oracle '[16]') is migrated to render author-date '(Kuhn, 1962)'. The base/processing detection picks author-date for a numeric-class style. Converter-level: detect_processing_mode or base_detector mis-routes. Repro: node scripts/oracle.js styles-legacy/bio-protocol.csl --json --force-migrate. Tail also: journal-of-contemporary-water-research-and-education.
