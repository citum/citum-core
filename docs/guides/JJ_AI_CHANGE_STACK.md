# jj AI Change Stack

## Purpose

Use jj as an optional local change-stack layer for AI-assisted work while keeping
Git and GitHub as the public collaboration interface.

This workflow is for Claude, Codex, and other LLM-assisted sessions. It captures
short-lived intent during development, makes exploratory changes easier to split
or abandon, and preserves Citum's existing commit, PR, and verification gates.

## Default Model

- Use GitHub branches and PRs for published collaboration.
- Use jj locally when `.jj` is present and the user has not requested a Git-only
  workflow.
- Keep one jj change per coherent plan step, bean/spec phase, style pass, or
  Rust fix.
- Let the parent session own jj stack curation. Subagents may implement or
  review bounded changes, but they must not independently split, squash, rebase,
  abandon, publish, or otherwise rewrite the shared stack.
- Treat raw prompt capture as temporary local provenance. In jj, intent files
  may be part of the evolving mutable change while work is in progress. They
  must be deleted from that change before export or publication, so the final
  Git-visible commit does not contain them unless the user explicitly asks for
  durable prompt provenance.

## Intent Capture

During local work, agents may create temporary intent files under:

```text
.ai-intents/INTENT-YYYY-MM-DD-HHMM-<slug>.md
```

Each file should contain only the information needed to curate the local stack:

- task, bean, issue, or PR identifier, if any
- model or tool used
- prompt summary or short quoted prompt excerpt
- plan step or change purpose
- files or subsystem touched
- verification command and result

These files are temporary. In a colocated jj/Git repository, they may be tracked
inside the current mutable jj change during drafting. jj snapshots workspace
edits into that change automatically, so there is no separate Git-style staging
step for the intent file. Before export or publication, delete the file and let
jj amend the current change so the final Git-visible snapshot has no
`.ai-intents/` paths. Do not publish intent files unless the user explicitly
chooses durable prompt provenance for a specific commit.

## Command Protocol

Before starting a local AI-assisted stack:

```bash
jj status
jj log
git status --short --branch
```

For each coherent step:

```bash
jj new
# create a temporary .ai-intents/ file if intent capture is useful
# implement one bounded step
jj status
jj diff
jj describe
```

During curation:

```bash
jj split
jj squash
jj rebase
jj describe
```

Before publishing through Git/GitHub:

```bash
jj status
# delete .ai-intents/ from the current change, if present
jj git export
git status --short --branch
```

If jj is unavailable, use the existing Git workflow. Do not block Citum work just
because jj is missing.

## Hook Gap

jj does not run git hooks. `commit-msg`, `pre-commit`, and `pre-push` are all
silently skipped. Run the equivalent checks manually before every push:

```bash
# 1. Validate commit message (subject format, 50-char limit, body presence)
jj log -r @ --no-graph --template 'description' > /tmp/jj-msg.txt \
  && bash .githooks/commit-msg /tmp/jj-msg.txt

# 2. For Rust changes — pre-commit gate
cargo fmt --check \
  && cargo clippy --all-targets --all-features -- -D warnings \
  && cargo nextest run

# 3. For any push — pre-push gate (schema regen, quality baseline)
bash .githooks/pre-push
```

Skipping step 1 is the most common CI failure when using jj. Always run it.

## Citum Adapter

The jj workflow is subordinate to Citum's repo rules:

- When the user asks for a PR, create a `codex/<scope>-<goal>` branch and open a
  PR instead of pushing to `main`.
- Keep commit messages conventional and follow the 50/72 rule.
- For Rust changes, run the required Rust gate before committing:
  `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run`.
- If `cargo nextest` is unavailable, use `cargo test`.
- If schema crate or CLI schema behavior changes, regenerate schemas and include
  the required schema/version footer.
- For style work, route through `/style-evolve` and preserve fidelity gates.
- For task-tracked work, keep bean state changes in the same final Git commit as
  the related work.

## Agent Coordination

Use jj to isolate and curate changes, not to distribute history ownership across
agents.

- The parent session decides when to create a new jj change and how to curate the
  stack.
- Implementation subagents receive one bounded task and report changed paths,
  verification, and risks.
- Review subagents inspect diffs and findings without rewriting history.
- If two LLMs produce competing designs, put each design on a separate jj change
  or branch, compare the diffs, then keep or squash only the selected approach.

## Publishing

Before push or PR creation:

- Ensure `jj status` and `git status --short --branch` agree on the intended
  published diff.
- Confirm `.ai-intents/` paths are absent from the final jj change and from
  `git status --short --branch`.
- Run the manual hook checks from the **Hook Gap** section above.
- Run the verification gate for the touched change type.
- Push the Git branch and check CI for PR branches.

Global Claude or Codex `ai-change-stack` skills may point at this guide, but
repo-local changes do not install or modify global skill files.
