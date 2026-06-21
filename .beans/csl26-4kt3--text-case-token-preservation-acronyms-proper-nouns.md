---
# csl26-4kt3
title: Text-case token preservation (acronyms + proper nouns)
status: todo
type: task
priority: normal
created_at: 2026-06-21T17:46:30Z
updated_at: 2026-06-21T17:46:30Z
---

Deferred from csl26-maim. Sentence-case transform clobbers acronyms (NIPS->Nips) and proper nouns (Cambridge->cambridge, AI->Ai). citeproc uses a stop-word/proper-noun heuristic. Known-hard. Root: crates/citum-engine/src/values/text_case.rs. Audit rows 213 + Springer Vancouver section.
