# Bean Lifecycle Policy

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-09
**Superseded by:** 
**Related:** `CLAUDE.md` task management and commit rules; `.claude/skills/beans/SKILL.md`

## Rule
Bean state must reflect repo reality. When work lands on `main`, the bean for that work must be completed in the same change series or immediate follow-up before the merge is considered done, and terminal beans must be archived promptly instead of remaining at `.beans/` root.

## Rationale
Open beans that describe already-landed work create false backlog, hide actual progress, and weaken `/beans next` recommendations. Root-level completed or scrapped beans create the opposite problem: they make finished work look active and allow hygiene drift to accumulate silently.

## Application
Use these lifecycle states consistently:

| State | Meaning | Required action |
|------|---------|-----------------|
| `draft` | Proposed work not yet ready to start | Refine or delete before active work begins |
| `todo` | Ready to start or queued | Move to `in-progress` when work starts |
| `in-progress` | Actively being worked | Keep checklist current |
| `completed` | Work is done and summarized | Archive promptly |
| `scrapped` | Work intentionally abandoned and summarized | Archive promptly |

The following conditions are hygiene failures:

| Condition | Severity | Required remediation |
|----------|----------|----------------------|
| `completed` or `scrapped` bean remains at `.beans/` root | Hard failure | Archive it with `beans archive` |
| Open bean collides with a completed/scrapped bean for the same normalized title | Hard failure | Resolve the duplicate and keep only the correct active bean open |
| Open bean has explicit bean-id evidence in a commit reachable from `main` | Hard failure | Complete the bean, append `## Summary of Changes`, then archive it if appropriate |
| Open bean only matches a `main` commit by normalized title | Advisory | Review the match; complete/archive if it is truly done, otherwise rename or clarify the bean |

Use `main` as the source of truth for landed work. Unmerged side branches do not count as completion.

When finishing work:

1. Update checklist items to checked.
2. Append `## Summary of Changes`.
3. Mark the bean `completed` or `scrapped`.
4. Archive terminal beans promptly.
5. Run `bash .claude/skills/beans/bin/citum-bean hygiene` before push when bean state changed.

## Exceptions
There is no automatic bean closing. If a stale-bean finding is a false positive, fix the bean title/body so it no longer ambiguously describes already-landed work, then rerun hygiene. Temporary exceptions require an explicit note in the bean body explaining why the bean remains open despite matching repo history.

## Changelog
- v1.0 (2026-03-09): Established bean lifecycle and stale-bean enforcement rules.
