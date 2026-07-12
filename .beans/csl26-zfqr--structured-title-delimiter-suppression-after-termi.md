---
# csl26-zfqr
title: Structured title delimiter suppression after terminal punctuation
status: todo
type: bug
priority: normal
tags:
    - punctuation
    - rendering
created_at: 2026-07-05T17:18:48Z
updated_at: 2026-07-12T18:52:06Z
---

Follow-up from GitHub issue #1010 / csl26-01jy.

When a structured title main part ends with significant terminal punctuation,
rendering should avoid blindly inserting the configured primary delimiter.
Example desired behavior: `Main title? And a subtitle`, not
`Main title?: And a subtitle`.

This likely belongs with broader punctuation-boundary work (`csl26-ztxq`) and
needs a spec decision for which terminal marks suppress or replace configured
structured-title delimiters across locales.

- [ ] Amend the title or punctuation spec with terminal-punctuation delimiter suppression rules
- [ ] Add structured-title tests for `?`, `!`, and other agreed terminal punctuation
- [ ] Implement delimiter suppression without breaking configurable `primary-delimiter` / `subtitle-delimiter` behavior

## Root Cause (2026-07-06 review)

`crates/citum-engine/src/values/title.rs::render_structured_title` (line 255) appends `primary_delimiter` unconditionally between the rendered main part and the subtitle group. Delimiters resolve in `structured_title_delimiters` (line 330): explicit `rendering.primary_delimiter` override, else `locale.grammar_options.title_subtitle_delimiter`. Nothing inspects the main part's terminal character, so `Main title?` + `: ` → `Main title?: And a subtitle`.

## Fix Design

Key simplification: the check is **semantic, not presentational** — test the terminal character of the *source* main-part text (`st.main` / the short-form variant) before `render_part_with_case` runs, not the rendered fragment. That avoids any markup-stripping dependency and decouples this bean from csl26-ztxq entirely (text-case transforms do not alter terminal punctuation; quote-wrapped mains are source text too, so mirror the quote-aware lookback in `ends_with_sentence_ending_visible_punctuation`, bibliography.rs:61).

1. **Spec first** (per todo): amend the title/punctuation spec with the suppression rule — when the main part ends in a suppressing terminal mark, the punctuation core of the configured primary delimiter is dropped and only its trailing whitespace is emitted. Recommended mark set: `?`, `!`, `…` (and `.` only when the delimiter core is itself `.`); encode the set in `locale.grammar_options` (new field, default "?!…") so inflected locales can override. Suppression applies to explicit style-level `primary_delimiter` overrides too, matching citeproc-js's global punctuation dedupe.
2. **Implementation:** in `render_structured_title`, split `primary_delimiter` into (punctuation core, whitespace tail); if the main part's last non-quote char is in the locale's suppressing set, push only the whitespace tail. `subtitle_delimiter` joins between multiple subtitles need the same rule (a subtitle can also end in `?`).
3. **Tests:** rstest matrix over ?/!/…/. mains × default and overridden delimiters × long/short forms × quoted mains, plus one multi-subtitle case, in values/tests.rs. Oracle spot-check on a style exercising structured titles (GitHub issue #1010 example).

Sizing: Sonnet-executable once the spec decision (mark set + locale field name) is confirmed.

## Update 2026-07-12

The "spec first" step is now tracked as `csl26-0vo3` (design
locale-configurable punctuation-collision system), which blocks this bean.
docs/specs/PUNCTUATION_NORMALIZATION.md now has a Recommended Design
section covering this — the suppression-set field this bean needs (default
"?!…") is meant to be the same field that design produces, not a second
one. Wait on `csl26-0vo3` before implementing.
