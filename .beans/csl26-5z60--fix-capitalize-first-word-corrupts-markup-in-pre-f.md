---
# csl26-5z60
title: 'Fix: capitalize_first_word corrupts markup in pre-formatted components'
status: in-progress
type: bug
priority: high
created_at: 2026-05-25T12:28:21Z
updated_at: 2026-05-25T12:28:21Z
---

capitalize_first_word in text_case.rs:124 treats rendered LaTeX/HTML/Typst markup as plain text. The bibliography sentence-initial pipeline applies case transforms after emph() formatting instead of before. Reproduction: texlua test_bib.lua → \Emph{521} vs expected \emph{521} (citum-labs). Fix: add capitalize_first_word_markup_aware gated behind pre_formatted flag. Parameterized rstest integration test across HTML, LaTeX, Typst, Plain formats.
