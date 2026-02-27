# Beans (Citum)

**Type:** User-Invocable, Agent-Invocable  
**LLM Access:** Yes  
**Purpose:** Task tracking and smart next-step selection using `beans`

## Bootstrap (Required)

1. Run `beans prime` at session start (or when task context changes).
2. Treat `beans prime` output as authoritative for command syntax and workflow.
3. Prefer `--json` for agent parsing and automation.
4. Do not use TodoWrite or ad-hoc todo lists; track work in beans.

## Citum Overlay

Use these project-specific rules on top of `beans prime`:

- Start by checking existing work: `beans list --json --ready` and `beans show --json <id>`.
- Always create beans with an explicit type (`-t`).
- Keep bean checklists current while work is in progress.
- Mark completed only when all checklist items are checked.
- When completing, append a `## Summary of Changes`.
- When scrapping, append a `## Reasons for Scrapping`.

## `/beans next` Helper

`/beans next` is a local helper that ranks and presents multiple ready options.

- Default output: top 3 options.
- Ranking: priority (`critical` > `high` > `normal` > `low` > `deferred`), then type (`bug` > `feature` > `task` > `milestone` > `epic`), then oldest first.
- Includes short rationale and parent title (when present).

**Implementation:** Always run via the wrapper script — never call `beans list --json --ready` directly, as that dumps raw JSON. The script handles ranking and formatting. After running, relay the output verbatim to the user as a plain code block or preformatted text — do not summarize or reformat it.

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
