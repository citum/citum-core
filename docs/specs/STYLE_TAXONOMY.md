# Style Taxonomy

**Status:** Active
**Version:** 1.1
**Date:** 2026-04-21
**Bean:** `csl26-v961`, `csl26-nrkn`
**Related:** `STYLE_PRESET_ARCHITECTURE.md`, `../architecture/2026-04-21_PROFILE_WRAPPER_VALIDATION_PASS.md`

## Purpose

Define a four-tier classification for all Citum styles. The taxonomy drives registry `kind` annotations, embedding decisions, and verification strategies.

## Tiers

| Tier | Kind | Definition | `extends:` field | Verification |
|------|------|------------|-----------------|--------------|
| 1 | `base` | Complete style with full templates; serves as inheritance root | No | CSL oracle (citeproc-js) for CSL-derived; biblatex snapshot for biblatex-derived |
| 2 | `profile` | Evidence-backed parent-plus-deltas style for a publisher, society, or standards body | Optional | Delta from its parent when a meaningful wrapper exists; otherwise direct oracle plus parent-diff evidence |
| 3 | `journal` | Pure alias in the registry; no YAML file | N/A | Inherits from parent |
| 4 | `independent` | Complete style; no aliases; no inheritance role | No | Own oracle |

## Profile Rule

`profile` is a semantic taxonomy, not a claim that the current YAML is already a
small `extends:` wrapper.

A style is a `profile` when the authority chain shows that it follows a known
publisher, society, or standards parent with bounded house deltas. The authority
order is:

1. publisher or journal guide
2. publisher house rules
3. named parent-style manual or standards reference
4. CSL/template-link evidence
5. current Citum YAML structure

Output similarity by itself is not enough.

## Implementation Note

Some public profile styles still remain bulkier than their taxonomy label would
ideally suggest.

That can happen for two reasons:

- the correct family root is not yet authored as a distinct Citum base
- the current `extends:` merge contract makes some child deltas expensive to
  express even when the parent relationship is real

This second point is a real limitation of the current authoring model, not just
an observation about file size. As documented in `STYLE_PRESET_ARCHITECTURE.md`,
objects deep-merge, but arrays and explicit `null` values replace inherited
content wholesale. In practice, bibliography templates and many
`type-variants` contain nested arrays or replace-whole structures, so changing
one child-specific component can force the child style to restate most of the
inherited block.

That is why taxonomy `profile` and “thin wrapper in YAML” can diverge. A style
can have valid evidence-backed parentage and still remain bulky because the
current inheritance model cannot express the delta compactly enough. This does
not mean the current merge behavior is wrong; it means wrapper compression may
require follow-up design work when the project wants more compact
parent-plus-delta authoring.

For that reason, a style may remain `kind: profile` in `registry/default.yaml`
while still carrying self-contained YAML today. Conversely, the presence of a
compiled `StyleBase` key is an implementation detail, not taxonomic proof that a
style is Tier 1 `base`.

## Current Classification

### Base Styles (Tier 1)

| Style | Origin |
|-------|--------|
| `apa-7th` | CSL-derived |
| `chicago-notes-18th` | CSL-derived |
| `chicago-author-date-18th` | CSL-derived |
| `ieee` | CSL-derived |
| `american-medical-association` | CSL-derived |
| `modern-language-association` | CSL-derived |

### Profile Styles (Tier 2)

| Style | Base | Notes |
|-------|------|-------|
| `chicago-shortened-notes-bibliography` | chicago-notes-18th | Proven shortened-notes wrapper |
| `elsevier-harvard` | pending dedicated Elsevier author-date base | Public profile handle currently carries family-root behavior |
| `elsevier-vancouver` | pending dedicated Elsevier numeric/NLM base | Public profile handle currently carries family-root behavior |
| `elsevier-with-titles` | pending dedicated Elsevier numeric-with-titles base | Public profile handle currently carries family-root behavior |
| `springer-basic-author-date` | pending dedicated Springer Basic family base | Public profile handle currently carries family-root behavior |
| `springer-basic-brackets` | springer-basic-author-date | Evidence-backed child of the Springer Basic author-date profile; still bulky because current merge rules force large bibliography/type-variant restatement |
| `springer-vancouver-brackets` | pending dedicated Springer Vancouver/NLM family base | Public profile handle currently carries family-root behavior |
| `taylor-and-francis-chicago-author-date` | chicago-author-date-18th | Guide-backed Chicago derivative; uses `extends:` |
| `taylor-and-francis-council-of-science-editors-author-date` | pending dedicated CSE family base | Standards-backed public profile still carried as self-contained YAML |
| `taylor-and-francis-national-library-of-medicine` | pending dedicated NLM family base | Standards-backed public profile still carried as self-contained YAML |

### Journal / Alias Styles (Tier 3)

Journal aliases are listed in `registry/default.yaml` under each entry's `aliases:` key. Each is a zero-config pointer to a profile or base style.

### Independent Styles (Tier 4)

Styles in `styles/*.yaml` that have no journal aliases and are not used as bases. Includes OSCOLA, MHRA variants, GOST, and similar discipline-specific styles.

## Embedding Policy

Only Tier 1 (base) and Tier 2 (profile) styles are embedded in the binary. Tier
3 (journal) styles resolve at runtime through the registry alias table. Tier 4
(independent) styles are loaded from disk or bundled separately.

---

## Changelog

- v1.1 (2026-04-21): Clarified that `profile` means evidence-backed parentage,
  not output similarity or already-small YAML. Recorded the semantic vs
  implementation distinction for bulky public profiles.
- v1.0 (2026-04-20): Initial spec. Defines four-tier model (base, profile, journal, independent).
  Classifies all 16 embedded styles. Documents embedding policy.
