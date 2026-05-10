# Citum Authoring Agent Skill Specification

**Status:** Draft
**Date:** 2026-05-10
**Related:** bean csl26-v2k9

## Overview

The Citum Authoring Agent Skill is a portable assistant-facing specification
that enables AI tools to create, revise, and validate Citum style files in a
reliable, schema-aware workflow. Citum is presented in its public docs and
repositories as a Rust citation formatting system for modern workflows, with
custom YAML styles, examples, and published schemas available to developers and
style authors.

This feature exists to make style authoring scalable across assistants such as
Cursor, Claude, Gemini, and ChatGPT without depending on a dedicated GUI wizard.
The skill should stay close to Citum core behavior so that authored styles
reflect the current schema, examples, published JSON Schema, and CLI validation
behavior.

## Problem

Authoring citation styles is structurally demanding because agents can easily
invent unsupported fields, misread YAML structure, or produce output that looks
plausible but fails validation. Citum’s value proposition depends on a
declarative YAML style model backed by a Rust engine and published schemas,
which means generated styles must be validated against machine-readable rules
and real tooling rather than accepted on prompt confidence alone.

Without a canonical skill, each assistant session must rediscover the schema,
examples, and workflow from scratch. That creates inconsistent outputs, weaker
portability across tools, and a higher support burden for users trying to create
styles through AI assistance.

## Goals

- Provide a canonical `SKILL.md` that can be loaded or adapted for major AI
  assistants.
- Ensure the skill directs assistants to produce valid Citum YAML rather than
  freeform pseudo-configurations.
- Require the use of Citum’s published JSON Schema as a first-line validation
  mechanism during authoring.
- Require the use of Citum CLI workflows to validate authored styles after
  schema validation and before presenting them as complete.
- Teach assistants to reuse documented Citum examples and inheritance patterns
  instead of inventing novel structures when uncertainty exists.
- Make behavior portable across assistants by defining the workflow and output
  contract independent of any one vendor platform.

## Non-goals

- This feature does not define a GUI style wizard or visual form-based authoring
  experience.
- This feature does not attempt to replace the Citum schema, published JSON
  Schema, examples, or CLI as the source of truth.
- This feature does not guarantee that every assistant can execute shell
  commands directly; it defines the ideal validation workflow and fallback
  expectations when execution is unavailable.
- This feature does not introduce assistant-specific prompt engineering beyond
  minimal loading instructions kept outside the canonical skill file.

## Validation model

The skill must define validation as a layered system rather than a single step.
Each layer serves a distinct purpose and should be applied in sequence when
available.

1. **Published JSON Schema validation** for structural correctness, allowed fields, required properties, and unsupported-key detection.
2. **CLI validation and rendering checks** for behavioral correctness against actual Citum engine behavior.
3. **Example comparison** for idiomatic use of inheritance, composition, and established style patterns.

Schema validation should be treated as the primary authoring-time validation
path because it is machine-readable, editor-friendly, and well-suited to fast
iteration. CLI validation should be treated as the execution-time check that
confirms the authored style behaves correctly in practice.

## Primary users

The primary users are developers, advanced users, and style maintainers who want
an AI assistant to help draft or modify Citum style files while staying grounded
in real Citum conventions. This aligns with Citum’s positioning around modern
programmatic citation workflows rather than manual style editing alone.

Secondary users include maintainers of future Citum Hub tooling or related
integrations who want a shared authoring protocol that can be reused in multiple
surfaces. Keeping the skill in or near `citum-core` supports that reuse because
the core repository is the closest stable source for schema behavior and
rendering logic.

## User stories

- As a style author, the user wants to describe a target citation style in plain
  language and receive a valid Citum YAML draft that reflects the requested
  behavior.
- As a maintainer, the user wants the assistant to validate the draft against
  the published schema before moving on to deeper behavior checks.
- As a maintainer, the user wants the assistant to validate the draft with Citum
  tooling before claiming success, reducing avoidable syntax and logic errors.
- As an advanced user, the user wants the assistant to explain uncertainties and
  assumptions rather than silently invent unsupported fields.
- As a platform user, the user wants roughly the same skill behavior in Cursor,
  Claude, Gemini, and ChatGPT so the workflow is portable.

## Deliverables

The feature should produce the following artifacts:

- `SKILL.md` as the canonical assistant-facing instruction file.
- Supporting example prompts and example outputs.
- A small validation fixture set for testing generated styles.
- Short platform-specific installation or loading notes stored separately from
  `SKILL.md`.

The canonical skill file should be concise enough to load into assistant
contexts but specific enough to constrain behavior. Assistant-specific wrapper
docs can adapt the loading format without changing the core behavior contract.

## Functional requirements

### 1. Skill contract

The skill must instruct the assistant to do the following in order:

1. Gather missing requirements before authoring when the request is underspecified.
2. Draft a Citum YAML style using documented patterns and examples.
3. Validate the draft against the published JSON Schema when schema-aware tooling is available.
4. Validate the draft with Citum CLI commands when available.
5. Revise until validation passes or a clear blocker is identified.
6. Return the final YAML plus a brief assumptions and validation summary.

This sequence is necessary because Citum documentation emphasizes examples,
schemas, and concrete engine behavior, and the feature’s core value is
dependable authoring rather than speculative generation.

### 2. Required input handling

The skill must instruct the assistant to identify or request at least these
inputs when relevant:

- Style or target publication name.
- Citation format goals, such as note-based, author-date, or numeric behavior.
- Bibliography ordering and entry formatting requirements.
- Name formatting conventions, including initials, particles, and et al.
  behavior.
- Locator and citation-specific requirements.
- Any jurisdictional, publisher, or institutional constraints.
- Whether the user is creating a new style, extending an existing one, or
  editing a current draft.

If these are not fully specified, the assistant should ask focused follow-up
questions or state explicit assumptions before drafting. The skill must
discourage the assistant from filling major gaps with invented schema
structures.

### 3. Output contract

The skill must require the assistant to return:

- A complete Citum YAML draft or patch.
- A short explanation of key assumptions.
- Validation results, including schema checks, commands attempted, and whether
  the output passed.
- If validation could not be executed, a clear statement that the YAML is
  unverified.

The assistant must not present a draft as final unless schema validation has
succeeded and CLI validation has succeeded, or unless the user explicitly
accepts an unverified draft. This keeps the skill aligned with Citum’s
tool-backed model.

### 4. Schema discipline

The skill must include strict instructions that the assistant:

- Must treat the published Citum JSON Schema as the primary source for
  field-level validity during authoring.
- Must not invent undocumented keys, sections, or inheritance mechanisms.
- Must prefer schema-backed autocomplete or validation where the environment
  supports it.
- Must prefer copying and adapting patterns from known Citum examples.
- Must preserve valid YAML structure and consistent indentation.
- Must flag uncertainty when the schema support for a requested feature is
  unknown.
- Must prefer minimal valid changes when editing an existing style.

This requirement is critical because schema hallucination is the main failure
mode for AI-based configuration authoring.

### 5. Published schema workflow

The skill must explicitly teach assistants how to use the published style schema
during drafting and review. At minimum, the assistant should be instructed to:

- Validate the draft against the published schema whenever a schema-aware
  editor, validator, or assistant environment supports it.
- Use schema validation to catch missing required fields, unsupported keys, and
  type mismatches early in the workflow.
- Treat schema validity as necessary but not sufficient for correctness, because
  a schema-valid style may still fail behavioral expectations at render time.
- Report schema-validation status separately from CLI-validation status.

This published schema workflow should be mandatory in the skill because it
enables fast feedback and prevents many invalid drafts from ever reaching the
CLI stage.

### 6. CLI validation workflow

The skill must include guidance for using Citum CLI commands for post-schema
verification, including rendering-based checks such as `citum render` when
appropriate. Citum publicly describes itself as a citation formatting system
with examples and behavior reporting, making CLI-backed validation central to
the skill’s credibility.

The workflow should support at least:

- Syntax validation of YAML structure where supported by the CLI.
- Behavioral validation using the Citum engine.
- Render checks using representative input data or fixtures.
- Iterative repair when command output reveals errors.

If the environment does not allow command execution, the assistant should switch
to a constrained fallback mode: produce the draft, report schema-validation
status if available, clearly state that CLI validation did not run, and
recommend local validation steps for the user.

### 7. Inheritance and composition guidance

The skill must explain how to reason about modular style composition and
parent-style inheritance using real Citum patterns where available. Because
examples are already part of the public documentation surface, they should be
treated as the preferred model source for assistants.

The assistant should be taught to:

- Extend existing styles before writing from scratch when that is semantically
  appropriate.
- Keep overrides small and localized.
- Explain the relationship between parent and child styles in plain language.
- Avoid duplicating entire structures when inheritance or composition can
  express the change more safely.

### 8. Error handling

The skill must define expected behavior for these failure modes:

- Missing style requirements.
- Unknown schema support.
- Schema validation failure.
- Invalid YAML.
- CLI validation failure.
- Conflicts between user instructions and observed example patterns.
- No shell, CLI, or schema-aware tooling access.

In each case, the assistant should prefer transparency over guesswork: ask a
question, narrow scope, or produce a clearly labeled partial draft.

## Non-functional requirements

- **Portability:** the skill must be plain-text and vendor-neutral so it can be
  adapted to multiple assistants.
- **Concision:** `SKILL.md` must be compact enough to fit comfortably into
  assistant context windows while retaining the core workflow.
- **Traceability:** the skill should direct assistants to report schema checks,
  commands run, assumptions made, and examples reused.
- **Maintainability:** the skill should live close to schema and CLI evolution
  so it can be updated when Citum changes.
- **Testability:** the workflow must be measurable with repeatable prompts and
  fixture-based validation.

## Proposed file layout

```text
citum-core/
  skills/
    authoring/
      SKILL.md
      EXAMPLES.md
      TEST_CASES.md
      fixtures/
        ...
docs/
  ai-authoring-skill/
    cursor.md
    claude.md
    gemini.md
    chatgpt.md
```

This layout keeps the canonical behavior in one place while letting installation
notes vary by assistant platform.

## `SKILL.md` structure

The canonical skill file should contain these sections:

1. Purpose and scope.
2. Source-of-truth rules.
3. Required authoring workflow.
4. Input checklist.
5. Output contract.
6. Schema-first validation rules.
7. CLI validation instructions.
8. Inheritance and composition guidance.
9. Failure handling.
10. Compact examples.

This structure is intentionally procedural. It should read like an operational
playbook for the assistant, not marketing copy.

## Test plan

The feature should include a lightweight conformance test set that can be run
manually across assistants. Suggested scenarios:

| Scenario | Goal | Pass criteria |
|---|---|---|
| Create a simple new style | Test baseline YAML generation | Produces valid YAML, passes schema validation, and passes the prescribed CLI workflow where available |
| Modify an existing style | Test minimal edits | Changes only relevant sections, stays schema-valid, and remains behaviorally valid |
| Extend via inheritance | Test composition behavior | Uses parent-style patterns correctly and documents assumptions |
| Ambiguous prompt | Test follow-up discipline | Asks targeted questions before drafting or states explicit assumptions |
| Unsupported feature request | Test schema restraint | Does not invent fields and clearly reports uncertainty |
| Schema failure case | Test fast feedback | Corrects unsupported keys or missing required fields before CLI validation |
| Validation failure case | Test repair loop | Revises draft in response to CLI output until resolved or blocked |

The same prompt set should be used in Cursor, Claude, Gemini, and ChatGPT to
compare output quality and portability.

## Acceptance criteria

The feature is complete when all of the following are true:

- A canonical `SKILL.md` exists and can be used in at least the target assistant
  platforms.
- The skill consistently produces Citum YAML drafts grounded in documented
  examples rather than invented structures.
- The skill treats the published JSON Schema as a mandatory first-line
  validation mechanism when supported by the environment.
- The skill instructs assistants to validate with Citum CLI commands after
  schema checks and to report both statuses clearly.
- A small assistant conformance suite exists and is documented.
- Platform-specific loading instructions exist outside the canonical skill file.
- Initial tests show that assistants can produce at least a small set of valid
  styles or edits with acceptable reliability.

## Open questions

The implementation should resolve these questions before finalizing `SKILL.md`:

- Which exact Citum CLI commands should be treated as canonical for behavioral
  validation?
- Which schema-aware workflows should be recommended for environments that can
  validate YAML against published JSON Schema but cannot run the CLI?
- Which example styles should be treated as the official few-shot patterns for
  inheritance and composition?
- How much schema detail should live directly in `SKILL.md` versus being
  delegated to linked example or schema files?
- Should the skill support patch-style edits in addition to full-file generation
  from the first version?
- What is the minimum viable conformance suite size for cross-assistant testing?

## Milestones

### Milestone 1: Spec approval

- Review and approve this feature specification.
- Confirm repository location and file layout.
- Confirm target assistants and the schema-first validation workflow.

### Milestone 2: Canonical skill draft

- Write `SKILL.md` based on the approved workflow.
- Draft compact examples for common style-authoring tasks.
- Draft platform-specific loading notes.

### Milestone 3: Validation fixtures

- Create fixture prompts and sample bibliographic data.
- Define expected schema and CLI validation steps and pass criteria.
- Run initial tests across target assistants.

### Milestone 4: Refinement

- Tighten instructions based on observed assistant failures.
- Reduce verbosity while preserving guardrails.
- Publish the first stable version in `citum-core`.

## Recommended next step

The next implementation step should be to convert this specification into a
first-pass `SKILL.md` plus 5 to 10 conformance scenarios. That sequence keeps
the work grounded in concrete authoring behavior instead of letting the idea
remain a high-level feature concept.

