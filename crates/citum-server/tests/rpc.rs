/*
SPDX-License-Identifier: MPL-2.0
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
    format!("{}/../../styles/apa-7th.yaml", env!("CARGO_MANIFEST_DIR"))
}

/// Minimal bibliography: one book (Hawking 1988) in native Citum schema format.
/// `issued` is a plain EDTF string; `author` is a ContributorList.
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
    let entries = result["result"].as_array().expect("result should be array");
    assert!(
        !entries.is_empty(),
        "expected at least one bibliography entry"
    );
    let entry = entries[0].as_str().unwrap();
    assert!(
        entry.contains("Hawking"),
        "entry should contain author name"
    );
    assert!(entry.contains("1988"), "entry should contain year");
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

// --- error handling ---

#[test]
fn unknown_method_returns_error() {
    let req = make_request(5, "frobnicate", json!({}));
    let err = dispatch(req).expect_err("unknown method should error");
    assert!(err.1.contains("unknown method"));
}

#[test]
fn missing_style_path_returns_error() {
    let req = make_request(6, "render_bibliography", json!({ "refs": hawking_refs() }));
    let err = dispatch(req).expect_err("missing style_path should error");
    assert!(err.1.contains("style_path"));
}

#[test]
fn missing_refs_returns_error() {
    let req = make_request(
        7,
        "render_bibliography",
        json!({ "style_path": apa_style_path() }),
    );
    let err = dispatch(req).expect_err("missing refs should error");
    assert!(err.1.contains("refs"));
}
