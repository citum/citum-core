//! BDD behavioral integration tests for bibliography I/O operations.
//!
//! Tests coverage for `load_bibliography_with_sets` across multiple input formats:
//! CSL-JSON arrays, Citum YAML, wrapped legacy JSON, and IndexMap structures.
//! Validates reference parsing, compound set preservation, and error handling
//! for invalid memberships and duplicates.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "Panicking is acceptable and often desired in tests."
)]

mod common;
use common::announce_behavior;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use citum_engine::io::load_bibliography_with_sets;
use rstest::rstest;

/// Generate a temp file path with unique suffix.
fn temp_path(stem: &str, ext: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{stem}-{now}.{ext}"))
}

/// **Given** a CSL-JSON array file,
/// **When** loaded with `load_bibliography_with_sets`,
/// **Then** it parses references and returns none for sets.
#[rstest]
#[case(
    r#"[{"id": "smith-2020", "type": "book", "title": "Test Book"}]"#,
    "csl_json_array",
    true,
    false
)]
/// **Given** a Citum YAML file with references,
/// **When** loaded with `load_bibliography_with_sets`,
/// **Then** it parses references and returns none for sets.
#[case(
    r#"references:
  - class: monograph
    id: yaml-ref-1
    type: book
    title: YAML Book
    issued: "2021"
"#,
    "citum_yaml",
    true,
    false
)]
/// **Given** a wrapped legacy JSON with references and null sets,
/// **When** loaded with `load_bibliography_with_sets`,
/// **Then** it parses references and returns none for sets.
#[case(
    r#"{"references": [{"id": "legacy-1", "type": "book", "title": "Legacy Book"}], "sets": null}"#,
    "wrapped_legacy",
    true,
    false
)]
/// **Given** an IndexMap YAML format with keyed references,
/// **When** loaded with `load_bibliography_with_sets`,
/// **Then** it parses all references by key.
#[case(
    r#"ref-yaml-1:
  id: ref-yaml-1
  type: book
  title: First Book
ref-yaml-2:
  id: ref-yaml-2
  type: journal-article
  title: Second Article
"#,
    "indexmap_format",
    true,
    false
)]
fn given_bibliography_file_when_loaded_then_refs_parsed(
    #[case] content: &str,
    #[case] format_name: &str,
    #[case] expect_refs: bool,
    #[case] expect_sets: bool,
) {
    announce_behavior(&format!(
        "Load bibliography file in {format_name} format and parse references"
    ));

    // Determine extension from content
    let ext = if content.starts_with('[') || content.starts_with('{') {
        "json"
    } else {
        "yaml"
    };

    let temp = temp_path("citum-bdd-io", ext);

    // Write temp file
    fs::write(&temp, content).expect("temp file should write");

    // Load bibliography
    let loaded =
        load_bibliography_with_sets(&temp).expect("bibliography should load without error");

    // Verify references were parsed
    if expect_refs {
        assert!(
            !loaded.references.is_empty(),
            "expected at least one reference in {format_name} format"
        );
    }

    // Verify sets presence
    if expect_sets {
        assert!(
            loaded.sets.is_some(),
            "expected sets to be Some in {format_name} format"
        );
    } else {
        assert!(
            loaded.sets.is_none(),
            "expected sets to be None in {format_name} format"
        );
    }

    // Cleanup
    let _ = fs::remove_file(temp);
}

/// **Given** a Citum YAML file with references and compound sets,
/// **When** loaded with `load_bibliography_with_sets`,
/// **Then** it returns sets containing the expected group key and membership.
#[test]
fn given_citum_yaml_with_sets_when_loaded_then_sets_preserved() {
    announce_behavior("Load Citum YAML with compound sets and preserve set membership");

    let yaml = r#"references:
  - class: monograph
    id: ref-a
    type: book
    title: Book A
    issued: "2020"
  - class: monograph
    id: ref-b
    type: book
    title: Book B
    issued: "2021"
sets:
  citation-group: [ref-a, ref-b]
"#;

    let temp = temp_path("citum-bdd-io-sets", "yaml");
    fs::write(&temp, yaml).expect("temp file should write");

    let loaded = load_bibliography_with_sets(&temp).expect("bibliography with sets should load");

    // Verify references
    assert_eq!(loaded.references.len(), 2, "should load 2 references");
    assert!(
        loaded.references.contains_key("ref-a"),
        "ref-a should exist"
    );
    assert!(
        loaded.references.contains_key("ref-b"),
        "ref-b should exist"
    );

    // Verify sets
    let sets = loaded.sets.expect("sets should be present");
    assert_eq!(sets.len(), 1, "should have 1 set");
    assert_eq!(
        sets.get("citation-group"),
        Some(&vec!["ref-a".to_string(), "ref-b".to_string()]),
        "citation-group should contain both refs in order"
    );

    // Cleanup
    let _ = fs::remove_file(temp);
}

/// **Given** a JSON object with Citum `InputBibliography` structure and sets,
/// **When** loaded with `load_bibliography_with_sets`,
/// **Then** it parses references and sets, validating membership against loaded refs.
#[test]
fn given_citum_json_with_sets_when_loaded_then_sets_validated() {
    announce_behavior(
        "Load Citum JSON with sets and validate membership against loaded references",
    );

    let json = r#"{
  "references": [
    {
      "class": "monograph",
      "id": "book-1",
      "type": "book",
      "title": "First Book",
      "issued": "2022"
    },
    {
      "class": "monograph",
      "id": "book-2",
      "type": "book",
      "title": "Second Book",
      "issued": "2023"
    }
  ],
  "sets": {
    "group-x": ["book-1"],
    "group-y": ["book-2"]
  }
}"#;

    let temp = temp_path("citum-bdd-io-json-sets", "json");
    fs::write(&temp, json).expect("temp file should write");

    let loaded =
        load_bibliography_with_sets(&temp).expect("JSON bibliography with sets should load");

    // Verify references
    assert_eq!(loaded.references.len(), 2, "should load 2 references");
    assert!(loaded.references.contains_key("book-1"));
    assert!(loaded.references.contains_key("book-2"));

    // Verify sets
    let sets = loaded.sets.expect("sets should be present");
    assert_eq!(sets.len(), 2, "should have 2 sets");
    assert_eq!(sets.get("group-x"), Some(&vec!["book-1".to_string()]));
    assert_eq!(sets.get("group-y"), Some(&vec!["book-2".to_string()]));

    // Cleanup
    let _ = fs::remove_file(temp);
}

/// **Given** a JSON file with references and invalid set membership (unknown ref ID),
/// **When** loaded with `load_bibliography_with_sets`,
/// **Then** it returns a parse error describing the unknown reference.
#[test]
fn given_invalid_set_membership_when_loaded_then_error_returned() {
    announce_behavior("Reject bibliography with set member not in references");

    let json = r#"{
  "references": [
    {
      "class": "monograph",
      "id": "ref-exists",
      "type": "book",
      "title": "Existing Book",
      "issued": "2020"
    }
  ],
  "sets": {
    "bad-group": ["ref-missing"]
  }
}"#;

    let temp = temp_path("citum-bdd-io-invalid-set", "json");
    fs::write(&temp, json).expect("temp file should write");

    let err = load_bibliography_with_sets(&temp).expect_err("should reject invalid set membership");

    let msg = err.to_string();
    assert!(
        msg.contains("ref-missing") && msg.contains("unknown"),
        "error should mention unknown member: {msg}"
    );

    // Cleanup
    let _ = fs::remove_file(temp);
}

/// **Given** a YAML file with references appearing in multiple sets,
/// **When** loaded with `load_bibliography_with_sets`,
/// **Then** it returns a parse error rejecting cross-set duplication.
#[test]
fn given_duplicate_set_membership_when_loaded_then_error_returned() {
    announce_behavior("Reject reference appearing in multiple compound sets");

    let yaml = r#"references:
  - class: monograph
    id: shared-ref
    type: book
    title: Shared Reference
    issued: "2020"
sets:
  group-1: [shared-ref]
  group-2: [shared-ref]
"#;

    let temp = temp_path("citum-bdd-io-dup-set", "yaml");
    fs::write(&temp, yaml).expect("temp file should write");

    let err = load_bibliography_with_sets(&temp).expect_err("should reject duplicate membership");

    let msg = err.to_string();
    assert!(
        msg.contains("shared-ref") && msg.contains("both compound sets"),
        "error should mention duplicate across sets: {msg}"
    );

    // Cleanup
    let _ = fs::remove_file(temp);
}

/// **Given** a YAML file with the same reference ID appearing twice in one set,
/// **When** loaded with `load_bibliography_with_sets`,
/// **Then** it returns a parse error rejecting within-set duplication.
#[test]
fn given_duplicate_within_set_when_loaded_then_error_returned() {
    announce_behavior("Reject reference appearing more than once within a single set");

    let yaml = r#"references:
  - class: monograph
    id: ref-id
    type: book
    title: Reference
    issued: "2020"
sets:
  group: [ref-id, ref-id]
"#;

    let temp = temp_path("citum-bdd-io-within-dup", "yaml");
    fs::write(&temp, yaml).expect("temp file should write");

    let err = load_bibliography_with_sets(&temp).expect_err("should reject within-set duplication");

    let msg = err.to_string();
    assert!(
        msg.contains("ref-id") && msg.contains("more than once"),
        "error should mention duplicate within set: {msg}"
    );

    // Cleanup
    let _ = fs::remove_file(temp);
}
