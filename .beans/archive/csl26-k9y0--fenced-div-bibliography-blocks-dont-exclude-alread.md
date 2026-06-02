---
# csl26-k9y0
title: Fenced-div bibliography blocks don't exclude already-placed entries
status: completed
type: bug
priority: normal
created_at: 2026-05-25T19:14:01Z
updated_at: 2026-06-02T10:44:31Z
---

When a document uses multiple ::: bibliography ::: fenced divs (e.g. primary/secondary split), entries rendered in an earlier block reappear in later unfiltered blocks. Each block calls render_document_bibliography_block independently with no shared 'assigned' state across blocks. The style-level grouping code (grouping.rs) does track assigned entries but only for the render_grouped_bibliography_with_format code path, not the fenced-div pipeline in pipeline.rs:replace_document_bibliography_blocks. Observed with type=manuscript primary block + unfiltered secondary block: harrington1891 appears in both.

## Summary of Changes

Work completed in 9130123. Added assigned: &mut HashSet<String> to render_bibliography_for_group and render_document_bibliography_block. replace_document_bibliography_blocks now initialises one HashSet before the per-block loop, threading it through every call so each block sees what was already placed.
