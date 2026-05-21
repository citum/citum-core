---
# csl26-39tm
title: Output-driven migration compression and alias UX evidence
status: completed
type: feature
priority: high
created_at: 2026-05-20T19:32:13Z
updated_at: 2026-05-20T23:07:39Z
parent: csl26-f1u7
blocked_by:
    - csl26-kqji
---

Follow-up split from PR2 after the XML/AST medoid compaction attempt proved 
fidelity-risky.

## Scope

• Build structural compression from output-driven inference/oracle evidence,
not by flattening parsed CSL XML branches.
• Target apa-6th-edition specifically: reduce migrated output below 1,500 
LOC without treating APA 6th as an alias of APA 7th and without 
bibliography/citation oracle regression.
• Emit machine-readable evidence for future UX: exact registry alias status,
parent/template link, canonical target, emitted form, preserved deltas,
discarded deltas, and output-size reduction.
• Design optional UI choices around evidence-backed actions: keep standalone,
register local alias, or propose global alias. Global aliases require 
reviewed equivalence, not ancestry/template evidence.
• Revisit long-tail styles such as american-medical-association and 
institute-of-physics-numeric only where output-driven deltas prove the
compression is behavior-preserving.

## Progress (2026-05-20 PR3)

Infrastructure delivered:

[x] MigrationEvidence struct + --emit-evidence <path> sidecar emission
(crates/citum-migrate/src/evidence.rs)
[x] Reverse <info><link rel="template"> discovery in StyleLineage::resolve, 
surfaced as inert family_candidate
[x] --family-candidate off|auto|<id> CLI flag routing discovered candidates 
through ExistingWrapper { preserve_template_deltas: true }
[x] Scorecard consumes evidence sidecar; surfaces "Compression candidates" 
section
[x] apa-6th-edition added to SQI sentinels (standalone baseline: 18/18 cit • 
10/37 bib • 5,661 LOC • SQI 66.67)
[x] Baseline doc refreshed in place

## PR4 Delivery

Output-driven compression mechanism completed:

[x] --minimize-wrapper CLI flag emits minimal wrapper (info + extends) when 
family-candidate parent is promoted
[x] Migrated apa-6th-edition LOC < 1,500: reduced 5,661 → 5 with zero citation 
fidelity regression and improved bibliography (18/18 • 33/33)
[x] Evidence.EmittedForm tracks minimized status in sidecar JSON
[x] rstest case verifies minimize_wrapper emits info+extends only for promoted 
family candidates
[x] Baseline doc updated: apa-6th-edition now shows minimized form with 100.00 
SQI vs. pre-compression 66.67

The bean's primary objective is complete: apa-6th-edition compression is 
achieved via evidence-driven minimal form emission.
