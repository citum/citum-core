---
name: beans
type: user-invocable, agent-invocable
description: >
  Task tracking and issue management for this Citum project using the `beans`
  CLI. Always use this skill — never TodoWrite — when the user mentions tasks,
  todos, work items, what to work on next, creating/updating/closing issues, or
  asks about project status. Trigger on: "what should I work on", "create a
  task for", "mark that done", "what's in progress", "next task", "check my
  tasks", "/beans next", "track this", "log a bug". Also trigger at the start
  of any multi-step task to check for an existing bean before creating one.
---

# Beans (Citum)

## Usage

1. Treat `beans prime` output as authoritative for command syntax and workflow.
2. Prefer `--json` for agent parsing and automation.
3. Do not use TodoWrite or ad-hoc todo lists; track work in beans.

## Citum Overlay

Use these project-specific rules on top of `beans prime`:

- Start by checking existing work: `beans list --json --ready` and `beans show --json <id>`.
- Always create beans with an explicit type (`-t`).
- Keep bean checklists current while work is in progress.
- Mark completed only when all checklist items are checked.
- When completing, append a `## Summary of Changes`.
- When scrapping, append a `## Reasons for Scrapping`.

## Commit Rule

**Always include the bean file in commits.** Use `git add -A` (not selective adds) so `.beans/` changes are never left out. Code changes and bean state must be committed together.

## `/beans next` Helper

`/beans next` is a local helper that ranks and presents multiple ready options.

- Default output: top 3 options.
- Ranking: prioritize executable work first (`bug`/`feature`/`task`), then use priority (`critical` > `high` > `normal` > `low` > `deferred`), then oldest first. `milestone` and `epic` beans are fallback suggestions only when there are not enough concrete ready items.
- Includes short rationale and parent title (when present).

**Implementation:** Always run via the wrapper script — never call `beans list --json --ready` directly, as that dumps raw JSON. The script handles ranking and formatting. After running, output the script result as plain text — no preamble, no commentary, nothing else.

```bash
bash .claude/skills/beans/bin/citum-bean next
bash .claude/skills/beans/bin/citum-bean next --limit 5
bash .claude/skills/beans/bin/citum-bean next --json
```

## Command Policy

- Canonical command behavior comes from `beans` itself.
- Do not duplicate CLI flag documentation here; use `beans prime` and `beans <cmd> --help`.
- If this file conflicts with `beans prime`, `beans prime` wins.

## See Also

- `beans prime`
- `beans help`
- `.beans.yml`
- `.beans/*.md`
