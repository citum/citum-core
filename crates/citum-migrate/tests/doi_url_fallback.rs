/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#![allow(missing_docs, reason = "test")]
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "Panicking is acceptable and often desired in tests."
)]

use citum_migrate::{Compressor, MacroInliner, TemplateCompiler, Upsampler};
use citum_schema::template::{SimpleVariable, TemplateComponent, TemplateGroupSelect};
use csl_legacy::parser::parse_style;
use roxmltree::Document;

const DOI_URL_FALLBACK_XML: &str = r#"
<style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="in-text">
  <info>
    <title>DOI URL fallback test</title>
    <id>https://example.org/doi-url-fallback-test</id>
  </info>
  <macro name="access">
    <choose>
      <if variable="DOI">
        <text variable="DOI" prefix="https://doi.org/"/>
      </if>
      <else-if variable="URL">
        <text variable="URL" prefix="Available: "/>
        <date variable="accessed">
          <date-part name="year"/>
        </date>
      </else-if>
    </choose>
  </macro>
  <citation>
    <layout><text macro="access"/></layout>
  </citation>
  <bibliography>
    <layout><text macro="access"/></layout>
  </bibliography>
</style>
"#;

fn parse_fixture() -> csl_legacy::model::Style {
    let document = Document::parse(DOI_URL_FALLBACK_XML).expect("fixture XML should parse");
    parse_style(document.root_element()).expect("fixture CSL should parse")
}

fn assert_ordered_fallback(template: &[TemplateComponent]) {
    let [TemplateComponent::Group(fallback)] = template else {
        panic!("expected one ordered fallback group, got {template:#?}");
    };
    assert_eq!(fallback.select, TemplateGroupSelect::First);
    let [
        TemplateComponent::Variable(doi),
        TemplateComponent::Group(url_branch),
    ] = fallback.group.as_slice()
    else {
        panic!("expected DOI followed by one atomic URL candidate");
    };
    assert_eq!(doi.variable, SimpleVariable::Doi);
    assert_eq!(url_branch.select, TemplateGroupSelect::All);
    assert_eq!(url_branch.group.len(), 2);
    assert!(matches!(
        url_branch.group.first(),
        Some(TemplateComponent::Variable(variable)) if variable.variable == SimpleVariable::Url
    ));
}

#[test]
fn bibliography_compiles_doi_then_url_as_one_ordered_fallback() {
    let style = parse_fixture();
    let inliner = MacroInliner::new(&style);
    let nodes = inliner
        .inline_bibliography(&style)
        .expect("fixture should have a bibliography");
    let ir = Compressor.compress_nodes(Upsampler::new().upsample_nodes(&nodes));
    let (template, _) = TemplateCompiler.compile_bibliography_with_types(&ir, true);

    assert_ordered_fallback(&template);
}

#[test]
fn citation_compiles_doi_then_url_as_one_ordered_fallback() {
    let style = parse_fixture();
    let inliner = MacroInliner::new(&style);
    let nodes = inliner.inline_citation(&style);
    let ir = Compressor.compress_nodes(Upsampler::new().upsample_nodes(&nodes));
    let template = TemplateCompiler.compile_citation(&ir);

    assert_ordered_fallback(&template);
}
