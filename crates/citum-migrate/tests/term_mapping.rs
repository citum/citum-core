use citum_migrate::{TemplateCompiler, Upsampler};
use citum_schema::CslnNode;
use citum_schema::locale::{GeneralTerm, TermForm};
use citum_schema::template::{NumberVariable, SimpleVariable, TemplateComponent};
use csl_legacy::model::{CslNode, Formatting, Label, Names, Text};

#[test]
fn test_upsample_term() {
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
    let csln_nodes = upsampler.upsample_nodes(&[legacy_node]);

    assert_eq!(csln_nodes.len(), 1);
    match &csln_nodes[0] {
        CslnNode::Term(term_block) => {
            assert_eq!(term_block.term, GeneralTerm::In);
            assert_eq!(term_block.form, TermForm::Long);
        }
        _ => panic!("Expected CslnNode::Term, got {:?}", csln_nodes[0]),
    }
}

#[test]
fn test_upsample_term_with_form() {
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
    let csln_nodes = upsampler.upsample_nodes(&[legacy_node]);

    assert_eq!(csln_nodes.len(), 1);
    match &csln_nodes[0] {
        CslnNode::Text { value } => {
            assert_eq!(value, "editor");
        }
        _ => panic!(
            "Expected CslnNode::Text for unknown term, got {:?}",
            csln_nodes[0]
        ),
    }
}

#[test]
fn test_upsample_term_preserves_strip_periods() {
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
    let csln_nodes = upsampler.upsample_nodes(&[legacy_node]);

    assert_eq!(csln_nodes.len(), 1);
    match &csln_nodes[0] {
        CslnNode::Term(term_block) => {
            assert_eq!(term_block.formatting.strip_periods, Some(true));
        }
        _ => panic!("Expected CslnNode::Term, got {:?}", csln_nodes[0]),
    }
}

#[test]
fn test_upsample_label_preserves_strip_periods() {
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
    let csln_nodes = upsampler.upsample_nodes(&[legacy_node]);

    assert_eq!(csln_nodes.len(), 1);
    match &csln_nodes[0] {
        CslnNode::Variable(variable) => {
            let label = variable
                .label
                .as_ref()
                .expect("label metadata should exist");
            assert_eq!(label.formatting.strip_periods, Some(true));
        }
        _ => panic!("Expected CslnNode::Variable, got {:?}", csln_nodes[0]),
    }
}

#[test]
fn test_upsample_strip_periods_defaults_when_absent_or_false() {
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
        CslnNode::Term(term_block) => assert_eq!(term_block.formatting.strip_periods, None),
        _ => panic!("Expected CslnNode::Term, got {:?}", term_nodes[0]),
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
        CslnNode::Variable(variable) => {
            let label = variable
                .label
                .as_ref()
                .expect("label metadata should exist");
            assert_eq!(label.formatting.strip_periods, Some(false));
        }
        _ => panic!("Expected CslnNode::Variable, got {:?}", label_nodes[0]),
    }
}

#[test]
fn test_template_compiler_preserves_strip_periods_through_compilation() {
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
    let locator = locator
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
    assert_eq!(locator.show_label, Some(true));
    assert_eq!(locator.strip_label_periods, Some(true));
}
