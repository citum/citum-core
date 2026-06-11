---
name: rust-simplify
description: One-file-at-a-time Rust quality pass for Citum, using local symbol-aware code navigation to target the highest-value cleanup.
---

# Rust Simplify

Use this skill for bounded Rust cleanup work in the Citum workspace when the task is to
improve code quality rather than fix a specific bug.

When `.jj` is present, use `docs/guides/JJ_AI_CHANGE_STACK.md` for optional
local change isolation and intent capture before publishing through Git/GitHub.

Read first:
- `docs/policies/AGENT_HARNESS_POLICY.md`
- `docs/guides/AGENT_ORCHESTRATION.md`
- `docs/guides/CODING_STANDARDS.md`
- `docs/guides/AGENT_SKILLS.md`

## Target Selection

- Prefer locally configured symbol and file analysis before reading large files.
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

## Self-Improvement

When you hit a recurring Rust smell not already caught by the audit script:

1. If it can be expressed as a regex rule, add it to `scripts/audit-rust-review-smells.py`
   as a new `Rule` entry in the `RULES` tuple — the script is the canonical home for
   automatable patterns.
2. If it's too contextual for a regex, add a short bullet to the Quality Pass section
   above instead.

For verification failure modes or code-navigation quirks, add a bullet to the
Verification section above. Include any file updates in the same commit. One
concrete rule per observation — keep the scope tight.
