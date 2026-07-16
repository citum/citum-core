---
# csl26-49sj
title: 'Design: conditional number labels (edition 版, volume 第..卷)'
status: todo
type: feature
created_at: 2026-07-16T15:52:20Z
updated_at: 2026-07-16T15:52:20Z
---

GB/T 7714 renders 版 after edition and 第{n}卷 as a title suffix only when the value is numeric (CSL: choose is-numeric + number form=ordinal + label; CSL-M %s terms like 第%s卷). Citum has no conditional-label mechanism on TemplateNumber and no circumfix number terms. Affects ~12 upstream-corpus entries (7.4:*, 7.2.3:*, 8.13.*, 9.2.1.3:4, plus 康熙字典 volume-title). Candidate designs: TemplateNumber label placement + when-numeric gate, or locale-owned MF2 messages with a $number arg selected per rendering locale. Also decide registered divergences for citeproc quirks (2nd 版 / 5th editors — en ordinal leak into zh, wrong per the standard text).
