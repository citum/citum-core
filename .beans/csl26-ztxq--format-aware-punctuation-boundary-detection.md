---
# csl26-ztxq
title: Format-aware punctuation boundary detection
status: todo
type: task
tags:
    - punctuation
    - rendering
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

visible_text strips only HTML tags, so separator/dedup logic misfires for LaTeX/Typst/Markdown (\emph{Title.} ends in }, producing 'Title.. Next'), violating the backends-differ-only-in-markup rule. cleanup_dangling_punctuation also runs global find/replace over full marked-up entries including attributes. Add a visible-text/logical-last-char hook to OutputFormat (or track logical boundaries in ProcTemplateComponent) and constrain the cleanup pass to text outside markup. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 13.
