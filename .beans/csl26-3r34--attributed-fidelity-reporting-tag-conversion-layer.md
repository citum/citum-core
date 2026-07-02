---
# csl26-3r34
title: 'Attributed fidelity reporting: tag conversion-layer suspects in oracle.js'
status: todo
type: task
created_at: 2026-07-02T13:19:55Z
updated_at: 2026-07-02T13:19:55Z
parent: csl26-cvfy
---

Phase 5 follow-on of the conversion-layer contract-test epic (see
docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md, Non-goals). Now that the
conversion layer has an explicit CSL 1.0.2 routing contract
(CSL_TYPES in crates/csl-legacy/src/csl_json.rs, contract tests in
crates/citum-schema-data/src/reference/conversion/contract_tests.rs),
expose per-reference conversion diagnostics from `citum render refs`
(e.g. a --json field noting 'rendered via fallback/default type') so
scripts/oracle.js and scripts/report-core.js can tag a failing fixture
case as 'conversion-layer suspect' before it is counted as a
style-fidelity failure. This mechanizes the classification rule in
docs/policies/STYLE_WORKFLOW_DECISION_RULES.md (style-defect /
migration-artifact / processor-defect / intentional divergence)
instead of leaving it to manual tracing.

Separate PR from the routing-closure work; see the parent epic
csl26-cvfy for the problem statement and the manual-attribution cost
this eliminates.
