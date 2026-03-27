# Contributing to Citum

Citum is a declarative, type-safe Rust implementation for managing and processing citation styles. We welcome contributions from domain experts, style authors, developers, and community members.

## How to Contribute

Citum follows an **AI-first development model** that values expertise over implementation speed. The most impactful contributions come from those who understand citation semantics:

**Domain Experts & Style Authors:**
- Surface real-world gaps: describe formatting requirements or edge cases that current systems handle poorly
- Provide contextual resources: share style guides, official manuals, and sample documents
- Report pain points: open GitHub issues describing what is difficult in the Citum model
- Refine instructions: suggest improvements to agent personas and skills

**Developers:**
- Focus on core engine architecture (`citum_engine`), schema design (`citum_schema`), and agent tooling
- Ensure all Rust code changes pass mandatory pre-commit checks before committing

## Development Setup

For full setup instructions (dependencies, Rust toolchain), see [README.md](./README.md#getting-started).

Quick start:
```bash
rustup update && cargo build && cargo test
```

## Task Management

Active development uses [beans](https://github.com/jdx/beans) for local task tracking. GitHub Issues remain open for bug reports and feature requests.

Quick task commands:
```bash
/beans list                              # Show all tasks
/beans next                              # Get recommended next task
/beans show BEAN_ID                      # View task details
/beans update BEAN_ID --status in-progress
/beans update BEAN_ID --status completed
```

See `.claude/skills/beans/SKILL.md` for full reference.

## Code Quality

**All Rust code changes must pass these checks before committing:**

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
```

If `cargo nextest` is not installed, use `cargo test` as fallback.

These checks are **mandatory** for all `.rs` files, `Cargo.toml`, and `Cargo.lock` changes. Documentation-only and style-only changes do not require these checks.

**Do not commit if any check fails — fix the issues first.**

## Commit Conventions

Follow [Conventional Commits](https://www.conventionalcommits.org/) format:
- **Format**: `type(scope): lowercase subject`
- **Length**: 50 character subject, 72 character body wrap
- **References**: Include issue references (e.g., `Refs: csl26-xxxx` or `Refs: #123`)
- **No Co-Authored-By**: Do not include co-author footers

Enable the repository commit hook to enforce this automatically:
```bash
git config core.hooksPath .githooks
```

With `core.hooksPath` enabled, the repository `pre-push` hook validates any
changed production styles in `styles/*.yaml` before the push is sent. Use
`./scripts/validate-production-styles.sh` for the full production-style gate.

For repository validation, prefer the workspace-backed commands:

```bash
cargo run --bin citum -- check -s styles/apa-7th.yaml
./scripts/validate-production-styles.sh
```

A globally installed `citum` binary may be older than the workspace and can
produce stale validation failures until it is rebuilt or reinstalled.

Example:
```
fix(processor): handle empty contributor list

Prevent rendering errors when contributor array is absent
or empty in input reference data.

Refs: #127
```

## Maintainer-Level Development

See [CLAUDE.md](./CLAUDE.md) for maintainer instructions:
- Dependency change confirmations
- Submodule operations protocols
- Agent integration (e.g., `@styleauthor` for style authoring)
- Benchmark requirements for performance changes

---

Thank you for contributing to Citum!
