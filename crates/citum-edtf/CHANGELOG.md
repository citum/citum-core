# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.19.0](https://github.com/citum/citum-core/compare/citum-edtf-v0.18.0...citum-edtf-v0.19.0) - 2026-03-25

### Added

- *(lint)* clippy all=deny, pedantic suppressions

### Fixed

- *(lint)* pedantic autofixes
- *(edtf)* validate explicit date parts

### Other

- move data structs to citum-schema-data
- *(lints)* centralize doc lint enforcement
- relicense to MIT OR Apache-2.0
- *(edtf)* rename crate csln-edtf → citum-edtf
