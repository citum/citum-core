# Journal-Profile Candidate Audit

**Date:** 2026-04-22
**Related:** `docs/specs/JOURNAL_PROFILE_TAXONOMY_AUDIT.md`, `docs/specs/STYLE_TAXONOMY.md`, commit `9e13a17b`

## Summary

This pass audited the eight backlog entries added in `9e13a17b` against four
evidence surfaces:

1. current guide or current public journal signal
2. CSL metadata and `rel="template"` links
3. `scripts/report-data/alias-candidates-2026-04-19.tsv`
4. `cargo run --bin citum-analyze -- styles-legacy --identify-profiles --json`

Outcome summary:

- `3` promoted to journal config-wrapper
- `0` promoted to alias
- `0` retained as structural descendants
- `5` demoted to false positives or unsupported parent links

This means the `9e13a17b` pass was useful for family triage, but it needed a
second guide-driven reduction pass before any inheritance claim was trustworthy.

## Evidence Table

| Candidate | Proposed Parent | Corrected Parent | Current guide or public signal | CSL metadata | Alias TSV | Analyzer audit | Outcome |
|-----------|-----------------|------------------|-------------------------------|--------------|-----------|----------------|---------|
| `pharmacoepidemiology-and-drug-safety` | `elsevier-with-titles` | `american-medical-association` | Wiley author-guidance link remains the concrete public style pointer in CSL metadata. | `rel="template"` points to AMA. | Best target `american-medical-association`, similarity `0.9881`, cite `1.00`, bib `0.00`. | Structural best match `elsevier-with-titles` with combined `0.92`, but authority evidence favors AMA. | `journal + config-wrapper` |
| `disability-and-rehabilitation` | `elsevier-with-titles` | `elsevier-with-titles` | The surviving public style evidence is the journal-specific Taylor & Francis PDF linked from CSL metadata. | `rel="template"` points to `cse-citation-sequence`. | Best target `elsevier-with-titles`, similarity `1.00`, cite `1.00`, bib `0.00`. | Structural best match `elsevier-with-titles` with combined `0.92`. | `journal + config-wrapper` |
| `zoological-journal-of-the-linnean-society` | `springer-basic-author-date` | — | Current OUP instructions still describe the journal as its own Linnean/OUP style surface, not a Springer family style: [General Instructions](https://academic.oup.com/zoolinnean/pages/General_Instructions). | `rel="template"` points to `biological-journal-of-the-linnean-society`. | Best target `springer-basic-author-date`, similarity `1.00`, cite `0.15`, bib `0.00`. | Structural best match `springer-basic-author-date` with combined `0.92`, but authority evidence contradicts the family mapping. | `false-positive` |
| `the-lichenologist` | `springer-basic-author-date` | — | Current Cambridge journal guidance is reachable and does not support a Springer-family relationship: [Author instructions](https://www.cambridge.org/core/journals/lichenologist/information/author-instructions). | no template parent in CSL metadata | Best target `elsevier-harvard`, similarity `0.9746`, cite `0.05`, bib `0.00`. | Structural best match `springer-basic-author-date` with combined `0.92`, but output evidence is weak and public authority contradicts the mapping. | `false-positive` |
| `memorias-do-instituto-oswaldo-cruz` | `springer-basic-author-date` | — | Current journal instructions remain reachable and describe a journal-specific publication style, not a Springer family style: [Instructions to authors](https://memorias.ioc.fiocruz.br/instructions-to-authors). | no template parent in CSL metadata | Best target `springer-basic-author-date`, similarity `1.00`, cite `0.65`, bib `0.00`. | Structural best match `springer-basic-author-date` with combined `0.92`, but the remaining style is effectively standalone after reduction. | `independent + standalone` |
| `techniques-et-culture` | `taylor-and-francis-council-of-science-editors-author-date` | — | Current public journal surface is OpenEdition, not Taylor & Francis: [journal page](https://journals.openedition.org/tc/1556#tocto3n5). | `rel="template"` points to `ethnologie-francaise`. | Best target `elsevier-harvard`, similarity `0.9546`, cite `0.05`, bib `0.00`. | Structural best match `taylor-and-francis-council-of-science-editors-author-date` with combined `0.93`, but authority evidence does not support the family claim. | `false-positive` |
| `hawaii-international-conference-on-system-sciences-proceedings` | `taylor-and-francis-national-library-of-medicine` | — | Current conference author page remains reachable and now says references and in-text citation should follow APA 7th: [HICSS authors](https://hicss.hawaii.edu/authors/). | `rel="template"` points to `acm-sigchi-proceedings`. | Best target `ieee`, similarity `0.9881`, cite `0.50`, bib `0.00`. | Structural best match `taylor-and-francis-national-library-of-medicine` with combined `0.93`, but current guide evidence contradicts any IEEE-family wrapper. | `false-positive; temporary IEEE-based legacy hold` |
| `cell-numeric` | `elsevier-with-titles` | `elsevier-with-titles` | Current public Cell-family signal remains the style's own journal guidance link from CSL metadata (`current-biology/authors`), which uses Current Biology numbered references. | no template parent in CSL metadata | Best target `elsevier-with-titles`, similarity `1.00`, cite `1.00`, bib `0.00`. | Structural best match `elsevier-with-titles` with combined `0.92`. | `journal + config-wrapper` |

## Interpretation

### What `9e13a17b` got right

- It surfaced several styles that do belong near an existing Citum family.
- The semantic skeleton comparison is good enough to rank likely families for
  manual audit.

### What `9e13a17b` got wrong

- It treated family similarity as if it were close to wrapper readiness.
- It did not normalize the HICSS shorthand candidate ID.
- It did not distinguish between "wrong hub" and "right family but still
  structural".

## Decision

Three candidates were converted into public journal config-wrappers with
corrected parents:

- `pharmacoepidemiology-and-drug-safety` → `american-medical-association`
- `disability-and-rehabilitation` → `elsevier-with-titles`
- `cell-numeric` → `elsevier-with-titles`

One candidate lost its parent link and remains standalone:

- `memorias-do-instituto-oswaldo-cruz`

One candidate remains a documented temporary legacy hold:

- `hawaii-international-conference-on-system-sciences-proceedings` → `ieee`

The audit recommendation is:

- keep only the parent links that survive a guide-driven reduction pass
- mark the unsupported mappings as `false-positive` or standalone legacy holds
- revise the public taxonomy so journal descendants are no longer forced into
  the same bucket as aliases or config-only wrappers
