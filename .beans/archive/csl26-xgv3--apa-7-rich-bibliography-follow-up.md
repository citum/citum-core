---
# csl26-xgv3
title: APA 7 rich bibliography follow-up
status: completed
type: task
priority: normal
created_at: 2026-04-09T12:48:29Z
updated_at: 2026-04-09T18:30:00Z
---

Follow up `csl26-qh84` after the APA rich bibliography benchmarks were wired
into official reporting.

Current verified state after the `csl26-qh84` co-evolution pass:
- primary APA gate remains green at 40 / 40 citations
- style-scoped report keeps `apa-7th` at headline fidelity `1.0`
- diagnostic APA test-library benchmark lands at 34 / 75

Bounded reduced-cluster evidence from the rich APA benchmark now shows a much
smaller residual surface:
- focused broadcast / motion-picture / interview / entry-reference cluster
  improved to 22 / 24
- resolved in `csl26-qh84`: podcast episode container routing, producer-led TV
  series contributor fallback, richer interview packaging, no-regression APA
  primary-gate preservation
- remaining residuals: one anonymous Wikipedia-style entry ordering/path issue
  and one placeholder extra-entry row citeproc currently omits

## Tasks

- [x] Classify the remaining APA residuals as style-defect, processor-defect,
  malformed-source exclusion, or intentional divergence.
- [x] Choose one bounded APA cluster for a `style-evolve` pass using reduced
  evidence first.
- [x] Land one net APA supplemental bibliography gain without regressing the
  40 / 40 primary gate.
- [x] Summarize any residual non-style defects in successor beans if needed.

## Summary of Changes

Confirmed the decisive APA residual as a processor-side dataset intake/render
gap rather than the already-covered Wikipedia anonymous-entry path.

Preserved titleless dataset metadata from CSL-JSON intake, synthesized the APA
fallback title/version text for that narrow case, and let APA bibliography URL
rendering fall back to DOI URLs for datasets without a standalone URL.

Added an engine bibliography regression that exercises the titleless APA
dataset shape directly and verifies the rendered entry no longer collapses to a
bare author/year stub.
