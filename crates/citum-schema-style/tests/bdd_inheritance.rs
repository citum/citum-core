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
#![allow(missing_docs, reason = "test/bench/bin crate")]

//! BDD tests for Template V3 variant inheritance and structural diffs.
//!
//! These tests verify that complex inheritance chains and structural overlays
//! are resolved deterministically and predictably in `merge_style_overlay`.

use citum_schema_style::{
    Style,
    locale::GeneralTerm,
    template::{SimpleVariable, TemplateComponent, TypeSelector},
};
use rstest::rstest;
use std::collections::HashSet;

fn create_base_style() -> Style {
    let yaml = r#"
version: "0.44.0"
info:
  title: Base Style
  id: base
bibliography:
  template:
    - title: primary
    - variable: doi
    - variable: url
  type-variants:
    book:
      - title: primary
      - variable: publisher
      - variable: url
"#;
    Style::from_yaml_str(yaml).expect("valid base style")
}

#[rstest]
#[case::override_rendering(
    r#"
extends: base
bibliography:
  type-variants:
    book:
      modify:
        - match: { title: primary }
          emph: true
"#,
    |style: &Style| {
        let bib = style.bibliography.as_ref().unwrap();
        let book = bib.type_variants.as_ref().unwrap().get(&TypeSelector::Single("book".into())).unwrap();
        let template = book.as_template().unwrap();
        assert_eq!(template[0].rendering().emph, Some(true));
        assert!(matches!(template[0], TemplateComponent::Title(_)));
    }
)]
#[case::add_component_before(
    r#"
extends: base
bibliography:
  type-variants:
    book:
      add:
        - before: { title: primary }
          component: { term: in }
"#,
    |style: &Style| {
        let bib = style.bibliography.as_ref().unwrap();
        let book = bib.type_variants.as_ref().unwrap().get(&TypeSelector::Single("book".into())).unwrap();
        let template = book.as_template().unwrap();
        assert_eq!(template.len(), 4);
        assert!(matches!(template[0], TemplateComponent::Term(_)));
        if let TemplateComponent::Term(t) = &template[0] {
            assert_eq!(t.term, GeneralTerm::In);
        }
    }
)]
#[case::remove_component(
    r#"
extends: base
bibliography:
  type-variants:
    book:
      remove:
        - match: { variable: publisher }
"#,
    |style: &Style| {
        let bib = style.bibliography.as_ref().unwrap();
        let book = bib.type_variants.as_ref().unwrap().get(&TypeSelector::Single("book".into())).unwrap();
        let template = book.as_template().unwrap();
        assert_eq!(template.len(), 2);
        assert!(!template.iter().any(|c| matches!(c, TemplateComponent::Variable(v) if v.variable == SimpleVariable::Publisher)));
    }
)]
#[case::deep_inheritance_explicit_extends(
    r#"
extends: base
bibliography:
  type-variants:
    book:
      extends: book
      modify:
        - match: { variable: publisher }
          strong: true
"#,
    |style: &Style| {
        let bib = style.bibliography.as_ref().unwrap();
        let book = bib.type_variants.as_ref().unwrap().get(&TypeSelector::Single("book".into())).unwrap();
        let template = book.as_template().unwrap();

        assert_eq!(template.len(), 3);
        let pub_comp = &template[1];
        assert_eq!(pub_comp.rendering().strong, Some(true));
    }
)]
fn test_template_variant_inheritance(#[case] overlay_yaml: &str, #[case] assertion: fn(&Style)) {
    let base = create_base_style();
    let overlay = Style::from_yaml_str(overlay_yaml).expect("valid overlay style");

    struct MockResolver(Style);
    impl citum_resolver_api::StyleResolver for MockResolver {
        type Style = Style;
        type Locale = citum_schema_style::locale::Locale;

        fn resolve_style(&self, _uri: &str) -> Result<Style, citum_schema_style::ResolverError> {
            Ok(self.0.clone())
        }

        fn resolve_locale(
            &self,
            id: &str,
        ) -> Result<Self::Locale, citum_schema_style::ResolverError> {
            Err(citum_schema_style::ResolverError::LocaleNotFound(
                std::borrow::Cow::Owned(id.to_string()),
            ))
        }
    }

    let resolver = MockResolver(base.clone());
    let mut visited = HashSet::new();
    let resolved = overlay
        .try_into_resolved_recursive_with(Some(&resolver), &mut visited)
        .expect("resolution should succeed");

    assertion(&resolved);
}

#[rstest]
#[case::cross_variant_extension(
    r#"
bibliography:
  template: [{ title: primary }]
  type-variants:
    book:
      modify: [{ match: { title: primary }, emph: true }]
    thesis:
      extends: book
      add: [{ after: { title: primary }, component: { variable: doi } }]
"#,
    |style: &Style| {
        let bib = style.bibliography.as_ref().unwrap();
        let variants = bib.type_variants.as_ref().unwrap();

        let book = variants.get(&TypeSelector::Single("book".into())).unwrap().as_template().unwrap();
        assert_eq!(book[0].rendering().emph, Some(true));

        let thesis = variants.get(&TypeSelector::Single("thesis".into())).unwrap().as_template().unwrap();
        assert_eq!(thesis.len(), 2);
        assert_eq!(thesis[0].rendering().emph, Some(true));
        assert!(matches!(thesis[1], TemplateComponent::Variable(ref v) if v.variable == SimpleVariable::Doi));
    }
)]
fn test_intra_style_variant_extension(#[case] yaml: &str, #[case] assertion: fn(&Style)) {
    let style = Style::from_yaml_str(yaml).expect("valid style");
    let mut visited = HashSet::new();
    let resolved = style
        .try_into_resolved_recursive_with(None, &mut visited)
        .expect("resolution should succeed");
    assertion(&resolved);
}

#[test]
fn test_multiple_inheritance_levels() {
    let base_yaml = r#"
version: "0.44.0"
info: { id: base }
bibliography:
  template: [{ title: primary }]
  type-variants:
    book:
      modify: [{ match: { title: primary }, emph: true }]
"#;
    let mid_yaml = r#"
extends: base
info: { id: mid }
bibliography:
  type-variants:
    book:
      extends: book
      add: [{ after: { title: primary }, component: { variable: doi } }]
"#;
    let top_yaml = r#"
extends: mid
info: { id: top }
bibliography:
  type-variants:
    book:
      extends: book
      modify: [{ match: { variable: doi }, strong: true }]
"#;

    let base = Style::from_yaml_str(base_yaml).unwrap();
    let mid = Style::from_yaml_str(mid_yaml).unwrap();
    let top = Style::from_yaml_str(top_yaml).unwrap();

    struct MultiResolver {
        base: Style,
        mid: Style,
    }
    impl citum_resolver_api::StyleResolver for MultiResolver {
        type Style = Style;
        type Locale = citum_schema_style::locale::Locale;

        fn resolve_style(&self, uri: &str) -> Result<Style, citum_schema_style::ResolverError> {
            match uri {
                "base" => Ok(self.base.clone()),
                "mid" => Ok(self.mid.clone()),
                _ => Err(citum_schema_style::ResolverError::StyleNotFound(
                    std::borrow::Cow::Owned(uri.to_string()),
                )),
            }
        }

        fn resolve_locale(
            &self,
            id: &str,
        ) -> Result<Self::Locale, citum_schema_style::ResolverError> {
            Err(citum_schema_style::ResolverError::LocaleNotFound(
                std::borrow::Cow::Owned(id.to_string()),
            ))
        }
    }

    let resolver = MultiResolver { base, mid };
    let mut visited = HashSet::new();
    let resolved = top
        .try_into_resolved_recursive_with(Some(&resolver), &mut visited)
        .expect("deep resolution should succeed");

    let bib = resolved.bibliography.as_ref().unwrap();
    let book = bib
        .type_variants
        .as_ref()
        .unwrap()
        .get(&TypeSelector::Single("book".into()))
        .unwrap();
    let template = book.as_template().unwrap();

    assert_eq!(template.len(), 2);
    // Component 0: title (emph: true from base)
    assert_eq!(template[0].rendering().emph, Some(true));
    // Component 1: doi (strong: true from top)
    assert_eq!(template[1].rendering().strong, Some(true));
}
