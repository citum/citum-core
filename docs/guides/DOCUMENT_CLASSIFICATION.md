# Document Classification Guide

Use this guide when creating a new project document or deciding whether an
existing document should remain in `docs/architecture/`, move to
`docs/policies/`, or be rewritten as a spec in `docs/specs/`.

## Core Rule

Classify documents by the question they answer:

| Question | Document type | Home |
|---------|---------------|------|
| "What are we building, and how will we know it is done?" | Specification | `docs/specs/` |
| "Why did we choose this design, and what alternatives did we consider?" | Architecture | `docs/architecture/` |
| "What rule must contributors and agents follow?" | Policy | `docs/policies/` |

If a document tries to answer more than one of those questions, split it.

## Use a Spec When

Write or convert to a spec when the document is the normative contract for
future implementation or verification.

Strong signals:
- It defines behavior that code and tests are expected to implement.
- It needs explicit scope and non-goals.
- It introduces or changes public interfaces, schema, or user-visible behavior.
- It should provide acceptance criteria that an implementer can verify.

Keep spec content focused on:
- purpose
- scope
- design
- implementation notes
- acceptance criteria

## Use Architecture When

Keep a document in architecture when it primarily records reasoning, tradeoffs,
or historical decisions.

Strong signals:
- It compares multiple options.
- It preserves context for why a model was chosen.
- It captures execution snapshots or dated plans.
- It is useful background, but not the implementation contract by itself.

Architecture docs should not be the only normative source for active behavior
once the design has settled. If implementers are treating an architecture doc
as the contract, extract a spec from the settled parts.

## Use Policy When

Use a policy when the document defines a binding rule that contributors and
agents must follow repeatedly.

Strong signals:
- It defines a required workflow or decision rule.
- It is enforced socially, by tooling, or by CI.
- The opening rule should be usable without reading the full document.

Policies should avoid carrying large design rationale that belongs in
architecture or feature detail that belongs in a spec.

## Legacy Document Triage

For existing mixed-purpose docs, use this decision path:

1. If the document is still an unresolved design discussion, keep it in
   architecture and mark its state clearly.
2. If the design is settled and the document is acting as the implementation
   contract, create a spec and leave the architecture doc as rationale/history.
3. If the document is really a recurring rule, move or rewrite it as a policy.
4. If the document is obsolete and superseded elsewhere, mark it as such or
   retire it.

Do not mechanically rename architecture docs to specs. Convert only when the
content is rewritten into a normative contract.

## Current Priority Candidates

These are good candidates for gradual follow-up, not immediate bulk
conversion:

| Document | Recommended treatment | Why |
|---------|------------------------|-----|
| `docs/architecture/design/TYPE_SYSTEM_ARCHITECTURE.md` | Keep as architecture; later add a separate spec if needed | It is still rationale and option analysis, not a clean normative contract |
| `docs/architecture/design/LEGAL_CITATIONS.md` | Likely spec candidate | It may be the de facto contract for legal-reference behavior |
| `docs/architecture/MULTILINGUAL.md` | Likely split | It appears broad enough to separate rationale from implementable behavior |
| `docs/architecture/design/BIBLIOGRAPHY_GROUPING.md` | Likely spec candidate | It likely describes processor behavior that should be testable |
| `docs/architecture/CITUM_STORE_PLAN.md` | Likely spec candidate | It appears feature-oriented if the store remains active roadmap work |

## Working Rule for New Docs

When in doubt:
- start with `docs/specs/` for non-trivial future implementation work
- keep supporting rationale in `docs/architecture/`
- extract reusable behavioral rules into `docs/policies/`

This avoids using one document to carry design rationale, implementation
contract, and workflow policy all at once.
