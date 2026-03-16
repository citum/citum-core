//! Integration tests for `StoreResolver`.

use crate::{StoreFormat, StoreResolver};
use std::fs;
use tempfile::TempDir;

/// Returns a minimal valid Citum style as YAML bytes.
fn minimal_style_yaml() -> &'static [u8] {
    include_bytes!("../../../styles/alpha.yaml")
}

/// Creates a temporary store directory and a resolver pointing at it.
fn make_resolver(format: StoreFormat) -> (TempDir, StoreResolver) {
    let dir = TempDir::new().expect("tempdir");
    let resolver = StoreResolver::new(dir.path().to_path_buf(), format);
    (dir, resolver)
}

#[test]
fn list_styles_empty_on_fresh_store() {
    let (_dir, resolver) = make_resolver(StoreFormat::Yaml);
    let styles = resolver.list_styles().expect("list_styles");
    assert!(styles.is_empty());
}

#[test]
fn install_and_list_yaml_style() {
    let (dir, resolver) = make_resolver(StoreFormat::Yaml);

    // Write the fixture to a temp file
    let src = dir.path().join("alpha.yaml");
    fs::write(&src, minimal_style_yaml()).unwrap();

    let name = resolver.install_style(&src).expect("install_style");
    assert_eq!(name, "alpha");

    let styles = resolver.list_styles().expect("list_styles");
    assert_eq!(styles, vec!["alpha"]);
}

#[test]
fn resolve_installed_yaml_style() {
    let (dir, resolver) = make_resolver(StoreFormat::Yaml);

    let src = dir.path().join("alpha.yaml");
    fs::write(&src, minimal_style_yaml()).unwrap();
    resolver.install_style(&src).expect("install_style");

    let style = resolver.resolve_style("alpha").expect("resolve_style");
    assert_eq!(style.info.title.as_deref(), Some("Alpha (biblatex-alpha)"));
}

#[test]
fn remove_installed_style() {
    let (dir, resolver) = make_resolver(StoreFormat::Yaml);

    let src = dir.path().join("alpha.yaml");
    fs::write(&src, minimal_style_yaml()).unwrap();
    resolver.install_style(&src).expect("install_style");
    resolver.remove_style("alpha").expect("remove_style");

    let styles = resolver.list_styles().expect("list_styles");
    assert!(styles.is_empty());
}

#[test]
fn resolve_missing_style_returns_error() {
    let (_dir, resolver) = make_resolver(StoreFormat::Yaml);
    assert!(resolver.resolve_style("nonexistent").is_err());
}

#[test]
fn remove_missing_style_returns_error() {
    let (_dir, resolver) = make_resolver(StoreFormat::Yaml);
    assert!(resolver.remove_style("nonexistent").is_err());
}

#[test]
fn install_and_resolve_json_style() {
    let (dir, resolver) = make_resolver(StoreFormat::Json);

    // Convert the YAML fixture to JSON via citum_schema
    let style: citum_schema::Style =
        serde_yaml::from_slice(minimal_style_yaml()).expect("parse yaml");
    let json = serde_json::to_vec(&style).expect("to json");

    let src = dir.path().join("alpha.json");
    fs::write(&src, &json).unwrap();
    resolver.install_style(&src).expect("install_style");

    let resolved = resolver.resolve_style("alpha").expect("resolve_style");
    assert_eq!(
        resolved.info.title.as_deref(),
        Some("Alpha (biblatex-alpha)")
    );
}

#[test]
fn install_and_resolve_cbor_style() {
    let (dir, resolver) = make_resolver(StoreFormat::Cbor);

    let style: citum_schema::Style =
        serde_yaml::from_slice(minimal_style_yaml()).expect("parse yaml");
    let mut cbor = Vec::new();
    ciborium::ser::into_writer(&style, &mut cbor).expect("to cbor");

    let src = dir.path().join("alpha.cbor");
    fs::write(&src, &cbor).unwrap();
    resolver.install_style(&src).expect("install_style");

    let resolved = resolver.resolve_style("alpha").expect("resolve_style");
    assert_eq!(
        resolved.info.title.as_deref(),
        Some("Alpha (biblatex-alpha)")
    );
}

#[test]
fn resolver_fallback_finds_any_format() {
    // Resolver configured for CBOR but only a YAML file exists — should still find it.
    let (dir, resolver) = make_resolver(StoreFormat::Cbor);

    let src = dir.path().join("alpha.yaml");
    fs::write(&src, minimal_style_yaml()).unwrap();
    // Install directly (bypassing the resolver so it lands as .yaml)
    let styles_dir = dir.path().join("styles");
    fs::create_dir_all(&styles_dir).unwrap();
    fs::copy(&src, styles_dir.join("alpha.yaml")).unwrap();

    let resolved = resolver.resolve_style("alpha").expect("fallback resolve");
    assert_eq!(
        resolved.info.title.as_deref(),
        Some("Alpha (biblatex-alpha)")
    );
}
