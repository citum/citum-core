---
name: rust-simplify
description: One-file-at-a-time Rust quality pass for Citum, using jcodemunch to target the highest-value cleanup.
---

# Rust Simplify

Use this skill for bounded Rust cleanup work in the Citum workspace when the task is to
improve code quality rather than fix a specific bug.

Read first:
- `docs/guides/CODING_STANDARDS.md`
- `docs/guides/CODEX_SKILLS.md`

## Target Selection

- Prefer `jcodemunch` symbol and file analysis before reading large files.
- Pick the highest-value Rust file in the affected crate or module.
- If a file path was supplied, confirm it is actually wired into the build.

## Quality Pass

- Reduce duplication and nested control flow.
- Prefer idiomatic Rust and explicit error handling.
- Review suspicious string ownership patterns. Prefer borrowed `&str` for lookup
  and comparison work, and allocate `String` values at real ownership boundaries.
- Do not perform broad allocation churn in hot paths without benchmark evidence.
- Add or update tests when behavior changes.
- When tests change, keep expected values independent of current implementation
  output and confirm behavior changes would have failed before the fix when
  practical.
- Keep the scope to one focused file or one tightly related cluster.

## Verification

- Run the repo-required Rust checks for any `.rs`, `Cargo.toml`, or `Cargo.lock` change.
- Run `python3 scripts/audit-rust-review-smells.py --changed` for Rust cleanup
  passes and review the advisory findings.
- Regenerate schemas if the touched files require it.
- Report the exact checks you ran and the result.
