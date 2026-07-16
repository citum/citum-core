---
# csl26-zmod
title: 'GB/T long-tail: periodicals, archives, conference containers, graphic'
status: todo
type: task
priority: normal
tags:
    - fidelity
    - migrate
    - style
created_at: 2026-07-16T15:52:20Z
updated_at: 2026-07-16T15:52:32Z
parent: csl26-8uxa
---

Remaining structural gaps for gb-t-7714-2025-numeric upstream corpus after wave 2 (~20 entries): whole-periodical entries (8.4.2:1-4) render via the flat default template; archival entries (8.12.3:1-4) need archive-location as title suffix and 收藏地：收藏者 imprint; conference proceedings container-title lost in conversion when only event-title present (8.6.3:1-3); graphic/audiovisual (8.11.3.2:2, 7.2.1:5) need [Z] marker and （create-date）[access-date] pattern; patent number source field mismatch (8.10.2:1-4, oracle uses call-number-ish application number); name particles (van der) dropped and Jr. period (8.5.3:10, 8.3.2:4); container-title-short strip-periods (7.1.3:2 Br Med J); serial issued full-date for online-first (8.5.1.1:7, 8.5.3:4); accessed-date conditional on missing issued (8.11.3.2:5, 8.13.3:3); CSTR tail dedupe (8.14.3:1).
