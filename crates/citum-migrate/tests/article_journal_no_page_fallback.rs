#![allow(missing_docs)]

use citum_migrate::{Compressor, MacroInliner, TemplateCompiler, Upsampler};
use citum_schema::template::{
    DateVariable, NumberVariable, SimpleVariable, TemplateComponent, TypeSelector,
};
use csl_legacy::parser::parse_style;
use roxmltree::Document;

fn parse_csl(xml: &str) -> Result<csl_legacy::model::Style, String> {
    let doc = Document::parse(xml).map_err(|err| err.to_string())?;
    parse_style(doc.root_element()).map_err(|err| err.to_string())
}

#[test]
fn migration_preserves_article_journal_detail_and_doi_components_for_exact_page_fallback_patterns()
{
    let xml = r#"<style>
        <citation><layout><text variable="title"/></layout></citation>
        <bibliography>
            <layout>
                <choose>
                    <if type="article-journal">
                        <group delimiter=", ">
                            <text variable="container-title"/>
                            <choose>
                                <if variable="page">
                                    <date variable="issued"><date-part name="year"/></date>
                                    <text variable="volume"/>
                                    <text variable="page"/>
                                </if>
                                <else>
                                    <text variable="DOI" prefix="DOI:"/>
                                </else>
                            </choose>
                        </group>
                    </if>
                </choose>
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
    let (_, type_templates) = compiler.compile_bibliography_with_types(&csln_bib, false);

    let template = type_templates
        .get(&TypeSelector::Single("article-journal".to_string()))
        .expect("article-journal type template should be preserved");

    assert!(template.iter().any(|component| matches!(
        component,
        TemplateComponent::Date(date) if date.date == DateVariable::Issued
    )));
    assert!(template.iter().any(|component| matches!(
        component,
        TemplateComponent::Number(number) if number.number == NumberVariable::Volume
    )));
    assert!(template.iter().any(|component| matches!(
        component,
        TemplateComponent::Number(number) if number.number == NumberVariable::Pages
    )));
    assert!(template.iter().any(|component| matches!(
        component,
        TemplateComponent::Variable(variable) if variable.variable == SimpleVariable::Doi
    )));
}
