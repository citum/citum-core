---
# csl26-uidd
title: Compound bibliography drops merged row when only a non-leader member is cited
status: completed
type: bug
priority: normal
created_at: 2026-07-11T14:22:11Z
updated_at: 2026-07-11T14:54:05Z
---

Found during code review of csl26-mnoo (branch codex/fix-grouped-bibliography-routing).

render_flat_compound_entries (processor/bibliography/grouping.rs) filters already-merged compound rows by the retained leader ID against cited_ids. merge_compound_entries keys each merged row under the first configured member present in the full processed set, so for a compound set [A, B] where only B is cited, the merged row carries id=A; A is not in cited_ids, and the entire row — including the cited member B — is silently dropped from cited-only content. Meanwhile DocumentBibliography.entries (computed unmerged from the cited subset) still includes B, so content and entries disagree.

This is pre-existing behavior (the removed render_with_legacy_grouping applied the same post-merge filter) and csl26-mnoo deliberately preserved it as the historical compound-selection contract. A fix would select cited membership before/during merging (a row is retained if ANY member is cited, or members are filtered pre-merge like custom groups do in render_with_custom_groups_filtered), which is a behavior change needing its own tests and a look at how citation-number assignment should behave for the uncited leader.

Related: with sort-partitioning active, the compound path filters refs by cited_ids BEFORE merging (render_with_partition_sections branch), so partitioned and unpartitioned compound styles already have divergent selection semantics for this edge case.

## Summary of Changes

Fixed `render_flat_compound_entries` (crates/citum-engine/src/processor/bibliography/grouping.rs) to treat a compound-numeric set as a single bibliographic unit: a merged row now renders in full when **any** configured member is cited, not just when the leader (the id the merged row is keyed under) is cited. Previously, citing only a non-leader member (e.g. `ref-b` of set `[ref-a, ref-b]`) dropped the entire merged row because the filter checked `cited_ids.contains(leader_id)`.

This makes the flat compound path symmetric with the already-tested leader-cited contract (`test_render_document_bibliography_compound_groups_use_full_render`): citing any set member now shows the full merged row (all sub-labels), while `DocumentBibliography.entries` stays cited-only and unmerged per member — so `content` and `entries` agree on which references are represented, even though `content` shows the whole set.

Added a mirror test, `given_only_non_leader_compound_member_cited_when_document_bibliography_rendered_then_full_row_is_retained`, next to the existing leader-cited test in `crates/citum-engine/src/processor/tests.rs`.

Left the partitioned compound path (`render_with_partition_sections`) unchanged — it filters members by `cited_ids` *before* merging, so it intentionally shows only cited members. This divergence between flat (full-set) and partitioned (cited-only) compound rendering is pre-existing and out of scope for this fix.

- [x] Fix render_flat_compound_entries retention predicate
- [x] Update doc comment
- [x] Add mirror test for non-leader-cited case
- [x] just pre-commit passes (fmt, clippy -D warnings, nextest: 1866 passed)
