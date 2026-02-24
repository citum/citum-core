---
# csl26-b76h
title: Ibid, op cit, etc citation support
status: todo
type: task
priority: normal
created_at: 2026-02-24T07:37:47Z
updated_at: 2026-02-24T12:21:47Z
---

CSL 1.0 has support for ibid, but I recall people sometimes wanting more.

https://github.com/citation-style-language/schema/issues/68

The issue discussion claims word-processor APIs can't track pages and such, but it appears that may no longer be true:

- Microsoft Word: the modern Office JavaScript API exposes a `Range`’s page number and other pagination info via properties like `range.getHorizontalPositionRelativeToPage()`, `range.getLocation`, etc., which client code can use to infer on which page a given citation field sits.[1]
- LibreOffice: the UNO API exposes page style and layout information at the document model level, and extensions can run pagination-aware operations (used by features like page-number fields, indexes, and cross‑references), which demonstrates that the layout engine’s page segmentation is visible to extension code. [2][3]
- Evidence from practice: the thread itself notes that Citavi implements “first on page” ibid behavior in recent versions of its Word integration, which would not be possible if there were truly no usable pagination interface.

I see two possibly way to implement this:

1. Have the document side (Word or LibreOffice plugin, LuaLaTeX or Typst, etc.) send complex position info to the processor, which it then incorporates.
2. Much simpler, and likely would work: the processor send the needed style features and the terms to use for each in specific contexts as part of payload metadata, and let the document-side plugin do the substitution.

If 2 works, it would be a very simple addition.

[1] Citation range in Word - Microsoft Q&A https://learn.microsoft.com/en-us/answers/questions/5566620/citation-range-in-word
[2] LibreOffice Developer's Guide: Chapter 7 - Text Documents https://wiki.documentfoundation.org/Documentation/DevGuide/Text_Documents
