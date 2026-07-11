---
# csl26-uidd
title: Compound bibliography drops merged row when only a non-leader member is cited
status: todo
type: bug
created_at: 2026-07-11T14:22:11Z
updated_at: 2026-07-11T14:22:11Z
---

Found during code review of csl26-mnoo (branch codex/fix-grouped-bibliography-routing).

render_flat_compound_entries (processor/bibliography/grouping.rs) filters already-merged compound rows by the retained leader ID against cited_ids. merge_compound_entries keys each merged row under the first configured member present in the full processed set, so for a compound set [A, B] where only B is cited, the merged row carries id=A; A is not in cited_ids, and the entire row — including the cited member B — is silently dropped from cited-only content. Meanwhile DocumentBibliography.entries (computed unmerged from the cited subset) still includes B, so content and entries disagree.

This is pre-existing behavior (the removed render_with_legacy_grouping applied the same post-merge filter) and csl26-mnoo deliberately preserved it as the historical compound-selection contract. A fix would select cited membership before/during merging (a row is retained if ANY member is cited, or members are filtered pre-merge like custom groups do in render_with_custom_groups_filtered), which is a behavior change needing its own tests and a look at how citation-number assignment should behave for the uncited leader.

Related: with sort-partitioning active, the compound path filters refs by cited_ids BEFORE merging (render_with_partition_sections branch), so partitioned and unpartitioned compound styles already have divergent selection semantics for this edge case.
