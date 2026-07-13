# Render-When Contract Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-07-13
**Supersedes:** None
**Related:** `csl26-qyub`

## Purpose

`render-when` is a field-presence conditional on `TemplateGroup`. A group
carrying `render-when` renders only when the reference data matches the
condition; otherwise it and everything inside it is skipped and claims no
variables. This spec defines its wire contract, field vocabulary, evaluation
semantics, and validation rules.

The mechanism is intentionally bounded: independent `field-present` and
`field-absent` probes combined with AND only. No OR, value comparisons, or
arbitrary boolean expressions.

## Scope

In scope:

- the wire contract for `TemplateGroup.render_when`,
  `TemplateGroupCondition`, and `TemplateConditionField`;
- the typed field vocabulary and its extension rules;
- validation and nesting semantics.

Out of scope:

- value comparisons, disjunction, arbitrary boolean expressions, or
  CSL-style `choose`/`if`/`else` control flow;
- migration emission — `citum-migrate` does not emit this feature, and this
  spec does not make it a migration target.

## Design

### Wire contract

`render-when` is valid only on `TemplateGroup`:

```yaml
- group:
  - contributor: author
  - contributor: recipient
    prefix: " to "
  render-when:
    field-present: recipient
    field-absent: title
```

`field-present` and `field-absent` are each optional, but at least one must
be supplied. When both are supplied they combine with AND: the group renders
only when the first field is present and the second is absent.

Conditions are evaluated from reference source accessors before the group
renders and before it can claim variables. They do not inspect formatted
text, substitution results, or whether another component already consumed a
value.

### Field vocabulary

| Field | Presence source |
|---|---|
| `author` | primary author contributor exists |
| `editor` | editor contributor exists |
| `recipient` | recipient contributor exists |
| `translator` | translator contributor exists |
| `title` | primary title accessor returns a value |
| `collection-title` | collection-title accessor returns a value |
| `issued` | effective issued date exists |
| `original-published` | original publication date exists |
| `publisher` | publisher string exists |
| `original-publisher` | original publisher string exists |
| `original-publisher-place` | original publisher place exists |
| `original-title` | original title exists |
| `doi` | DOI exists |
| `genre` | genre exists |
| `archive` | archive or repository name exists |
| `archive-location` | archive location or shelfmark exists |

New fields may be added when all of the following hold:

- presence has one unambiguous, documented reference accessor meaning;
- a real style forcing case needs it, and the need is a field-presence
  layout or value selection within one reference type — not a stand-in for
  a distinction an option, preset, or type-variant should own instead;
- schema parse, present, absent, and combined-condition tests cover it;
- engine behavior stays generic and does not inspect style identity;
- this contract and generated schema documentation are updated.

Field growth does not imply operator growth. Multiple-field lists, OR,
comparisons, arbitrary expressions, and new branch forms each require a
separate design proposal.

### Validation

Style validation rejects:

- `render-when: {}`, an unconditional no-op;
- the same field in both `field-present` and `field-absent`, which can never
  match;
- `render-when` on any component other than a group — already impossible in
  the typed schema.

### Nesting

Conditioned groups may nest. Each condition is evaluated independently
before its own group renders; a suppressed group claims no variables.

## Implementation Notes

Not yet implemented:

- validation rejecting empty and same-field present/absent conditions;
- behavior tests for present, absent, combined-AND, and nested cases;
- schema regeneration (`docs/schemas/style.json`,
  `docs/schemas/server.json`) for the new validation.

`citum-migrate` continues to not emit `render-when`.

## Acceptance Criteria

- [ ] Schema validation rejects empty and same-field present/absent
      conditions.
- [ ] Behavior tests cover present, absent, combined-AND, and nested cases.
- [ ] Generated schemas document the validated contract.
- [ ] Status promoted to Active in the implementation commit.

## Changelog

- v1.0 (2026-07-13): Initial contract specification.
