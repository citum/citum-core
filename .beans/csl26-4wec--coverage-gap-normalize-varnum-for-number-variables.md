---
# csl26-4wec
title: 'coverage-gap: normalize var:/num: for number variables in legacy extractor'
status: todo
type: task
priority: normal
created_at: 2026-06-16T15:48:49Z
updated_at: 2026-06-16T15:48:49Z
---

coverage_gap.rs::collect_legacy_features emits var:page, var:volume, var:issue, var:edition from CSL <text variable="X"/> nodes. But migrate likely converts these to NumberVariable (Number(NumberVariable::Volume)), producing num:volume in compiled output. This creates false-positive gaps for ~2782 (volume), ~2798 (page), ~1701 (issue), ~2422 (edition) styles.

Investigate: check what compile_from_xml actually produces for <text variable="volume"/> — is it TemplateComponent::Number(NumberVariable::Volume) or TemplateComponent::Variable(SimpleVariable::Volume)?

If NumberVariable: update collect_legacy_features to emit num:{citum_key} for the known num-normalized variables (page→pages, volume, issue, edition, number-of-pages, number-of-volumes, citation-number, collection-number, chapter-number, number). Also handle the CSL 'page' → Citum 'pages' rename.

This is a follow-up to csl26-t56t (coverage-gap mode). Run --coverage-gap after the fix to verify these gaps collapse.

Files: crates/citum-analyze/src/coverage_gap.rs (collect_legacy_features)
