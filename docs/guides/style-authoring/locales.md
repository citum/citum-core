---
title: Locales and Gender-Aware Terms
nav: Locales
description: Use locale terms for roles, labels, locators, and grammatical agreement.
features:
  - gender-aware-locale-terms
---

## [translate] Locale-owned text

Use locale terms for text that varies by language: page labels, role labels,
and other bibliographic terms. Style templates should express structure; locale
files should own language.

```yaml
- term: volume
  form: short
  suffix: " "
- number: volume
```

## [wc] Contributor gender metadata

Contributor-driven role labels can use contributor gender metadata where the
locale provides gendered forms.

```yaml
contributors:
  - role: editor
    contributor:
      family: "Martinez"
      given: "Ana"
    gender: feminine
```

Mixed-gender contributor groups prefer a locale's neutral or common form when
one exists. Citum does not silently choose a masculine-specific label for a
mixed group when only gendered forms are available.

## [settings] Template gender overrides

Use a template-level `gender` override when the style needs to request a
specific agreement form directly.

```yaml
- contributor: editor
  form: long
  gender: feminine

- term: volume
  form: short
  gender: masculine

- number: volume
  label-form: short
  gender: feminine
```

## [language] Locale YAML

Locale terms can be plain strings or gendered maps.

```yaml
roles:
  editor:
    long:
      singular:
        masculine: editor
        feminine: editora
        common: persona editora
      plural:
        masculine: editores
        feminine: editoras
        common: equipo editorial
```

Locator terms can also declare lexical gender metadata.

```yaml
locators:
  page:
    long:
      singular: página
      plural: páginas
    short:
      singular: p.
      plural: pp.
    gender: feminine
```

> [!TIP]
> **Current scope**
> Gender-aware rendering applies to locale term selection and contributor role
> labels. Verb-form role terms remain ungendered until the follow-up adds the
> required plumbing.
