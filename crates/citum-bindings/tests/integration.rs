#![allow(missing_docs, reason = "test")]

/// Integration tests for citum-bindings.
///
/// These tests verify the public API using minimal inline fixtures that match
/// the expected caller contract (clean JSON, no metadata keys).
use citum_bindings::{render_bibliography, render_citation, validate_style};

const STYLE_YAML: &str = include_str!("../../../styles/embedded/apa-7th.yaml");

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
    assert!(!result.unwrap().is_empty());
}

#[test]
fn render_bibliography_returns_string() {
    let result = render_bibliography(STYLE_YAML, REFS_JSON);
    assert!(result.is_ok(), "render_bibliography failed: {result:?}");
    assert!(!result.unwrap().is_empty());
}

#[test]
fn validate_style_accepts_valid_style() {
    assert!(validate_style(STYLE_YAML).is_ok());
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
