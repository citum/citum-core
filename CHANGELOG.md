# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

### Miscellaneous Tasks

- **tooling**: Regenerate changelog via git-cliff

## [0.7.0] - 2026-03-01

### Bug Fixes

- **processor**: Stabilize numeric cite ordering
- **processor**: Align note locator and serial metadata
- **locale**: Preserve locator term forms from yaml
- **core**: Match default overrides only as fallback
- **compat**: Normalize oracle text and lift springer
- **compat**: Raise springer-vancouver above 90
- **oracle**: Use token-similarity matching
- **migrate**: Make inferred mode cache-only
- **migrate**: Guard inferred citation templates
- **migrate**: Preserve help flag in pr rewrite
- **migrate**: Recover branch-specific numeric fidelity
- **migrate**: Scope inferred type-template merges
- **report**: Include tier-2 wave in compat
- **ci**: Skip metadata-only compat commits
- **ci**: Normalize compat hash-only marker
- **ci**: Harden core-quality style resolution
- **report**: Infer format for !custom and missing processing styles
- **styles**: Fix citation templates across 27 styles
- **migrate**: Improve numeric sort fidelity
- **migrate**: Improve wave3 citation fidelity
- **chicago-shortened**: Close patent and personal-comm bibliography gaps
- **processor**: Calculate group_length for author-format disambiguation
- **cli**: Add --builtin support to render doc and check commands
- **styles**: Modernize MLA and refine citations
- **latex**: Escape bare & in finish() for valid LaTeX output
- **styles**: Correct gost processing variants
- **cli**: Wire grouped bibliography rendering to render refs
- **styles**: Bring all styles to 51%+ fidelity and 80%+ SQI
- **styles**: Fix 3 springer-physics bib failures
- **core**: Normalize type-name casing in TypeSelector::matches
- **migrate**: Improve migration logic for legal and special types
- **core**: Switch InputReference from untagged to class-tagged dispatch
- **docs**: Prevent nav overlap
- **docs**: Restore nav on md screens
- **docs**: Align compat branding to citum
- **oracle**: Strip bibliography numbering after whitespace normalization
- **beans**: Filter non-actionable statuses in next
- **delimiter**: Normalize enum parsing across engine and migrate
- **engine**: Normalize space-only initials formatting
- **docs**: Add soon badges to Hub and Labs in nav
- **labels**: Make et-al name count configurable per preset
- **locale**: Lowercase editor short terms (ed./eds.)
- **labels,names**: Docs and test coverage for label mode and space separator
- **chicago**: Add bibliography sort for anon works
- **beans**: Prefer executable next targets
- **chicago-author-date-classic**: Clean up duplicate keys
- **styles**: Close top-10 bibliography deltas
- **schema**: Drop is_base from StyleInfo and csl-legacy Info
- **styles**: Rename CSLN to Citum in apa-7th title
- **styles**: Remove parenthetical from apa-7th title
- **examples**: Validate and fix refs YAML files
- **processing**: Make bibliography sort defaults explicit
- **schema**: Address missing field_languages field in ref_article_authors macro
- **render**: Make HTML bibliography separators markup-aware
- **ci**: Stabilize local and oracle gates
- **processor**: Repair bibliography block rendering
- **processor**: Add trailing newline after bibliography blocks
- **processor**: Strip YAML frontmatter from rendered output
- **engine**: Sort undated bibliography entries last
- **rendering**: Smarten leading single quotes
- **engine**: Complete high-fit wave 1 regressions
- **engine**: Format title apostrophe logic
- **schema**: Restore schema publishing outputs
- **tooling**: Align schema validator with repo inputs
- **engine**: Thread position into et-al renderer
- **migrate**: Fix subsequent et-al propagation

### Documentation

- **styleauthor**: Add template strategy guardrails
- **compat**: Report core styles without template col
- **compat**: Treat core styles and note-only fidelity
- **compat**: Refresh core-style fidelity dashboard
- **compat**: Refresh dashboard snapshot
- **compat**: Exclude annals from core style set
- Update JSON schemas
- Update JSON schemas
- **migrate**: Add csln-migrate guide
- **beans**: Record 100-style benchmark update
- **beans**: Add SQI evolution tracking task
- **workflow**: Add fidelity-first SQI guidance
- Refactor CONTRIBUTING.md for clarity and concision
- Comprehensive project review and sync
- Remove stale roadmap from README
- Update migration sections to reflect current state
- Rewrite readme and align docs site
- **readme**: Point status links to published site
- Add style author guide to website
- **guide**: Add IDE editor integration callout
- **guide**: Fix enum values from schema review
- **styleauthor**: Add migrate+enhance batch mode
- **status**: Refresh tier metrics
- **claude**: Ban literal \n in commits
- Update JSON schemas
- **guide**: Expand preset catalog with all 14 contributor presets
- Update JSON schemas
- **architecture**: Add migrate wave handoff docs
- **architecture**: Consolidate wave runbook
- Update JSON schemas
- Link to comprehensive YAML editor support list
- Add embedded styles section and CLI builtin examples
- Update JSON schemas
- **beans**: Mark csl26-wms5 and csl26-reux as completed
- Update JSON schemas
- Highlight LuaLaTeX integration proof-of-concept
- **arch**: Add citum modularization plan and beans
- Enhance examples page with navigation
- **architecture**: Define style target roadmap
- **arch**: Correct biblatex placement, fix phase refs, add sections
- **arch**: Expand git history section for three-repo split
- **arch**: Correct repo split — styles stay in core, hub transfers
- **examples**: Add multilingual and bib-grouping runnable examples
- Unify navigation and fix SQI Tier sorting
- **arch**: Add citum-server mode plan and beans
- **compat**: Document fidelity and SQI
- **architecture**: Refresh sqi plan status
- **beans**: Add ibid, etc
- **beans**: Refine csl26-b76h scope and structure
- **beans**: Tighten csl26-b76h post-review fixes
- Elevate note styles to parallel with numeric wave
- Update JSON schemas
- Update JSON schemas
- **claude**: Slim CLAUDE.md from ~722 to 157 lines
- **architecture**: Update Section 3 to reflect deny_unknown_fields trade-off
- **architecture**: Add architectural soundness assessment and gap beans
- **citum**: Keep server in core during early phase
- **server**: Clarify dependency graph in plan
- **branding**: Finish citum command and website naming sweep
- **migration**: Align labs repo name to citum-citum-labs
- **branding**: Rename CSLN docs to Citum
- **site**: Fix cargo install command snippet
- **site**: Fix homepage hero branding
- **site**: Finish Citum branding in compat page
- **web**: Align domains and nav
- **beans**: Enforce citum-bean script for /beans next
- **beans**: Suppress redundant output repeat
- **beans**: Relay citum-bean next output to user verbatim
- **beans**: Relay script output as plain text
- **architecture**: Close csl26-o18j with verification
- **beans**: Rescope rh2u and ifiw status
- **server**: Add HTTP curl example to README
- **server**: Fix stdio example with valid JSON
- **index**: Promote JSON-RPC server card from roadmap to shipped
- **multilingual**: Clarify field-scoped language metadata
- **beans**: Mark csl26-p363 complete
- Update name
- **schema,engine,migrate**: Add public API doc comments
- **claude**: Require /// doc comments on all public Rust items
- **testing**: Record deferred wave 1 follow-up
- **architecture**: Use bun for csl intake examples
- Update JSON schemas

### Features

- **processor**: Normalize note citations and locator rules
- **report**: Add note citation type coverage
- **processor**: Add locator label controls
- **migrate**: Automate output-driven templates
- **migrate**: Score inferred template confidence
- **report**: Add SQI and sortable compat table
- **style**: Complete core fidelity and probes
- **migrate**: Carry scope-aware name/disambig config
- **report**: Replace status column with sqi tier
- **label**: Implement label citation mode
- **core**: Migrate and refine next five styles
- **report**: Auto-discover styles for compat
- **report**: Add style search to compat table
- **styles**: Add 58 priority styles with presets
- **presets**: Expand sqi preset extraction
- **presets**: Add 6 new ContributorPreset variants
- **core**: Migrate workspace to Rust 2024 edition
- **processor**: Sort numeric label assignment
- Implement declarative macros for CSLN
- **presets**: Add numeric contributor variants
- **styles**: Migrate wave1 and wave2 batches
- **wave3**: Seed baseline styles and metrics
- **styles**: Add ams-label and alpha label styles
- **embedded**: Add 12 priority styles with builtin CLI support
- **mla**: Add disambiguate_only title field and MLA style
- **core**: Add FamilyOnly and DayMonthAbbrYear to schema
- **processor**: Support FamilyOnly and DayMonthAbbrYear rendering
- **cli**: Comprehensive UX overhaul
- **bindings/latex**: Add LuaLaTeX integration
- **grouping**: Add style targets and document tests
- **styles**: Add basic multilingual config to apa and mla
- **styles**: Localize sectional group headings
- **styles**: Add gost-r-7-0-5-2008 grouping styles
- **processor**: Add position-based citation variants (ibid, subsequent)
- **migrate**: Finish phase3/4 and wave4
- **beans**: Add citum wrapper and smart next
- **migrate**: Complete variable-once cross-list deduplication
- **migration**: Add component coverage gate
- **ci**: Add oracle regression baseline gate
- **styles**: Multilingual YAML styles csl26-mls1
- **server**: Add citum-server crate
- **testing**: Add interactive fixture generator
- **multilingual**: Support language-aware title templates
- **document**: Auto-note djot citations
- **document**: Configure note marker punctuation
- **edtf**: Implement time component rendering
- **schema**: Add SortPreset; use in chicago
- **multilingual**: Add preferred-transliteration
- **note-styles**: Ibid/subsequent to chicago-notes
- **cli**: Default output format to html
- **server**: Support output formats
- **schema**: Add CitationField, StyleSource provenance to StyleInfo
- **processor**: Add document-level bibliography grouping via djot and YAML
- **typst**: Add native rendering and pdf output
- **testing**: Add csl intake audit
- **engine**: Support container short titles
- **scripts**: Add csl intake progress summary
- **schema,engine**: Add subsequent et-al controls
- **tests**: Extract CJK/Arabic CSL fixtures + native test
- **tooling**: Add --dry-run flag to release.sh

### Miscellaneous Tasks

- **ci**: Update compat report
- **ci**: Update compat report
- **compat**: Rename report-top10 to report-core
- **ci**: Update compat report
- **beans**: Track label processing mode as feature
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **beans**: Add gen block refactoring task (csl26-j80k)
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **plan**: Add ams and alpha planning tasks
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **beans**: Add MLA processor handoff tasks
- **ci**: Update compat report
- **ci**: Update compat report
- Update gitignore to ignore dylib
- **beans**: Mark csl26-grp1 in progress
- **beans**: Record grouping fixture progress
- **beans**: Note localized grouping heading progress
- **beans**: Close csl26-grp1 as completed
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **skills**: Add metadata, delete styleauthor
- **skills**: Update
- **beans**: Close csl26-b76h
- Clean out temp files
- **beans**: Add detail to x08y
- **beans**: Add versioning and schema dispatch tasks
- **claude**: Remove styleauthor legacy references
- **beans**: Add bodies to gap beans from soundness assessment
- **ci**: Update compat report
- **repo**: Remove bindings from core after labs extraction
- **ci**: Update compat report
- **ci**: Update compat report
- **docs**: Align beans and docs hygiene
- **docs**: Remove CSLN branding from compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **beans**: Fix l2hg epic — add checklist of blocking children
- **beans**: Close l2hg epic and completed child beans
- **beans**: Mark 13 migration beans completed, remove phantom dep
- **ci**: Update compat report
- **beans**: Close iek4
- **beans**: Close csl26-6n3c
- **beans**: Move prime hook to settings, clean skill
- **beans**: Close 7yfp and vdrn as resolved
- **beans**: Close csl26-yrri
- **beans**: Close csl26-4rg8
- **beans**: Close csl26-cih2 and csl26-w0gt
- **ci**: Update compat report
- **beans**: Clean up
- **ci**: Update compat report
- **beans**: Close csl26-srvr epic and children
- **testing**: Consolidate infrastructure contracts
- **beans**: Close testing infrastructure epic
- **beans**: Close n79w
- **beans**: Close x08y
- **beans**: Update t052
- **beans**: Close csl26-2tjp
- **beans**: Scrap csl26-5axq (ICU not needed)
- **styles**: Apply sort presets to 22 styles
- **beans**: Track duplicate sort key cleanup
- **ci**: Update compat report
- **styles**: Dedup sort keys in 16 styles
- **beans**: Close csl26-1mkv
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **beans**: Mark csl26-erd0 completed
- Update rules
- **ci**: Update compat report
- **beans**: Add toasty note
- **beans**: Normalize completed icu evaluation
- **ci**: Update compat report
- **ci**: Update compat report
- **beans**: Bulk update
- **beans**: Complete csl26-extg
- **beans**: Complete csl26-380v
- **beans**: Complete csl26-mo6c
- **release**: Bump workspace version to 0.7.0
- **tooling**: Add release.sh for workspace version bumps
- **tooling**: Add patch/minor/major bumping and major confirmation to release.sh

### Refactor

- **sqi**: Align core scoring across styles
- **migrate**: Harden inferred template merge
- **tests**: Add reference-builder macros and migrate test boilerplate
- **citation**: Move suppress-author to citation level
- **workflow**: Unify style-evolve workflow
- **processor**: Remove unused clap dependency
- **core**: Decouple csln_core from csl_legacy and biblatex
- **workspace**: Migrate csln namespace to citum
- **cli**: Make citum the only public binary name
- **beans**: Clean up /beans next human output
- **edtf**: Rename crate csln-edtf → citum-edtf
- **migrate**: Trim redundant bibliography sorts
- **migrate**: Modularize template_compiler

### Styling

- **vancouver**: Raise elsevier-vancouver match
- **top10**: Add springer-socpsych-author-date
- **chicago-notes**: Reach oracle citation parity
- **core**: Improve citation fidelity across core set
- **core**: Remove annals from repository
- **tfca**: Raise fidelity above 25 percent
- **core**: Use locator label controls
- **american-medical-association**: Expand bib types
- **chicago-notes**: Cover note reference types
- **batch**: Migrate and enhance next 10
- **priority**: Complete next-10 wave
- **priority**: Migrate and enhance next 20
- **batch**: Raise wave-100 fidelity floor
- **mla**: Apply new template components to MLA
- Improve SQI metrics
- **core**: Lift SQI via numeric citation presetization
- **migrate**: Update generated styles and work logs

### Testing

- **metadata**: Add test coverage for new MLA forms
- **fixtures**: Add legal hierarchy grouping fixture
- **grouping**: Cover jm legal heading order
- **grouping**: Cover localized heading fallback
- **server**: Add RPC dispatcher integration tests
- **server**: Cover http mode
- **engine**: Add sort oracle tests
- **citations**: Cover empty-date citation sort

### Ci

- **sqi**: Add core fidelity gate and drift checks
- **compat**: Run report on all main pushes
- **release**: Fix release-plz package override

### Revert

- **cli**: Restore plain as default output format

## [0.6.0] - 2026-02-19

### Bug Fixes

- Add missing variables and enable strict validation
- **schema**: Allow string presets for contributors, dates, and titles
- **ci**: Remove npm install step — node_modules is committed to repo
- **ci**: Add workflow_dispatch to compat-report
- **ci**: Trigger Pages deploy after compat report commit
- **ci**: Track package.json and restore npm install step
- **ci**: Add concurrency group to prevent parallel runs
- **proc**: Improve grouped cites and quotes
- **examples**: Improve example bibliography and citation files
- **lua**: Portable FFI loading and lifecycle
- **springer**: Raise full-oracle bibliography fidelity above 90%
- **apa**: Handle legal/personal citation edges
- **csln**: Align process alias with render refs flags
- **scripts**: Harden compatibility reporting and fix ama style

### Documentation

- **web**: Remove paragraph/broken link
- Update JSON schemas
- Update JSON schemas
- Add editor integration note for json schemas
- **web**: Add other editors for schemas
- Add copy buttons and dev note to index
- **web**: Announce LaTeX renderer and C-FFI integration
- **beans**: Update rendering and FFI tasks
- **web**: Remove sentence on lualatex
- Update JSON schemas
- **bindings**: Remove unneeded ref
- Update JSON schemas
- **processor**: Fix table formatting
- **site**: Update cli examples to render/check contract
- **bean**: Refine boltffi plan with phased binding strategy
- **styleauthor**: Add targeted fidelity efficiency guidance
- **bean**: Update apa-7th fidelity progress
- **styleauthor**: Standardize on full-fixture oracle workflow
- **web**: Sync schema publishing and style scope

### Features

- **beans**: Document-level bibliography grouping
- Overhaul djot citation syntax
- **core**: Introduce ComponentOverride enum
- **core**: Achieve strict template validation
- **processor**: Add native LaTeX renderer
- **processor**: Add universal C-FFI bridge
- **bindings**: Add LuaJIT FFI binding for LuaLaTeX
- **processor**: Fix djot citation parsing and rendering
- **processor**: Simplify citation model and djot support
- **render**: Full latex support in csln process
- **report**: Add top-10 style compatibility report
- **report**: Richer compatibility metrics and detail view
- **style**: Modernize apa-7th conjunctions
- **bean**: Add top-10 style reporting task
- **core**: Add TypeSelector and semantic number fields
- **style**: Improve apa-7th fidelity and add documentation
- **cli**: Improve error handling for input files
- **apa-7th**: Push bibliography fidelity above 20 matches
- Add complex citations to oracle
- **processor**: Implement citation sorting and improved grouping

### Miscellaneous Tasks

- Add json schema link to style, bib files
- Duplicate yaml header line
- Formalize benchmark workflow
- Improve benchmark script transparency
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **tool**: Enhance oracle scripts
- **ci**: Update compat report
- **tool**: Refine oracle matching and detection
- **ci**: Update compat report
- Remove temporary workfile
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **styles**: Remove obsolete test style fixtures
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- **git**: Ignore .oracle-cache
- **ci**: Update compat report
- **ci**: Update compat report
- **ci**: Update compat report
- Gitignore
- **ci**: Update compat report
- **workflow**: Align oracle and style catalog
- **ci**: Update compat report
- **release**: Bump workspace version to 0.6.0

### Refactor

- **processor**: Format-aware value extraction pipeline
- **csln**: Unify cli ux around render/check
- **csln**: Remove process command alias
- **i18n**: Localize bib group headings

### Styling

- Improve elsevier-with-titles bibliography fidelity
- Improve elsevier-harvard bibliography coverage
- **elsevier-harvard**: Raise fidelity >90%
- **apa-7th**: Reach full oracle fidelity

### Testing

- **i18n**: Align multilingual tests
- **processor**: Wire domain fixtures into CI runs

### Beans

- Explore boltffi for binding gen
- **kjzk**: Add boltffi planning analysis

### Core

- Map interview-like monographs to interview ref type
- **render**: Fix substitution and type matching

### Examples

- **djot**: Make it more representative

## [0.5.0] - 2026-02-16

### Bug Fixes

- Document processing for numeric styles
- Correct date format in document-refs.json
- **release**: Remove unsupported update_all_packages field
- **processor**: Resolve clippy warnings in document tests
- Remove duplicate default profile in nextest config
- **nextest**: Correct config field types
- **ci**: Remove nextest to avoid yanked deps
- Correct lint errors and model name mismatches
- **release**: Consolidate to root changelog and align versioning

### Documentation

- Add bibliography grouping design
- **web**: Add grouping feature
- Reorder features and add date examples
- Plan multilingual/grouped disambiguation
- Align docs with test reorg and cleanup files
- Add Quick Start integration guide for interactivity
- Connect interactive demo with configuration examples
- Clarify interactive enhancements framing
- Restructure documentation directory
- Integrate schema generation into docs/ directory
- Add Schemas link to navigation
- Update JSON schemas
- Add JSON schemas section to index
- Add project origins and vision to landing page
- Add localized disambiguation example
- Complete bibliography grouping documentation
- Update JSON schemas
- **grouping**: Add primary/secondary sources
- **beans**: Track external grouping and heading localization

### Features

- **style**: Support integral citations in Springer Vancouver
- **djot**: Implement citation visibility modifiers and grouping 
- Configurable bibliography grouping
- **beans**: Add typst output format
- **beans**: Add interactive html css/js
- **beans**: Add deno evaluation task
- **test**: Add CSL test suite for disambiguation
- Implement full disambiguation system
- **web**: Implement interactive HTML renderer
- **dx**: Optimize binary size and automate schema publishing
- **dx**: Export and publish all top-level schemas
- **core**: Support legal reference conversion
- **grouping**: Implement group disambiguation

### Miscellaneous Tasks

- **examples**: Tweak document.djot
- **release**: Sync workspace versions and tag format
- Shorten project name
- Release v0.3.0 (#168)
- Integrate nextest for parallel testing
- Update dependency versions
- Bump version to 0.4.0 and sync changelog
- Release v0.4.0
- **beans**: Update grouping task metadata
- Bump version to 0.5.0
- Release v0.5.0

### Refactor

- **test**: Use pure CSLN types

### Testing

- Add author-date integral locator test
- Remove broken integral locator test
- **processor**: Expand native test suite and refactor existing tests
- **processor**: Reorganize integration tests into functional targets

## [0.3.0] - 2026-02-15

### Bug Fixes

- **csln_migrate**: Improve substitute extraction for real styles
- Variable-once rule and serde parsing for Variable components
- Handle is-uncertain-date condition in migration
- **migrate**: Improve template compilation
- **processor**: Use container_title for chapter book titles
- **migrate**: Disable auto chapter type_template generation
- **migrate**: Improve citation delimiter extraction
- **processor**: Correctly map ParentSerial/ParentMonograph to container_title
- **migrate**: Extract date wrapping from original CSL style
- **migrate**: Add editor/container-title for chapters, suppress journal publisher
- **migrate**: Add page formatting overrides for journals and chapters
- **migrate**: Resolve template nesting regression with recursive deduplication
- **migrate**: Context-aware contributor option extraction
- **migrate**: Recursive type overrides for nested components
- **migrate**: Improve CSL extraction and template generation
- **processor**: Implement contributor verb and label forms
- **migrate**: Suppress pages for chicago chapters
- **migrate**: Chicago publisher-place visibility rules
- **migrate**: Remove comma before volume for chicago journals
- **migrate**: Use space suffix for chicago journal titles
- **processor**: Add context-aware delimiter for two-author bibliographies
- **processor**: Implement variable-once rule for substituted titles
- **migrate**: Extract 'and' configuration from citation macros
- **migrate**: Use full names in bibliography for styles without style-level initialize-with (#56)
- **processor**: Improve bibliography sorting with proper key chaining
- **sort**: Strip leading articles and fix anonymous work formatting
- **render**: Suppress trailing period after URLs in nested lists
- **reference**: Extract actual day from EDTF dates
- **migrate**: Improve bibliography template extraction
- **processor**: Add contributor labels and sorting fixes
- **migrate**: Deduplicate nested lists and fix volume-issue grouping
- Position year at end of bibliography for numeric styles
- Restore working template compiler from pre-modularization
- **migrate**: Extract author from substitute when primary is rare role
- **migrate**: Detect numeric styles and position year at end
- **migrate**: Add space prefix to volume after journal name
- **workflow**: Implement opus review critical fixes and strategy updates
- **gitignore**: Improve baselines directory exclusion pattern
- **migrate**: Extract correct citation delimiter from innermost group
- **migrate**: Handle Choose blocks in delimiter extraction
- **migrate**: Extract bibliography delimiter from nested groups
- **migrate**: Improve bibliography component extraction for nested variables
- **migrate**: Prevent duplicate list variables
- **migrate**: Improve contributor and bibliography migration
- **migrate**: Add text-case support for term nodes and deduplicate numbers
- **migrate**: Use IndexMap to preserve component ordering
- **migrate**: Disable hardcoded component sorting
- **migrate**: Add date deduplication in lists
- **migrate**: Preserve label_form from CSL 1.0 Label nodes
- Improve list component merging in template compiler
- **migrate**: Preserve macro call order across choose branches
- **scripts**: Attach overrides to template objects for JSON output
- **scripts**: Per-type confidence metric for template inferrer
- **scripts**: Detect prefixes and emit delimiter in inferrer output
- **render,inferrer**: Improve delimiter detection and URL suffix handling
- Improve bibliography separator handling for wrapped components
- **migrate**: Correct contributor name order logic
- **core**: Enable initialize-with override on contributor components
- **processor**: Resolve mode-dependent conjunctions and implement deep config merging
- **bibliography**: Preserve component suffixes in separator deduplication
- **core**: Alias DOI/URL/ISBN/ISSN for CSL-JSON
- **web**: Add scroll margin to example anchors
- **styles**: Update metadata to CSLN conventions
- **locale**: Handle nested Forms in role term extraction
- **processor**: Allow variable repetition with different context
- **processor**: Author substitution and grouping bugs
- **processor**: Use correct jotdown API
- **processor**: Prevent HTML escaping in docs
- **styles**: APA integral and config
- Release-plz, actions config
- Remove per-crate changelogs and configure single release

### Documentation

- Add refactor plan for csln core alignment and baseline analysis
- Add architecture principles and improve code comments
- Add PERSONAS.md for feature evaluation
- Update README and AGENTS.md with analyzer info
- Update status and add branch workflow rule
- Explicitly forbid agent from merging to main
- Update development principles and roadmap
- Add Domain Expert persona to PERSONAS.md
- Integrate skills and personas into agents.md
- **agent**: Add prior art research and roadmap
- Establish ai-driven contribution workflow
- Add table of contents to readme
- 3 -> 4 agents
- Fix links
- Update agent state for chapter formatting work
- Update state after disabling chapter auto-gen
- **state**: Update state.json with delimiter fix progress
- **agent**: Add style editor vision document
- Update status and reduce APA-centric guidance
- Update project status and development guidelines
- Add PR workflow for AI agent
- Add style aliasing and presets design document
- Clarify AI contribution model in README
- Use doc comments for schema-visible preset descriptions
- Add presets and embedded templates to README
- Mark preset-aware migration as complete in roadmap
- Update STYLE_ALIASING.md with implementation status
- **workflow**: Require maintainer approval before merging PRs
- **design**: Update style aliasing decisions
- Update roadmap with recent progress
- **workflow**: Strict message formatting rules
- Add delimiter enum refactoring TODO
- Add test data expansion TODO
- Add punctuation normalization design doc
- Add oracle parity session summary
- Add comprehensive style test results
- Add source link for punctuation rules
- Document remaining APA bibliography issues
- Minor clarification
- Populate high priority features table
- **reference**: Convert parent-by-id TODO
- **migrate**: Convert remaining TODOs to issues
- Update status tracking for recent fixes
- Update success criteria with actual scores
- Add tier 3 plan for numeric styles and tooling improvements
- Update tier 3 plan with implemented tooling
- Update tier 3 plan with tooling insights
- Broaden style editor vision to platform
- Link to new style-editor repository
- Update architecture diagram in README
- Add autonomous command whitelist
- Add rendering fidelity workflow analysis
- Add pull request convention rule
- Add task list to version control
- Replace tier plans with agent-friendly status document
- Rewrite workflow analysis for llm agents and consolidate skills
- Add mandatory pre-pr checklist to prevent ci failures
- Fix binary path in task skill documentation
- Add task management section to readme
- Enable rapid development workflow on main
- **architecture**: Add migration strategy analysis
- **architecture**: Revise migration strategy analysis
- **architecture**: Add validation results to migration strategy analysis
- Document mode-dependent citation formatting
- Adopt hybrid core vs. community style strategy
- **beans**: Track punctuation suppression bug
- **skills**: Prefer wrap and delimiters for semantic joining
- **beans**: Track support for inner and outer affixes
- Update rendering workflow and migration strategy analysis
- Update README with output formatting and semantic markup instructions
- Align rendering workflow with hybrid migration strategy
- **beans**: Add csl26-5qh6 to track semantic tagging bug
- **multilingual**: Add architectural design for multilingual support
- Add multilingual graceful degradation principles to CLAUDE.md
- Add documentation website and deployment workflow
- Update install command to use git repo
- Comment out incorrect version banner
- Fix incorrect syntax in examples page
- Fix incorrect syntax in feature examples
- **claude**: Enhance autonomous command whitelist with safety tiers
- Add CLAUDE.md and task files to autonomous whitelist
- **claude**: Fix autonomous command whitelist
- **examples**: Clarify EDTF uses locale terms not hardcoded values
- **examples**: Add info field and restructure bibliography files
- **bench**: Add benchmark requirements policy
- **beans**: Add multilingual punctuation scope
- **instructions**: Add humanizer skill integration
- **web**: Link landing page features to examples
- **web**: Expand examples and align with landing page features
- Align with hybrid migration strategy
- Align with hybrid migration strategy
- **web**: Add style hub links
- **examples**: Add chaucer with edtf approximate date
- **web**: Add pluggable output formats to documentation
- Add legal citations design document
- Add type system architecture analysis
- Add type addition policy and finalize architecture
- Audit current types against 4-factor policy
- Add project roadmap to index page
- Scope pre-commit checks to rust changes
- Update website and beans for document processing progress
- Promote document processing to features grid
- **web**: Clarify DJOT support
- Expand advanced citation examples
- Refine narrative citation examples
- Reorder features by importance
- Add wasm support design doc and bean
- Sync beans and roadmap with project state
- Add schema versioning policy

### Features

- Initial commit of CSLN Architecture
- Proof-of-concept CSLN Renderer
- Enhanced Names handling and Verification
- Locale Ingestion and Advanced Name Mapping
- **csln_core, csln_migrate**: Add CSLN schema and OptionsExtractor
- **csln_migrate**: Integrate OptionsExtractor into migration CLI
- **csln_migrate**: Add TemplateCompiler for clean CSLN output
- **scripts**: Add citeproc-js oracle for verification
- **csln_migrate**: Improve template ordering and author-date citation
- Type-specific overrides for APA formatting
- Achieve 5/5 oracle match with name_order control
- Add style-level options for name initialization
- Add csln_analyze tool for corpus analysis
- Implement page-range-format (minimal, chicago, expanded)
- Implement delimiter-precedes-et-al (786 styles)
- Implement demote-non-dropping-particle (2,570 styles)
- Add GitHub CI workflow for Rust
- Implement name and given-name disambiguation
- **bib**: Implement subsequent author substitution
- **core**: Implement style versioning and forward compatibility
- Update legacy name parsing and processor support
- **core**: Add json schema generation support and docs
- **processor**: Add citation layout support
- **core**: Add multi-language locale support
- **migrate**: Extract bibliography sort and fix citation delimiter
- **processor**: Add bibliography entry numbering for numeric styles
- **processor**: Fix name initials formatting and extraction
- **processor**: Support per-component name conjunction override
- **core**: Add overrides support to contributor and date components
- **core**: Add else-if branches and type-specific bibliography templates
- **migrate**: Add type-specific template extraction (disabled)
- **migrate**: Add chapter type_template for author-date styles
- **processor**: Implement declarative title and contributor rendering logic
- **render**: Add title quotes and fix period-inside-quotes
- **locale**: Implement punctuation-in-quote as locale option
- **migrate**: Extract volume-pages delimiter from CSL styles
- **migrate**: Extract bibliography entry suffix from CSL layout
- Add new CSLN reference model and biblatex crate
- **analyze**: Add parent style ranking for dependent styles
- **core**: Add style preset vocabulary for Phase 1
- **core**: Add embedded priority templates for Phase 2
- **migrate**: Add preset detection for extracted configs
- **core**: Expose embedded templates via use-preset
- **contributor**: Implement et-al-use-last truncation
- **render**: Implement structured hyperlinking in templates
- **core**: Enhance citation model and add bibliography separator config
- **core,processor**: Implement curly quote rendering
- Complete Chicago Author-Date bibliography rendering (5/5) (#52)
- **test**: Expand test data to 15 reference items (#53)
- **processor**: Achieve 15/15 oracle parity for Chicago and APA (#54)
- **processor**: Add citation grouping and year suffix ordering
- **options**: Add configurable URL trailing period
- **options**: Add substitute presets and style-aware contributor matching
- **locale**: Expose locator terms for page labels
- **migrate**: Infer month format from CSL date-parts
- **reference**: Support parent reference by ID
- **migrate**: Support type-conditional substitution extraction
- **core**: Implement editor label format standardization
- **processor**: Improve bibliography separator handling
- **migrate**: Improve migration fidelity and deduplication
- **migrate**: Implement publisher-place visibility rules
- **core**: Add prefix_inside_wrap for flexible wrap ordering
- **scripts**: Add structured diff oracle for component-level comparison
- **scripts**: Add batch oracle aggregator for pattern detection
- **scripts**: Add parallel execution and --all flag for corpus analysis
- Add comprehensive bibliographic examples and schema updates
- Unify reference models and fix processor tests
- Improve APA bibliography formatting and core infrastructure
- Implement phase 1 workflow improvements
- **workflow**: Add regression detection with baseline tracking
- **migrate**: Add variable provenance debugger
- **csl-tasks**: Implement task management CLI
- **csl-tasks**: Add ux improvements for local-first workflow
- **task-cli**: Fill out task management skill
- **csl-tasks**: Implement GitHub issue number alignment for task IDs
- **csl-tasks**: Improve GitHub sync error handling
- **csl-tasks**: Add duplicate detection for github sync
- Convert github issues to beans for rendering workflow
- **migrate**: Add custom delimiter support for CSL 1.0 compatibility
- **migrate**: Implement complete source_order tracking system
- **fixtures**: Expand test references to 28 items across 17 types
- **scripts**: Add output-driven template inference engine
- **scripts**: Add prefix, wrap, and items grouping to inferrer
- **scripts**: Show per-type confidence in verbose output
- **scripts**: Add formatting inference and parent-monograph detection
- **migrate**: Integrate template resolution cascade with per-component delimiters
- Refine container template inference grouping
- **core,processor**: Add locator support and refine punctuation rendering
- **styles**: Add APA 7th edition CSLN style
- **core,processor**: Add locator support, mode-dependent logic, and integral citation templates
- **styles**: Add APA 7th edition CSLN style with integral/narrative support
- **skills**: Add styleauthor skill and agent for LLM-driven style creation
- **github**: Add style request issue template
- Wire up three-tier options architecture
- Improve template inference and sync tests
- **migration**: Add styleauthor migration pathway
- **styles**: Add elsevier-with-titles style
- **rendering**: Implement inner/outer affixes
- Implement pluggable output rendering and semantic markup
- Migrate and process localized terms
- Add structured html output for bibliography
- **skills**: Add update to styleauthor
- Adapt styleauthor to minmax tri-agent workflow
- **core**: Add InputBibliography and TemplateDate fallback support
- **processor**: Improve rendering engine and test dataset
- **styles**: Add chicago manual of style 18th edition (author-date)
- **processor**: Add integral citation mode to CLI output
- **multilingual**: Implement holistic parallel metadata for names and titles
- **multilingual**: Implement multilingual support
- **styles**: Add elsevier-harvard author-date style
- **styleauthor**: Add workflow optimizations
- Implement elsevier-vancouver style (WIP)
- **styles**: Add elsevier-vancouver
- **core**: Implement declarative hyperlink configuration
- **styleauthor**: Add autonomous command whitelist
- **cli**: Merge process and validate into csln
- **processor**: Implement multilingual BCP 47 resolution
- Add CBOR binary format support and conversion tool
- **dates**: Implement EDTF uncertainty, approximation, and range rendering
- **styles**: Implement springer-vancouver-brackets.yaml
- **styles**: Add springer-vancouver-brackets style
- **workflow**: Optimize styleauthor migration workflow
- **edtf**: Implement modern winnow-based parser
- Add performance benchmarking
- Implement schema generation, validation
- Automated migration workflow with infer-template.js
- **processor**: Implement strip-periods in term and number labels
- **styles**: Add strip-periods to springer-basic
- **cli**: Add --show-keys flag to process command
- **workflow**: Migration workflow optimizations
- Fix override fallback and migrate Springer style
- **presets**: Add options-level preset support
- **styles**: Add taylor-and-francis-chicago-author-date
- **core**: Add Tier 1 legal reference types
- **styles**: Add legal-case override to APA 7th
- **core**: Add Patent and Dataset reference types
- **core**: Add Standard and Software types
- **cli**: Support complex citation models as input
- **processor**: Add document-level processing prototype
- **processor**: Implement WinnowCitationParser for Djot syntax
- **processor**: Simplify Djot citation syntax by removing mandatory attribute
- **processor**: Support hybrid and structured locators in Djot parser
- **processor**: Implement djot document processing and structured locators
- **processor**: Add HTML output for Djot document processing
- **citations**: Add infix support for integrals
- **core**: Add locale term role labels
- **styles**: Add label config to AMA
- **processor**: Support infix variable in integral citations

### Miscellaneous Tasks

- Update GEMINI_STATE for phase 2c completion
- Update state - Phase 2 complete with APA semantic match
- Add git-advanced skill
- Update analysis tool with new known name attributes
- Add claude code symlinks
- Consolidate design docs and gitignore debug artifacts
- Add claude code permission whitelist
- Remove obsolete state file and temporary test outputs
- Add task-next skill
- Remove task-next skill
- Add tasks directory to gitignore
- Update gitignore for local data files
- Remove tasks dir
- **beans**: Restructure task hierarchy with migration and rendering epics
- **beans**: Update
- **beans**: Update
- **claude**: Refining permissions
- **beans**: Document failed source_order approach
- **beans**: Integrate hybrid migration strategy
- **beans**: Restructure migration strategy beans
- **beans**: Add AI-assisted authoring notes to csl26-o3ek
- **beans**: Mark csl26-qb6h as completed
- **beans**: Mark csl26-l05b as completed
- **beans**: Mark csl26-z8rc as completed
- **beans**: Add docs and enhancement tasks for template inferrer
- **beans**: Update csl26-lhxi progress (4/6 done, 2 deferred)
- **beans**: Mark csl26-lhxi as completed
- **beans**: Mark csl26-25v6 completed, create csl26-9a89 for rendering
- **beans**: Update csl26-9a89 with rendering progress and next steps
- **github**: Remove domain exp templ
- Rename styles directory to styles-legacy
- Fix paths
- **beans**: Add ai enhancement migration
- **beans**: Update
- **skills**: Bean -> beans
- **beans**: Update
- **beans**: Add doc todo
- Fix path
- **context**: Add context files
- **skills**: Update workflow
- **tasks**: Mark csl26-0s2b completed
- **tasks**: Link DOI bug to hyperlink feature
- **tasks**: Add bean for EDTF time component followup
- **config**: Simplify autonomous operations config
- Update apa-7th.yaml with verified auto-migrated results
- Mark strip-periods bean as completed
- Update gitignore and remove bin files
- **cli**: Use explicit short flags and fix annals citation
- Final clippy fixes and document processing polish
- **tasks**: Update hybrid migration task
- Add automated code versioning
- Bump initial version to 0.3.0
- Release v0.3.0

### Refactor

- **core**: Use DelimiterPunctuation enum for volume_pages_delimiter
- **core**: Remove feature gate from embedded templates
- Modularize core crates
- Modularize core and processor crates
- Migrate to claude code native tasks
- **migrate**: Implement occurrence-based template compilation
- Migrate from csl-tasks to beans for local task management
- **beans**: Reorganize tasks into epic structure
- **scripts**: Harden oracle component parser
- Rename binaries from underscores to hyphens
- Remove processor magic and fix punctuation suppression
- **styleauthor**: Use Sonnet + checkpoints
- **cli**: Csln-processor -> csln process
- **core**: Strict typing with custom fields
- **processor**: Modularize document processing

### Styling

- Fix formatting and clippy warnings
- **migrate**: Fix doc comment indentation for clippy
- Format code with cargo fmt
- Fix narrative citation rendering and CMOS delimiters

### Agent

- Remove generated line from pr

### Ci

- Add rust caching for faster builds

