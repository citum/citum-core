# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.19.0](https://github.com/citum/citum-core/compare/citum-schema-v0.18.0...citum-schema-v0.19.0) - 2026-03-25

### Added

- *(bindings)* promote wasm api + specta types
- *(lint)* clippy all=deny, pedantic suppressions
- *(core)* split schema and convert namespace
- *(citations)* add integral name memory
- *(citation)* unify locator model
- *(engine)* support expanded verification cases
- *(compound-sets)* implement sets and subentry
- *(schema)* locator ergonomics
- *(schema)* compound locator support
- *(schema)* csl support and pr schema gate
- *(schema)* add NameForm to ContributorConfig
- *(schema,engine)* add subsequent et-al controls
- *(engine)* support container short titles
- *(typst)* add native rendering and pdf output
- *(schema)* add CitationField, StyleSource provenance to StyleInfo
- *(multilingual)* add preferred-transliteration
- *(schema)* add SortPreset; use in chicago
- *(edtf)* implement time component rendering
- *(document)* configure note marker punctuation
- *(multilingual)* support language-aware title templates

### Fixed

- *(lint)* pedantic autofixes
- *(engine)* preserve harvard no-date citations
- *(engine)* finish compound numeric rendering
- *(scripts)* make bump workflow schema-only
- *(scripts)* replace bump workflow with python tool
- *(schema)* address missing field_languages field in ref_article_authors macro
- *(processing)* make bibliography sort defaults explicit
- *(examples)* validate and fix refs YAML files
- *(schema)* drop is_base from StyleInfo and csl-legacy Info
- *(locale)* lowercase editor short terms (ed./eds.)
- *(labels)* make et-al name count configurable per preset
- *(delimiter)* normalize enum parsing across engine and migrate

### Other

- release v0.18.0
- *(lints)* centralize doc lint enforcement
- *(schema)* dedupe facade crate
- *(release)* bump to 0.10.0 and align schema
- *(release)* bump version to 0.8.0
- relicense to MIT OR Apache-2.0
- *(deps)* replace serde_yaml with serde_yaml_ng
- *(deps)* replace serde_cbor with ciborium
- *(schema)* document and test processing options
- *(schema)* document and test locale types
- *(schema)* document and test renderer
- *(schema)* cover locale support docs
- *(schema)* cover citation locator docs
- *(schema)* cover renderer docs
- *(schema)* cover root style-model docs
- *(schema,engine,migrate)* add public API doc comments
- *(migrate)* trim redundant bibliography sorts
- *(edtf)* rename crate csln-edtf → citum-edtf
- *(beans)* track duplicate sort key cleanup
- *(workspace)* migrate csln namespace to citum

## [0.18.0](https://github.com/citum/citum-core/compare/citum-schema-v0.17.0...citum-schema-v0.18.0) - 2026-03-25

### Added

- *(bindings)* promote wasm api + specta types
