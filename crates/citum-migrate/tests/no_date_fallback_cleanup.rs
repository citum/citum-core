#![allow(missing_docs, reason = "test")]

use citum_migrate::{Compressor, MacroInliner, TemplateCompiler, Upsampler};
use citum_schema::{
    locale::GeneralTerm,
    template::{DateVariable, TemplateComponent},
};
use csl_legacy::parser::parse_style;
use roxmltree::Document;

fn parse_csl(xml: &str) -> Result<csl_legacy::model::Style, String> {
    let doc = Document::parse(xml).map_err(|err| err.to_string())?;
    parse_style(doc.root_element()).map_err(|err| err.clone())
}

#[test]
fn migration_drops_explicit_no_date_terms_when_issued_is_already_present() {
    let xml = r#"<style>
        <citation><layout><text variable="title"/></layout></citation>
        <bibliography>
            <layout>
                <group prefix="(" suffix=")">
                    <choose>
                        <if variable="issued">
                            <date variable="issued">
                                <date-part name="year"/>
                            </date>
                        </if>
                        <else>
                            <text term="no date" form="short"/>
                        </else>
                    </choose>
                </group>
            </layout>
        </bibliography>
    </style>"#;
    let style = parse_csl(xml).expect("legacy style should parse");

    let inliner = MacroInliner::new(&style);
    let flattened = inliner
        .inline_bibliography(&style)
        .expect("bibliography should exist");
    let raw_bib = Upsampler::new().upsample_nodes(&flattened);
    let compressor = Compressor;
    let csln_bib = compressor.compress_nodes(raw_bib);
    let compiler = TemplateCompiler;
    let template = compiler.compile_bibliography(&csln_bib, false);

    assert!(template.iter().any(component_contains_issued_date));
    assert!(!template.iter().any(component_contains_no_date_term));
}

fn component_contains_issued_date(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Date(date) => date.date == DateVariable::Issued,
        TemplateComponent::Group(group) => group.group.iter().any(component_contains_issued_date),
        _ => false,
    }
}

fn component_contains_no_date_term(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Term(term) => term.term == GeneralTerm::NoDate,
        TemplateComponent::Group(group) => group.group.iter().any(component_contains_no_date_term),
        _ => false,
    }
}
