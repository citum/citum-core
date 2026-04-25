---
# csl26-k7nf
title: Typst Citation Interactivity Follow-up
status: todo
type: research
priority: normal
tags:
    - research
created_at: 2026-03-01T15:12:10Z
updated_at: 2026-04-25T20:20:07Z
blocking:
    - csl26-93yh
---

## Overview

Determine what additional citation interactivity is realistically possible after `csl26-93yh`, with special attention to Typst-generated PDFs versus the already-planned interactive HTML path.

## Goals

* Confirm whether newer Typst or PDF annotation APIs allow custom citation hover metadata.
* Confirm whether any viewer-portable tooltip or popup mechanism can be generated from Typst.
* Evaluate whether a subtle always-on citation color treatment is preferable to unsupported hover-only styling in PDF output.
* Define the split between PDF-safe enhancements and richer HTML-only enhancements.

## Questions to Answer

1. Can Typst emit any custom PDF annotations suitable for citation metadata popups?
2. If not, is there a stable way to attach viewer-visible metadata beyond the default link destination tooltip?
3. Should citation affordances in Typst default to static styling instead of interaction semantics that PDFs do not support reliably?
4. Which interaction features should be documented as HTML-only rather than promised for Typst/PDF?

## Deliverables

* Short technical note in `docs/architecture/` summarizing supported versus unsupported interaction affordances.
* Recommendation on whether to add static citation styling options to Typst output.
* Recommendation on whether citation metadata previews should live exclusively in the HTML renderer path.

## Success Criteria

* Clear yes/no answers on Typst/PDF support for hover styling and metadata popups.
* A documented product decision on where advanced citation interactivity belongs.
* No ambiguity in user-facing docs about what Typst/PDF can and cannot do.
