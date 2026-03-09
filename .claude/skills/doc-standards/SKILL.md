# doc-standards Skill

**Trigger phrases:** "create a spec", "write a design doc", "add a policy",
"document this decision", "new architecture doc", "write a spec for",
"spec this out", "design doc for", "I need a policy for"

## Directory Selection

| Intent | Directory | Template source |
|--------|-----------|-----------------|
| Feature/design specification | `docs/specs/` | `docs/specs/README.md` |
| Active behavioral rule | `docs/policies/` | `docs/policies/README.md` |
| Execution plan or snapshot | `docs/architecture/` | date-stamp filename |
| Operational how-to | `docs/guides/` | — |
| Reference lookup table | `docs/reference/` | — |

## Spec Template

```markdown
# [Feature Name] Specification

**Status:** Draft | Active | Superseded
**Version:** 1.0
**Date:** YYYY-MM-DD
**Supersedes:** (path, if any)
**Related:** (policy, bean, or issue)

## Purpose
One paragraph: what feature this specifies and why.

## Scope
In scope. Explicitly out of scope.

## Design
(Core content — decisions, data models, examples.)

## Implementation Notes
(Non-normative hints, known constraints.)

## Acceptance Criteria
- [ ] Verifiable condition 1
- [ ] Verifiable condition 2

## Changelog
- v1.0 (DATE): Initial version.
```

## Policy Template

```markdown
# [Policy Name]

**Status:** Active | Superseded
**Version:** 1.0
**Date:** YYYY-MM-DD
**Superseded by:** (path, if Superseded)
**Related:** (spec or CLAUDE.md section)

## Rule
The binding rule in 1–2 sentences. Agents must be able to apply this
without reading further.

## Rationale
Why this rule exists.

## Application
Decision table or flowchart for common scenarios.

## Exceptions
When the rule may be bypassed, and the approval process.

## Changelog
- v1.0 (DATE): Established.
```

## Pre-Commit Validation Checklist

- [ ] Correct directory (see table above)
- [ ] Frontmatter complete: Status, Version, Date, Related
- [ ] Spec: Acceptance Criteria section populated with at least one item
- [ ] Policy: Rule section is self-contained (≤2 sentences, agent-readable)
- [ ] No broken relative links
- [ ] Run `/humanizer` if the document is user-facing prose

## Feature Design Workflow (Spec-First)

1. Create `docs/specs/FEATURE_NAME.md` with Status `Draft` before writing code.
2. Commit the spec. Get it merged/pushed.
3. Set Status to `Active` in the same commit as the first implementation commit.
4. Add the spec path to the bean description (`beans update <id> -d "..."`).
5. After the feature ships, update Acceptance Criteria checkboxes.

## Registering a New Active Policy

1. Create `docs/policies/POLICY_NAME.md` using the policy template.
2. Add an entry to `docs/policies/README.md` table.
3. If the policy must be loaded by agents on every session, add a reference
   to CLAUDE.md (Prior Art & Design Documents section or Autonomous Operations).
