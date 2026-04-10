---
# csl26-2pey
title: 'Note parser: continue past non-matching lines'
status: completed
type: feature
priority: normal
created_at: 2026-04-04T14:00:15Z
updated_at: 2026-04-10T18:09:06Z
---

The current \`parse_note_field_hacks\` heuristic stops scanning at the first
non-matching line (after skipping line 0). This means recognized \`key: value\`
pairs later in the note are never extracted if any free-text line precedes them.

Example: \`EXVHRUDT\` broadcast item — \`event-location: United States\` is not
extracted because a line with a space in the key appears first and halts parsing.

## Tasks
- [ ] Investigate whether changing break→continue causes regressions in existing
  oracle/citation tests before committing.
- [ ] If safe, replace the break with continue (accumulate non-matching lines
  into residual note but keep scanning for recognized pairs).
- [ ] Add a unit test in csl_json.rs covering mixed free-text + recognized keys.

## Summary of Changes

Replaced `break` with `continue` in `parse_note_field_hacks` so the scanner
continues past non-matching free-text lines. Two unit tests added:
- `test_parse_note_field_recognized_keys_after_free_text`: broadcast with free-text before `event-place`
- `test_parse_note_field_recognized_keys_after_midnote_free_text`: bill with free-text between `genre` and `status`

All 16 note-parser tests pass. Oracle 18/18 unaffected.
