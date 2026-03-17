#![allow(missing_docs, reason = "test")]
/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Tests for cross-list variable deduplication (variable-once rule).

use citum_migrate::passes::deduplicate::deduplicate_variables_cross_lists;
use citum_schema::template::{
    ComponentOverride, ContributorRole, DateVariable, SimpleVariable, TemplateComponent,
    TemplateContributor, TemplateDate, TemplateList, TemplateVariable, TypeSelector,
};

fn announce_behavior(summary: &str) {
    println!("behavior: {summary}");
}

#[test]
fn test_contributor_cross_list_duplicate_suppressed() {
    announce_behavior(
        "When two migrated sibling lists both render author, the later author branch is suppressed so CSL variable-once behavior is preserved.",
    );
    // Setup: Create two sibling lists where 'author' appears in both.
    // After deduplication, the second list should have 'author' suppressed.
    let mut components = vec![
        TemplateComponent::List(TemplateList {
            items: vec![TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            })],
            ..Default::default()
        }),
        TemplateComponent::List(TemplateList {
            items: vec![TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Author,
                ..Default::default()
            })],
            ..Default::default()
        }),
    ];

    deduplicate_variables_cross_lists(&mut components);

    // Verify the first author is unsuppressed
    #[allow(clippy::collapsible_if, reason = "pattern matching readability")]
    if let TemplateComponent::List(ref list) = components[0] {
        if let TemplateComponent::Contributor(ref contrib) = list.items[0] {
            assert!(
                contrib.overrides.is_none()
                    || !contrib
                        .overrides
                        .as_ref()
                        .unwrap()
                        .contains_key(&TypeSelector::Single("all".to_string())),
                "First author should not be suppressed"
            );
        }
    }

    // Verify the second author is suppressed
    #[allow(clippy::collapsible_if, reason = "pattern matching readability")]
    if let TemplateComponent::List(ref list) = components[1] {
        if let TemplateComponent::Contributor(ref contrib) = list.items[0] {
            assert!(
                contrib.overrides.is_some(),
                "Second author should have overrides"
            );
            let overrides = contrib.overrides.as_ref().unwrap();
            let key = TypeSelector::Single("all".to_string());
            assert!(
                overrides.contains_key(&key),
                "Second author should have 'all' override"
            );
            if let ComponentOverride::Rendering(rendering) = &overrides[&key] {
                assert_eq!(
                    rendering.suppress,
                    Some(true),
                    "Second author should be suppressed"
                );
            }
        }
    }
}

#[test]
fn test_date_cross_list_duplicate_suppressed() {
    announce_behavior(
        "When two migrated sibling lists both render issued dates, the later date branch is suppressed to preserve CSL variable-once behavior.",
    );
    // Setup: Create two sibling lists where 'issued' appears in both.
    // After deduplication, the second list should have 'issued' suppressed.
    let mut components = vec![
        TemplateComponent::List(TemplateList {
            items: vec![TemplateComponent::Date(TemplateDate {
                date: DateVariable::Issued,
                ..Default::default()
            })],
            ..Default::default()
        }),
        TemplateComponent::List(TemplateList {
            items: vec![TemplateComponent::Date(TemplateDate {
                date: DateVariable::Issued,
                ..Default::default()
            })],
            ..Default::default()
        }),
    ];

    deduplicate_variables_cross_lists(&mut components);

    // Verify the first date is unsuppressed
    #[allow(clippy::collapsible_if, reason = "pattern matching readability")]
    if let TemplateComponent::List(ref list) = components[0] {
        if let TemplateComponent::Date(ref date) = list.items[0] {
            assert!(
                date.overrides.is_none()
                    || !date
                        .overrides
                        .as_ref()
                        .unwrap()
                        .contains_key(&TypeSelector::Single("all".to_string())),
                "First date should not be suppressed"
            );
        }
    }

    // Verify the second date is suppressed
    #[allow(clippy::collapsible_if, reason = "pattern matching readability")]
    if let TemplateComponent::List(ref list) = components[1] {
        if let TemplateComponent::Date(ref date) = list.items[0] {
            assert!(
                date.overrides.is_some(),
                "Second date should have overrides"
            );
            let overrides = date.overrides.as_ref().unwrap();
            let key = TypeSelector::Single("all".to_string());
            assert!(
                overrides.contains_key(&key),
                "Second date should have 'all' override"
            );
            if let ComponentOverride::Rendering(rendering) = &overrides[&key] {
                assert_eq!(
                    rendering.suppress,
                    Some(true),
                    "Second date should be suppressed"
                );
            }
        }
    }
}

#[test]
fn test_variable_cross_list_duplicate_suppressed() {
    announce_behavior(
        "When a migrated top-level variable and sibling list both render publisher, the later list rendering is suppressed to avoid duplicate output.",
    );
    // Setup: Create a top-level variable and a sibling list with the same variable.
    // After deduplication, the list variable should be suppressed.
    let mut components = vec![
        TemplateComponent::Variable(TemplateVariable {
            variable: SimpleVariable::Publisher,
            ..Default::default()
        }),
        TemplateComponent::List(TemplateList {
            items: vec![TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Publisher,
                ..Default::default()
            })],
            ..Default::default()
        }),
    ];

    deduplicate_variables_cross_lists(&mut components);

    // Verify the first variable is unsuppressed
    if let TemplateComponent::Variable(ref var) = components[0] {
        assert!(
            var.overrides.is_none()
                || !var
                    .overrides
                    .as_ref()
                    .unwrap()
                    .contains_key(&TypeSelector::Single("all".to_string())),
            "First variable should not be suppressed"
        );
    }

    // Verify the second variable is suppressed
    #[allow(clippy::collapsible_if, reason = "pattern matching readability")]
    if let TemplateComponent::List(ref list) = components[1] {
        if let TemplateComponent::Variable(ref var) = list.items[0] {
            assert!(
                var.overrides.is_some(),
                "Second variable should have overrides"
            );
            let overrides = var.overrides.as_ref().unwrap();
            let key = TypeSelector::Single("all".to_string());
            assert!(
                overrides.contains_key(&key),
                "Second variable should have 'all' override"
            );
            if let ComponentOverride::Rendering(rendering) = &overrides[&key] {
                assert_eq!(
                    rendering.suppress,
                    Some(true),
                    "Second variable should be suppressed"
                );
            }
        }
    }
}

#[test]
fn test_nested_list_variable_once_per_branch() {
    announce_behavior(
        "Nested migrated lists track variable-once suppression per branch so inner duplicates are handled without leaking outer-list state.",
    );
    // Setup: Create nested lists where a variable appears at different nesting levels.
    // Each nesting level should track its own scope across all sibling components.
    let mut components = vec![
        TemplateComponent::List(TemplateList {
            items: vec![
                TemplateComponent::Variable(TemplateVariable {
                    variable: SimpleVariable::Doi,
                    ..Default::default()
                }),
                TemplateComponent::List(TemplateList {
                    items: vec![TemplateComponent::Variable(TemplateVariable {
                        variable: SimpleVariable::Publisher,
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
            ],
            ..Default::default()
        }),
        TemplateComponent::List(TemplateList {
            items: vec![TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Publisher,
                ..Default::default()
            })],
            ..Default::default()
        }),
    ];

    deduplicate_variables_cross_lists(&mut components);

    // Verify Doi is unsuppressed in the first list
    #[allow(clippy::collapsible_if, reason = "pattern matching readability")]
    if let TemplateComponent::List(ref list) = components[0] {
        if let TemplateComponent::Variable(ref var) = list.items[0] {
            assert!(
                var.overrides.is_none()
                    || !var
                        .overrides
                        .as_ref()
                        .unwrap()
                        .contains_key(&TypeSelector::Single("all".to_string())),
                "Doi in first list should not be suppressed"
            );
        }
    }

    // Verify publisher in the inner nested list is unsuppressed (first occurrence)
    #[allow(clippy::collapsible_if, reason = "pattern matching readability")]
    if let TemplateComponent::List(ref list) = components[0] {
        if let TemplateComponent::List(ref inner_list) = list.items[1] {
            if let TemplateComponent::Variable(ref var) = inner_list.items[0] {
                assert!(
                    var.overrides.is_none()
                        || !var
                            .overrides
                            .as_ref()
                            .unwrap()
                            .contains_key(&TypeSelector::Single("all".to_string())),
                    "Publisher in inner list should not be suppressed"
                );
            }
        }
    }

    // Verify publisher in the second list is suppressed (duplicate)
    #[allow(clippy::collapsible_if, reason = "pattern matching readability")]
    if let TemplateComponent::List(ref list) = components[1] {
        if let TemplateComponent::Variable(ref var) = list.items[0] {
            assert!(
                var.overrides.is_some(),
                "Second publisher should have overrides"
            );
            let overrides = var.overrides.as_ref().unwrap();
            let key = TypeSelector::Single("all".to_string());
            assert!(
                overrides.contains_key(&key),
                "Second publisher should have 'all' override"
            );
            if let ComponentOverride::Rendering(rendering) = &overrides[&key] {
                assert_eq!(
                    rendering.suppress,
                    Some(true),
                    "Second publisher should be suppressed"
                );
            }
        }
    }
}
