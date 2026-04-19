---
# csl26-hs0y
title: Auto-bump workspace version on publishable crate changes
status: completed
type: task
priority: normal
created_at: 2026-04-19T20:18:34Z
updated_at: 2026-04-19T20:23:06Z
---

Extend the schema-check hook pipeline to also detect API changes in publishable Rust crates and auto-bump [workspace.package].version. Mirrors the Schema-Bump footer contract with a Version-Bump footer. Includes rust-check.py, validate-version-release.py, hook updates, CI job (cargo-semver-checks-action), and VERSION_BUMPING.md reference.

## Summary of Changes

- **scripts/rust-check.py**: pre-commit detects publishable crate changes,
  runs cargo-semver-checks (with diff heuristic fallback), bumps
  [workspace.package].version in Cargo.toml, and writes .git/VERSION_BUMP
  handoff. commit-msg reads handoff and appends Version-Bump: footer.
- **scripts/validate-version-release.py**: validates Version-Bump footers
  against the actual version delta in a commit range (used by pre-push + CI).
- **.githooks/pre-commit**: calls rust-check.py pre-commit after schema check.
- **.githooks/commit-msg**: calls rust-check.py commit-msg after schema-check.
- **.githooks/pre-push**: validates version contract via validate-version-release.py.
- **.github/workflows/ci.yml**: adds 'Enforce version bump footer contract on PRs'
  step in checks job, and new semver-check job using cargo-semver-checks-action.
- **docs/reference/VERSION_BUMPING.md**: reference doc for the policy.
- **scripts/install-hooks.sh**: updated hook descriptions.
