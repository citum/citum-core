# Canonical Genre and Medium Values

**Version:** 1.0
**Date:** 2026-03-29
**Related:** [ENUM_VOCABULARY_POLICY.md](../policies/ENUM_VOCABULARY_POLICY.md)

## Overview

`genre` and `medium` are free-text `Option<String>` fields on reference types.
Both fields use **kebab-case canonical identifiers** as their stored form.
Matching at the application layer should be case-insensitive, and new
integrations SHOULD perform case-normalization before comparison.
Producers SHOULD emit the canonical forms listed here.
(Full case-insensitive normalization is tracked in bean csl26-qqfa.)

Canonical values are advisory, not exhaustive. Any value may be stored. Styles
that branch on `genre` or `medium` should handle unrecognised values gracefully
(typically by rendering the raw string as-is).

New canonical values are added by PR to this file only — no Rust code change is
required.

---

## Genre

`genre` describes the intellectual or documentary nature of a work within its
reference type. It provides finer discrimination than the type alone (e.g.,
distinguishing a PhD thesis from a master's thesis, or a short film from a
feature film).

| Canonical value | Human-readable (en) | Notes |
|---|---|---|
| `phd-thesis` | PhD thesis | Doctoral dissertation |
| `masters-thesis` | Master's thesis | Graduate thesis below doctoral level |
| `undergraduate-thesis` | Undergraduate thesis | Honours or senior thesis |
| `technical-report` | Technical report | Numbered reports from institutions |
| `annual-report` | Annual report | Corporate or institutional annual report |
| `assessment-report` | Assessment report | Commissioned or evaluative report |
| `working-paper` | Working paper | Pre-publication / discussion paper |
| `white-paper` | White paper | Policy or technical position paper |
| `conference-paper` | Conference paper | Presented at a named conference |
| `short-film` | Short film | Cinematographic work ≤40 min |
| `feature-film` | Feature film | Cinematographic work >40 min |
| `documentary` | Documentary | Non-fiction film or video |
| `television-episode` | Television episode | Single episode of a series |
| `radio-broadcast` | Radio broadcast | Audio broadcast episode or programme |
| `podcast-episode` | Podcast episode | Single episode of a podcast series |
| `video-interview` | Video interview | An interview recorded and distributed as video |
| `manuscript-scroll` | Manuscript scroll | Antique scroll manuscript |
| `holograph-manuscript` | Holograph manuscript | Handwritten original document |
| `letter` | Letter | Personal or official correspondence |
| `email` | Email | Electronic mail message |
| `preprint` | Preprint | Pre-peer-review version (use with `Monograph(Preprint)`) |
| `dataset` | Dataset | Research dataset (use with `Dataset` type) |
| `software` | Software | Software release (use with `Software` type) |
| `map` | Map | Cartographic work |
| `artwork` | Artwork | Visual or fine art work |
| `score` | Musical score | Notated music |
| `sound-recording` | Sound recording | Audio recording of music or speech |
| `patent` | Patent | Use with `Patent` type; `genre` for sub-type (e.g., design patent) |
| `standard` | Standard | Use with `Standard` type |

### Fixture values (not yet normalized)

The following values appear in test fixtures and have not yet been migrated to
kebab-case canonical form. They will be normalized in a future data migration
pass (tracked as bean csl26-qqfa).

| Current fixture value | Field | Canonical target |
|---|---|---|
| `"PhD thesis"` | `genre` | `"phd-thesis"` |
| `"PhD dissertation"` | `genre` | `"phd-thesis"` |
| `"Technical report"` | `genre` | `"technical-report"` |
| `"Annual report"` | `genre` | `"annual-report"` |
| `"Assessment report"` | `genre` | `"assessment-report"` |
| `"Short film"` | `genre` | `"short-film"` |
| `"Manuscript scroll"` | `genre` | `"manuscript-scroll"` |
| `"Holograph manuscript"` | `genre` | `"holograph-manuscript"` |
| `"Letter"` | `genre` | `"letter"` |
| `"Video interview"` | `medium` | move to `genre: "video-interview"`; set `medium` to appropriate channel |

---

## Medium

`medium` describes how a work is delivered or experienced. Values cover both
physical carriers (the object in your hands: `film`, `dvd`, `vinyl`) and
distribution channels (how you receive or access it: `television`,
`streaming`, `print`). This mirrors the unified-medium convention of MLA,
APA, and Chicago, which use a single field for both concepts.

When choosing a value, prefer the most specific applicable term. If both
carrier and channel are relevant to the same work, prefer the channel — the
channel describes how the citing author experienced the work, which is what
citation styles care about (e.g., use `streaming` rather than `blu-ray` for a
documentary watched on a streaming service).

### Physical carriers

| Canonical value | Human-readable (en) | Notes |
|---|---|---|
| `film` | Film | Theatrical or archival film print |
| `video` | Video | Generic digital video; use a more specific channel if known |
| `audio-cd` | Audio CD | Compact disc audio |
| `vinyl` | Vinyl | Vinyl record |
| `dvd` | DVD | Digital Video Disc |
| `blu-ray` | Blu-ray | Blu-ray disc |
| `vhs` | VHS | Video Home System tape |
| `microfiche` | Microfiche | Micrographic storage medium |
| `manuscript` | Manuscript | Handwritten or typed physical document |
| `photograph` | Photograph | Photographic print or slide |

### Distribution channels and access modes

| Canonical value | Human-readable (en) | Notes |
|---|---|---|
| `television` | Television | Broadcast or cable television |
| `radio` | Radio | Broadcast radio |
| `streaming` | Streaming | Online streaming service (audio or video) |
| `online` | Online | General digital / web access |
| `print` | Print | Physical printed publication |

### Fixture values (not yet normalized)

| Current fixture value | Field | Canonical target |
|---|---|---|
| `"Television"` | `medium` | `"television"` |
| `"film"` | `medium` | already canonical |
| `"Video interview"` | `medium` | move to `genre: "video-interview"` (see Genre section above) |

---

## Localization

The canonical keys in this table serve as locale-map keys. A future
`locale/en/vocab.yaml` would provide:

```yaml
genre:
  phd-thesis: "PhD thesis"
  short-film: "Short film"
  video-interview: "Video interview"
medium:
  film: "Film"
  television: "Television"
  streaming: "Streaming"
```

The stored value in YAML/JSON/CBOR is always the canonical kebab-case key;
display text is resolved at render time by the locale layer. See
[ENUM_VOCABULARY_POLICY.md](../policies/ENUM_VOCABULARY_POLICY.md) for the
full localization policy.
