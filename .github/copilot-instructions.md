# Copilot Review Instructions

## Workspace versioning

Do flag manual workspace or schema version bumps in feature or fix PRs.

This repo uses the release workflow to infer patch, minor, or major impact from
conventional commits after changes merge to `main`. The workflow opens a
release PR that updates `[workspace.package].version`, schema versions, generated
schemas, and release notes together.

Feature PRs should not edit `[workspace.package].version`,
`STYLE_SCHEMA_VERSION`, or add release tags. Schema-affecting PRs should commit
regenerated `docs/schemas/*` output when it changes.

**Consequence:** A normal feature or fix PR can modify Rust code and generated
schemas without carrying version bump commits or schema bump footers.

## Bean files

`.beans/` contains task-tracking files managed by the `beans` CLI (an agentic issue tracker). These files are intentionally committed alongside code changes. Do not flag them as unrelated or suggest removing them from commits.

Archived beans move to `.beans/archive/` when completed. Both paths are expected in the repository.

## Commit footer conventions

Commits may include:

- `Refs: <bean-id>` — links the commit to a bean tracking entry
