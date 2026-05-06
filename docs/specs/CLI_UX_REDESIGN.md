# CLI UX Redesign Specification

**Status:** Active
**Date:** 2026-05-06
**Related:** csl26-j242, docs/specs/STYLE_REGISTRY.md, docs/specs/DISTRIBUTED_RESOLVER.md

## Purpose

This specification defines a clearer command-line user experience for Citum. It
evaluates the current CLI from a human interface design and CLI UX perspective,
then defines a target command model that makes common user tasks discoverable:
rendering documents, finding styles, installing styles, adding registries,
validating inputs, and authoring style or locale files.

The redesign optimizes for clean, consistent UX. Backward compatibility is not a
goal for this work: confusing duplicate commands should be removed or reorganized
rather than kept as aliases.

## Scope

In scope:

- User-facing command names, grouping, help text, examples, and error guidance.
- Style discovery, style installation, registry management, installed locale UX,
  and diagnostics.
- Command behavior where current behavior contradicts help text or user intent.
- The `csl26-j242` interactive style browser as part of the style discovery UX.

Out of scope:

- Changes to citation rendering semantics.
- Changes to style, locale, bibliography, or citation schema shape.
- Distributed resolver protocol details beyond the CLI workflows already needed
  by style and registry management.

## Current UX Evaluation

### Top-level command model

The current command tree is:

```text
citum
  render doc|refs
  check
  convert refs|style|citations|locale
  styles list
  registry list|resolve
  store list|install|remove
  style list|search|info|lint
  locale lint
  schema
  bindings
  completions
  doc       (hidden legacy)
  validate  (hidden legacy)
```

The main problem is that the command tree mixes object names, implementation
layers, and user tasks. `style` and `styles` are nearly identical names but have
different meanings. `registry` claims to manage sources but only inspects a
partial view. `store` exposes the storage mechanism, while users usually want to
add or remove styles and locales without learning where Citum stores them.

### Command findings

`render` is the clearest command group. `render doc` and `render refs` describe
tasks users understand. Its main UX issue is style discovery: users who do not
know a valid `--style` value must leave the render workflow and guess whether
`styles`, `style`, `registry`, or `store` is the right next command.

`check` is broad but acceptable. It validates styles, bibliography files, and
citations. It should stay a top-level command because validation is a cross-file
workflow. Help text should distinguish schema validation from style-locale linting.

`convert` has a good object-oriented shape. `convert refs`, `convert style`,
`convert citations`, and `convert locale` are predictable. It needs consistent
input/output examples and should use the same singular/plural terms as the rest
of the CLI: `refs` for bibliography/reference files, `style` for one style file,
`locale` for one locale file, and `citations` for citation input files.

`styles` is the most confusing command. It lists embedded builtins only, but the
name looks like the general style browsing surface. Because `style` already owns
the unified catalog commands, `styles` should be removed from the clean target
UX. Embedded-only browsing should be expressed as `citum style list --source
embedded`.

`style` is conceptually correct as the main style namespace, but its current
top-level help says "Validate a style against a locale file" even though the
subcommands are `list`, `search`, `info`, and `lint`. This teaches users the
wrong model before they see the useful discovery commands.

`style lint` is too narrowly described as locale-file validation. It should be
framed as style authoring validation, with `--locale` as one input to that
validation. The command should explain what it checks and what it does not check.

`registry` currently says "manage" but only provides `list` and `resolve`.
`registry list` includes a local `citum-registry.yaml` if present, while
`registry resolve` currently resolves only against the embedded default registry.
That violates the user's expectation that commands under one noun share the same
source model. A clean UX requires real management commands and consistent source
resolution.

`store` is an implementation-layer command exposed as a user-facing noun. It is
useful because it reveals platform paths and installed files, but those are
diagnostic concerns. Style installation should live under `style add/remove`,
locale installation should live under `locale add/remove`, and store-path
inspection should live under a diagnostic command.

`locale` is a valid authoring namespace. `locale lint` is clear, but it should
share validation language with `style lint` and should be discoverable from
`check --help` when users are validating project inputs.

`schema`, `bindings`, and `completions` are developer/tooling commands. They are
acceptable as top-level commands, but help text should label them as tooling so
new end users do not confuse them with citation workflows.

Hidden `doc` and `validate` legacy commands should be removed in the clean target
UX. If retained temporarily during implementation, they should not shape public
documentation, examples, or tests.

## Design

### User stories

1. As an author, I can render a document with a style I already know:
   `citum render doc manuscript.djot -b refs.json -s apa`.
2. As an author, I can search for a style by title, alias, field, or publisher:
   `citum style search chicago`.
3. As an author, I can inspect a style before using it:
   `citum style info chicago-author-date-18th`.
4. As an author, I can install a style without copying a full ID from terminal
   output: `citum style add chicago` lets me choose from ranked matches.
5. As an institutional admin, I can add a registry source:
   `citum registry add https://styles.example.org/citum-registry.yaml --name example`.
6. As an institutional admin, I can refresh or remove that source:
   `citum registry update example` and `citum registry remove example`.
7. As a style author, I can validate a style against locale-sensitive behavior:
   `citum style lint my-style.yaml --locale locales/en-US.yaml`.
8. As a locale author, I can validate locale messages and aliases:
   `citum locale lint locales/en-US.yaml`.

### Target command tree

```text
citum
  render doc|refs
  check
  convert refs|style|citations|locale
  style list|search|info|browse|add|remove|lint
  registry list|add|remove|update|resolve
  locale list|add|remove|lint
  doctor
  schema
  bindings
  completions
```

`citum style` is the main user-facing style namespace. It owns discovery,
inspection, installation, removal, interactive browsing, and style authoring
validation.

`citum registry` owns registry source management. A registry is a source of style
metadata and style locations, not a style itself.

`citum locale` owns installed locale workflows and locale authoring validation.

`citum doctor` owns diagnostics that are not citation tasks: data directory,
cache directory, configured registries, installed counts, and environment
problems.

`citum styles` is removed from the target UX.

`citum store` is removed from the target UX. The underlying store remains an
implementation detail used by `style`, `locale`, and `doctor`.

### Style catalog semantics

The style catalog must have one documented source model. All of these commands
must use that model unless a flag narrows it:

- `citum style list`
- `citum style search`
- `citum style info`
- `citum style browse`
- `citum style add`
- style-not-found suggestions from `render`, `check`, and `style add`

The default catalog should include every style users can reasonably choose by
name: embedded styles, configured registry styles, and installed user styles.
`--source` may narrow the view, but source names must be user concepts rather
than implementation leaks:

```text
--source all
--source embedded
--source installed
--source registry:<name>
```

Rows should include enough information to choose confidently: ID, title, source,
aliases when useful, kind, and field/domain tags when available. Text output
should stay concise; JSON output should expose the full row.

### Style commands

`citum style list` lists catalog rows. It should default to a readable table and
support `--format json`, `--source`, `--limit`, and `--offset`.

`citum style search <query>` searches IDs, aliases, titles, descriptions, kind,
and field tags. It should use the same output shape as `style list`.

`citum style info <id-or-alias>` prints a detail view for one style. Text output
should be field-oriented rather than a one-row table:

```text
ID:       apa-7th
Title:    American Psychological Association 7th edition
Source:   embedded
Aliases:  apa
Kind:     base
```

`citum style browse` implements `csl26-j242`. It is an interactive Ratatui TUI
over the same catalog rows used by `list`, `search`, and `info`. It must provide
a scrollable list, incremental filtering, a detail pane, and install/remove
actions without defining a separate data model or registry path.

The browser is a style discovery tool, not a monochrome table prompt. It should
use restrained color to make state legible:

- a header showing the active source filter, search query, result count, and
  installed count;
- a selectable table of styles with aligned `Status`, `Source`, `ID`, and
  `Title` columns;
- a visible installed indicator for styles already present in the user store,
  shown as a high-contrast `INSTALLED` cell in the `Status` column rather than
  appended after the ID;
- a detail pane with ID, title, source, aliases, fields, description, and URL
  when available;
- a footer with the active key bindings and transient success/error messages.

Catalog rows should be merged for display. If an embedded or registry style is
already installed, it appears once with its original source and an installed
status. Styles that only exist in the user store appear with source `installed`.
The TUI must never require the user to copy a full style ID from terminal output.

Required keys:

- `/` focuses search and live-filters the list;
- `Esc` clears search or returns focus to the list;
- `Up`/`Down` and `j`/`k` move selection;
- `PageUp`/`PageDown`, `Home`, and `End` page through results;
- `Enter` or `d` focuses the detail pane;
- `i` installs the selected style when it is not already installed;
- `r` removes the selected installed style after a `y`/`n` confirmation modal;
- `q` quits from any focus state.

The command must keep a non-TTY fallback that prints the same rows as
`style list/search`, because scripts and pipes should not enter alternate-screen
mode. In a narrow terminal, the TUI should collapse to a single-pane list/detail
toggle with each item split across two lines: status/source first, then ID/title.
The footer must be mode-specific: search mode only shows search controls, while
list/detail mode shows install or remove depending on the selected style state.

`citum style add <query-or-url-or-path>` installs a style into the user store. If
given a URL or path, it validates and installs that style directly. If given text
that is not a path or URL, it first tries exact ID and alias resolution, then
falls back to catalog search.

The command must not require ordinary terminal users to copy and paste a full
style ID. In an interactive terminal:

- one exact or high-confidence match installs directly after showing the style
  title and source;
- multiple plausible matches show a numbered selection list;
- no matches show the same next-step guidance as style-not-found errors.

In non-interactive mode, ambiguous input fails with ranked matches and a clear
instruction to rerun with an exact ID, alias, URL, or path. Automation can still
use exact IDs, but the human workflow is query-first.

`citum style browse` should include an install action so a user can search,
inspect, and install without leaving the TUI.

`citum style remove <id-or-alias>` removes an installed style. It must not remove
embedded styles. For automation, it should support `--yes`.

`citum style lint <style>` validates style authoring rules, including
locale-driven terms when `--locale` is provided or when the style declares a
default locale. Its help text should not imply that locale resolution is the only
purpose of style linting.

### Registry commands

`citum registry list` lists configured registry sources and the embedded source.
It should report name, URL/source, priority or precedence, cache status, style
count, and last update where available.

`citum registry add <url> --name <name>` fetches and validates a registry before
adding it. It must fail clearly if the registry cannot be read or parsed.

`citum registry remove <name>` removes a configured registry. It should support
`--yes` for non-interactive use.

`citum registry update [<name>|--all]` refreshes cached registry metadata.

`citum registry resolve <id-or-alias>` resolves through the same registry source
chain used by style catalog commands and prints the selected style plus source.
It must not use a narrower source model than `registry list`.

### Locale commands

`citum locale list` lists installed and embedded locales. It should support
`--source all|embedded|installed` and `--format text|json`.

`citum locale add <path>` installs a locale file after validating it.

`citum locale remove <id-or-name>` removes an installed locale. It must not
remove embedded locales. For automation, it should support `--yes`.

`citum locale lint <locale>` validates locale message syntax, alias targets, and
supported MessageFormat behavior.

### Diagnostics

`citum doctor` reports local environment and configuration state. It should
include the platform data directory, cache directory, configured registry
sources, installed style count, installed locale count, and any unreadable or
invalid local files. It should not be required for normal browse, install,
render, or validation workflows.

### Help text rules

Every command summary should answer "what user task does this perform?" in one
sentence. Avoid implementation-first phrases such as "unified catalog" unless
the help text immediately defines what sources are included.

Use consistent argument labels:

- `<style>` for a path, URL, ID, or alias accepted by the style resolver.
- `<style-id>` for a catalog ID or alias only.
- `<registry>` for a configured registry name.
- `<locale>` for a locale path or locale ID accepted by the command.

Every command that accepts style names should include one example using `apa`
and one example using a fully qualified style ID when space allows.

Errors should point to the next useful command:

```text
style not found: "apaa"

Did you mean "apa"?
Search styles with: citum style search apaa
List installed styles with: citum style list --source installed
```

## Implementation Notes

The first implementation pass should focus on command semantics and help text
before adding the TUI. A clean sequence is:

1. Remove the public `styles` command and route embedded-only browsing through
   `style list --source embedded`.
2. Remove the public `store` command and move its user-facing behavior to
   `style`, `locale`, and `doctor`.
3. Update `style` top-level help and `style lint` copy.
4. Make `registry list` and `registry resolve` use the same source chain.
5. Add query-first `style add/remove` as the user-facing style installation path.
6. Make `style list/search/info/add` include installed styles or document an
   explicit source filter when a source cannot be loaded.
7. Add `locale list/add/remove`.
8. Add `registry add/remove/update`.
9. Add `doctor` diagnostics.
10. Implement `style browse` for `csl26-j242` over the finalized catalog model.

The style catalog should be implemented as a shared service rather than rebuilt
inside each command. The service should expose one row model for table, JSON, and
TUI consumers.

If implementation continues in this PR, keep commits scoped by user-visible
workflow:

1. `docs(cli): specify clean cli ux` - this spec.
2. `refactor(cli): share style catalog resolution` - introduce the catalog row
   service used by list, search, info, add, and errors.
3. `feat(cli): simplify style discovery commands` - remove `styles`, fix style
   help, and make list/search/info use the shared catalog.
4. `feat(cli): add query-first style installation` - add `style add/remove` with
   interactive disambiguation and non-interactive ambiguity errors.
5. `feat(cli): manage locales outside store namespace` - add locale list/add/remove
   and keep locale lint under the same namespace.
6. `feat(cli): manage registries consistently` - add registry add/remove/update
   and make resolve use the same sources as list.
7. `feat(cli): add doctor diagnostics` - expose paths, cache, registry, and
   installed-resource diagnostics without a public store noun.
8. `feat(cli): add interactive style browser` - implement `csl26-j242` on top of
   the shared catalog after the command model is stable.

## Acceptance Criteria

- [ ] The spec evaluates every current command group and identifies user-visible
      confusion or confirms that the group is already coherent.
- [ ] The target command tree has no duplicate `style`/`styles` split.
- [ ] The target UX includes browse, inspect, query-first install,
      registry-add, registry refresh, registry-remove, render, style-lint, and
      locale-lint workflows.
- [ ] `csl26-j242` is specified as `citum style browse` over the shared style
      catalog, including source badges and installed-state indicators.
- [ ] The target command tree has no public `store` noun; diagnostics live under
      `doctor`, styles under `style`, and locales under `locale`.
- [ ] The implementation sequence is explicit enough that follow-up commits can
      continue in the same PR after spec review.

## Changelog

- 2026-05-06: Initial version.
