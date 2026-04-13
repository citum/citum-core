# Personal Communication Citation Specification

**Status:** Active
**Date:** 2026-04-13
**Related:** csl26-zr4q

## Purpose

Defines how personal communications (letters, emails, conversations) are rendered
in-text citations across APA, Chicago, and MLA style families.

Personal communications are never listed in the reference list. They exist only
as in-text citations, which drives a number of rendering constraints.

## Scope

In scope: APA 7th integral and non-integral citation rendering; Chicago Notes
author/recipient rendering; locale term registration for `personal-communication`.

Out of scope: MLA (noted for completeness only); bibliography suppression
mechanism (that is a separate engine concern).

## Design

### Style guide requirements

#### APA 7th (Section 8.9)

**Non-integral:** `(J. Oglethorpe, personal communication, January 13, 1733)`

**Integral:** `Oglethorpe (personal communication, January 13, 1733)`

- Author appears with initials (given-first order).
- "personal communication" label appears inside the parenthetical group,
  after the author name component (non-integral) or after the prose name (integral).
- Date uses full month-day-year form ("January 13, 1733").
- No reference-list entry: `bibliography.type-variants.personal-communication: []`.

#### Chicago Notes and Bibliography (17th ed., 14.111)

Format in a footnote/endnote: `James Oglethorpe to the Trustees, 1733 [...]`

- Author rendered in full (given-first).
- Recipient rendered after "to".
- Archive location follows if applicable.
- No reference-list entry in bibliography.

#### MLA 9th

- Recipient shown with "Received by" label.
- Not applicable to current Citum styles; noted for completeness.

### Input data requirements

Personal communications must carry explicit contributors with roles, not an
embedded title string:

```yaml
type: personal-communication
contributors:
- role: author
  contributor: {given: James, family: Oglethorpe}
- role: recipient
  contributor: {name: "the Trustees"}
issued: '1733'
```

The original Zotero export embeds author and recipient in a `title` field
("James Oglethorpe to the Trustees"). This is incorrect input for Citum.
The `recipient` role is supported via `citum_schema::reference::ContributorRole::Recipient`.

### Template design (APA 7th)

#### Non-integral

Items are delimited by `citation.delimiter` (", ") and wrapped by
`non-integral.wrap: parentheses`:

```yaml
personal-communication:
- contributor: author
  form: long
  name-order: given-first
- term: personal-communication
- date: issued
  form: full
- variable: locator
```

Renders: `(J. Oglethorpe, personal communication, January 13, 1733)`

#### Integral

The "personal communication" label belongs inside the parenthetical group,
not as a suffix on the author name component:

```yaml
personal-communication:
- contributor: author
  form: long
  name-order: given-first
- delimiter: ", "
  wrap:
    punctuation: parentheses
  group:
  - term: personal-communication
  - date: issued
    form: full
  - variable: locator
```

Renders: `Oglethorpe (personal communication, January 13, 1733)`

### Locale term

The `personal-communication` locale term is defined in all supported locales
under `terms.personal_communication` in the locale YAML.

`GeneralTerm::PersonalCommunication` was added to the schema and is referenced
in templates as `term: personal-communication`.

### Processor principle

The processor (engine) must not hard-code personal-communication rendering.
Removed special-cases:

- `contributor/mod.rs`: was injecting `", personal communication"` suffix and
  `given-first` name order regardless of the style template.
- `date.rs`: was overriding the date form to `DateForm::Full` regardless of
  the form declared in the style template.

Both behaviors are now declared entirely by the style template. The processor
stays dumb; the style declares behavior.

## Implementation Notes

The en-US locale YAML (`locales/en-US.yaml`) has a pre-existing parse failure
at line 906 (unicode curly quotes in `grammar-options`). This causes
`Locale::load` to fall back to the hardcoded `Terms::en_us()` method, which
must include the `PersonalCommunication` entry for the term to resolve. Fixing
the YAML parse failure is tracked separately.

## Acceptance Criteria

- [x] `GeneralTerm::PersonalCommunication` is registered in schema and locale.
- [x] APA 7th integral citation renders label inside the parenthetical group.
- [x] APA 7th non-integral citation renders label as a standalone component.
- [x] Date renders in full form (`form: full`) as declared by the style template.
- [x] Engine has no special-casing for `personal-communication` type.
- [x] Personal-communication items do not appear in bibliography.

## Changelog

- 2026-04-13: Initial version. Active from first implementation commit.
