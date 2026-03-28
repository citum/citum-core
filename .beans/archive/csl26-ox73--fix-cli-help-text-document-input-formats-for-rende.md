---
# csl26-ox73
title: 'Fix CLI help text: document input formats for render/convert refs'
status: completed
type: task
priority: normal
created_at: 2026-03-28T14:37:53Z
updated_at: 2026-03-28T14:40:19Z
---

Fix CLI help text gaps:
- `render refs --help`: document that `--bibliography` accepts Citum YAML, Citum JSON, or CSL-JSON (auto-detected)
- `convert refs --help`: enumerate all six `RefsFormat` variants with one-line descriptions
- Add help text to `--from` and `--to` args in `ConvertRefsArgs`

Also refactor `input_reference_from_biblatex` to return `InputReference` directly (move `From<>` call inside), eliminating the `csl_json::Reference` pivot type from the public signature.

## Summary of Changes

- `render refs --help`: added INPUT FORMATS section documenting Citum YAML, Citum JSON, and CSL-JSON, with a note that BibLaTeX/RIS inputs require `citum convert refs` first
- `convert refs --help` / `ConvertRefsArgs`: added `long_about` enumerating all six `RefsFormat` variants with one-line descriptions and examples; clarified `--from`/`--to` help strings
- `input_reference_from_biblatex`: return type changed from `csl_json::Reference` to `InputReference`; `From<>` call moved inside the function

Deferred bean for post-1.0 CSL-JSON removal: csl26-o0s8
