# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.17.0](https://github.com/citum/citum-core/compare/citum-engine-v0.16.0...citum-engine-v0.17.0) - 2026-03-22

### Added

- *(engine)* add secondary role label presets
- *(engine)* annotate preview html with template indices
- *(engine)* resolve style presets
- *(locale)* ICU MF1 locale system
- *(engine)* render_locator subsystem
- *(lint)* clippy all=deny, pedantic suppressions
- *(engine)* implement title text-case semantics
- *(note)* implement note-start conformance
- *(notes)* split note-shortening audit layers
- *(note)* audit and complete note styles
- *(citation)* align repeated-note position semantics
- *(engine)* support djot title markup
- *(render)* preserve link URL in djot rendering
- *(doc)* add pandoc markdown citations
- *(citations)* add integral name memory
- *(citation)* unify locator model
- *(multilingual)* prove locale bib layouts
- *(engine)* support expanded verification cases
- *(bindings)* add citum-bindings crate
- *(compound-sets)* implement sets and subentry
- *(schema)* locator ergonomics
- *(schema)* compound locator support
- *(schema)* csl support and pr schema gate
- *(schema)* add NameForm to ContributorConfig
- *(migrate)* tighten inferred bibliography parity heuristics
- *(engine)* add org-mode input/output
- *(engine)* djot inline rendering for annotations
- *(engine,cli)* annotated bibliography support
- *(engine)* overhaul and rebrand Citum FFI bindings
- *(tests)* extract CJK/Arabic CSL fixtures + native test
- *(schema,engine)* add subsequent et-al controls
- *(engine)* support container short titles
- *(testing)* add csl intake audit
- *(typst)* add native rendering and pdf output
- *(processor)* add document-level bibliography grouping via djot and YAML
- *(note-styles)* ibid/subsequent to chicago-notes
- *(multilingual)* add preferred-transliteration
- *(schema)* add SortPreset; use in chicago
- *(edtf)* implement time component rendering
- *(document)* configure note marker punctuation
- *(document)* auto-note djot citations
- *(multilingual)* support language-aware title templates
- *(server)* add citum-server crate

### Fixed

- *(citations)* use prose joining for integral multicites
- *(schema)* null-aware preset overlay merging
- *(lint)* pedantic autofixes
- *(schema)* defaults + drop semantic_classes
- *(bibliography)* add journal doi fallback policy
- *(engine)* per-cite suffix in grouped citations
- *(engine)* integral citation rendering
- *(engine)* address review feedback
- *(oracle)* make scoring case-aware
- *(styles)* add note repeat overrides
- *(engine)* integral ibid in authored notes
- *(engine)* correct note-style ibid rendering
- *(engine)* drop cited-subset numbering
- *(engine)* preserve harvard no-date citations
- *(engine)* sort missing-name works by title
- *(engine)* finish compound numeric rendering
- address copilot review comments
- *(engine)* implement render_org_inline properly
- *(engine)* annotation rendering for non-HTML formats
- *(engine)* thread position into et-al renderer
- *(engine)* format title apostrophe logic
- *(engine)* complete high-fit wave 1 regressions
- *(rendering)* smarten leading single quotes
- *(engine)* sort undated bibliography entries last
- *(processor)* strip YAML frontmatter from rendered output
- *(processor)* add trailing newline after bibliography blocks
- *(processor)* repair bibliography block rendering
- *(ci)* stabilize local and oracle gates
- *(render)* make HTML bibliography separators markup-aware
- *(processing)* make bibliography sort defaults explicit
- *(chicago)* add bibliography sort for anon works
- *(labels,names)* docs and test coverage for label mode and space separator
- *(locale)* lowercase editor short terms (ed./eds.)
- *(labels)* make et-al name count configurable per preset
- *(engine)* normalize space-only initials formatting
- *(delimiter)* normalize enum parsing across engine and migrate

### Other

- release v0.16.0
- *(ci)* reduce compilation time A+B
- release v0.14.0
- release v0.14.0
- move data structs to citum-schema-data
- *(engine)* bundle Renderer::new params
- *(engine)* convert priority-list test to rstest
- *(engine)* seal citation parser boundary
- *(engine)* remove too_many_args suppressions
- *(lint)* enforce too_many_lines and cognitive_complexity
- *(djot)* add adapter pipeline tests
- *(grouped)* add regression tests for grouped modes
- *(djot)* split adapter and parsing
- *(engine)* split grouped.rs into submodules
- *(engine)* remove too_many_arguments allows
- *(engine,migrate)* rust-simplify pass
- *(engine)* split processor files
- *(citum-engine)* address copilot review feedback
- *(citum-engine)* simplify grouped rendering and disambiguation
- *(engine)* extract ffi biblatex module
- *(engine)* split contributor module
- *(citum-engine)* simplify grouping helpers
- *(citum-engine)* split rendering module
- *(citum-engine)* split document module
- *(migrate)* extract fixups module
- *(engine)* extract helpers from format_names/format_single_name
- *(engine)* extract title multilingual config helper
- *(engine)* simplify values/* and io hotspots
- *(lints)* centralize doc lint enforcement
- *(processor)* simplify document and rendering flows
- *(repo)* refresh snapshots and SPDX headers
- CSLN -> Citum
- *(citum-engine)* thin processor facade
- *(engine)* simplify rendering helpers
- *(schema,engine)* add rendering_mut
- *(engine)* simplify citation rendering
- *(engine)* simplify citation helpers
- *(engine)* simplify rendering.rs
- *(engine)* simplify hint calculation
- *(engine)* add disambiguation benchmarks
- *(engine)* expand behavior report coverage
- *(engine)* expand behavior report coverage
- *(engine)* publish behavior coverage reports
- *(citum-engine)* simplify rendering pass
- relicense to MIT OR Apache-2.0
- *(deps)* replace serde_yaml with serde_yaml_ng
- *(deps)* replace serde_cbor with ciborium
- *(engine)* annotation rendering unit tests
- *(engine)* add /// and unit tests
- *(engine)* enforce missing docs coverage
- *(engine)* cover public support APIs
- *(processor)* cover rendered disambiguation paths
- *(processor)* correct review-driven docs
- enhance processor docs and tests
- *(citations)* cover empty-date citation sort
- *(engine)* add sort oracle tests
- *(schema,engine,migrate)* add public API doc comments
- *(beans)* complete csl26-extg
- update name
- *(edtf)* rename crate csln-edtf → citum-edtf
- *(beans)* track duplicate sort key cleanup
- *(workspace)* migrate csln namespace to citum

## [0.16.0](https://github.com/citum/citum-core/compare/citum-engine-v0.15.0...citum-engine-v0.16.0) - 2026-03-22

### Added

- *(engine)* add secondary role label presets

### Fixed

- *(citations)* use prose joining for integral multicites

### Other

- *(ci)* reduce compilation time A+B

## [0.15.0](https://github.com/citum/citum-core/compare/citum-engine-v0.14.0...citum-engine-v0.15.0) - 2026-03-19

### Added

- *(engine)* annotate preview html with template indices

## [0.14.0](https://github.com/citum/citum-core/compare/citum-engine-v0.13.0...citum-engine-v0.14.0) - 2026-03-19

### Added

- *(engine)* resolve style presets
- *(locale)* ICU MF1 locale system

### Fixed

- *(schema)* null-aware preset overlay merging
