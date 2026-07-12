---
# csl26-0vo3
title: Design locale-configurable punctuation-collision system
status: completed
type: task
priority: normal
tags:
    - punctuation
    - rendering
    - locale
    - schema
created_at: 2026-07-12T18:51:44Z
updated_at: 2026-07-12T20:42:36Z
blocking:
    - csl26-zfqr
---

Unify and finish the design work docs/specs/PUNCTUATION_NORMALIZATION.md
has flagged since 2026-02-15 but never resolved: the punctuation-*collision*
half of the problem (adjacent marks at join points, e.g. "Titel!," should
collapse to "Titel!" in German) as opposed to the quote-*movement* half the
spec already models.

Motivated by two independent paths landing on the same need:
- Upstream CSL schema#379 ("Make punctuation collapsing localisable"),
  tracked as bucket-1-partial in
  docs/architecture/audits/2026-07-12_CSL_SCHEMA_ISSUE_TRIAGE.md — Citum's
  current resolve_punctuation_collision (citation.rs:15-55) and
  DANGLING_PUNCTUATION_PATTERNS (bibliography.rs:374-390) are both
  hardcoded, English-only, with no locale hook.
- csl26-zfqr (structured-title delimiter suppression after terminal
  punctuation), whose own fix design says "spec first" and already proposes
  the same kind of mechanism (a locale-overridable terminal-mark set,
  default "?!…", in grammar-options).

The spec now has a "Recommended Design" section: narrow, named
grammar-options fields (locale default + per-style override, matching the
existing note-punctuation/note-number/note-marker-order precedent) rather
than a general pattern-rewrite table (the CSL-M `<punct-handling>`
proposal, deliberately rejected — see spec for reasoning). This bean is the
concrete design decision + implementation that section leaves open.

- [x] Finalize exact grammar-options field names and the full set of
      class-pair collision policies (spec sketches StrongTerminal/WeakTerminal/
      CommaLike classes and a strong-plus-comma-policy field; needs a
      final decision, not just a sketch)
- [x] Confirm the terminal-mark suppression-set field csl26-zfqr needs
      (default "?!…") is the same field this design produces, not a
      second competing one
- [x] Add new field(s) to citum-schema-style with #[serde(default)],
      regenerate JSON schema in the same commit (per
      crates/citum-schema/CLAUDE.md)
- [x] Update docs/guides/AUTHORING_LOCALES.md's grammar-options example
- [x] Populate en-US.yaml (and any other embedded locales) with defaults
- [x] Wire the new field(s) into resolve_punctuation_collision and/or
      DANGLING_PUNCTUATION_PATTERNS as appropriate
- [x] Flip docs/specs/PUNCTUATION_NORMALIZATION.md Status to Active in the
      implementation commit
- [x] French spacing (NBSP/narrow-NBSP before : ; ! ? and guillemets) is
      scoped OUT of this pass per the spec — file as a separate follow-up
      if still wanted after this lands

## Summary of Changes

- Added locale defaults and direct style overrides for strong-terminal comma
  collisions, with compatibility-first English behavior.
- Shared the resolved policy across citation and bibliography joins, including
  markup-aware dangling-punctuation cleanup.
- Defined `delimiter-suppressing-terminal-marks` for `csl26-zfqr`; structured
  title consumption remains part of that separate bean.
- Activated the punctuation-normalization spec, updated locale documentation,
  and regenerated the public locale and style schemas.
- Kept French spacing outside this implementation as specified.
