---
# csl26-57a7
title: Audit TemplateComponent::Group consumers for an Alternatives arm
status: scrapped
type: task
priority: normal
tags:
    - engine
    - style
    - schema
    - fidelity
created_at: 2026-09-07T00:42:55Z
updated_at: 2026-09-08T12:49:16Z
parent: csl26-40n4
---

docs/specs/ALTERNATIVES.md (Draft) proposed a new TemplateComponent::Alternatives variant -- since redesigned as select: first, docs/specs/GROUP_SELECT.md. A grep found TemplateComponent::Group matched in 21 files across citum-engine and citum-schema-style -- not just the renderer. Three were checked directly and confirmed to key on specific component kinds (Title, Date(Issued), Number(Volume), Variable(Url/Doi)) for non-rendering purposes:

- crates/citum-engine/src/values/list.rs (is_term_based -- already accounted for in the spec)
- crates/citum-engine/src/processor/rendering/grouped/component_predicates.rs (citation grouping / contributor-stripping)
- crates/citum-engine/src/processor/rendering/grouped/template_policy.rs (article-journal bibliography template filtering, the same file csl26-8z39 extends)

If a title, contributor, date, number, or url/doi variable is wrapped inside an alternatives: candidate, these consumers won't see it structurally, regardless of nesting depth -- restricting nesting does not fix this, since the cause is what a candidate contains, not how deep it sits.

Update 2026-09-07: audit complete for all 18 files. Findings folded into ALTERNATIVES.md v1.1. Most are already safe under the content restriction (key on title/contributor, which v1 already forbids as candidate content) or are compiler-enforced-safe (dispatch_component! in macros.rs is exhaustive, no wildcard). Two real gaps found and closed by widening the restriction / adding requirements in the spec itself: sorting.rs's first_date_component_ref searches any date, not just issued (restriction widened from 'issued date' to 'any date'); helpers.rs's leading_group_affix/strip_leading_group_affixes are kind-independent structural helpers needing their own Alternatives arm regardless of content restriction (added to Implementation Notes/AC). Two low-severity, non-blocking gaps remain, deferred to this bean's remaining scope: lint.rs's collect_template_requirements and api/warnings.rs's unknown-enum-variant scanner both special-case Group but not Alternatives -- tooling completeness, not rendering correctness.

## Todo
- [x] Enumerate the remaining 18 files matching TemplateComponent::Group (grep -rln 'TemplateComponent::Group' crates/citum-engine/src crates/citum-schema-style/src, minus the 3 already checked)
- [x] For each, determine whether it needs an Alternatives arm to remain correct once alternatives: is in general use
- [ ] Add the arm to each file that needs one, with a regression test per file
- [ ] Once complete, lift ALTERNATIVES.md's v1 placement restriction (update Scope/'v1 placement restriction' section, promote to v2)

## Reasons for Scrapping

This bean's entire premise -- `docs/specs/ALTERNATIVES.md` proposing a new `TemplateComponent::Alternatives` variant, requiring an audit of every structural consumer of `TemplateComponent::Group` for a matching `Alternatives` arm -- no longer applies. Design review (PR #1268) found a cleaner shape: `select: first` as a mode on `TemplateGroup` itself, not a new component type. Since a `select: first` group is still `TemplateComponent::Group`, every consumer this bean was going to audit (component_predicates.rs, template_policy.rs, sorting.rs, the affix helpers) already recurses into it with no new arm needed. See docs/specs/GROUP_SELECT.md.
