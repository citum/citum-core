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

/// Integration tests for citum-bindings.
///
/// These tests verify the public API using minimal inline fixtures that match
/// the expected caller contract (clean JSON, no metadata keys).
use citum_bindings::{
    format_document, get_style_metadata, materialize_style, render_bibliography, render_citation,
    validate_style,
};
use citum_schema::Style;

const STYLE_YAML: &str = include_str!("../../../styles/embedded/apa-7th.yaml");
const CITATION_ONLY_STYLE_YAML: &str =
    include_str!("../../../styles/embedded/chicago-notes-18th.yaml");
const BIBLIOGRAPHY_ONLY_STYLE_YAML: &str = r#"version: "0.77.0"
info:
  title: Bibliography Only
bibliography:
  template:
    - title: primary
"#;

/// Minimal bibliography with one book reference (Citum-native JSON format).
const REFS_JSON: &str = r#"{
  "ITEM-1": {
    "id": "ITEM-1",
    "class": "monograph",
    "type": "book",
    "title": "The Structure of Scientific Revolutions",
    "author": [{"family": "Kuhn", "given": "Thomas S."}],
    "issued": "1962"
  }
}"#;

const CITATION_JSON: &str = r#"{"id":"c1","items":[{"id":"ITEM-1"}]}"#;

#[test]
fn render_citation_returns_string() {
    let result = render_citation(STYLE_YAML, REFS_JSON, CITATION_JSON, None);
    assert!(result.is_ok(), "render_citation failed: {result:?}");
    assert_ne!(result.unwrap(), "");
}

#[test]
fn render_bibliography_returns_string() {
    let result = render_bibliography(STYLE_YAML, REFS_JSON);
    assert!(result.is_ok(), "render_bibliography failed: {result:?}");
    assert_ne!(result.unwrap(), "");
}

#[test]
fn builtin_selector_returns_real_metadata_and_bibliography() {
    let metadata_json = get_style_metadata("id: apa-7th")
        .expect("the bundled APA selector should resolve to style metadata");
    assert_ne!(metadata_json, "{}", "issue #1282 returned empty metadata");
    let metadata: serde_json::Value =
        serde_json::from_str(&metadata_json).expect("metadata should be JSON");
    assert_eq!(
        metadata.get("title").and_then(serde_json::Value::as_str),
        Some("American Psychological Association 7th edition")
    );

    let bibliography = render_bibliography("id: apa-7th", REFS_JSON)
        .expect("the bundled APA selector should render a bibliography");
    assert_ne!(
        bibliography, "",
        "issue #1282 returned an empty bibliography"
    );
}

#[test]
fn mhra_selector_and_alias_return_exact_authored_surfaces() {
    let metadata = get_style_metadata("id: mhra")
        .expect("the bundled MHRA alias should resolve to style metadata");
    let metadata: serde_json::Value =
        serde_json::from_str(&metadata).expect("metadata should be JSON");
    assert_eq!(
        metadata.get("title").and_then(serde_json::Value::as_str),
        Some("MHRA Style Guide 4th edition (notes)")
    );
    assert_eq!(
        metadata
            .get("default-locale")
            .and_then(serde_json::Value::as_str),
        Some("en-GB")
    );

    let citation = render_citation("id: mhra-notes", REFS_JSON, CITATION_JSON, None)
        .expect("the bundled MHRA ID should render a citation");
    assert_eq!(
        citation,
        "<span class=\"citum-citation\" data-ref=\"ITEM-1\">Thomas S. Kuhn, <span class=\"citum-title\"><em>The Structure of Scientific Revolutions</em></span> (<span class=\"citum-issued\">1962</span>)</span>."
    );

    let bibliography_refs = REFS_JSON.replace(
        "\"issued\": \"1962\"",
        "\"issued\": \"1962\", \"publisher\": \"Example Press\"",
    );
    let bibliography = render_bibliography("id: mhra", &bibliography_refs)
        .expect("the bundled MHRA alias should render a bibliography");
    assert_eq!(
        bibliography,
        "<div class=\"citum-bibliography citum-bibliography--hanging-indent\">\n<div class=\"citum-entry\" id=\"ref-ITEM-1\" data-author=\"Kuhn\" data-year=\"1962\" data-title=\"The Structure of Scientific Revolutions\"><span class=\"citum-author\">Kuhn, Thomas S.</span>, <span class=\"citum-title\"><em>The Structure of Scientific Revolutions</em></span><span class=\"citum-publisher\"> (Example Press</span><span class=\"citum-issued\">, 1962)</span></div>\n</div>"
    );
}

#[test]
fn render_bibliography_without_a_resolved_spec_returns_an_error() {
    let error = render_bibliography(CITATION_ONLY_STYLE_YAML, REFS_JSON)
        .expect_err("citation-only styles must not silently return empty bibliographies");

    assert_eq!(
        error,
        "Style has no bibliography section; renderBibliography cannot produce output"
    );
}

#[test]
fn render_citation_without_a_resolved_spec_returns_an_error() {
    let error = render_citation(BIBLIOGRAPHY_ONLY_STYLE_YAML, REFS_JSON, CITATION_JSON, None)
        .expect_err("bibliography-only styles must not invent a citation");

    assert_eq!(
        error,
        "Style has no citation section; renderCitation cannot produce output"
    );
}

#[test]
fn materialize_style_preserves_an_absent_bibliography() {
    let materialized = materialize_style(CITATION_ONLY_STYLE_YAML)
        .expect("citation-only style should materialize");
    let style = Style::from_yaml_str(&materialized)
        .expect("materialized citation-only style should remain valid");

    assert!(style.bibliography.is_none());
}

#[test]
fn materialize_style_resolves_inheritance_and_template_references() {
    let materialized = materialize_style("id: chicago-author-date-18th")
        .expect("bundled dependent style should materialize");
    let style = Style::from_yaml_str(&materialized)
        .expect("materialized dependent style should remain valid");

    assert!(style.extends.is_none());
    assert!(style.extends_pin.is_none());
    let citation = style.citation.expect("citation should be inherited");
    assert!(citation.template.is_some());
    assert!(citation.template_ref.is_none());
    let bibliography = style
        .bibliography
        .expect("bibliography should be inherited");
    assert!(bibliography.template.is_some());
    assert!(bibliography.template_ref.is_none());
}

#[test]
fn format_document_accepts_a_bundled_style_alias() {
    let request = serde_json::json!({
        "style": { "kind": "id", "value": "apa" },
        "output_format": "html",
        "refs": serde_json::from_str::<serde_json::Value>(REFS_JSON).expect("refs fixture"),
        "citations": [{ "id": "c1", "items": [{ "id": "ITEM-1" }] }]
    });

    let result = format_document(&request.to_string()).expect("bundled ID should resolve");
    let result: serde_json::Value = serde_json::from_str(&result).expect("result should be JSON");
    assert_eq!(
        result["formatted_citations"].as_array().map(Vec::len),
        Some(1)
    );
    assert!(
        result["bibliography"]["content"]
            .as_str()
            .is_some_and(|content| !content.is_empty())
    );
}

#[test]
fn validate_style_accepts_valid_style() {
    assert!(validate_style(STYLE_YAML).is_ok());
    assert!(validate_style("id: apa").is_ok());
}

#[test]
fn builtin_selector_rejects_unknown_remote_and_mixed_ids() {
    assert_eq!(
        validate_style("id: not-a-builtin").expect_err("unknown IDs must fail"),
        "Style ID 'not-a-builtin' is not bundled with @citum/engine; fetch version-pinned style YAML and pass the YAML content instead"
    );
    assert_eq!(
        validate_style("id: acm-sig-proceedings")
            .expect_err("remote-only catalog IDs must fail in the binding"),
        "Style ID 'acm-sig-proceedings' is not bundled with @citum/engine; fetch version-pinned style YAML and pass the YAML content instead"
    );
    assert_eq!(
        validate_style("id: apa-7th\ninfo:\n  title: Wrong level")
            .expect_err("a selector cannot also be a style document"),
        "Invalid style selector: top-level `id` must be the only field; use `info.id` for style metadata"
    );
}

#[test]
fn direct_render_errors_and_structured_format_warns_for_missing_locale() {
    let style = STYLE_YAML.replace(
        "  title: American Psychological Association 7th edition\n",
        "  title: American Psychological Association 7th edition\n  default-locale: cy-GB\n",
    );
    let error = render_citation(&style, REFS_JSON, CITATION_JSON, None)
        .expect_err("direct string rendering cannot return locale warnings");
    assert_eq!(
        error,
        "Locale resolution error: Requested locale 'cy-GB' is not bundled; falling back to en-US"
    );

    let request = serde_json::json!({
        "style": { "kind": "yaml", "value": style },
        "output_format": "html",
        "refs": serde_json::from_str::<serde_json::Value>(REFS_JSON).expect("refs fixture"),
        "citations": [{ "id": "c1", "items": [{ "id": "ITEM-1" }] }]
    });
    let result =
        format_document(&request.to_string()).expect("structured formatting should fall back");
    let result: serde_json::Value = serde_json::from_str(&result).expect("result should be JSON");
    assert_eq!(result["warnings"][0]["code"], "locale_fallback");
    assert_eq!(
        result["warnings"][0]["message"],
        "Requested locale 'cy-GB' is not bundled; falling back to en-US"
    );
}

#[test]
fn format_document_rejects_uri_and_path_inputs_in_wasm_contract() {
    for (style, expected) in [
        (
            serde_json::json!({ "kind": "uri", "value": "https://example.test/style.yaml" }),
            "Style URI 'https://example.test/style.yaml' cannot be fetched by @citum/engine; fetch version-pinned YAML and pass it with kind 'yaml'",
        ),
        (
            serde_json::json!({ "kind": "path", "value": "/tmp/style.yaml" }),
            "Style path '/tmp/style.yaml' is unavailable in WASM; read the file and pass its contents with kind 'yaml'",
        ),
    ] {
        let request = serde_json::json!({
            "style": style,
            "refs": {},
            "citations": []
        });
        assert_eq!(
            format_document(&request.to_string()).expect_err("unsupported style input must fail"),
            expected
        );
    }
}

#[test]
fn validate_style_rejects_invalid_yaml() {
    assert!(validate_style("not: valid: yaml: [[[").is_err());
}

#[test]
fn render_citation_bad_style_returns_error() {
    let result = render_citation("not yaml", REFS_JSON, CITATION_JSON, None);
    assert!(result.is_err());
}

#[test]
fn render_bibliography_bad_refs_returns_error() {
    let result = render_bibliography(STYLE_YAML, "not json");
    assert!(result.is_err());
}
