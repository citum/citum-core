//! Tests for the resolver architecture.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]

use citum_store::resolver::{
    ChainResolver, EmbeddedResolver, FileResolver, ResolverError, StyleResolver,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_file_resolver_uri_handling() {
    let temp = TempDir::new().unwrap();
    let style_path = temp.path().join("test.yaml");
    fs::write(&style_path, "info:\n  title: Test Style\n").unwrap();

    let resolver = FileResolver;

    // Test with absolute file path
    let uri = style_path.to_str().unwrap();
    let style = resolver.resolve_style(uri).unwrap();
    assert_eq!(style.info.title.as_deref(), Some("Test Style"));
}

#[test]
fn test_file_resolver_locale_resolution() {
    // FileResolver checks locales/{id}.{ext} relative to CWD.
    // We only verify the NotFound path here to avoid polluting the workspace.
    let resolver = FileResolver;
    match resolver.resolve_locale("nonexistent-test-locale-1234") {
        Err(ResolverError::LocaleNotFound(_)) => (),
        _ => panic!("Expected LocaleNotFound"),
    }
}

#[test]
fn test_chain_resolver_fallback() {
    let temp = TempDir::new().unwrap();
    let style_path = temp.path().join("test.yaml");
    fs::write(&style_path, "info:\n  title: Test Style\n").unwrap();

    let file_resolver = Box::new(FileResolver);
    let embedded_resolver = Box::new(EmbeddedResolver);

    let chain = ChainResolver::new(vec![file_resolver, embedded_resolver]);

    // Should find test.yaml
    let style_uri = style_path.to_str().unwrap();
    let style = chain.resolve_style(style_uri).unwrap();
    assert_eq!(style.info.title.as_deref(), Some("Test Style"));

    // Should find embedded style
    let style = chain.resolve_style("apa").unwrap();
    assert_eq!(
        style.info.title.as_deref(),
        Some("American Psychological Association 7th edition")
    );

    // Should return NotFound for missing style
    match chain.resolve_style("nonexistent") {
        Err(ResolverError::StyleNotFound(_)) => (),
        _ => panic!("Expected StyleNotFound error"),
    }
}

#[test]
fn test_chain_resolver_error_propagation() {
    let temp = TempDir::new().unwrap();
    let style_path = temp.path().join("invalid.yaml");
    fs::write(&style_path, "invalid yaml content: [").unwrap();

    let file_resolver = Box::new(FileResolver);
    let embedded_resolver = Box::new(EmbeddedResolver);

    let chain = ChainResolver::new(vec![file_resolver, embedded_resolver]);

    // Should propagate YamlError rather than falling back to next resolver
    let style_uri = style_path.to_str().unwrap();
    match chain.resolve_style(style_uri) {
        Err(ResolverError::YamlError(_)) => (),
        err => panic!("Expected YamlError, got {:?}", err),
    }
}
