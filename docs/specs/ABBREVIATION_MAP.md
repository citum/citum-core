# Abbreviation Map

Status: Active

## Overview

The `abbreviation-map` is a document-level feature that substitutes full rendered
string values with user-defined abbreviations.

## YAML Frontmatter

```yaml
abbreviation-map:
  Estates Gazette: EG
  "Lloyd's Law Reports": "Lloyd's Rep"
```

Both `abbreviation-map` (YAML) and `abbreviation_map` (JSON) key forms are accepted.

Inline map form:

```yaml
abbreviation-map: { Estates Gazette: EG }
```

## Semantics

- Keys are full rendered strings (exact, case-sensitive match).
- Values are the replacement abbreviations.
- Substitution is applied after value extraction, before output assembly.
- If no map entry exists for a value, the original value is returned unchanged.
- The map is style-agnostic — it applies to any template component that renders a simple string (titles and variables).

## Scope

This is a clean break from CSL/Pandoc's `citation-abbreviations` JSON format.
No nested structure, no variable-group bindings — just a flat key→value map
applied uniformly across rendered string fields.

Abbreviations are applied to:
- Title fields (main title, container title, collection title)
- Variable fields (publisher, archive, series)
- Contributor literal names (corporate/institutional authors)

## Non-goals

- Fuzzy or case-insensitive matching (not in scope for this release)
- Per-field or per-variable-type targeting
- Abbreviation of structured contributor names (handled separately by name formatting)
