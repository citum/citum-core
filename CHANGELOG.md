# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-03-01

### Bug Fixes

**beans**

- Filter non-actionable statuses in next

- Prefer executable next targets


**chicago**

- Add bibliography sort for anon works


**chicago-author-date-classic**

- Clean up duplicate keys


**chicago-shortened**

- Close patent and personal-comm bibliography gaps


**ci**

- Skip metadata-only compat commits

- Normalize compat hash-only marker

- Harden core-quality style resolution

- Stabilize local and oracle gates


**cli**

- Add --builtin support to render doc and check commands

- Wire grouped bibliography rendering to render refs


**compat**

- Normalize oracle text and lift springer

- Raise springer-vancouver above 90


**core**

- Match default overrides only as fallback

- Normalize type-name casing in TypeSelector::matches

- Switch InputReference from untagged to class-tagged dispatch


**delimiter**

- Normalize enum parsing across engine and migrate


**docs**

- Prevent nav overlap

- Restore nav on md screens

- Align compat branding to citum

- Add soon badges to Hub and Labs in nav


**engine**

- Normalize space-only initials formatting

- Sort undated bibliography entries last

- Complete high-fit wave 1 regressions

- Format title apostrophe logic

- Thread position into et-al renderer


**examples**

- Validate and fix refs YAML files


**labels**

- Make et-al name count configurable per preset


**labels,names**

- Docs and test coverage for label mode and space separator


**latex**

- Escape bare & in finish() for valid LaTeX output


**locale**

- Preserve locator term forms from yaml

- Lowercase editor short terms (ed./eds.)


**migrate**

- Make inferred mode cache-only

- Guard inferred citation templates

- Preserve help flag in pr rewrite

- Recover branch-specific numeric fidelity

- Scope inferred type-template merges

- Improve numeric sort fidelity

- Improve wave3 citation fidelity

- Improve migration logic for legal and special types

- Fix subsequent et-al propagation


**oracle**

- Use token-similarity matching

- Strip bibliography numbering after whitespace normalization


**processing**

- Make bibliography sort defaults explicit


**processor**

- Stabilize numeric cite ordering

- Align note locator and serial metadata

- Calculate group_length for author-format disambiguation

- Repair bibliography block rendering

- Add trailing newline after bibliography blocks

- Strip YAML frontmatter from rendered output


**render**

- Make HTML bibliography separators markup-aware


**rendering**

- Smarten leading single quotes


**report**

- Include tier-2 wave in compat

- Infer format for !custom and missing processing styles


**schema**

- Drop is_base from StyleInfo and csl-legacy Info

- Address missing field_languages field in ref_article_authors macro

- Restore schema publishing outputs


**styles**

- Fix citation templates across 27 styles

- Modernize MLA and refine citations

- Correct gost processing variants

- Bring all styles to 51%+ fidelity and 80%+ SQI

- Fix 3 springer-physics bib failures

- Close top-10 bibliography deltas

- Rename CSLN to Citum in apa-7th title

- Remove parenthetical from apa-7th title


**tooling**

- Align schema validator with repo inputs



### Documentation

**architecture**

- Add migrate wave handoff docs

- Consolidate wave runbook

- Define style target roadmap

- Refresh sqi plan status

- Update Section 3 to reflect deny_unknown_fields trade-off

- Add architectural soundness assessment and gap beans

- Close csl26-o18j with verification

- Use bun for csl intake examples


**examples**

- Add multilingual and bib-grouping runnable examples


**guide**

- Add IDE editor integration callout

- Fix enum values from schema review

- Expand preset catalog with all 14 contributor presets


**migrate**

- Add csln-migrate guide


**multilingual**

- Clarify field-scoped language metadata


**schema,engine,migrate**

- Add public API doc comments


**server**

- Clarify dependency graph in plan

- Add HTTP curl example to README

- Fix stdio example with valid JSON



### Features

**beans**

- Add citum wrapper and smart next


**bindings/latex**

- Add LuaLaTeX integration


**ci**

- Add oracle regression baseline gate


**cli**

- Comprehensive UX overhaul

- Default output format to html


**core**

- Migrate and refine next five styles

- Migrate workspace to Rust 2024 edition

- Add FamilyOnly and DayMonthAbbrYear to schema


**document**

- Auto-note djot citations

- Configure note marker punctuation


**edtf**

- Implement time component rendering


**embedded**

- Add 12 priority styles with builtin CLI support


**engine**

- Support container short titles


**grouping**

- Add style targets and document tests


**label**

- Implement label citation mode


**migrate**

- Automate output-driven templates

- Score inferred template confidence

- Carry scope-aware name/disambig config

- Finish phase3/4 and wave4

- Complete variable-once cross-list deduplication


**migration**

- Add component coverage gate


**mla**

- Add disambiguate_only title field and MLA style


**multilingual**

- Support language-aware title templates

- Add preferred-transliteration


**note-styles**

- Ibid/subsequent to chicago-notes


**presets**

- Expand sqi preset extraction

- Add 6 new ContributorPreset variants

- Add numeric contributor variants


**processor**

- Normalize note citations and locator rules

- Add locator label controls

- Sort numeric label assignment

- Support FamilyOnly and DayMonthAbbrYear rendering

- Add position-based citation variants (ibid, subsequent)

- Add document-level bibliography grouping via djot and YAML


**report**

- Add note citation type coverage

- Add SQI and sortable compat table

- Replace status column with sqi tier

- Auto-discover styles for compat

- Add style search to compat table


**schema**

- Add SortPreset; use in chicago

- Add CitationField, StyleSource provenance to StyleInfo


**schema,engine**

- Add subsequent et-al controls


**scripts**

- Add csl intake progress summary


**server**

- Add citum-server crate

- Support output formats


**style**

- Complete core fidelity and probes


**styles**

- Add 58 priority styles with presets

- Migrate wave1 and wave2 batches

- Add ams-label and alpha label styles

- Add basic multilingual config to apa and mla

- Localize sectional group headings

- Add gost-r-7-0-5-2008 grouping styles

- Multilingual YAML styles csl26-mls1


**testing**

- Add interactive fixture generator

- Add csl intake audit


**tests**

- Extract CJK/Arabic CSL fixtures + native test


**tooling**

- Add --dry-run flag to release.sh


**typst**

- Add native rendering and pdf output


**wave3**

- Seed baseline styles and metrics



### Refactor

**beans**

- Clean up /beans next human output


**citation**

- Move suppress-author to citation level


**cli**

- Make citum the only public binary name


**core**

- Decouple csln_core from csl_legacy and biblatex


**edtf**

- Rename crate csln-edtf → citum-edtf


**migrate**

- Harden inferred template merge

- Trim redundant bibliography sorts

- Modularize template_compiler


**processor**

- Remove unused clap dependency


**sqi**

- Align core scoring across styles


**tests**

- Add reference-builder macros and migrate test boilerplate


**workflow**

- Unify style-evolve workflow


**workspace**

- Migrate csln namespace to citum



### Styling

**american-medical-association**

- Expand bib types


**batch**

- Migrate and enhance next 10

- Raise wave-100 fidelity floor


**chicago-notes**

- Reach oracle citation parity

- Cover note reference types


**core**

- Improve citation fidelity across core set

- Remove annals from repository

- Use locator label controls

- Lift SQI via numeric citation presetization


**migrate**

- Update generated styles and work logs


**mla**

- Apply new template components to MLA


**priority**

- Complete next-10 wave

- Migrate and enhance next 20


**tfca**

- Raise fidelity above 25 percent


**top10**

- Add springer-socpsych-author-date


**vancouver**

- Raise elsevier-vancouver match



### Testing

**citations**

- Cover empty-date citation sort


**engine**

- Add sort oracle tests


**fixtures**

- Add legal hierarchy grouping fixture


**grouping**

- Cover jm legal heading order

- Cover localized heading fallback


**metadata**

- Add test coverage for new MLA forms


**server**

- Add RPC dispatcher integration tests

- Cover http mode


## [0.6.0] - 2026-02-19

### Bug Fixes

**apa**

- Handle legal/personal citation edges


**ci**

- Remove npm install step — node_modules is committed to repo

- Add workflow_dispatch to compat-report

- Trigger Pages deploy after compat report commit

- Track package.json and restore npm install step

- Add concurrency group to prevent parallel runs


**csln**

- Align process alias with render refs flags


**examples**

- Improve example bibliography and citation files


**lua**

- Portable FFI loading and lifecycle


**proc**

- Improve grouped cites and quotes


**schema**

- Allow string presets for contributors, dates, and titles


**scripts**

- Harden compatibility reporting and fix ama style


**springer**

- Raise full-oracle bibliography fidelity above 90%



### Documentation

**bean**

- Refine boltffi plan with phased binding strategy

- Update apa-7th fidelity progress


**bindings**

- Remove unneeded ref


**processor**

- Fix table formatting



### Features

**apa-7th**

- Push bibliography fidelity above 20 matches


**bean**

- Add top-10 style reporting task


**beans**

- Document-level bibliography grouping


**bindings**

- Add LuaJIT FFI binding for LuaLaTeX


**cli**

- Improve error handling for input files


**core**

- Introduce ComponentOverride enum

- Achieve strict template validation

- Add TypeSelector and semantic number fields


**processor**

- Add native LaTeX renderer

- Add universal C-FFI bridge

- Fix djot citation parsing and rendering

- Simplify citation model and djot support

- Implement citation sorting and improved grouping


**render**

- Full latex support in csln process


**report**

- Add top-10 style compatibility report

- Richer compatibility metrics and detail view


**style**

- Modernize apa-7th conjunctions

- Improve apa-7th fidelity and add documentation



### Refactor

**csln**

- Unify cli ux around render/check

- Remove process command alias


**i18n**

- Localize bib group headings


**processor**

- Format-aware value extraction pipeline



### Styling

**apa-7th**

- Reach full oracle fidelity


**elsevier-harvard**

- Raise fidelity >90%



### Testing

**i18n**

- Align multilingual tests


**processor**

- Wire domain fixtures into CI runs


## [0.5.0] - 2026-02-16

### Bug Fixes

**ci**

- Remove nextest to avoid yanked deps


**nextest**

- Correct config field types


**processor**

- Resolve clippy warnings in document tests


**release**

- Remove unsupported update_all_packages field

- Consolidate to root changelog and align versioning



### Documentation

**grouping**

- Add primary/secondary sources



### Features

**beans**

- Add typst output format

- Add interactive html css/js

- Add deno evaluation task


**core**

- Support legal reference conversion


**djot**

- Implement citation visibility modifiers and grouping 


**dx**

- Optimize binary size and automate schema publishing

- Export and publish all top-level schemas


**grouping**

- Implement group disambiguation


**style**

- Support integral citations in Springer Vancouver


**test**

- Add CSL test suite for disambiguation


**web**

- Implement interactive HTML renderer



### Refactor

**test**

- Use pure CSLN types



### Testing

**processor**

- Expand native test suite and refactor existing tests

- Reorganize integration tests into functional targets


## [0.3.0] - 2026-02-15

### Bug Fixes

**bibliography**

- Preserve component suffixes in separator deduplication


**core**

- Enable initialize-with override on contributor components

- Alias DOI/URL/ISBN/ISSN for CSL-JSON


**csln_migrate**

- Improve substitute extraction for real styles


**gitignore**

- Improve baselines directory exclusion pattern


**locale**

- Handle nested Forms in role term extraction


**migrate**

- Improve template compilation

- Disable auto chapter type_template generation

- Improve citation delimiter extraction

- Extract date wrapping from original CSL style

- Add editor/container-title for chapters, suppress journal publisher

- Add page formatting overrides for journals and chapters

- Resolve template nesting regression with recursive deduplication

- Context-aware contributor option extraction

- Recursive type overrides for nested components

- Improve CSL extraction and template generation

- Suppress pages for chicago chapters

- Chicago publisher-place visibility rules

- Remove comma before volume for chicago journals

- Use space suffix for chicago journal titles

- Extract 'and' configuration from citation macros

- Use full names in bibliography for styles without style-level initialize-with (#56)

- Improve bibliography template extraction

- Deduplicate nested lists and fix volume-issue grouping

- Extract author from substitute when primary is rare role

- Detect numeric styles and position year at end

- Add space prefix to volume after journal name

- Extract correct citation delimiter from innermost group

- Handle Choose blocks in delimiter extraction

- Extract bibliography delimiter from nested groups

- Improve bibliography component extraction for nested variables

- Prevent duplicate list variables

- Improve contributor and bibliography migration

- Add text-case support for term nodes and deduplicate numbers

- Use IndexMap to preserve component ordering

- Disable hardcoded component sorting

- Add date deduplication in lists

- Preserve label_form from CSL 1.0 Label nodes

- Preserve macro call order across choose branches

- Correct contributor name order logic


**processor**

- Use container_title for chapter book titles

- Correctly map ParentSerial/ParentMonograph to container_title

- Implement contributor verb and label forms

- Add context-aware delimiter for two-author bibliographies

- Implement variable-once rule for substituted titles

- Improve bibliography sorting with proper key chaining

- Add contributor labels and sorting fixes

- Resolve mode-dependent conjunctions and implement deep config merging

- Allow variable repetition with different context

- Author substitution and grouping bugs

- Use correct jotdown API

- Prevent HTML escaping in docs


**reference**

- Extract actual day from EDTF dates


**render**

- Suppress trailing period after URLs in nested lists


**render,inferrer**

- Improve delimiter detection and URL suffix handling


**scripts**

- Attach overrides to template objects for JSON output

- Per-type confidence metric for template inferrer

- Detect prefixes and emit delimiter in inferrer output


**sort**

- Strip leading articles and fix anonymous work formatting


**styles**

- Update metadata to CSLN conventions

- APA integral and config


**web**

- Add scroll margin to example anchors


**workflow**

- Implement opus review critical fixes and strategy updates



### Documentation

**agent**

- Add prior art research and roadmap

- Add style editor vision document


**architecture**

- Add migration strategy analysis

- Revise migration strategy analysis

- Add validation results to migration strategy analysis


**bench**

- Add benchmark requirements policy


**design**

- Update style aliasing decisions


**examples**

- Clarify EDTF uses locale terms not hardcoded values

- Add info field and restructure bibliography files

- Add chaucer with edtf approximate date


**instructions**

- Add humanizer skill integration


**migrate**

- Convert remaining TODOs to issues


**multilingual**

- Add architectural design for multilingual support


**reference**

- Convert parent-by-id TODO


**skills**

- Prefer wrap and delimiters for semantic joining


**state**

- Update state.json with delimiter fix progress



### Features

**analyze**

- Add parent style ranking for dependent styles


**bib**

- Implement subsequent author substitution


**citations**

- Add infix support for integrals


**cli**

- Merge process and validate into csln

- Add --show-keys flag to process command

- Support complex citation models as input


**contributor**

- Implement et-al-use-last truncation


**core**

- Implement style versioning and forward compatibility

- Add json schema generation support and docs

- Add multi-language locale support

- Add overrides support to contributor and date components

- Add else-if branches and type-specific bibliography templates

- Add style preset vocabulary for Phase 1

- Add embedded priority templates for Phase 2

- Expose embedded templates via use-preset

- Enhance citation model and add bibliography separator config

- Implement editor label format standardization

- Add prefix_inside_wrap for flexible wrap ordering

- Add InputBibliography and TemplateDate fallback support

- Implement declarative hyperlink configuration

- Add Tier 1 legal reference types

- Add Patent and Dataset reference types

- Add Standard and Software types

- Add locale term role labels


**core,processor**

- Implement curly quote rendering

- Add locator support and refine punctuation rendering

- Add locator support, mode-dependent logic, and integral citation templates


**csl-tasks**

- Implement task management CLI

- Add ux improvements for local-first workflow

- Implement GitHub issue number alignment for task IDs

- Improve GitHub sync error handling

- Add duplicate detection for github sync


**csln_core, csln_migrate**

- Add CSLN schema and OptionsExtractor


**csln_migrate**

- Integrate OptionsExtractor into migration CLI

- Add TemplateCompiler for clean CSLN output

- Improve template ordering and author-date citation


**dates**

- Implement EDTF uncertainty, approximation, and range rendering


**edtf**

- Implement modern winnow-based parser


**fixtures**

- Expand test references to 28 items across 17 types


**github**

- Add style request issue template


**locale**

- Implement punctuation-in-quote as locale option

- Expose locator terms for page labels


**migrate**

- Extract bibliography sort and fix citation delimiter

- Add type-specific template extraction (disabled)

- Add chapter type_template for author-date styles

- Extract volume-pages delimiter from CSL styles

- Extract bibliography entry suffix from CSL layout

- Add preset detection for extracted configs

- Infer month format from CSL date-parts

- Support type-conditional substitution extraction

- Improve migration fidelity and deduplication

- Implement publisher-place visibility rules

- Add variable provenance debugger

- Add custom delimiter support for CSL 1.0 compatibility

- Implement complete source_order tracking system

- Integrate template resolution cascade with per-component delimiters


**migration**

- Add styleauthor migration pathway


**multilingual**

- Implement holistic parallel metadata for names and titles

- Implement multilingual support


**options**

- Add configurable URL trailing period

- Add substitute presets and style-aware contributor matching


**presets**

- Add options-level preset support


**processor**

- Add citation layout support

- Add bibliography entry numbering for numeric styles

- Fix name initials formatting and extraction

- Support per-component name conjunction override

- Implement declarative title and contributor rendering logic

- Achieve 15/15 oracle parity for Chicago and APA (#54)

- Add citation grouping and year suffix ordering

- Improve bibliography separator handling

- Improve rendering engine and test dataset

- Add integral citation mode to CLI output

- Implement multilingual BCP 47 resolution

- Implement strip-periods in term and number labels

- Add document-level processing prototype

- Implement WinnowCitationParser for Djot syntax

- Simplify Djot citation syntax by removing mandatory attribute

- Support hybrid and structured locators in Djot parser

- Implement djot document processing and structured locators

- Add HTML output for Djot document processing

- Support infix variable in integral citations


**reference**

- Support parent reference by ID


**render**

- Add title quotes and fix period-inside-quotes

- Implement structured hyperlinking in templates


**rendering**

- Implement inner/outer affixes


**scripts**

- Add citeproc-js oracle for verification

- Add structured diff oracle for component-level comparison

- Add batch oracle aggregator for pattern detection

- Add parallel execution and --all flag for corpus analysis

- Add output-driven template inference engine

- Add prefix, wrap, and items grouping to inferrer

- Show per-type confidence in verbose output

- Add formatting inference and parent-monograph detection


**skills**

- Add styleauthor skill and agent for LLM-driven style creation

- Add update to styleauthor


**styleauthor**

- Add workflow optimizations

- Add autonomous command whitelist


**styles**

- Add APA 7th edition CSLN style

- Add APA 7th edition CSLN style with integral/narrative support

- Add elsevier-with-titles style

- Add chicago manual of style 18th edition (author-date)

- Add elsevier-harvard author-date style

- Add elsevier-vancouver

- Implement springer-vancouver-brackets.yaml

- Add springer-vancouver-brackets style

- Add strip-periods to springer-basic

- Add taylor-and-francis-chicago-author-date

- Add legal-case override to APA 7th

- Add label config to AMA


**task-cli**

- Fill out task management skill


**test**

- Expand test data to 15 reference items (#53)


**workflow**

- Add regression detection with baseline tracking

- Optimize styleauthor migration workflow

- Migration workflow optimizations



### Refactor

**beans**

- Reorganize tasks into epic structure


**cli**

- Csln-processor -> csln process


**core**

- Use DelimiterPunctuation enum for volume_pages_delimiter

- Remove feature gate from embedded templates

- Strict typing with custom fields


**migrate**

- Implement occurrence-based template compilation


**processor**

- Modularize document processing


**scripts**

- Harden oracle component parser


**styleauthor**

- Use Sonnet + checkpoints



### Styling

**migrate**

- Fix doc comment indentation for clippy


