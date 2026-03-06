---
name: beans
type: user-invocable, agent-invocable
description: >
  Task tracking and issue management for this Citum project using the `beans`
  CLI. Always use this skill — never TodoWrite — when the user mentions tasks,
  todos, work items, what to work on next, creating/updating/closing issues, or
  asks about project status. Trigger on: "what should I work on", "what's next",
  "create a task for", "mark that done", "I finished", "close this", "what's
  in progress", "next task", "check my tasks", "/beans next", "track this",
  "log a bug", "what's blocking me", "mark it complete", "I'm done with".
  Also trigger at the start of any multi-step task to check for an existing
  bean before creating one.
---

# Beans (Citum)

## Citum Overlay

The `beans prime` guide is already injected into every session — no need to
call it again. Use these project-specific rules on top of it:

- Before starting work: check `beans list --json --ready` and `beans show --json <id>`.
- Always create beans with an explicit type (`-t bug | feature | task | epic | milestone`).
- Keep bean checklists current while work is in progress (`- [ ]` → `- [x]`).
- Mark completed only when all checklist items are checked.
- When completing, append a `## Summary of Changes` section.
- When scrapping, append a `## Reasons for Scrapping` section.

## Commit Rule

**Always include the bean file in commits.** Use `git add -A` (not selective
adds) so `.beans/` changes are never left out. Code changes and bean state must
be committed together.

## Common Patterns

**Check off a checklist item** (exact match required — copy text verbatim):
```bash
beans update <id> \
  --body-replace-old "- [ ] Do the thing" \
  --body-replace-new "- [x] Do the thing"
```

**Append summary and mark complete in one shot:**
```bash
beans update <id> \
  --body-replace-old "- [ ] Final step" --body-replace-new "- [x] Final step" \
  --body-append "## Summary of Changes\n\nWhat was done and why." \
  -s completed
```

**Multiple checkbox updates atomically** (use GraphQL to avoid multiple etag conflicts):
```bash
beans query 'mutation {
  updateBean(id: "<id>", input: {
    bodyMod: {
      replace: [
        { old: "- [ ] Step A", new: "- [x] Step A" }
        { old: "- [ ] Step B", new: "- [x] Step B" }
      ]
    }
  }) { id etag }
}'
```

## `/beans next` Helper

`/beans next` ranks ready options using the full dependency graph and shows
what is currently in progress, so you can pick without manual analysis.

**Ranking:** priority → leverage (how many open beans this unblocks) desc →
type (bug > feature > task) → oldest first. Epics/milestones appear only when
concrete work is insufficient to fill the limit.

**Output includes:**
- In-progress context header (what's already running)
- `· unblocks N` badge when completing a bean would unblock other work

```bash
bash .claude/skills/beans/bin/citum-bean next           # top 3
bash .claude/skills/beans/bin/citum-bean next --limit 5
bash .claude/skills/beans/bin/citum-bean next --json
```

Always run via the wrapper — never call `beans list --json --ready` directly,
as that skips leverage scoring and the in-progress header. Output the script
result as plain text with no preamble or commentary.

## Command Policy

- Canonical command behaviour comes from `beans` itself.
- Do not duplicate CLI flag docs here; use `beans <cmd> --help`.
- If this file conflicts with `beans prime`, `beans prime` wins.

## See Also

- `beans prime`
- `beans help`
- `.beans.yml`
- `.beans/*.md`
