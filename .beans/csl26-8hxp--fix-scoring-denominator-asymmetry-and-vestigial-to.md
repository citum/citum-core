---
# csl26-8hxp
title: Fix scoring denominator asymmetry and vestigial tokenizer split
status: todo
type: task
priority: low
created_at: 2026-07-06T18:42:31Z
updated_at: 2026-07-06T18:42:31Z
parent: csl26-al39
---

Audit F5 (2026-07-06 migrate review): (1) score_bibliography_entries counts unmatched references AND unmatched rendered entries in items, while invalid_candidate_score counts only references — candidates compared on pass counts use different denominators. Unify or document. (2) tokenize is a pure alias of tokenize_normalized yet doc comments describe a distinct historical raw citation scorer; collapse token_jaccard/token_jaccard_normalized and fix the comments.
