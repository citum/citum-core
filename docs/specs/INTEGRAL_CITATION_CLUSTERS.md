# Integral Citation Clusters Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-06-03
**Related:** `crates/citum-schema-data/src/citation.rs`,
`crates/citum-engine/src/processor/citation.rs`,
`docs/specs/INTEGRAL_NAME_MEMORY.md`,
`docs/specs/REPEATED_NOTE_CITATION_STATE_MODEL.md`

---

## Purpose

Define the authoritative vocabulary for the in-text citation model, document
the default rendering behavior for multi-work narrative citations, specify the
`sentence_start` signal for sentence-initial capitalization, and record future
extension points.

---

## Terminology and Glossary

| Concept | Citum name | Serialized form | Notes |
|---|---|---|---|
| One in-text citation event | **citation cluster** | `Citation` (struct) | A single call-out in the text; may cite one or more works |
| One reference within a cluster | **cite** / **citation item** | `CitationItem` (struct) | Carries the citekey, locator, item-level prefix/suffix |
| Narrative / in-text form | **integral** | `"mode": "integral"` | Author rendered inline; e.g. "Smith (2020) argues…" |
| Parenthetical form | **non-integral** | `"mode": "non-integral"` (default) | Author inside parens; e.g. "(Smith, 2020)" |

### Key invariant: mode is cluster-scoped

`CitationMode` (`Integral` / `NonIntegral`) is a property of the **cluster**,
not of individual citation items. A mixed-mode cluster — some items narrative,
others parenthetical — is typographically incoherent and is not supported.
`Citation.mode` applies uniformly to all items in `Citation.items`. The same
invariant applies to `suppress_author`: per-item suppression is not supported.

---

## Multi-Work Narrative Clusters (default behavior)

When a cluster cites multiple works in integral/narrative mode, the default
rendering is **list-like prose**:

> Smith (2020) and Jones (2021) both argue…

> Smith (2020), Jones (2021), and Brown (2022) all contend…

The join is locale-aware (conjunction term, optional serial comma) and is
handled by `join_integral_groups` in
`crates/citum-engine/src/processor/citation.rs`.

### Non-goal: cluster-like (semicolon) narrative form

A style might conceptually want to preserve semicolons in a narrative cluster:

> Smith (2020); Jones (2021) both argue…

This form is **not currently supported** and is not on the roadmap. If a
concrete style requirement arises, it can be implemented as an opt-in under
`citation.integral.multi-cite-form: cluster` (list-like remains the default)
using the existing `multi-cite-delimiter` override as an internal escape hatch.
Until a real request exists, that escape hatch is intentionally undocumented.

---

## Cluster-Level Affixes ("see also", prefixes)

Cluster-level affixes are placed by **conventional position relative to the
citation wrap**:

| Mode | Affix placement | Example |
|---|---|---|
| Non-integral (parenthetical) | **Inside** the parentheses | `(see also Smith 2020)` |
| Integral (narrative) | **Outside** / part of prose | `see also Smith (2020)` |

This is the current implemented behavior (`apply_citation_input_affixes` +
`apply_spec_wrap_and_affixes` in `citation.rs`) and is the documented contract.

### Future enhancement: configurable affix placement

Some styles may want to override the convention (e.g. keep affixes inside
parentheses even for narrative clusters). This can be added as
`citation.affix-placement: inside | outside` (defaulting to the conventional
behavior) if users request it. This is **not currently implemented**.

---

## Sentence-Initial Capitalization (`sentence_start`)

### The problem

When an integral cluster begins a sentence and the cluster starts with a
lowercase affix, the leading character must be capitalized:

```
# With prefix "see also" (lowercase)
    integral, mid-sentence:    see also Smith (2020) argues …
    integral, sentence-start:  See also Smith (2020) argues …
```

The standalone processor cannot detect sentence position from rendered text. An
author-led cluster (e.g. "Smith (2020)") is already capitalized by virtue of
the author's name, so this only matters when a lowercase element leads.

### Prior art: natbib / biblatex capitalized commands

LaTeX resolves the same ambiguity by requiring the author to choose a command
variant at the call site:

| System | Lowercase | Capitalized |
|---|---|---|
| natbib | `\citet{key}` | `\Citet{key}` |
| biblatex | `\parencite{key}` | `\Parencite{key}` |
| biblatex | `\textcite{key}` | `\Textcite{key}` |
| biblatex | `\citeauthor{key}` | `\Citeauthor{key}` |

The capital variant signals to the formatter that the citation opens a sentence
and any lowercase prefix (or name particle such as "von" / "van der") should be
capitalized. Citum adopts the same explicit-signal model: the *host* supplies
the flag; the *processor* applies the transform.

### `Citation.sentence_start`

```rust
pub sentence_start: bool,  // default false
```

When `true`, the engine applies `CapitalizeFirst` to the **first alphabetic
character** of the fully-composed cluster output (after affixes and wrap).
The markup-aware variant is used, so leading punctuation (e.g. an opening
parenthesis), HTML tags, and LaTeX/Typst command prefixes are skipped rather
than corrupted:

```
"(see also Smith 2020)"   →  "(See also Smith 2020)"
"see also Smith (2020)"   →  "See also Smith (2020)"
"Smith (2020)"            →  "Smith (2020)"   (no-op: already capital)
```

The transform is applied regardless of citation mode and is a no-op when the
composed output already starts with a capital letter.

### Who sets the flag?

The flag is **caller-supplied**, not auto-detected:

- **Document pipeline / `format_document`**: the Djot/Markdown document
  processor can inspect surrounding text to auto-populate the flag. This is a
  **deferred follow-up** (see below).
- **WASM bridge / hub editor**: the editor knows cursor position and can set
  `sentence_start` when inserting a citation after a sentence boundary.
- **FFI callers**: pass `sentence_start: true` when the citation is known to
  begin a sentence.
- **Standalone `process_citation` API**: default `false`; callers that know
  the context set it explicitly.

### Deferred: auto-detection in the document pipeline

Automatic detection of sentence boundaries in `format_document` (Djot/Markdown)
is a follow-up task. Until implemented, the flag must be set explicitly by the
host. This matches natbib/biblatex behavior: the author chooses the form.
