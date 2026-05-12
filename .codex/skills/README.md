Legacy compatibility shims for Codex installs that still point at `.codex/skills/`.

Canonical public skills now live under `.skills/`. Keep these entries as thin
symlinks so older `~/.codex/skills/*` installations continue to resolve after
the migration.
