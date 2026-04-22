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
| 3 | `journal` | Alias or config-only wrapper for a specific journal | Optional | Inherits from parent |
| 4 | `independent` | Complete style; no aliases; no inheritance role | No | Own oracle |

## Profile Rule

`profile` is both a semantic taxonomy and an implementation contract.

A style is a `profile` only when the authority chain shows that it follows a
known publisher, society, or standards parent with bounded house deltas, and
its local YAML remains config-only. The authority order is:

1. publisher or journal guide
2. publisher house rules
3. named parent-style manual or standards reference
4. CSL/template-link evidence
5. current Citum YAML structure

Output similarity by itself is not enough.

## Implementation Note

Profile styles now use config-only wrappers over hidden compiled roots.

- public `kind: profile` styles may keep local identity and use the normal
  typed options surface
- profile-only schema namespaces are forbidden
- public `kind: profile` styles may not keep local templates, `type-variants`,
  or template-clearing `null` values
- hidden compiled roots are an implementation detail and do not appear in the
  public registry

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
| `chicago-shortened-notes-bibliography` | hidden Chicago shortened-notes root | Public shortened-notes wrapper |
| `elsevier-harvard` | hidden Elsevier Harvard root | Public profile handle is config-only |
| `elsevier-vancouver` | hidden Elsevier Vancouver root | Public profile handle is config-only |
| `elsevier-with-titles` | hidden Elsevier with-titles root | Public profile handle is config-only |
| `springer-basic-author-date` | hidden Springer Basic author-date root | Public profile handle is config-only |
| `springer-basic-brackets` | hidden Springer Basic brackets root | Public profile handle is config-only |
| `springer-vancouver-brackets` | hidden Springer Vancouver root | Public profile handle is config-only |
| `taylor-and-francis-chicago-author-date` | hidden Taylor & Francis Chicago root | Public profile handle is config-only |
| `taylor-and-francis-council-of-science-editors-author-date` | hidden Taylor & Francis CSE root | Public profile handle is config-only |
| `taylor-and-francis-national-library-of-medicine` | hidden Taylor & Francis NLM root | Public profile handle is config-only |

### Journal / Alias Styles (Tier 3)

Journal aliases are listed in `registry/default.yaml` under each entry's `aliases:` key. Each is a zero-config pointer to a profile or base style.

Like Tier 2 styles, Tier 3 styles can use `extends:` to inherit a base structure while applying journal-specific config-only overrides (e.g., localized delimiters or specific locator labels).

### Profile Candidates (Backlog)

The following styles have been identified via automated semantic skeleton analysis as high-probability candidates for `extends:` conversion:

| Legacy Style | Target Base | Semantic Similarity |
|--------------|-------------|---------------------|
| `pharmacoepidemiology-and-drug-safety` | `elsevier-with-titles` | 0.84 |
| `disability-and-rehabilitation` | `elsevier-with-titles` | 0.83 |
| `zoological-journal-of-the-linnean-society` | `springer-basic-author-date` | 0.83 |
| `the-lichenologist` | `springer-basic-author-date` | 0.83 |
| `memorias-do-instituto-oswaldo-cruz` | `springer-basic-author-date` | 0.83 |
| `techniques-et-culture` | `taylor-and-francis-cse` | 0.87 |
| `hawaii-int-conf-system-sciences` | `taylor-and-francis-nlm` | 0.86 |
| `cell-numeric` | `elsevier-with-titles` | 0.83 |

## Embedding Policy

Only Tier 1 (base) and Tier 2 (profile) styles are embedded in the binary. Tier
3 (journal) styles resolve at runtime through the registry alias table. Tier 4
(independent) styles are loaded from disk or bundled separately.

---

## Changelog

- v1.4 (2026-04-22): Replaced `options.profile` with normal scoped options.
- v1.2 (2026-04-21): Enforced config-only profile wrappers over hidden compiled
  roots.
- v1.1 (2026-04-21): Clarified that `profile` means evidence-backed parentage,
  not output similarity or already-small YAML.
- v1.0 (2026-04-20): Initial spec. Defines four-tier model (base, profile, journal, independent).
  Classifies all 16 embedded styles. Documents embedding policy.
