# Authoring styles and locales

Citum publishes separate schemas for style behavior and localized language
data. Start with a complete document below, then use the linked authoring pages
for the full option and template vocabulary.

- [`style.json`](../schemas/style.json) validates citation and bibliography
  templates, processing options, metadata, and inheritance.
- [`locale.json`](../schemas/locale.json) validates terms, dates, messages,
  grammar options, and punctuation realization.

The schemas describe the accepted wire format. The [style authoring
guide](style-authoring/start.html) explains how to choose templates and options,
while its [locale section](style-authoring/locales.html) explains term lookup,
MessageFormat 2 messages, and grammatical selection.

## Write a style

Add the schema comment as the first line so an editor with YAML language-server
support can provide completion and validation. This complete style renders an
author and year in citations, then an author, year, and title in the
bibliography.

```yaml
# schema: style
# yaml-language-server: $schema=https://docs.citum.org/schemas/style.json
info:
  title: "My Style"
  id: "my-style"
  default-locale: "en-US"

options:
  processing: author-date

citation:
  template:
    - contributor: author
      form: short
    - date: issued
      form: year
  delimiter: ", "

bibliography:
  options:
    separator: ". "
  template:
    - contributor: author
      form: long
    - date: issued
      form: year
    - title: primary
```

Run the end-to-end validator before publishing the file:

```console
citum style validate path/to/my-style.yaml
```

Continue with [Style anatomy](style-authoring/style-anatomy.html) for the
document sections, [Templates](style-authoring/templates.html) for renderable
components, and [Options](style-authoring/options.html) for reusable formatting
rules.

## Write a locale

A locale document supplies language data independently of a style. The locale
identifier selects the document at runtime. Version 2 locales can define MF2
messages in addition to structured terms.

```yaml
# schema: locale
# yaml-language-server: $schema=https://docs.citum.org/schemas/locale.json
locale: en-US-x-citum-guide
locale-schema-version: "2"

evaluation:
  message-syntax: mf2

terms:
  and: "and"

roles:
  editor:
    long:
      singular: "editor"
      plural: "editors"
    short:
      singular: "ed."
      plural: "eds."
    verb: "edited by"

locators:
  page:
    long:
      singular: "page"
      plural: "pages"
    short:
      singular: "p."
      plural: "pp."

messages:
  pattern.editor-credit: "Edited by {$name}"
```

Check message syntax, aliases, and locale structure with:

```console
citum locale lint path/to/en-US-x-citum-guide.yaml
```

The [Locales authoring page](style-authoring/locales.html) covers message
arguments, gender and plural selection, fallback rules, and compositional
patterns. For repository-maintained locale files, also follow the
[locale contribution guide](AUTHORING_LOCALES.md).

## Validate in other tooling

JSON Schema validators can consume the published files directly. Regenerate
the repository copies after changing schema documentation or Rust schema types:

```console
just schema-gen
```

Schema metadata changes such as clearer descriptions do not change accepted
documents. Review structural schema changes separately because they can affect
editor validation even when deserialization still succeeds.
