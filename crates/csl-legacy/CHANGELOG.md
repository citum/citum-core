# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.17.0](https://github.com/citum/citum-core/compare/csl-legacy-v0.16.0...csl-legacy-v0.17.0) - 2026-03-22

### Added

- *(lint)* clippy all=deny, pedantic suppressions
- *(engine)* support expanded verification cases
- *(schema)* add CitationField, StyleSource provenance to StyleInfo

### Fixed

- *(lint)* pedantic autofixes
- *(schema)* drop is_base from StyleInfo and csl-legacy Info

### Other

- *(ci)* reduce compilation time A+B
- *(lints)* centralize doc lint enforcement
- *(repo)* refresh snapshots and SPDX headers
- CSLN -> Citum
- relicense to MIT OR Apache-2.0
- *(csl-legacy, citum-cli)* add doc and test coverage
- *(workspace)* migrate csln namespace to citum
