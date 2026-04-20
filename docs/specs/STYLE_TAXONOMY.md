# Style Taxonomy

**Status:** Active
**Version:** 1.0
**Date:** 2026-04-20
**Bean:** `csl26-v961`
**Related:** `STYLE_PRESET_ARCHITECTURE.md`

## Purpose

Define a four-tier classification for all Citum styles. The taxonomy drives registry `kind` annotations, embedding decisions, and verification strategies.

## Tiers

| Tier | Kind | Definition | `extends:` field | Verification |
|------|------|------------|-----------------|--------------|
| 1 | `base` | Complete style with full templates; serves as inheritance root | No | CSL oracle (citeproc-js) for CSL-derived; biblatex snapshot for biblatex-derived |
| 2 | `profile` | Adapts a base for a specific organization; may override options without full templates | Optional | Delta from its base; or direct oracle if self-contained |
| 3 | `journal` | Pure alias in the registry; no YAML file | N/A | Inherits from parent |
| 4 | `independent` | Complete style; no aliases; no inheritance role | No | Own oracle |

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
| `chicago-shortened-notes-bibliography` | chicago-notes-18th | Chicago variant |
| `elsevier-harvard` | — | Self-contained publisher profile |
| `elsevier-vancouver` | — | Self-contained publisher profile |
| `elsevier-with-titles` | — | Self-contained publisher profile |
| `springer-basic-author-date` | — | Self-contained publisher profile |
| `springer-basic-brackets` | — | Self-contained publisher profile |
| `springer-vancouver-brackets` | — | Self-contained publisher profile |
| `taylor-and-francis-chicago-author-date` | chicago-author-date-18th | Uses `extends:` |
| `taylor-and-francis-council-of-science-editors-author-date` | — | Self-contained |
| `taylor-and-francis-national-library-of-medicine` | — | Self-contained |

### Journal / Alias Styles (Tier 3)

Journal aliases are listed in `registry/default.yaml` under each entry's `aliases:` key. Each is a zero-config pointer to a profile or base style.

### Independent Styles (Tier 4)

Styles in `styles/*.yaml` that have no journal aliases and are not used as bases. Includes OSCOLA, MHRA variants, GOST, and similar discipline-specific styles.

## Embedding Policy

Only Tier 1 (base) and Tier 2 (profile) styles are embedded in the binary via `StyleBase`. Tier 3 (journal) styles resolve at runtime through the registry alias table. Tier 4 (independent) styles are loaded from disk or bundled separately.

---

## Changelog

- v1.0 (2026-04-20): Initial spec. Defines four-tier model (base, profile, journal, independent).
  Classifies all 16 embedded styles. Documents embedding policy.
