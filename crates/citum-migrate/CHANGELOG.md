# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.17.0](https://github.com/citum/citum-core/compare/citum-migrate-v0.16.0...citum-migrate-v0.17.0) - 2026-03-22

### Added

- *(schema)* short_name + edition on StyleInfo
- *(lint)* clippy all=deny, pedantic suppressions
- *(report)* add migration behavior coverage
- *(engine)* implement title text-case semantics
- *(migrate)* support mixed note position trees
- *(citation)* align repeated-note position semantics
- *(migrate)* normalize legal-case fields by style id
- *(migrate)* tighten inferred bibliography parity heuristics
- *(migrate)* improve inferred parity and migrate-only oracle tooling
- *(schema,engine)* add subsequent et-al controls
- *(schema)* add CitationField, StyleSource provenance to StyleInfo
- *(schema)* add SortPreset; use in chicago
- *(migrate)* complete variable-once cross-list deduplication

### Fixed

- *(lint)* pedantic autofixes
- *(bibliography)* add journal doi fallback policy
- *(oracle)* make scoring case-aware
- *(migrate)* support complex position trees
- *(migrate)* preserve strip-periods in migration
- *(migrate)* normalize locator labels
- *(migrate)* fix subsequent et-al propagation
- *(schema)* drop is_base from StyleInfo and csl-legacy Info
- *(delimiter)* normalize enum parsing across engine and migrate

### Other

- *(migrate)* drop locator label fields
- *(lint)* enforce too_many_lines and cognitive_complexity
- *(migrate)* split fixups modules
- *(migrate)* remove only_used_in_recursion allow
- *(engine,migrate)* rust-simplify pass
- *(citum-migrate)* simplify upsampler
- *(migrate)* extract fixups module
- *(lints)* centralize doc lint enforcement
- *(repo)* refresh snapshots and SPDX headers
- CSLN -> Citum
- *(migration)* expand csl-to-citum reporting
- *(tests)* rename strip-periods regression test
- relicense to MIT OR Apache-2.0
- *(deps)* replace serde_yaml with serde_yaml_ng
- *(migrate)* add /// to options_extractor fns
- *(migrate)* modularize template_compiler
- *(schema,engine,migrate)* add public API doc comments
- *(migrate)* trim redundant bibliography sorts
- *(beans)* track duplicate sort key cleanup
- *(workspace)* migrate csln namespace to citum
