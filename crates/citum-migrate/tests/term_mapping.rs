/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#![allow(missing_docs, reason = "test")]
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

use citum_migrate::ir::Node;
use citum_migrate::{TemplateCompiler, Upsampler};
use citum_schema::locale::{GeneralTerm, TermForm};
use citum_schema::template::{NumberVariable, SimpleVariable, TemplateComponent};
use csl_legacy::model::{CslNode, Formatting, Label, Names, Text};

fn announce_behavior(summary: &str) {
    tracing::debug!("behavior: {summary}");
}

#[test]
fn test_upsample_term() {
    announce_behavior(
        "A CSL text term migrates to a Citum term node with the matching normalized term and default long form.",
    );
    let legacy_node = CslNode::Text(Text {
        term: Some("in".to_string()),
        formatting: Formatting::default(),
        value: None,
        variable: None,
        macro_name: None,
        form: None,
        prefix: None,
        suffix: None,
        quotes: None,
        text_case: None,
        strip_periods: None,
        plural: None,
        macro_call_order: None,
    });

    let upsampler = Upsampler::new();
    let ir_nodes = upsampler.upsample_nodes(&[legacy_node]);

    assert_eq!(ir_nodes.len(), 1);
    match &ir_nodes[0] {
        Node::Term(term_block) => {
            assert_eq!(term_block.term, GeneralTerm::In);
            assert_eq!(term_block.form, TermForm::Long);
        }
        _ => panic!("Expected Node::Term, got {:?}", ir_nodes[0]),
    }
}

#[test]
fn test_upsample_term_with_form() {
    announce_behavior(
        "An unknown CSL term value stays literal during migration instead of being coerced into an unsupported Citum term.",
    );
    let legacy_node = CslNode::Text(Text {
        term: Some("editor".to_string()),
        form: Some("short".to_string()),
        formatting: Formatting::default(),
        value: None,
        variable: None,
        macro_name: None,
        prefix: None,
        suffix: None,
        quotes: None,
        text_case: None,
        strip_periods: None,
        plural: None,
        macro_call_order: None,
    });

    let upsampler = Upsampler::new();
    let ir_nodes = upsampler.upsample_nodes(&[legacy_node]);

    assert_eq!(ir_nodes.len(), 1);
    match &ir_nodes[0] {
        Node::Text { value } => {
            assert_eq!(value, "editor");
        }
        _ => panic!(
            "Expected Node::Text for unknown term, got {:?}",
            ir_nodes[0]
        ),
    }
}

#[test]
fn test_upsample_term_preserves_strip_periods() {
    announce_behavior(
        "A CSL term with strip-periods keeps that formatting flag when migrated into a Citum term node.",
    );
    let legacy_node = CslNode::Text(Text {
        term: Some("in".to_string()),
        formatting: Formatting::default(),
        value: None,
        variable: None,
        macro_name: None,
        form: None,
        prefix: None,
        suffix: None,
        quotes: None,
        text_case: None,
        strip_periods: Some(true),
        plural: None,
        macro_call_order: None,
    });

    let upsampler = Upsampler::new();
    let ir_nodes = upsampler.upsample_nodes(&[legacy_node]);

    assert_eq!(ir_nodes.len(), 1);
    match &ir_nodes[0] {
        Node::Term(term_block) => {
            assert_eq!(term_block.formatting.strip_periods, Some(true));
        }
        _ => panic!("Expected Node::Term, got {:?}", ir_nodes[0]),
    }
}

#[test]
fn test_upsample_label_preserves_strip_periods() {
    announce_behavior(
        "A CSL label keeps strip-periods metadata when migration lifts it into Citum variable label settings.",
    );
    let legacy_node = CslNode::Label(Label {
        variable: Some("page".to_string()),
        form: Some("short".to_string()),
        prefix: None,
        suffix: None,
        text_case: None,
        strip_periods: Some(true),
        plural: None,
        macro_call_order: None,
        formatting: Formatting::default(),
    });

    let upsampler = Upsampler::new();
    let ir_nodes = upsampler.upsample_nodes(&[legacy_node]);

    assert_eq!(ir_nodes.len(), 1);
    match &ir_nodes[0] {
        Node::Variable(variable) => {
            let label = variable
                .label
                .as_ref()
                .expect("label metadata should exist");
            assert_eq!(label.formatting.strip_periods, Some(true));
        }
        _ => panic!("Expected Node::Variable, got {:?}", ir_nodes[0]),
    }
}

#[test]
fn test_upsample_strip_periods_preserves_none_and_false() {
    announce_behavior(
        "Migration preserves both omitted and explicit false strip-periods settings instead of normalizing them away.",
    );
    let upsampler = Upsampler::new();

    let term_nodes = upsampler.upsample_nodes(&[CslNode::Text(Text {
        term: Some("in".to_string()),
        formatting: Formatting::default(),
        value: None,
        variable: None,
        macro_name: None,
        form: None,
        prefix: None,
        suffix: None,
        quotes: None,
        text_case: None,
        strip_periods: None,
        plural: None,
        macro_call_order: None,
    })]);
    match &term_nodes[0] {
        Node::Term(term_block) => assert_eq!(term_block.formatting.strip_periods, None),
        _ => panic!("Expected Node::Term, got {:?}", term_nodes[0]),
    }

    let label_nodes = upsampler.upsample_nodes(&[CslNode::Label(Label {
        variable: Some("page".to_string()),
        form: Some("short".to_string()),
        prefix: None,
        suffix: None,
        text_case: None,
        strip_periods: Some(false),
        plural: None,
        macro_call_order: None,
        formatting: Formatting::default(),
    })]);
    match &label_nodes[0] {
        Node::Variable(variable) => {
            let label = variable
                .label
                .as_ref()
                .expect("label metadata should exist");
            assert_eq!(label.formatting.strip_periods, Some(false));
        }
        _ => panic!("Expected Node::Variable, got {:?}", label_nodes[0]),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "test functions naturally exceed 100 lines"
)]
#[test]
fn test_template_compiler_preserves_strip_periods_through_compilation() {
    announce_behavior(
        "Strip-periods formatting survives the full CSL-to-Citum migration pipeline through template compilation for terms, labels, and contributors.",
    );
    let upsampler = Upsampler::new();
    let compiler = TemplateCompiler;
    let legacy_nodes = vec![
        CslNode::Text(Text {
            term: Some("in".to_string()),
            formatting: Formatting::default(),
            value: None,
            variable: None,
            macro_name: None,
            form: None,
            prefix: None,
            suffix: None,
            quotes: None,
            text_case: None,
            strip_periods: Some(true),
            plural: None,
            macro_call_order: None,
        }),
        CslNode::Label(Label {
            variable: Some("page".to_string()),
            form: Some("short".to_string()),
            prefix: None,
            suffix: None,
            text_case: None,
            strip_periods: Some(true),
            plural: None,
            macro_call_order: None,
            formatting: Formatting::default(),
        }),
        CslNode::Names(Names {
            variable: "editor".to_string(),
            delimiter: None,
            delimiter_precedes_et_al: None,
            et_al_min: None,
            et_al_use_first: None,
            et_al_subsequent_min: None,
            et_al_subsequent_use_first: None,
            prefix: None,
            suffix: None,
            children: vec![CslNode::Label(Label {
                variable: Some("editor".to_string()),
                form: Some("short".to_string()),
                prefix: None,
                suffix: None,
                text_case: None,
                strip_periods: Some(true),
                plural: None,
                macro_call_order: None,
                formatting: Formatting::default(),
            })],
            macro_call_order: None,
            formatting: Formatting::default(),
        }),
    ];

    let compiled = compiler.compile(&upsampler.upsample_nodes(&legacy_nodes));

    let term = compiled
        .iter()
        .find_map(|component| match component {
            TemplateComponent::Term(term) if term.term == GeneralTerm::In => Some(term),
            _ => None,
        })
        .expect("term component should compile");
    assert_eq!(term.rendering.strip_periods, Some(true));

    let pages = compiled
        .iter()
        .find_map(|component| match component {
            TemplateComponent::Number(number) if number.number == NumberVariable::Pages => {
                Some(number)
            }
            _ => None,
        })
        .expect("page label should compile into a number component");
    assert_eq!(pages.rendering.strip_periods, Some(true));

    let editor = compiled
        .iter()
        .find_map(|component| match component {
            TemplateComponent::Contributor(contributor)
                if contributor.contributor == citum_schema::template::ContributorRole::Editor =>
            {
                Some(contributor)
            }
            _ => None,
        })
        .expect("editor names should compile into a contributor component");
    assert_eq!(editor.rendering.strip_periods, Some(true));

    let locator = compiler.compile(&upsampler.upsample_nodes(&[CslNode::Label(Label {
        variable: Some("locator".to_string()),
        form: Some("short".to_string()),
        prefix: None,
        suffix: None,
        text_case: None,
        strip_periods: Some(true),
        plural: None,
        macro_call_order: None,
        formatting: Formatting::default(),
    })]));
    let _locator = locator
        .iter()
        .find_map(|component| match component {
            TemplateComponent::Variable(variable)
                if variable.variable == SimpleVariable::Locator =>
            {
                Some(variable)
            }
            _ => None,
        })
        .expect("locator label should compile into a locator variable");
    // Locator label config is now handled by style-level LocatorConfig, not on template variables
}

#[test]
fn csl_m_cstr_variable_compiles_to_a_canonical_identifier_component() {
    let legacy_node = CslNode::Text(Text {
        variable: Some("CSTR".to_string()),
        prefix: Some("CSTR: ".to_string()),
        suffix: Some(".".to_string()),
        formatting: Formatting::default(),
        value: None,
        macro_name: None,
        term: None,
        form: None,
        quotes: None,
        text_case: None,
        strip_periods: None,
        plural: None,
        macro_call_order: None,
    });

    let compiled = TemplateCompiler.compile(&Upsampler::new().upsample_nodes(&[legacy_node]));

    let [TemplateComponent::Identifier(identifier)] = compiled.as_slice() else {
        panic!("CSTR should compile to exactly one identifier component: {compiled:?}");
    };
    assert_eq!(identifier.identifier.as_str(), "cstr");
    assert_eq!(identifier.rendering.prefix.as_deref(), Some("CSTR: "));
    assert_eq!(identifier.rendering.suffix.as_deref(), Some("."));
}
