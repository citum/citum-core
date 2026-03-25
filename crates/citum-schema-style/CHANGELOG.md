# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.19.0](https://github.com/citum/citum-core/compare/citum-schema-style-v0.18.0...citum-schema-style-v0.19.0) - 2026-03-25

### Added

- *(bindings)* promote wasm api + specta types
- *(template-v2)* implement template schema v2
- *(schema)* role preset shorthand
- *(locale)* pivot to MF2 message syntax
- *(engine)* add secondary role label presets
- *(schema)* add StyleRegistry type, registry, and CLI
- *(schema)* layered style preset overrides
- *(locale)* ICU MF1 locale system
- *(schema)* style-level locator rendering config
- *(schema)* short_name + edition on StyleInfo
- *(lint)* clippy all=deny, pedantic suppressions
- *(engine)* implement title text-case semantics
- *(note)* implement note-start conformance
- *(citation)* align repeated-note position semantics
- *(core)* split schema and convert namespace

### Fixed

- *(migrate)* co-emit name_form on initialize_with
- *(release)* schema pre-1.0 major bump guard
- *(scripts)* oracle-yaml test JS and doc fixes
- *(engine)* split name-form from initialize-with
- *(schema)* null-aware preset overlay merging
- *(lint)* pedantic autofixes
- *(schema)* defaults + drop semantic_classes
- *(bibliography)* add journal doi fallback policy
- *(convert)* preserve refs fidelity across csl-json and ris

### Other

- *(specs)* reconcile template-v2 with main
- release v0.18.0
- release v0.17.0
- release v0.16.0
- *(ci)* reduce compilation time A+B
- release v0.14.0
- release v0.13.0
- move data structs to citum-schema-data
- *(lint)* enforce too_many_lines and cognitive_complexity
- *(lints)* centralize doc lint enforcement
- *(repo)* refresh snapshots and SPDX headers
- CSLN -> Citum
- *(schema,engine)* add rendering_mut
- *(schema)* dedupe facade crate
- *(release)* bump to 0.10.0 and align schema

## [0.18.0](https://github.com/citum/citum-core/compare/citum-schema-style-v0.17.0...citum-schema-style-v0.18.0) - 2026-03-25

### Added

- *(bindings)* promote wasm api + specta types
- *(template-v2)* implement template schema v2

### Fixed

- *(scripts)* oracle-yaml test JS and doc fixes
- *(engine)* split name-form from initialize-with

## [0.17.0](https://github.com/citum/citum-core/compare/citum-schema-style-v0.16.0...citum-schema-style-v0.17.0) - 2026-03-22

### Added

- *(schema)* role preset shorthand

## [0.16.0](https://github.com/citum/citum-core/compare/citum-schema-style-v0.15.0...citum-schema-style-v0.16.0) - 2026-03-22

### Added

- *(locale)* pivot to MF2 message syntax
- *(engine)* add secondary role label presets
- *(schema)* add StyleRegistry type, registry, and CLI

### Other

- *(ci)* reduce compilation time A+B

## [0.14.0](https://github.com/citum/citum-core/compare/citum-schema-style-v0.13.0...citum-schema-style-v0.14.0) - 2026-03-19

### Added

- *(schema)* layered style preset overrides
- *(locale)* ICU MF1 locale system

### Fixed

- *(schema)* null-aware preset overlay merging
