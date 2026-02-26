# CSL Style Targets: Multilingual and Sectional Bibliography

**Status:** Active - Implementation Pending  
**Date:** 2026-02-22  
**Beans:** csl26-mls1 (multilingual styles), csl26-grp1 (grouping styles)  
**Precondition:** csl26-mlt2 (processor multilingual logic) must be completed first

## Overview

This document records the priority style families identified as primary testbeds
for multilingual rendering and sectional bibliography grouping in Citum. It maps
those families to existing architecture and pending implementation tasks.

## Architecture Status

The schema and infrastructure are complete. Remaining work is processor logic
and concrete style YAML files + test fixtures for each family.

### Completed

- MultilingualConfig schema (title-mode, name-mode, preferred-script, scripts)
- BibliographyGroup schema (selector, sort, heading, disambiguate)
- SelectorEvaluator with type/field/cited-status predicates and negation
- GroupSorter with type-order, name-order (FamilyGiven/GivenFamily), composite
- Localizable GroupHeading (Literal, Term, Localized)
- Processor integration: render_grouped_bibliography_with_format
- Experimental styles: bluebook-legal.yaml, multilingual-academic.yaml

### Pending (Dependencies)

- csl26-mlt2: resolve_multilingual_string/name in citum_engine (HIGH - blocks all multilingual styles)
- csl26-mlt3: preferred-transliteration priority list field
- csl26-mlt4: CSL-M/Juris-M test fixture extraction
- csl26-extg: Document-level grouping override (Djot frontmatter/CLI)

## 1. Multilingual Style Targets (Bean: csl26-mls1)

Priority order for building YAML styles with multilingual config sections.

### 1.1 APA 7th (`styles/apa-7th.yaml`) - EXTEND EXISTING

**Rules (from APA Publication Manual 7th ed.):**
- Keep all reference elements in original language
- Add English translation of non-English title in square brackets when document locale is en
- Format: `Original title [English translation].`
- Exception: titles already in English need no bracketed translation

**YAML extension needed:**

```yaml
options:
  multilingual:
    title-mode: combined        # original [translation]
    name-mode: primary          # keep original script
    preferred-script: Latn
```

Test fixture: Add 2-3 references with non-English titles and English translations
to `tests/fixtures/multilingual/apa-multilingual.json`.

### 1.2 ISO-690 Family - NEW STYLES

Target files: `styles/iso690-author-date.yaml`, `styles/iso690-numeric.yaml`

**Rules (from biblatex-iso690 documentation):**
- Document language: used for labels, connectors, terms
- Resource language: used for titles, contributor names, publisher info
- No default-locale = processor resolves locale from document context
- Original title shown first; translated title in brackets if resource language differs from document language

**YAML pattern:**

```yaml
options:
  multilingual:
    title-mode: combined        # original [translation]
    name-mode: transliterated   # romanize non-Latin names for Latin-script documents
    preferred-script: Latn
```

Style note: ISO-690 is locale-agnostic by design; the style YAML should omit
default-locale and rely on the processor's document locale.

### 1.3 GOST R 7.0.5-2008 Family - NEW STYLES

Target files: `styles/gost-r-7-0-5-2008-numeric.yaml`,
`styles/gost-r-7-0-5-2008-author-date.yaml`

**Rules (from GOST 7.0.5-2008 and 7.1-2003 standards):**
- Default locale: ru-RU (Russian)
- Non-Russian titles: show original title, optionally add Russian translation in brackets
- Cyrillic/Latin script mixing: items in non-Cyrillic scripts may be transliterated

**YAML pattern:**

```yaml
options:
  multilingual:
    title-mode: combined        # original [translated-ru]
    name-mode: transliterated   # Cyrillic romanization for non-Russian names
    preferred-script: Cyrl      # default is Cyrillic for Russian docs
    scripts:
      Latn:
        use-native-ordering: false
```

### 1.4 Juris-M/JM Turabian Multilingual - NEW STYLE

Target file: `styles/experimental/jm-turabian-multilingual.yaml`

**Rules (from Juris-M tutorial, Journal of Asian Studies conventions):**
- Store both original and romanized forms for CJK names
- Sort bibliography by romanized names (given-family for Vietnamese)
- Display: original script with romanization in parentheses or vice versa
- Japanese names: Hepburn romanization (`ja-Latn-hepburn`)

**YAML pattern:**

```yaml
options:
  multilingual:
    title-mode: combined        # "romanized title [original script]"
    name-mode: transliterated   # show romanized, original in parens
    preferred-script: Latn
    scripts:
      Hans:
        use-native-ordering: true
        delimiter: ""
      Hant:
        use-native-ordering: true
        delimiter: ""
      Jpan:
        use-native-ordering: true
        delimiter: ""
```

### 1.5 MLA (`styles/modern-language-association.yaml`) - EXTEND EXISTING

**Rules (from MLA Handbook 9th ed.):**
- Title in original language, followed by translation in brackets
- Format: `Original Title [Translated Title].`
- Same pattern as APA but without italics on translated title

Status note: style already exists as `styles/modern-language-association.yaml`.

## 2. Sectional Bibliography Style Targets (Bean: csl26-grp1)

Priority order for building YAML styles with `bibliography.groups` sections.

### 2.1 Chicago Author-Date - EXTEND EXISTING

Target: `styles/chicago-author-date.yaml`

**Rules (Chicago Manual of Style 17th ed., section 14.59-14.63):**
- Primary sources: manuscripts, archival documents, interviews, unpublished
- Secondary sources: books, journal articles, chapters
- Optional: separate "Archival Sources" or "Newspapers" sections
- No requirement; depends on discipline and instructor/editor convention

**YAML groups extension:**

```yaml
bibliography:
  groups:
    - id: primary-sources
      heading:
        literal: "Primary Sources"
      selector:
        type: [manuscript, interview, personal_communication]

    - id: archival
      heading:
        literal: "Archival Sources"
      selector:
        type: [archival-document]

    - id: secondary
      heading:
        literal: "Secondary Sources"
      selector:
        not:
          type: [manuscript, interview, personal_communication, archival-document]
```

### 2.2 Juris-M Chicago Legal - NEW STYLE

Target: `styles/experimental/jm-chicago-legal.yaml`

**Rules (from Juris-M legal styles and The Bluebook):**
- Cases (legal-case) sorted by case name + court hierarchy
- Statutes (statute) sorted alphabetically by short title
- Treaties (treaty) sorted by date
- Secondary sources: books and articles alphabetically by author

**YAML groups pattern:**

```yaml
bibliography:
  groups:
    - id: cases
      heading:
        literal: "Cases"
      selector:
        type: legal-case
      sort:
        template:
          - key: title
            ascending: true

    - id: statutes
      heading:
        literal: "Statutes"
      selector:
        type: statute
      sort:
        template:
          - key: title

    - id: treaties
      heading:
        literal: "Treaties and International Agreements"
      selector:
        type: treaty
      sort:
        template:
          - key: issued

    - id: secondary
      heading:
        literal: "Secondary Sources"
      selector:
        not:
          type: [legal-case, statute, treaty]
```

### 2.3 Multilingual Academic with Groups - EXTEND EXISTING

Target: `styles/experimental/multilingual-academic.yaml` (already exists)

Enhancement: add explicit `disambiguate: locally` per language group so year
suffixes restart within each language section.

```yaml
bibliography:
  groups:
    - id: vietnamese
      heading:
        localized:
          vi: "Tài liệu tiếng Việt"
          en-US: "Vietnamese Sources"
      selector:
        field:
          language: vi
      sort:
        template:
          - key: author
            sort-order: given-family
      disambiguate: locally

    - id: western
      selector:
        not:
          field:
            language: vi
      sort:
        template:
          - key: author
            sort-order: family-given
      disambiguate: locally
```

### 2.4 GOST with Category Sections - ADD TO GOST STYLES

**Rules (Russian library science conventions):**
Standard GOST bibliographies are often organized by document category in Russian
academic and archival practice.

**YAML groups pattern:**

```yaml
bibliography:
  groups:
    - id: books
      heading:
        literal: "Книги"     # books
      selector:
        type: [book, edited-book]

    - id: articles
      heading:
        literal: "Статьи"    # articles
      selector:
        type: [article-journal, article-magazine]

    - id: legal
      heading:
        literal: "Нормативные документы"    # regulatory documents
      selector:
        type: [statute, legal-case, treaty]

    - id: other
      selector:
        not:
          type: [book, edited-book, article-journal, article-magazine, statute, legal-case, treaty]
```

## 3. Test Fixture Requirements

### Multilingual Fixtures

Location: `tests/fixtures/multilingual/`

Required fixture files:
- `multilingual-cjk.json`: 3 CJK references (Japanese, Chinese, Korean) with transliterations in Hepburn/Pinyin/McCune-Reischauer
- `multilingual-cyrillic.json`: 3 Russian references with ALA-LC transliterations
- `multilingual-mixed.json`: mixed-language bibliography for disambiguation testing (2 references with same romanized author name, different originals)

### Grouping Fixtures

Location: `tests/fixtures/grouping/`

Required fixture files:
- `legal-hierarchy.json`: cases, statutes, treaties (already exists per csl26-group phase 5)
- `primary-secondary.json`: manuscripts + journal articles for Chicago grouping test
- `multilingual-groups.json`: Vietnamese + English references for per-group sorting test (already exists per csl26-group phase 6, needs `disambiguate: locally` validation)

## 4. Dependency Graph

```text
csl26-mlt2 (processor multilingual resolve)
    └─► csl26-mls1 (this bean: multilingual style YAMLs + fixtures)
            └─► csl26-mlt3 (preferred-transliteration config)
            └─► csl26-mlt4 (Juris-M test fixture extraction)

csl26-group (COMPLETED)
    └─► csl26-grp1 (this bean: grouping style YAMLs + per-group disambig validation)
            └─► csl26-extg (document-level override, lower priority)
```

## 5. Success Criteria

### Multilingual (csl26-mls1)

- resolve_multilingual_string in processor resolves correct variant per MultilingualMode
- APA 7th renders non-English title + [English translation] with combined mode
- ISO-690 styles author-date and numeric created with locale-agnostic config
- GOST styles created with ru-RU default-locale and Cyrillic/Latin handling
- JM-Turabian experimental style created with CJK name ordering
- Multilingual test fixtures in `tests/fixtures/multilingual/` (CJK, Cyrillic, mixed)
- All existing oracle tests pass with no regressions (APA 7th 8/8 citations)

### Grouping (csl26-grp1)

- Chicago author-date extended with primary/secondary groups
- JM-Chicago-legal style created with cases/statutes/treaties/secondary groups
- Multilingual-academic style updated with `disambiguate: locally` per group
- GOST styles extended with category-based groups
- Integration tests using `process_document()` API for group validation
- Per-group disambiguation (locally scope) validated with mixed-year fixture

## References

- `/Users/brucedarcus/Code/csl26/docs/architecture/MULTILINGUAL.md`
- `/Users/brucedarcus/Code/csl26/docs/architecture/design/BIBLIOGRAPHY_GROUPING.md`
- `/Users/brucedarcus/Code/csl26/docs/architecture/DISAMBIGUATION_MULTILINGUAL_GROUPING.md`
- `/Users/brucedarcus/Code/csl26/crates/citum-schema/src/options/multilingual.rs`
- `/Users/brucedarcus/Code/csl26/crates/citum-schema/src/grouping.rs`
- `/Users/brucedarcus/Code/csl26/crates/citum-engine/src/grouping/selector.rs`
- `/Users/brucedarcus/Code/csl26/crates/citum-engine/src/grouping/sorting.rs`
- biblatex-iso690 documentation ([PDF](http://mirrors.ibiblio.org/CTAN/macros/latex/contrib/biblatex-contrib/biblatex-iso690/biblatex-iso690.pdf))
- Juris-M tutorial ([docs](https://juris-m.readthedocs.io/en/latest/tutorial.html))
