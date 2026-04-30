#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in test, benchmark, and example code."
)]
#![allow(missing_docs, reason = "test")]
/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Tests for cross-list variable deduplication (variable-once rule).
//!
//! In template-v2, duplicates are removed rather than suppressed via overrides.

use citum_migrate::passes::deduplicate::deduplicate_variables_cross_lists;
use citum_schema::template::{
    ContributorRole, DateVariable, SimpleVariable, TemplateComponent, TemplateContributor,
    TemplateDate, TemplateGroup, TemplateVariable,
};

fn announce_behavior(summary: &str) {
    println!("behavior: {summary}");
}

#[test]
fn test_contributor_cross_list_duplicate_removed() {
    announce_behavior(
        "When two migrated sibling lists both render author, the later author branch is removed so CSL variable-once behavior is preserved.",
    );
    let mut components = vec![
        TemplateComponent::Group(TemplateGroup {
            group: vec![TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            })],
            ..Default::default()
        }),
        TemplateComponent::Group(TemplateGroup {
            group: vec![TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            })],
            ..Default::default()
        }),
    ];

    deduplicate_variables_cross_lists(&mut components);

    // First author remains
    if let TemplateComponent::Group(ref list) = components[0] {
        assert_eq!(list.group.len(), 1, "First list should still have author");
    }

    // Second author is removed
    if let TemplateComponent::Group(ref list) = components[1] {
        assert!(
            list.group.is_empty(),
            "Second list should have author removed"
        );
    }
}

#[test]
fn test_date_cross_list_duplicate_removed() {
    announce_behavior(
        "When two migrated sibling lists both render issued dates, the later date branch is removed to preserve CSL variable-once behavior.",
    );
    let mut components = vec![
        TemplateComponent::Group(TemplateGroup {
            group: vec![TemplateComponent::Date(TemplateDate {
                date: DateVariable::Issued,
                ..Default::default()
            })],
            ..Default::default()
        }),
        TemplateComponent::Group(TemplateGroup {
            group: vec![TemplateComponent::Date(TemplateDate {
                date: DateVariable::Issued,
                ..Default::default()
            })],
            ..Default::default()
        }),
    ];

    deduplicate_variables_cross_lists(&mut components);

    if let TemplateComponent::Group(ref list) = components[0] {
        assert_eq!(list.group.len(), 1, "First list should still have date");
    }
    if let TemplateComponent::Group(ref list) = components[1] {
        assert!(
            list.group.is_empty(),
            "Second list should have date removed"
        );
    }
}

#[test]
fn test_variable_cross_list_duplicate_removed() {
    announce_behavior(
        "When a migrated top-level variable and sibling list both render publisher, the later list rendering is removed to avoid duplicate output.",
    );
    let mut components = vec![
        TemplateComponent::Variable(TemplateVariable {
            variable: SimpleVariable::Publisher,
            ..Default::default()
        }),
        TemplateComponent::Group(TemplateGroup {
            group: vec![TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Publisher,
                ..Default::default()
            })],
            ..Default::default()
        }),
    ];

    deduplicate_variables_cross_lists(&mut components);

    // Top-level variable remains
    assert!(
        matches!(components[0], TemplateComponent::Variable(_)),
        "First variable should remain"
    );

    // Nested duplicate is removed
    if let TemplateComponent::Group(ref list) = components[1] {
        assert!(
            list.group.is_empty(),
            "Nested duplicate publisher should be removed"
        );
    }
}

#[test]
fn test_nested_list_variable_once_per_branch() {
    announce_behavior(
        "Nested migrated lists track variable-once removal per branch so inner duplicates are handled without leaking outer-list state.",
    );
    let mut components = vec![
        TemplateComponent::Group(TemplateGroup {
            group: vec![
                TemplateComponent::Variable(TemplateVariable {
                    variable: SimpleVariable::Doi,
                    ..Default::default()
                }),
                TemplateComponent::Group(TemplateGroup {
                    group: vec![TemplateComponent::Variable(TemplateVariable {
                        variable: SimpleVariable::Publisher,
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
            ],
            ..Default::default()
        }),
        TemplateComponent::Group(TemplateGroup {
            group: vec![TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Publisher,
                ..Default::default()
            })],
            ..Default::default()
        }),
    ];

    deduplicate_variables_cross_lists(&mut components);

    // Doi stays in first list
    if let TemplateComponent::Group(ref list) = components[0] {
        assert!(
            matches!(list.group[0], TemplateComponent::Variable(_)),
            "Doi in first list should remain"
        );
        // Publisher in inner nested list stays (first occurrence)
        if let TemplateComponent::Group(ref inner_list) = list.group[1] {
            assert_eq!(
                inner_list.group.len(),
                1,
                "Publisher in inner list should remain (first occurrence)"
            );
        }
    }

    // Publisher in second list is removed (duplicate)
    if let TemplateComponent::Group(ref list) = components[1] {
        assert!(
            list.group.is_empty(),
            "Second publisher should be removed (duplicate)"
        );
    }
}
