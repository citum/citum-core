# Bean Lifecycle Policy

**Status:** Active
**Version:** 1.1
**Date:** 2026-03-09
**Superseded by:** 
**Related:** `CLAUDE.md` task management and commit rules; `.claude/skills/beans/SKILL.md`

## Rule
Bean state must reflect repo reality. When work lands on `main`, the bean for that work must be completed in the same change series or immediate follow-up before the merge is considered done, and terminal beans must be archived promptly instead of remaining at `.beans/` root.

Duplicate, superseded, and stale-open beans must be made explicit in bean
metadata or body text instead of being left as implicit project memory.

## Rationale
Open beans that describe already-landed work create false backlog, hide actual progress, and weaken `/beans next` recommendations. Root-level completed or scrapped beans create the opposite problem: they make finished work look active and allow hygiene drift to accumulate silently.

The failure mode is not only "forgot to archive." It is also:
- duplicate beans with different wording for the same work
- beans that were partly completed, then silently absorbed into broader work
- open beans that became vague umbrellas after their concrete bug fix already landed

If those cases are not represented explicitly, the tracker stops being a source
of truth and becomes a second-hand memory aid.

## Application
Use these lifecycle states consistently:

| State | Meaning | Required action |
|------|---------|-----------------|
| `draft` | Proposed work not yet ready to start | Refine or delete before active work begins |
| `todo` | Ready to start or queued | Move to `in-progress` when work starts |
| `in-progress` | Actively being worked | Keep checklist current |
| `completed` | Work is done and summarized | Archive promptly |
| `scrapped` | Work intentionally abandoned and summarized | Archive promptly |

Use the following continuity markers whenever they apply:

| Situation | Required marker |
|----------|------------------|
| Bean duplicates another open or completed bean | Add `duplicate-of: <bean-id>` in frontmatter or the opening body lines |
| Bean is replaced by a broader or newer tracker | Add `superseded-by: <bean-id or spec path>` |
| Bean remains open only because adjacent work absorbed part of it | Add a dated note naming the remaining gap explicitly |

Open beans without one of these markers must describe one concrete, still-open
unit of work. If the remaining work is no longer concrete, split it or close it.

The following conditions are hygiene failures:

| Condition | Severity | Required remediation |
|----------|----------|----------------------|
| `completed` or `scrapped` bean remains at `.beans/` root | Hard failure | Archive it with `beans archive` |
| Open bean collides with a completed/scrapped bean for the same normalized title | Hard failure | Resolve the duplicate and keep only the correct active bean open |
| Open bean has explicit bean-id evidence in a commit reachable from `main` | Hard failure | Complete the bean, append `## Summary of Changes`, then archive it if appropriate |
| Open bean duplicates another bean's acceptance criteria or remediation steps | Hard failure | Collapse to one canonical bean and mark the duplicate `scrapped` with `duplicate-of` evidence |
| Open bean contains completed-work notes plus only broad portfolio next steps | Hard failure | Close it and move residual work to a successor bean/spec with an explicit handoff |
| Open bean only matches a `main` commit by normalized title | Advisory | Review the match; complete/archive if it is truly done, otherwise rename or clarify the bean |
| Open bean depends on project memory to explain why it is still distinct | Advisory | Add `duplicate-of`, `superseded-by`, or a dated residual-gap note |

Use `main` as the source of truth for landed work. Unmerged side branches do not count as completion.

When finishing work:

1. Update checklist items to checked.
2. Append `## Summary of Changes`.
3. Mark the bean `completed` or `scrapped`.
4. Archive terminal beans promptly.
5. Run `bash .claude/skills/beans/bin/citum-bean hygiene` before push when bean state changed.

When opening or revising a bean:

1. Search existing beans before creating a new one.
2. Reuse the existing bean if the acceptance criteria and remediation path are materially the same.
3. If you keep a second bean, explain the distinction immediately in the new bean body.
4. If a bean becomes a portfolio tracker rather than a concrete fix, move that role to a spec or epic and close the stale bean.

## Exceptions
There is no automatic bean closing. If a stale-bean finding is a false positive, fix the bean title/body so it no longer ambiguously describes already-landed work, then rerun hygiene. Temporary exceptions require an explicit note in the bean body explaining why the bean remains open despite matching repo history.

## Changelog
- v1.0 (2026-03-09): Established bean lifecycle and stale-bean enforcement rules.
- v1.1 (2026-03-09): Added duplicate/supersession markers and stale-open umbrella rules.
