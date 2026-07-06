---
# csl26-ztxq
title: Format-aware punctuation boundary detection
status: todo
type: task
priority: normal
tags:
    - punctuation
    - rendering
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-06T18:45:57Z
parent: csl26-8m2p
---

visible_text strips only HTML tags, so separator/dedup logic misfires for LaTeX/Typst/Markdown (\emph{Title.} ends in }, producing 'Title.. Next'), violating the backends-differ-only-in-markup rule. cleanup_dangling_punctuation also runs global find/replace over full marked-up entries including attributes. Add a visible-text/logical-last-char hook to OutputFormat (or track logical boundaries in ProcTemplateComponent) and constrain the cleanup pass to text outside markup. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 13.

## Root Cause (2026-07-06 review)

Confirmed at source: `render/bibliography.rs::visible_text` (line 26) strips only HTML `<...>` tags. All punctuation-boundary helpers (`first_visible_char`, `last_visible_non_space_char`, `ends_with_sentence_ending_visible_punctuation`) build on it, so for LaTeX/Typst/Markdown/Djot fragments the 'last visible char' is markup (`}`, `_`, `*`), and separator/dedup decisions misfire (\emph{Title.} → 'Title.. Next'). Independently, `cleanup_dangling_punctuation` (line 386) runs a fixed-point global find/replace over the fully marked-up entry, so patterns like \" ,\" / '  ' can rewrite inside markup, attributes, and URLs.

## Fix Design

1. **OutputFormat hook:** add `fn visible_text<'a>(&self, fragment: &'a str) -> Cow<'a, str>` to the `OutputFormat` trait with per-backend implementations: HTML = current tag strip; PlainText = passthrough; LaTeX = strip \\command tokens and brace delimiters (keep brace contents); Typst = strip `#func[...]` wrappers and emphasis markers; Markdown/Djot = strip emphasis/strong markers and link syntax keeping link text. Default impl = passthrough so third-party formats degrade safely.
2. **Thread the format through** the boundary helpers in bibliography.rs (they become generic over F like their callers already are) and any citation.rs users of the same pattern.
3. **Constrain the cleanup pass:** rewrite `cleanup_dangling_punctuation` to operate per visible segment — lex the fragment with the backend's markup boundaries (reuse the visible_text lexer, but map spans instead of discarding them) and apply the pattern table only inside visible runs. If span-mapping is too invasive for a first commit, an acceptable intermediate is: apply the pass only for PlainText/HTML (current behavior) and skip it for LaTeX/Typst/Markdown until spans land — that converts silent corruption into a known, documented gap.
4. **Tests:** per-backend rstest matrix (same entry rendered via each format asserts identical `visible_text` output — the backends-differ-only-in-markup rule from DESIGN_PRINCIPLES), plus regression cases from engine review part2 finding 13 ('Title.. Next' in LaTeX, marker-terminated Typst emphasis, URL containing ', .').

Note: csl26-zfqr no longer depends on this work — its check moved to source text (see that bean). Sizing: item 1-2 Sonnet-executable; item 3 span-mapping needs a review pass.
