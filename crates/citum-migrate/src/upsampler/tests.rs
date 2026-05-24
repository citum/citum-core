/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]

use super::*;
use crate::ir;
use csl_legacy::model::{Choose, ChooseBranch, CslNode, Formatting, Group, Text};

fn literal_text(value: &str) -> CslNode {
    CslNode::Text(Text {
        value: Some(value.to_string()),
        variable: None,
        macro_name: None,
        term: None,
        form: None,
        prefix: None,
        suffix: None,
        quotes: None,
        text_case: None,
        strip_periods: None,
        plural: None,
        macro_call_order: None,
        formatting: Formatting::default(),
    })
}

fn choose_branch(position: Option<&str>, children: Vec<CslNode>) -> ChooseBranch {
    ChooseBranch {
        match_mode: None,
        type_: None,
        variable: None,
        is_numeric: None,
        is_uncertain_date: None,
        locator: None,
        position: position.map(ToString::to_string),
        children,
    }
}

fn choose_branch_with_conditions(
    position: Option<&str>,
    type_: Option<&str>,
    variable: Option<&str>,
    locator: Option<&str>,
    children: Vec<CslNode>,
) -> ChooseBranch {
    ChooseBranch {
        match_mode: None,
        type_: type_.map(ToString::to_string),
        variable: variable.map(ToString::to_string),
        is_numeric: None,
        is_uncertain_date: None,
        locator: locator.map(ToString::to_string),
        position: position.map(ToString::to_string),
        children,
    }
}

fn group(children: Vec<CslNode>) -> CslNode {
    CslNode::Group(Group {
        delimiter: None,
        prefix: None,
        suffix: None,
        children,
        macro_call_order: None,
        formatting: Formatting::default(),
    })
}

fn collect_text_values<'a>(nodes: &'a [ir::Node], output: &mut Vec<&'a str>) {
    for node in nodes {
        match node {
            ir::Node::Text { value } => output.push(value.as_str()),
            ir::Node::Group(group) => collect_text_values(&group.children, output),
            ir::Node::Condition(condition) => {
                collect_text_values(&condition.then_branch, output);
                for branch in &condition.else_if_branches {
                    collect_text_values(&branch.children, output);
                }
                if let Some(else_branch) = &condition.else_branch {
                    collect_text_values(else_branch, output);
                }
            }
            _ => {}
        }
    }
}

fn text_values(nodes: &[ir::Node]) -> Vec<&str> {
    let mut values = Vec::new();
    collect_text_values(nodes, &mut values);
    values
}

#[test]
fn extract_position_templates_preserves_nested_choose_siblings() {
    let choose = Choose {
        if_branch: choose_branch(Some("subsequent"), vec![literal_text("SHORT")]),
        else_if_branches: vec![],
        else_branch: Some(vec![literal_text("FULL")]),
    };
    let legacy_nodes = vec![group(vec![
        literal_text("PREFIX"),
        CslNode::Choose(choose),
        literal_text("SUFFIX"),
    ])];

    let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

    assert!(!extracted.unsupported_mixed_conditions);
    assert_eq!(
        text_values(
            extracted
                .first
                .as_deref()
                .expect("first variant should exist")
        ),
        vec!["PREFIX", "FULL", "SUFFIX"]
    );
    assert_eq!(
        text_values(
            extracted
                .subsequent
                .as_deref()
                .expect("subsequent variant should exist")
        ),
        vec!["PREFIX", "SHORT", "SUFFIX"]
    );
}

#[test]
fn extract_position_templates_merges_multiple_position_chooses() {
    let choose = Choose {
        if_branch: choose_branch(Some("subsequent"), vec![literal_text("SHORT")]),
        else_if_branches: vec![choose_branch(
            Some("first"),
            vec![literal_text("FIRST-ONLY")],
        )],
        else_branch: Some(vec![literal_text("FULL")]),
    };
    let locator_choose = Choose {
        if_branch: choose_branch(Some("ibid-with-locator"), vec![literal_text("LOC")]),
        else_if_branches: vec![choose_branch(Some("ibid"), vec![literal_text("IBID")])],
        else_branch: Some(vec![literal_text("DATE")]),
    };
    let legacy_nodes = vec![
        literal_text("A"),
        CslNode::Choose(choose),
        literal_text("MID"),
        CslNode::Choose(locator_choose),
        literal_text("Z"),
    ];

    let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

    assert!(!extracted.unsupported_mixed_conditions);
    assert_eq!(
        text_values(
            extracted
                .first
                .as_deref()
                .expect("first variant should exist")
        ),
        vec!["A", "FIRST-ONLY", "MID", "DATE", "Z"],
    );
    assert_eq!(
        text_values(
            extracted
                .subsequent
                .as_deref()
                .expect("subsequent variant should exist")
        ),
        vec!["A", "SHORT", "MID", "DATE", "Z"]
    );
    assert_eq!(
        text_values(
            extracted
                .ibid
                .as_deref()
                .expect("ibid variant should exist")
        ),
        vec!["A", "FULL", "MID", "IBID", "Z"]
    );
}

#[test]
fn extract_position_templates_selects_ibid_per_choose() {
    let ibid_only_choose = Choose {
        if_branch: choose_branch(Some("ibid"), vec![literal_text("IBID-ONLY")]),
        else_if_branches: vec![],
        else_branch: Some(vec![literal_text("DEFAULT-A")]),
    };
    let ibid_with_locator_choose = Choose {
        if_branch: choose_branch(
            Some("ibid-with-locator"),
            vec![literal_text("IBID-WITH-LOC")],
        ),
        else_if_branches: vec![],
        else_branch: Some(vec![literal_text("DEFAULT-B")]),
    };
    let legacy_nodes = vec![
        CslNode::Choose(ibid_only_choose),
        literal_text("MID"),
        CslNode::Choose(ibid_with_locator_choose),
    ];

    let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

    assert!(!extracted.unsupported_mixed_conditions);
    assert_eq!(
        text_values(
            extracted
                .ibid
                .as_deref()
                .expect("ibid variant should exist")
        ),
        vec!["IBID-ONLY", "MID", "IBID-WITH-LOC"]
    );
}

#[test]
fn extract_position_templates_preserves_nested_non_position_choose() {
    let nested_choose = Choose {
        if_branch: ChooseBranch {
            match_mode: None,
            type_: None,
            variable: Some("title".to_string()),
            is_numeric: None,
            is_uncertain_date: None,
            locator: None,
            position: None,
            children: vec![literal_text("LEGAL")],
        },
        else_if_branches: vec![],
        else_branch: Some(vec![literal_text("GENERAL")]),
    };
    let choose = Choose {
        if_branch: choose_branch(Some("subsequent"), vec![CslNode::Choose(nested_choose)]),
        else_if_branches: vec![],
        else_branch: Some(vec![literal_text("FULL")]),
    };
    let legacy_nodes = vec![CslNode::Choose(choose)];

    let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

    assert!(!extracted.unsupported_mixed_conditions);
    assert!(matches!(
        extracted.subsequent.as_deref(),
        Some([ir::Node::Condition(_)])
    ));
}

#[test]
fn extract_position_templates_rewrites_position_type_branches_into_conditionals() {
    let choose = Choose {
        if_branch: choose_branch_with_conditions(
            None,
            None,
            Some("archive"),
            None,
            vec![literal_text("ARCHIVE")],
        ),
        else_if_branches: vec![
            choose_branch_with_conditions(
                Some("first"),
                Some("interview"),
                None,
                None,
                vec![literal_text("INTERVIEW-FIRST")],
            ),
            choose_branch_with_conditions(
                Some("first"),
                Some("personal_communication"),
                None,
                None,
                vec![literal_text("PERSONAL-FIRST")],
            ),
        ],
        else_branch: None,
    };
    let legacy_nodes = vec![CslNode::Choose(choose)];

    let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

    assert!(!extracted.unsupported_mixed_conditions);
    assert!(matches!(
        extracted.first.as_deref(),
        Some([ir::Node::Condition(condition)])
            if condition.if_variables.len() == 1
                && condition.else_if_branches.len() == 2
                && condition.else_if_branches.iter().all(|branch| branch.if_item_type.len() == 1)
                && condition.else_branch.is_none()
    ));
    assert_eq!(
        text_values(
            extracted
                .first
                .as_deref()
                .expect("first variant should exist")
        ),
        vec!["ARCHIVE", "INTERVIEW-FIRST", "PERSONAL-FIRST"]
    );
}

#[test]
fn extract_position_templates_preserves_non_position_siblings_in_mixed_subsequent_variant() {
    let choose = Choose {
        if_branch: choose_branch_with_conditions(
            None,
            None,
            Some("archive"),
            None,
            vec![literal_text("ARCHIVE")],
        ),
        else_if_branches: vec![
            choose_branch_with_conditions(
                Some("subsequent"),
                Some("interview"),
                None,
                None,
                vec![literal_text("SHORT-INTERVIEW")],
            ),
            choose_branch_with_conditions(
                Some("subsequent"),
                Some("personal_communication"),
                None,
                None,
                vec![literal_text("SHORT-PERSONAL")],
            ),
        ],
        else_branch: Some(vec![literal_text("FULL")]),
    };
    let legacy_nodes = vec![group(vec![
        literal_text("PREFIX"),
        CslNode::Choose(choose),
        literal_text("SUFFIX"),
    ])];

    let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

    assert!(!extracted.unsupported_mixed_conditions);
    assert!(matches!(
        extracted.subsequent.as_deref(),
        Some([ir::Node::Group(group)])
            if matches!(group.children.as_slice(), [_, ir::Node::Condition(_), _])
    ));
    assert_eq!(
        text_values(
            extracted
                .subsequent
                .as_deref()
                .expect("subsequent variant should exist")
        ),
        vec![
            "PREFIX",
            "ARCHIVE",
            "SHORT-INTERVIEW",
            "SHORT-PERSONAL",
            "FULL",
            "SUFFIX"
        ]
    );
}

#[test]
fn extract_position_templates_supports_position_locator_branches() {
    let choose = Choose {
        if_branch: choose_branch_with_conditions(
            Some("first"),
            None,
            None,
            Some("chapter"),
            vec![literal_text("CHAPTER-FIRST")],
        ),
        else_if_branches: vec![choose_branch_with_conditions(
            Some("subsequent"),
            None,
            None,
            Some("page"),
            vec![literal_text("PAGE-SUBSEQUENT")],
        )],
        else_branch: Some(vec![literal_text("FALLBACK")]),
    };
    let legacy_nodes = vec![CslNode::Choose(choose)];

    let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

    assert!(!extracted.unsupported_mixed_conditions);
    assert_eq!(
        text_values(
            extracted
                .first
                .as_deref()
                .expect("first variant should exist")
        ),
        vec!["CHAPTER-FIRST", "FALLBACK"]
    );
    assert_eq!(
        text_values(
            extracted
                .subsequent
                .as_deref()
                .expect("subsequent variant should exist")
        ),
        vec!["PAGE-SUBSEQUENT", "FALLBACK"]
    );
}

#[test]
fn extract_position_templates_marks_ambiguous_fallbacks_as_unsupported() {
    let choose = Choose {
        if_branch: choose_branch_with_conditions(
            None,
            None,
            Some("title"),
            None,
            vec![literal_text("TITLE")],
        ),
        else_if_branches: vec![choose_branch(
            Some("subsequent"),
            vec![literal_text("SHORT")],
        )],
        else_branch: Some(vec![literal_text("FULL")]),
    };
    let legacy_nodes = vec![CslNode::Choose(choose)];

    let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

    assert!(extracted.unsupported_mixed_conditions);
    assert!(!extracted.has_overrides());
    assert!(extracted.first.is_none());
}

#[test]
fn extract_position_templates_marks_duplicate_position_branches_as_unsupported() {
    let choose = Choose {
        if_branch: choose_branch(Some("subsequent"), vec![literal_text("SHORT")]),
        else_if_branches: vec![choose_branch(
            Some("subsequent"),
            vec![literal_text("AGAIN")],
        )],
        else_branch: Some(vec![literal_text("FULL")]),
    };
    let legacy_nodes = vec![CslNode::Choose(choose)];

    let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

    assert!(extracted.unsupported_mixed_conditions);
}

#[test]
fn map_choose_prefers_else_branch_for_uncertain_dates() {
    let choose = Choose {
        if_branch: ChooseBranch {
            match_mode: None,
            type_: None,
            variable: None,
            is_numeric: None,
            is_uncertain_date: Some("issued".to_string()),
            locator: None,
            position: None,
            children: vec![literal_text("UNCERTAIN")],
        },
        else_if_branches: vec![],
        else_branch: Some(vec![literal_text("CERTAIN")]),
    };

    let mapped = Upsampler::new()
        .map_choose(&choose)
        .expect("uncertain-date choose should map");

    assert_eq!(text_values(std::slice::from_ref(&mapped)), vec!["CERTAIN"]);
}

#[test]
fn map_choose_drops_uncertain_date_markers_without_default_branch() {
    let choose = Choose {
        if_branch: ChooseBranch {
            match_mode: None,
            type_: None,
            variable: None,
            is_numeric: None,
            is_uncertain_date: Some("issued".to_string()),
            locator: None,
            position: None,
            children: vec![literal_text("circa")],
        },
        else_if_branches: vec![],
        else_branch: None,
    };

    let mapped = Upsampler::new()
        .map_choose(&choose)
        .expect("uncertain-date choose should map");

    match mapped {
        ir::Node::Group(group) => assert!(group.children.is_empty()),
        other => panic!("expected empty group fallback, got {other:?}"),
    }
}

#[test]
fn map_choose_prefers_first_citation_content_for_position_fallbacks() {
    let choose = Choose {
        if_branch: choose_branch(Some("subsequent"), vec![literal_text("SHORT")]),
        else_if_branches: vec![],
        else_branch: Some(vec![literal_text("FULL")]),
    };

    let mapped = Upsampler::new()
        .map_choose(&choose)
        .expect("position choose should map");

    assert_eq!(text_values(std::slice::from_ref(&mapped)), vec!["FULL"]);
}

#[test]
fn map_choose_uses_negated_else_if_as_effective_fallback() {
    let choose = Choose {
        if_branch: choose_branch_with_conditions(
            None,
            Some("book"),
            None,
            None,
            vec![literal_text("BOOK")],
        ),
        else_if_branches: vec![ChooseBranch {
            match_mode: Some("none".to_string()),
            type_: Some("book".to_string()),
            variable: None,
            is_numeric: None,
            is_uncertain_date: None,
            locator: None,
            position: None,
            children: vec![literal_text("NOT-BOOK")],
        }],
        else_branch: None,
    };

    let mapped = Upsampler::new()
        .map_choose(&choose)
        .expect("choose should map");

    match mapped {
        ir::Node::Condition(condition) => {
            assert!(condition.else_if_branches.is_empty());
            assert_eq!(text_values(&condition.then_branch), vec!["BOOK"]);
            assert_eq!(
                text_values(
                    condition
                        .else_branch
                        .as_deref()
                        .expect("negated else-if should become else branch")
                ),
                vec!["NOT-BOOK"]
            );
        }
        other => panic!("expected condition node, got {other:?}"),
    }
}

#[test]
fn map_choose_preserves_existing_else_over_negated_if_fallback() {
    let choose = Choose {
        if_branch: ChooseBranch {
            match_mode: Some("none".to_string()),
            type_: Some("book".to_string()),
            variable: None,
            is_numeric: None,
            is_uncertain_date: None,
            locator: None,
            position: None,
            children: vec![literal_text("NOT-BOOK")],
        },
        else_if_branches: vec![choose_branch_with_conditions(
            None,
            Some("article-journal"),
            None,
            None,
            vec![literal_text("ARTICLE")],
        )],
        else_branch: Some(vec![literal_text("EXISTING-ELSE")]),
    };

    let mapped = Upsampler::new()
        .map_choose(&choose)
        .expect("choose should map");

    match mapped {
        ir::Node::Condition(condition) => {
            assert!(condition.then_branch.is_empty());
            assert_eq!(
                text_values(
                    condition
                        .else_branch
                        .as_deref()
                        .expect("existing else should remain in place")
                ),
                vec!["EXISTING-ELSE"]
            );
            let mut all_text = text_values(
                condition
                    .else_branch
                    .as_deref()
                    .expect("existing else should remain in place"),
            );
            all_text.extend(text_values(&condition.then_branch));
            for branch in &condition.else_if_branches {
                all_text.extend(text_values(&branch.children));
            }
            assert!(!all_text.contains(&"NOT-BOOK"));
            assert!(all_text.contains(&"ARTICLE"));
        }
        other => panic!("expected condition node, got {other:?}"),
    }
}
