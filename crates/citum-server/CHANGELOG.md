# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.19.0](https://github.com/citum/citum-core/compare/citum-server-v0.18.0...citum-server-v0.19.0) - 2026-03-25

### Added

- *(template-v2)* implement template schema v2
- *(engine)* annotate preview html with template indices
- *(lint)* clippy all=deny, pedantic suppressions
- *(server)* upgrade CLI with clap and custom styling
- *(typst)* add native rendering and pdf output
- *(server)* support output formats
- *(server)* add citum-server crate

### Fixed

- *(schema)* null-aware preset overlay merging
- *(lint)* pedantic autofixes
- *(server)* enforce docs and invalid formats

### Other

- release v0.18.0
- release v0.14.0
- *(lints)* centralize doc lint enforcement
- *(repo)* refresh snapshots and SPDX headers
- relicense to MIT OR Apache-2.0
- *(deps)* replace serde_yaml with serde_yaml_ng
- *(cli)* unify help output with a summary-and-detail model
- *(server)* cover http mode
- *(server)* fix stdio example with valid JSON
- *(server)* add HTTP curl example to README
- *(server)* add RPC dispatcher integration tests

## [0.18.0](https://github.com/citum/citum-core/compare/citum-server-v0.17.0...citum-server-v0.18.0) - 2026-03-25

### Added

- *(template-v2)* implement template schema v2

## [0.15.0](https://github.com/citum/citum-core/compare/citum-server-v0.14.0...citum-server-v0.15.0) - 2026-03-19

### Added

- *(engine)* annotate preview html with template indices
