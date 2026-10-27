# Using registry and auxiliary schemas

Citum publishes three small artifacts alongside its style, bibliography, and
server schemas:

| Artifact | Use it when you need to… | Contract |
|---|---|---|
| [`registry.json`](../schemas/registry.json) | publish or consume a named style registry | JSON Schema for `version` and `styles` |
| [`abbrev-map.json`](../schemas/abbrev-map.json) | replace exact rendered strings at document time | JSON Schema for a flat string-to-string map |
| [`type-map.json`](../schemas/type-map.json) | inspect the converter's BibLaTeX and CSL-JSON mappings | Generated data dump, not a JSON Schema |

Use the [JSON Schemas page](../schemas/) for editor integration and the
[generated reference pages](../reference/BIBLATEX_MAPPING.md) for mapping
details. This guide focuses on authoring and integration decisions that a raw
schema cannot explain.

## Publish a style registry

A registry is a YAML or JSON document with a `version` and a list of style
entries. It is a discovery and alias-resolution layer; it does not contain the
style body. A local `citum-registry.yaml` is discovered from the current
working directory, and `citum registry add` can load a registry from a local
path or an HTTP(S) URL.

Add the schema hint to a YAML registry when your editor supports the YAML
language server:

```yaml
# yaml-language-server: $schema=https://docs.citum.org/schemas/registry.json
# schema: registry
version: "1"
styles:
  - id: example-journal
    aliases: [ej]
    path: styles/example-journal.yaml
    title: Example Journal
    description: House style for Example Journal
    fields: [science]
    kind: independent
  - id: example-remote
    url: https://styles.example.org/example-remote.yaml
    title: Example remote style
    kind: profile
```

Each entry must provide exactly one style source:

- `builtin` names a style embedded in the default registry;
- `path` points to a YAML style file, relative to the registry's local base
  directory; or
- `url` points to an HTTP(S) YAML style file.

The JSON Schema checks the field shapes. The registry loader also checks this
exactly-one-source rule, so do not combine `path` and `url` on one entry. Use
`id` for the canonical name and `aliases` for names that should resolve to the
same entry. `title`, `description`, `fields`, and `kind` are discovery metadata;
they do not replace the style's own `info` block.

Useful commands for a local or configured registry are:

```console
citum registry list
citum registry resolve ej
citum registry add https://styles.example.org/registry.yaml --name example
```

The registry schema is also available directly for tools that need to validate
before invoking the CLI:

```console
citum schema registry
```

For the resolver chain, caching, and remote-source workflow, see the
[distributed registries guide](DISTRIBUTED_REGISTRIES.md). Use the current
`version`/`styles` shape shown above when authoring a registry consumed by the
current loader.

## Apply document-level abbreviations

An abbreviation map is a flat map from a full rendered string to its
replacement. Keys are exact and case-sensitive. The engine applies the map
after extracting a value and before assembling output, so a map can affect
titles, container titles, publisher names, archive names, and literal
contributors without becoming part of a style definition.

The standalone map file has this shape:

```yaml
# yaml-language-server: $schema=https://docs.citum.org/schemas/abbrev-map.json
# schema: abbrev-map
Estates Gazette: EG
"Lloyd's Law Reports": "Lloyd's Rep"
World Health Organization: WHO
```

When rendering a document, put that same map under the document options block:

```yaml
---
options:
  abbreviation-map:
    Estates Gazette: EG
    "Lloyd's Law Reports": "Lloyd's Rep"
---

Text with citations goes here.
```

For the JSON API, use the equivalent `document_options.abbreviation_map` key.
The `abbreviation-map` YAML alias and `abbreviation_map` JSON spelling are
accepted at the document-options boundary; the map itself remains flat.

An abbreviation map is appropriate for document-specific output conventions.
Use a style option or a contributor's `short-name` when the abbreviation is a
stable part of the style or source data. Validate the standalone map with:

```console
citum schema abbrev-map
```

## Read the generated type map

`type-map.json` is produced by `citum schema type-map` as a plain data dump.
It records the converter's current BibLaTeX entry-type and field mappings and
the CSL-JSON type conversion table. It is useful to build migration reports,
inspect supported mappings, or generate documentation; it is not an input
format for style or bibliography authors and it is not a JSON Schema.

The generated, human-readable views are:

- [BibLaTeX field mapping](../reference/BIBLATEX_MAPPING.md), generated from
  `biblatex_entry_types` and `biblatex_fields`;
- [CSL-JSON type mapping](../reference/generated/CSL_JSON_MAPPING.md),
  generated from `csl_type_map`; and
- the [CSL type conversion contract](../specs/CSL_TYPE_CONVERSION_CONTRACT.md),
  which explains the conversion and round-trip guarantees.

Treat the type-map keys and row meanings as versioned converter metadata, not
as a stable authoring API. Rows may be added or corrected when conversion
support changes, and notes may become more precise without changing the native
wire formats. Do not edit `type-map.json` or either generated Markdown table
by hand. The Rust mapping tables are the source of truth; regenerate all
published schemas and references with:

```console
just schema-gen
```

That command refreshes `docs/schemas/` and the generated reference pages from
the current code. It should produce no schema-structure change for a
documentation-only update.
