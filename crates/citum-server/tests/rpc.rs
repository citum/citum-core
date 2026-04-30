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

//! Integration tests for the JSON-RPC dispatcher.
//!
//! Uses a real Citum style (apa-7th.yaml) and minimal inline reference data
//! to exercise all three methods without touching stdin/stdout.

use citum_server::rpc::{RpcRequest, dispatch};
use serde_json::json;

/// Absolute path to the APA style.
/// `CARGO_MANIFEST_DIR` is the crate root; workspace root is two levels up.
fn apa_style_path() -> String {
    format!(
        "{}/../../styles/embedded/apa-7th.yaml",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Minimal bibliography: one book (Hawking 1988) in native Citum schema format.
/// `issued` is a plain EDTF string; `author` is a `ContributorList`.
fn hawking_refs() -> serde_json::Value {
    json!({
        "ITEM-2": {
            "id": "ITEM-2",
            "class": "monograph",
            "type": "book",
            "title": "A Brief History of Time",
            "author": [{"family": "Hawking", "given": "Stephen"}],
            "issued": "1988"
        }
    })
}

fn make_request(id: u32, method: &str, params: serde_json::Value) -> RpcRequest {
    serde_json::from_value(json!({
        "id": id,
        "method": method,
        "params": params
    }))
    .unwrap()
}

// --- validate_style ---

#[test]
fn validate_style_valid() {
    let req = make_request(
        1,
        "validate_style",
        json!({ "style_path": apa_style_path() }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    assert_eq!(result["id"], 1);
    assert_eq!(result["result"]["valid"], true);
    assert!(result["result"]["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn validate_style_missing_file() {
    let req = make_request(
        2,
        "validate_style",
        json!({ "style_path": "styles/does-not-exist.yaml" }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    assert_eq!(result["id"], 2);
    assert_eq!(result["result"]["valid"], false);
    assert!(!result["result"]["warnings"].as_array().unwrap().is_empty());
}

// --- render_bibliography ---

#[test]
fn render_bibliography_returns_entries() {
    let req = make_request(
        3,
        "render_bibliography",
        json!({
            "style_path": apa_style_path(),
            "refs": hawking_refs()
        }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    assert_eq!(result["id"], 3);
    assert_eq!(result["result"]["format"], "plain");
    let entries = result["result"]["entries"]
        .as_array()
        .expect("entries should be array");
    assert!(
        !entries.is_empty(),
        "expected at least one bibliography entry"
    );
    let entry = entries[0].as_str().unwrap();
    assert_eq!(entry, "Hawking, S. (1988). _A Brief History of Time_");
    assert_eq!(
        result["result"]["content"].as_str().unwrap(),
        "Hawking, S. (1988). _A Brief History of Time_"
    );
}

#[test]
fn render_bibliography_html_returns_wrapped_markup() {
    let req = make_request(
        8,
        "render_bibliography",
        json!({
            "style_path": apa_style_path(),
            "refs": hawking_refs(),
            "output_format": "html"
        }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    assert_eq!(result["id"], 8);
    assert_eq!(result["result"]["format"], "html");
    assert!(result["result"]["entries"].is_null());
    let content = result["result"]["content"]
        .as_str()
        .expect("content should be a string");
    assert!(
        content.contains("citum-bibliography"),
        "html bibliography should include wrapper markup"
    );
}

#[test]
fn render_bibliography_html_injects_template_indices_when_requested() {
    let req = make_request(
        13,
        "render_bibliography",
        json!({
            "style_path": apa_style_path(),
            "refs": hawking_refs(),
            "output_format": "html",
            "inject_ast_indices": true
        }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    let content = result["result"]["content"]
        .as_str()
        .expect("content should be a string");
    assert!(
        content.contains(r#"data-index="0""#),
        "html bibliography should include template indices when requested: {content}"
    );
}

// --- render_citation ---

#[test]
fn render_citation_returns_string() {
    let req = make_request(
        4,
        "render_citation",
        json!({
            "style_path": apa_style_path(),
            "refs": hawking_refs(),
            "citation": {
                "id": "cite-1",
                "items": [{"id": "ITEM-2"}]
            }
        }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    assert_eq!(result["id"], 4);
    let citation = result["result"].as_str().expect("result should be string");
    assert!(
        citation.contains("Hawking") || citation.contains("1988"),
        "citation should reference the work: {citation}"
    );
}

#[test]
fn render_citation_html_returns_markup() {
    let req = make_request(
        9,
        "render_citation",
        json!({
            "style_path": apa_style_path(),
            "refs": hawking_refs(),
            "output_format": "html",
            "citation": {
                "id": "cite-1",
                "items": [{"id": "ITEM-2"}]
            }
        }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    assert_eq!(result["id"], 9);
    let citation = result["result"].as_str().expect("result should be string");
    assert!(
        citation.contains("citum-citation"),
        "html citation should contain citation wrapper: {citation}"
    );
}

#[test]
fn render_citation_html_injects_template_indices_when_requested() {
    let req = make_request(
        14,
        "render_citation",
        json!({
            "style_path": apa_style_path(),
            "refs": hawking_refs(),
            "output_format": "html",
            "inject_ast_indices": true,
            "citation": {
                "id": "cite-1",
                "items": [{"id": "ITEM-2"}]
            }
        }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    let citation = result["result"].as_str().expect("result should be string");
    assert!(
        citation.contains(r#"class="citum-issued" data-index="0""#),
        "html citation should annotate the rendered citation component when requested: {citation}"
    );
}

#[test]
fn render_citation_typst_returns_internal_link_markup() {
    let req = make_request(
        11,
        "render_citation",
        json!({
            "style_path": apa_style_path(),
            "refs": hawking_refs(),
            "output_format": "typst",
            "citation": {
                "id": "cite-1",
                "items": [{"id": "ITEM-2"}]
            }
        }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    assert_eq!(result["id"], 11);
    let citation = result["result"].as_str().expect("result should be string");
    assert!(
        citation.contains("#link(<ref-ITEM-2>)"),
        "typst citation should contain an internal link: {citation}"
    );
}

// --- error handling ---

#[test]
fn unknown_method_returns_error() {
    let req = make_request(5, "frobnicate", json!({}));
    let err = dispatch(req).expect_err("should error");
    assert_eq!(err.1, "unknown method: frobnicate");
}

#[test]
fn missing_style_path_returns_error() {
    let req = make_request(6, "render_bibliography", json!({ "refs": hawking_refs() }));
    let err = dispatch(req).expect_err("should error");
    assert_eq!(err.1, "missing required field: style_path");
}

#[test]
fn missing_refs_returns_error() {
    let req = make_request(
        7,
        "render_bibliography",
        json!({ "style_path": apa_style_path() }),
    );
    let err = dispatch(req).expect_err("should error");
    assert_eq!(err.1, "missing required field: refs");
}

#[test]
fn invalid_output_format_returns_error() {
    let req = make_request(
        12,
        "render_bibliography",
        json!({
            "style_path": apa_style_path(),
            "refs": hawking_refs(),
            "output_format": "pdf"
        }),
    );
    let err = dispatch(req).expect_err("invalid output format should error");
    assert_eq!(err.0, Some(json!(12)));
    assert!(err.1.contains("unsupported output format"));
    assert!(err.1.contains("pdf"));
}

#[test]
fn render_bibliography_typst_returns_labeled_markup() {
    let req = make_request(
        10,
        "render_bibliography",
        json!({
            "style_path": apa_style_path(),
            "refs": hawking_refs(),
            "output_format": "typst"
        }),
    );
    let result = dispatch(req).expect("dispatch should succeed");
    assert_eq!(result["id"], 10);
    assert_eq!(result["result"]["format"], "typst");
    assert!(result["result"]["entries"].is_null());
    let content = result["result"]["content"]
        .as_str()
        .expect("content should be a string");
    assert_eq!(
        content,
        "Hawking, S. (1988). _A Brief History of Time_ <ref-ITEM-2>"
    );
}
