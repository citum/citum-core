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

## Schema & Style Design Philosophy

Citum uses a **declarative schema**: styles are data, not programs. This is the
sharpest divergence from CSL 1.0 and the most important design constraint for
contributors working on the schema (`citum_schema`) or the engine
(`citum_engine`).

### What "declarative" means here

A style YAML file describes *what* a citation should look like — which fields
appear, in what order, wrapped in what punctuation — without encoding *how* the
processor should compute it. The processor stays dumb; the style stays explicit.

The primary mechanism for reusable formatting variation is **`options`**, not
template branching. Contributor formatting, date presentation, processing mode,
and label behavior are all declared in `options.*` blocks. Templates handle
structure, field order, and local rendering details such as `prefix`, `suffix`,
and `wrap`. This is why CSL 1.0 conditionals and macro chains have no equivalent
in Citum styles — those decisions are either lifted into named `options` presets,
expressed as explicit structural `type-variants`, or made unnecessary by the
type system.

**Do not add procedural logic to styles.** A style is valid Citum if every
rendering decision can be read off the YAML without knowing Rust. If you need to
read engine source to understand why a style produces a given output, that logic
belongs in the style, not the engine.

### What this replaces in CSL 1.0

| CSL 1.0 pattern | Citum equivalent |
|-----------------|-----------------|
| `<choose><if type="article-journal">…</if></choose>` | Prefer scoped `options.*` for reusable formatting policy; use `type-variants:` only when the template structure differs by type |
| `<if variable="DOI">…</if>` (optional-field branching) | Template items are skipped automatically when the variable is absent |
| `<text macro="author"/>` (macro call chain) | Named presets (`options.contributors`, `options.dates`) |
| Hardcoded separator by type in the processor | `suffix:` / `prefix:` on the relevant template item in the style |
| `<choose><if position="first-in-cluster">` | Engine-managed; styles configure via `citation.options.group-delimiter` / `options.processing.group` |

The goal is that a style author — not a Rust programmer — can express any
supported formatting difference without touching the engine.

### Rules for schema contributors

**Prefer flat, typed fields over open maps.**
New schema fields go on the appropriate typed struct, not into a
`HashMap<String, serde_json::Value>`. Open maps
(`custom: Option<HashMap<String, serde_json::Value>>`) exist only as an explicit
escape hatch for user-defined extensions.

**New fields must deserialize when absent and default to a neutral/off value.**
Use `Option<T>` or `#[serde(default)]` / `#[serde(default = "...")]` on concrete
types as appropriate so older styles remain valid against newer engine versions.

**No `unwrap()`, no `unsafe`, no silent fallbacks.**
Missing data surfaces as `None` and is handled by the template skip mechanism.
Do not paper over missing fields with magic defaults in the processor.

**`type-variants` are a last resort.**
Before reaching for `type-variants`, ask whether the base template can handle
the case naturally. Reserve `type-variants` for genuine structural differences
(e.g., a journal article omits `publisher`; a dataset adds `version`). Do not
use `type-variants` for punctuation-only variation.

**Processor logic requires a style escape hatch.**
If the engine must behave differently for two reference types, there must be a
style-level knob that controls it. Hardcoded type-conditional logic in Rust is a
design smell — open a discussion before adding it.

### Quick reference

```yaml
# Good: style declares the structural difference explicitly
bibliography:
  template:
    - contributor: author
    - date: issued
      form: year
      wrap: parentheses
    - title: primary
    - variable: publisher       # skipped automatically for types without a publisher
  type-variants:
    article-journal:
      - contributor: author
      - date: issued
        form: year
        wrap: parentheses
      - title: primary
      - title: parent-serial  # replaces publisher for journal articles
```

```rust
// Bad: processor encodes type knowledge that belongs in the style
if ref_type == RefType::ArticleJournal {
    output.push(render_journal_title(ctx));  // ← this logic belongs in style YAML
}
```

For the full rationale, see
[docs/architecture/DESIGN_PRINCIPLES.md](./docs/architecture/DESIGN_PRINCIPLES.md).

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
