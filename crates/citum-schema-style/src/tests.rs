/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

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

use super::*;
use indexmap::IndexMap;

#[test]
fn test_style_minimal_deserialization() {
    let yaml = r#"
info:
  title: Test Style
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(style.info.title.as_ref().unwrap(), "Test Style");
}

#[test]
fn test_style_with_citation() {
    let yaml = r#"
info:
  title: Test
citation:
  template:
    - contributor: author
      form: short
    - date: issued
      form: year
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();
    let citation = style.citation.unwrap();
    assert_eq!(citation.resolve_template().unwrap().len(), 2);
}

#[test]
fn test_style_with_options() {
    let yaml = r#"
info:
  title: APA
options:
  processing: author-date
  contributors:
    display-as-sort: first
    and: symbol
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();
    let options = style.options.unwrap();
    assert_eq!(options.processing, Some(options::Processing::AuthorDate));
}

#[test]
fn test_resolve_for_position_ibid_falls_back_to_subsequent() {
    let citation = CitationSpec {
        suffix: Some("base".to_string()),
        subsequent: Some(Box::new(CitationSpec {
            suffix: Some("subseq".to_string()),
            ..Default::default()
        })),
        ..Default::default()
    };

    let resolved = citation
        .resolve_for_position(Some(&crate::citation::Position::Ibid))
        .into_owned();
    assert_eq!(resolved.suffix, Some("subseq".to_string()));

    let resolved_with_locator = citation
        .resolve_for_position(Some(&crate::citation::Position::IbidWithLocator))
        .into_owned();
    assert_eq!(resolved_with_locator.suffix, Some("subseq".to_string()));
}

#[test]
fn test_resolve_for_position_ibid_precedes_subsequent() {
    let citation = CitationSpec {
        suffix: Some("base".to_string()),
        subsequent: Some(Box::new(CitationSpec {
            suffix: Some("subseq".to_string()),
            ..Default::default()
        })),
        ibid: Some(Box::new(CitationSpec {
            suffix: Some("ibid".to_string()),
            ..Default::default()
        })),
        ..Default::default()
    };

    let resolved = citation
        .resolve_for_position(Some(&crate::citation::Position::Ibid))
        .into_owned();
    assert_eq!(resolved.suffix, Some("ibid".to_string()));

    let resolved_subsequent = citation
        .resolve_for_position(Some(&crate::citation::Position::Subsequent))
        .into_owned();
    assert_eq!(resolved_subsequent.suffix, Some("subseq".to_string()));
}

#[test]
fn test_resolve_for_position_merges_note_start_text_case() {
    let citation = CitationSpec {
        note_start_text_case: Some(NoteStartTextCase::Lowercase),
        ibid: Some(Box::new(CitationSpec {
            note_start_text_case: Some(NoteStartTextCase::CapitalizeFirst),
            ..Default::default()
        })),
        ..Default::default()
    };

    let resolved = citation
        .resolve_for_position(Some(&crate::citation::Position::Ibid))
        .into_owned();
    assert_eq!(
        resolved.note_start_text_case,
        Some(NoteStartTextCase::CapitalizeFirst)
    );

    let unresolved = citation.resolve_for_position(None).into_owned();
    assert_eq!(
        unresolved.note_start_text_case,
        Some(NoteStartTextCase::Lowercase)
    );
}

#[test]
fn test_citum_first_yaml() {
    // Test parsing the actual citum-first.yaml file structure
    let yaml = r#"
info:
  title: APA
options:
  substitute:
    contributor-role-form: short
    template:
      - editor
      - title
  processing: author-date
  contributors:
    display-as-sort: first
    and: symbol
citation:
  template:
    - contributor: author
      form: short
    - date: issued
      form: year
bibliography:
  template:
    - contributor: author
      form: long
    - date: issued
      form: year
      wrap: parentheses
    - title: primary
    - title: parent-monograph
      prefix: "In "
      emph: true
    - number: volume
    - variable: doi
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();

    // Verify info
    assert_eq!(style.info.title.as_ref().unwrap(), "APA");

    // Verify options
    let options = style.options.unwrap();
    assert_eq!(options.processing, Some(options::Processing::AuthorDate));
    assert!(options.substitute.is_some());

    // Verify citation
    let citation = style.citation.unwrap();
    let citation_template = citation.resolve_template().unwrap();
    assert_eq!(citation_template.len(), 2);

    // Verify bibliography
    let bib = style.bibliography.unwrap();
    let bib_template = bib.resolve_template().unwrap();
    assert_eq!(bib_template.len(), 6);

    // Verify flattened rendering worked
    match &bib_template[1] {
        template::TemplateComponent::Date(d) => {
            assert_eq!(
                d.rendering.wrap,
                Some(template::WrapConfig {
                    punctuation: template::WrapPunctuation::Parentheses,
                    inner_prefix: None,
                    inner_suffix: None,
                })
            );
        }
        _ => panic!("Expected Date"),
    }

    match &bib_template[3] {
        template::TemplateComponent::Title(t) => {
            assert_eq!(t.rendering.prefix, Some("In ".to_string()));
            assert_eq!(t.rendering.emph, Some(true));
        }
        _ => panic!("Expected Title"),
    }
}

#[test]
fn test_style_custom_fields() {
    let yaml = r#"
version: "1.1"
info:
  title: Custom Fields Test
custom:
  my-extension: true
  author-tool: "StyleAuthor v2.0"
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(style.version, SchemaVersion::parse("1.1").unwrap());
    let custom = style.custom.as_ref().unwrap();
    assert_eq!(
        custom.get("my-extension").unwrap(),
        &serde_json::Value::Bool(true)
    );
    assert_eq!(
        custom.get("author-tool").unwrap(),
        &serde_json::Value::String("StyleAuthor v2.0".to_string())
    );

    // Round-trip test
    let round_tripped = serde_yaml::to_string(&style).unwrap();
    assert!(
        round_tripped.contains("version: 1.1")
            || round_tripped.contains("version: \"1.1\"")
            || round_tripped.contains("version: '1.1'")
    );
    assert!(round_tripped.contains("my-extension: true"));
    assert!(round_tripped.contains("author-tool:"));
}

#[test]
fn test_style_with_template_ref() {
    let yaml = r#"
info:
  title: Preset Test
citation:
  template-ref: apa
bibliography:
  template-ref: vancouver
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();

    // Test Citation template reference (APA)
    let citation = style.citation.unwrap();
    assert!(citation.template_ref.is_some());
    assert!(citation.template.is_none());

    let citation_template = citation.resolve_template().unwrap();
    assert_eq!(citation_template.len(), 3); // APA citation is (Author, Year, Locator)

    // precise check for APA structure
    match &citation_template[0] {
        template::TemplateComponent::Contributor(c) => {
            assert_eq!(c.contributor, template::ContributorRole::Author);
        }
        _ => panic!("Expected Contributor"),
    }

    // Test Bibliography template extension (Vancouver)
    let bib = style.bibliography.unwrap();
    let bib_template = bib.resolve_template().unwrap();
    // Vancouver bib has roughly 8 components
    assert!(bib_template.len() >= 5);

    // Verify first component is citation number
    match &bib_template[0] {
        template::TemplateComponent::Number(n) => {
            assert_eq!(n.number, template::NumberVariable::CitationNumber);
        }
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_template_overrides_template_ref_precedence() {
    let yaml = r#"
info:
  title: Override Test
citation:
  template-ref: apa
  template:
    - variable: doi
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();
    let citation = style.citation.unwrap();

    // Should have both
    assert!(citation.template_ref.is_some());
    assert!(citation.template.is_some());

    // Template should win
    let resolved = citation.resolve_template().unwrap();
    assert_eq!(resolved.len(), 1);
    match &resolved[0] {
        template::TemplateComponent::Variable(v) => {
            assert_eq!(v.variable, template::SimpleVariable::Doi);
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn cid_and_integrity_fields_round_trip() {
    let yaml = r#"
extends: https://hub.citum.org/styles/apa-7th.yaml
extends-pin: bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa
info:
  title: Pinned APA Variant
  citum-version: ">=0.38.0"
"#;
    let style: Style = Style::from_yaml_str(yaml).expect("yaml parses");
    assert!(style.extends.is_some(), "extends preserved");
    assert_eq!(
        style.extends_pin.as_deref(),
        Some("bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa")
    );
    assert_eq!(style.info.citum_version.as_deref(), Some(">=0.38.0"));

    let serialized = serde_yaml::to_string(&style).expect("serializes");
    assert!(
        serialized.contains("extends-pin: bafkrei"),
        "extends-pin field name preserved on serialization: {serialized}"
    );
    assert!(
        serialized.contains("citum-version:"),
        "citum-version field name preserved on serialization: {serialized}"
    );
}

#[test]
fn citum_version_too_high_rejects_resolution() {
    let yaml = r#"
info:
  title: From the Future
  citum-version: ">=999.0.0"
"#;
    let style = Style::from_yaml_str(yaml).unwrap();
    let err = style.try_into_resolved().expect_err("must reject");
    assert!(
        matches!(err, ResolutionError::VersionMismatch { .. }),
        "expected VersionMismatch, got {err:?}"
    );
}

#[test]
fn citum_version_satisfied_resolves_normally() {
    let yaml = r#"
info:
  title: Satisfied
  citum-version: ">=0.0.1"
"#;
    let style = Style::from_yaml_str(yaml).unwrap();
    let resolved = style.try_into_resolved().expect("must resolve");
    assert_eq!(resolved.info.title.as_deref(), Some("Satisfied"));
}

#[test]
fn extends_pin_on_builtin_base_is_rejected() {
    let yaml = r#"
extends: apa-7th
extends-pin: bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa
info:
  title: Bad Pin Target
"#;
    let style = Style::from_yaml_str(yaml).unwrap();
    let err = style.try_into_resolved().expect_err("must reject");
    assert!(
        matches!(err, ResolutionError::UriResolutionFailed { .. }),
        "expected UriResolutionFailed for builtin pin, got {err:?}"
    );
}

#[test]
fn cid_extends_uri_round_trips_as_uri_variant() {
    let yaml = r#"
extends: cid:bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa
info:
  title: CID-extended Style
"#;
    let style: Style = Style::from_yaml_str(yaml).expect("yaml parses");
    let extends = style.extends.expect("extends present");
    assert!(extends.is_cid(), "cid: prefix detected as CID URI");
    assert_eq!(
        extends.key(),
        "cid:bafkreigh2akiscaildc6mzfo4qtbk3rjmxa3ofkpzxqzd2cs6ftxvtsqfa"
    );
}

#[test]
fn old_section_extends_key_is_captured_for_forward_compat() {
    let yaml = r#"
info:
  title: Old Section Template Reuse Key
citation:
  extends: apa
"#;
    let style: Style = serde_yaml::from_str(yaml).expect("parse succeeds");
    let citation = style.citation.expect("citation section present");
    assert!(
        citation.unknown_fields.contains_key("extends"),
        "removed `extends` key should land in unknown_fields for SoftDegrade detection, \
             got: {:?}",
        citation.unknown_fields,
    );
}

#[test]
fn test_citation_localized_templates() {
    let yaml = r#"
info:
  title: Localized Citation
citation:
  template:
    - variable: note
  locales:
    - locale: [de]
      template:
        - variable: publisher
    - default: true
      template:
        - variable: doi
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();
    let citation = style.citation.unwrap();

    assert_eq!(
        citation
            .resolve_template_for_language(Some("de-AT"))
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        citation
            .resolve_template_for_language(Some("fr"))
            .unwrap()
            .len(),
        1
    );
    match &citation.resolve_template_for_language(Some("de")).unwrap()[0] {
        template::TemplateComponent::Variable(v) => {
            assert_eq!(v.variable, template::SimpleVariable::Publisher);
        }
        _ => panic!("Expected Variable"),
    }
    match &citation.resolve_template_for_language(Some("fr")).unwrap()[0] {
        template::TemplateComponent::Variable(v) => {
            assert_eq!(v.variable, template::SimpleVariable::Doi);
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_bibliography_localized_templates() {
    let yaml = r#"
info:
  title: Localized Bibliography
bibliography:
  template:
    - variable: note
  locales:
    - locale: [ja, zh]
      template:
        - title: primary
    - default: true
      template:
        - contributor: author
          form: long
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();
    let bibliography = style.bibliography.unwrap();

    match &bibliography
        .resolve_template_for_language(Some("ja-JP"))
        .unwrap()[0]
    {
        template::TemplateComponent::Title(_) => {}
        _ => panic!("Expected Title"),
    }
    match &bibliography
        .resolve_template_for_language(Some("en-US"))
        .unwrap()[0]
    {
        template::TemplateComponent::Contributor(_) => {}
        _ => panic!("Expected Contributor"),
    }
}

#[test]
fn test_bibliography_with_groups() {
    let yaml = r#"
info:
  title: Grouped Bibliography Test
bibliography:
  template:
    - contributor: author
      form: long
  groups:
    - id: vietnamese
      heading:
        localized:
          vi: "Tài liệu tiếng Việt"
          en-US: "Vietnamese Sources"
      selector:
        field:
          language: vi
      sort:
        template:
          - key: author
            sort-order: given-family
    - id: other
      selector:
        not:
          field:
            language: vi
"#;
    let style: Style = serde_yaml::from_str(yaml).unwrap();
    let bib = style.bibliography.unwrap();

    assert!(bib.groups.is_some());
    let groups = bib.groups.unwrap();
    assert_eq!(groups.len(), 2);

    // First group
    assert_eq!(groups[0].id, "vietnamese");
    match groups[0].heading.as_ref().unwrap() {
        grouping::GroupHeading::Localized { localized } => {
            assert_eq!(localized.get("vi").unwrap(), "Tài liệu tiếng Việt");
        }
        _ => panic!("expected localized heading"),
    }
    assert!(groups[0].sort.is_some());

    // Second group (fallback with negation)
    assert_eq!(groups[1].id, "other");
    assert!(groups[1].heading.is_none());
    assert!(groups[1].selector.not.is_some());
}

#[test]
fn validate_type_name_accepts_known_types() {
    assert!(template::validate_type_name("article-journal"));
    assert!(template::validate_type_name("legal-case"));
    assert!(template::validate_type_name("all"));
    assert!(template::validate_type_name("default"));
}

#[test]
fn validate_type_name_normalizes_underscores() {
    assert!(template::validate_type_name("article_journal"));
    assert!(template::validate_type_name("legal_case"));
}

#[test]
fn validate_type_name_rejects_unknown() {
    assert!(!template::validate_type_name("article-journall"));
    assert!(!template::validate_type_name("unknown-type"));
    assert!(!template::validate_type_name(""));
}

#[test]
fn style_validate_emits_warning_for_unknown_type_in_bib_type_variants() {
    let mut type_variants = IndexMap::new();
    type_variants.insert(
        template::TypeSelector::Single("typo-type".to_string()),
        TemplateVariant::Full(vec![]),
    );

    let style = Style {
        bibliography: Some(BibliographySpec {
            type_variants: Some(type_variants),
            ..Default::default()
        }),
        ..Default::default()
    };

    let warnings = style.validate();
    assert_eq!(warnings.len(), 1);
    match &warnings[0] {
        SchemaWarning::UnknownTypeName { name, location } => {
            assert_eq!(name, "typo-type");
            assert_eq!(location, "bibliography.type-variants");
        }
    }
}

#[test]
fn style_validate_no_warnings_for_valid_style() {
    let mut type_variants = IndexMap::new();
    type_variants.insert(
        template::TypeSelector::Single("legal-case".to_string()),
        TemplateVariant::Full(vec![]),
    );

    let style = Style {
        bibliography: Some(BibliographySpec {
            type_variants: Some(type_variants),
            ..Default::default()
        }),
        ..Default::default()
    };

    let warnings = style.validate();
    assert!(warnings.is_empty());
}

#[test]
fn null_type_variants_override_clears_preset_type_variants() {
    let child_yaml = r#"
extends: chicago-notes-18th
citation:
  type-variants: ~
  template:
  - contributor: author
    form: short
"#;
    let style = Style::from_yaml_str(child_yaml).expect("parses");
    let resolved = style.into_resolved();
    let citation = resolved.citation.unwrap();
    assert!(
        citation.type_variants.is_none(),
        "type_variants should be None after null override, got: {:?}",
        citation.type_variants.as_ref().map(|tv| tv.keys().count())
    );
}

#[test]
fn template_v3_diff_resolves_to_full_type_variant() {
    let yaml = r#"
bibliography:
  template:
  - contributor: author
    form: long
  - title: primary
  - variable: publisher
  - variable: url
  type-variants:
    article-journal:
      modify:
      - match: { variable: publisher }
        suppress: true
      remove:
      - match: { variable: url }
      add:
      - after: { title: primary }
        component: { title: parent-serial, emph: true }
"#;
    let resolved = Style::from_yaml_str(yaml)
        .expect("style should parse")
        .try_into_resolved()
        .expect("diff should resolve");
    let variants = resolved
        .bibliography
        .expect("bibliography should exist")
        .type_variants
        .expect("variants should exist");
    let template = variants
        .get(&TypeSelector::Single("article-journal".to_string()))
        .and_then(TemplateVariant::as_template)
        .expect("variant should resolve to a full template");

    assert_eq!(template.len(), 4);
    assert!(matches!(
        &template[2],
        TemplateComponent::Title(title)
            if title.title == template::TitleType::ParentSerial
                && title.rendering.emph == Some(true)
    ));
    assert!(template.iter().any(|component| matches!(
        component,
        TemplateComponent::Variable(variable)
            if variable.variable == template::SimpleVariable::Publisher
                && variable.rendering.suppress == Some(true)
    )));
}

#[test]
fn template_v3_modify_can_set_number_label_form() {
    let yaml = r#"
bibliography:
  template:
  - number: pages
  type-variants:
    chapter:
      modify:
      - match: { number: pages }
        label-form: short
"#;
    let resolved = Style::from_yaml_str(yaml)
        .expect("style should parse")
        .try_into_resolved()
        .expect("diff should resolve");
    let variants = resolved
        .bibliography
        .expect("bibliography should exist")
        .type_variants
        .expect("variants should exist");
    let template = variants
        .get(&TypeSelector::Single("chapter".to_string()))
        .and_then(TemplateVariant::as_template)
        .expect("variant should resolve to a full template");

    assert!(matches!(
        &template[0],
        TemplateComponent::Number(number)
            if number.number == template::NumberVariable::Pages
                && number.label_form == Some(template::LabelForm::Short)
    ));
}

#[test]
fn template_v3_add_with_missing_anchor_returns_resolution_error() {
    let yaml = r#"
bibliography:
  template:
  - title: primary
  type-variants:
    book:
      add:
      - after: { variable: publisher }
        component: { date: issued, form: year }
"#;
    let err = Style::from_yaml_str(yaml)
        .expect("style should parse")
        .try_into_resolved()
        .expect_err("missing add anchor should reject the diff");

    assert!(matches!(
        err,
        ResolutionError::TemplateVariantAnchorNotFound { location }
            if location == "bibliography.type-variants[book]"
    ));
}

#[test]
fn template_v3_nested_citation_diff_can_use_outer_template() {
    let style = Style {
        citation: Some(CitationSpec {
            template: Some(vec![
                TemplateComponent::Contributor(template::TemplateContributor {
                    contributor: template::ContributorRole::Author,
                    ..Default::default()
                }),
                TemplateComponent::Title(template::TemplateTitle {
                    title: template::TitleType::Primary,
                    ..Default::default()
                }),
            ]),
            integral: Some(Box::new(CitationSpec {
                type_variants: Some(indexmap::IndexMap::from([(
                    TypeSelector::Single("personal_communication".to_string()),
                    TemplateVariant::Diff(TemplateVariantDiff {
                        remove: vec![TemplateRemoveOperation {
                            match_selector: TemplateComponentSelector {
                                fields: std::collections::BTreeMap::from([(
                                    "title".to_string(),
                                    serde_json::json!("primary"),
                                )]),
                            },
                        }],
                        ..Default::default()
                    }),
                )])),
                ..Default::default()
            })),
            ..Default::default()
        }),
        ..Default::default()
    };
    let resolved = style
        .try_into_resolved()
        .expect("nested diff should resolve against outer citation template");
    let variants = resolved
        .citation
        .expect("citation should exist")
        .integral
        .expect("integral citation should exist")
        .type_variants
        .expect("variants should exist");
    let template = variants
        .get(&TypeSelector::Single("personal_communication".to_string()))
        .and_then(TemplateVariant::as_template)
        .expect("variant should resolve");

    assert_eq!(template.len(), 1);
    assert!(matches!(template[0], TemplateComponent::Contributor(_)));
}

#[test]
fn template_v3_overlay_variant_defaults_to_inherited_variant() {
    #[derive(Clone)]
    struct FakeResolver {
        style: Style,
    }

    impl citum_resolver_api::StyleResolver for FakeResolver {
        type Style = Style;
        type Locale = Locale;

        fn resolve_style(&self, uri: &str) -> Result<Style, ResolverError> {
            if uri == "parent-style" {
                Ok(self.style.clone())
            } else {
                Err(ResolverError::StyleNotFound(std::borrow::Cow::Owned(
                    uri.to_string(),
                )))
            }
        }

        fn resolve_locale(&self, id: &str) -> Result<Self::Locale, ResolverError> {
            Err(ResolverError::LocaleNotFound(std::borrow::Cow::Owned(
                id.to_string(),
            )))
        }
    }

    let parent = Style::from_yaml_str(
        r#"
bibliography:
  template:
  - title: primary
  type-variants:
    book:
    - title: primary
      emph: true
"#,
    )
    .expect("parent should parse");
    let child = Style::from_yaml_str(
        r#"
extends: parent-style
bibliography:
  type-variants:
    book:
      modify:
      - match: { title: primary }
        quote: true
"#,
    )
    .expect("child should parse");

    let resolved = child
        .try_into_resolved_with(Some(&FakeResolver { style: parent }))
        .expect("child diff should resolve");
    let template = resolved
        .bibliography
        .expect("bibliography should exist")
        .type_variants
        .expect("variants should exist")
        .get(&TypeSelector::Single("book".to_string()))
        .and_then(TemplateVariant::as_template)
        .expect("book variant should resolve")
        .to_vec();

    assert!(matches!(
        &template[0],
        TemplateComponent::Title(title)
            if title.rendering.emph == Some(true)
                && title.rendering.quote == Some(true)
    ));
}

#[test]
fn citation_options_parse_valid_citation_fields() {
    let yaml = r#"
citation:
  options:
    contributors:
      shorten: {min: 3, use-first: 1}
    links:
      doi: true
"#;

    let style = Style::from_yaml_str(yaml).expect("citation options should parse");
    let options = style
        .citation
        .and_then(|citation| citation.options)
        .expect("citation options should exist");
    assert!(options.contributors.is_some());
    assert_eq!(options.links.and_then(|links| links.doi), Some(true));
}

#[test]
fn citation_options_capture_bibliography_only_fields_for_forward_compat() {
    let yaml = r#"
citation:
  options:
    entry-suffix: "."
"#;

    // Misplaced keys land in `unknown_fields` (SoftDegrade contract); strict-mode
    // surfaces them via `citum check --strict`.
    let style = Style::from_yaml_str(yaml).expect("citation options must tolerate unknown keys");
    let options = style
        .citation
        .and_then(|c| c.options)
        .expect("citation.options must exist");
    assert!(options.unknown_fields.contains_key("entry-suffix"));
}

#[test]
fn bibliography_options_parse_valid_bibliography_fields() {
    let yaml = r#"
bibliography:
  options:
    entry-suffix: "."
    separator: ", "
"#;

    let style = Style::from_yaml_str(yaml).expect("bibliography options should parse");
    let options = style
        .bibliography
        .and_then(|bibliography| bibliography.options)
        .expect("bibliography options should exist");
    assert_eq!(options.entry_suffix.as_deref(), Some("."));
    assert_eq!(options.separator.as_deref(), Some(", "));
}

#[test]
fn bibliography_options_capture_citation_only_fields_for_forward_compat() {
    let yaml = r#"
bibliography:
  options:
    locators:
      form: short
"#;

    let style =
        Style::from_yaml_str(yaml).expect("bibliography options must tolerate unknown keys");
    let options = style
        .bibliography
        .and_then(|b| b.options)
        .expect("bibliography.options must exist");
    assert!(options.unknown_fields.contains_key("locators"));
}

#[test]
fn top_level_options_capture_unknown_fields() {
    let yaml = r#"
options:
  bibliography:
    entry-suffix: "."
"#;

    // Unknown keys under `options:` are captured for forward-compat rather than rejected.
    let style = Style::from_yaml_str(yaml).expect("unknown top-level option key must be tolerated");
    assert!(!style.options.unwrap().unknown_fields.is_empty());
}

#[test]
fn profile_rejects_top_level_templates() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
templates:
  foo:
    - title: primary
"#;
    let err = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect_err("profile template override must fail");
    assert!(matches!(
        err,
        ResolutionError::InvalidProfileOverride { location } if location == "templates"
    ));
}

#[test]
fn profile_rejects_citation_template_override() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
citation:
  template:
    - title: primary
"#;
    let err = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect_err("profile citation template override must fail");
    assert!(matches!(
        err,
        ResolutionError::InvalidProfileOverride { location } if location == "citation.template"
    ));
}

#[test]
fn profile_rejects_bibliography_type_variants_override() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
bibliography:
  type-variants:
    default:
      - title: primary
"#;
    let err = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect_err("profile bibliography type variants must fail");
    assert!(matches!(
        err,
        ResolutionError::InvalidProfileOverride { location } if location == "bibliography.type-variants"
    ));
}

#[test]
fn profile_rejects_null_template_clear() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
bibliography:
  template: ~
"#;
    let err = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect_err("profile null template clear must fail");
    assert!(matches!(
        err,
        ResolutionError::InvalidProfileOverride { location } if location == "bibliography.template"
    ));
}

#[test]
fn profile_rejects_bibliography_template_ref_override() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
bibliography:
  template-ref: apa
"#;
    let err = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect_err("profile bibliography template-ref override must fail");
    assert!(matches!(
        err,
        ResolutionError::InvalidProfileOverride { location } if location == "bibliography.template-ref"
    ));
}

#[test]
fn profile_rejects_citation_localized_template_override() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
citation:
  locales:
    - locale: [fr-FR]
      template:
        - title: primary
"#;
    let err = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect_err("profile citation localized template override must fail");
    assert!(matches!(
        err,
        ResolutionError::InvalidProfileOverride { location } if location == "citation.locales"
    ));
}

#[test]
fn profile_rejects_citation_subsequent_template_override() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
citation:
  subsequent:
    template:
      - title: primary
"#;
    let err = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect_err("profile citation subsequent template override must fail");
    assert!(matches!(
        err,
        ResolutionError::InvalidProfileOverride { location } if location == "citation.subsequent.template"
    ));
}

#[test]
fn profile_rejects_citation_ibid_type_variants_override() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
citation:
  ibid:
    type-variants:
      default:
        - title: primary
"#;
    let err = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect_err("profile citation ibid type variants override must fail");
    assert!(matches!(
        err,
        ResolutionError::InvalidProfileOverride { location } if location == "citation.ibid.type-variants"
    ));
}

#[test]
fn profile_allows_normal_options() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
options:
  page-range-format: expanded
"#;
    let resolved = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect("profile wrappers should accept normal options");
    assert_eq!(
        resolved
            .options
            .as_ref()
            .and_then(|options| options.page_range_format.clone()),
        Some(options::PageRangeFormat::Expanded)
    );
}

#[test]
fn profile_rejects_removed_options_profile_surface() {
    let yaml = r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
options:
  profile:
    citation-label-wrap: brackets
"#;
    let err = Style::from_yaml_str(yaml).expect_err("legacy profile surface must fail");
    assert!(err.to_string().contains("`options.profile` was removed"));
}

#[test]
fn profile_resolution_leaves_hidden_core_templates_intact() {
    let base = StyleBase::ElsevierHarvardCore.base();
    let wrapper = Style::from_yaml_str(
        r#"
info:
  id: elsevier-harvard
extends: elsevier-harvard-core
"#,
    )
    .unwrap()
    .try_into_resolved()
    .expect("wrapper should resolve");

    assert_eq!(
        wrapper
            .citation
            .as_ref()
            .and_then(|citation| citation.resolve_template()),
        base.citation
            .as_ref()
            .and_then(|citation| citation.resolve_template())
    );
    assert_eq!(
        wrapper
            .bibliography
            .as_ref()
            .and_then(|bib| bib.resolve_template()),
        base.bibliography
            .as_ref()
            .and_then(|bib| bib.resolve_template())
    );
}

#[test]
fn scoped_options_apply_to_profile_wrappers() {
    let resolved = Style::from_yaml_str(
        r#"
info:
  id: elsevier-vancouver
extends: elsevier-vancouver-core
citation:
  options:
    label-wrap: brackets
    group-delimiter: comma
bibliography:
  options:
    title-terminator: comma
    repeated-author-rendering: dash
"#,
    )
    .unwrap()
    .try_into_resolved()
    .expect("scoped wrapper config should resolve");

    assert_eq!(
        resolved
            .citation
            .as_ref()
            .and_then(|citation| citation.wrap.clone()),
        Some(template::WrapConfig::from(
            template::WrapPunctuation::Brackets
        ))
    );
    assert_eq!(
        resolved
            .citation
            .as_ref()
            .and_then(|citation| citation.multi_cite_delimiter.clone())
            .as_deref(),
        Some(", ")
    );
    assert_eq!(
        resolved
            .bibliography
            .as_ref()
            .and_then(|bib| bib.options.as_ref())
            .and_then(|options| options.subsequent_author_substitute.clone())
            .as_deref(),
        Some("———")
    );
}

#[test]
fn options_contributors_replaces_profile_contributor_slot() {
    let resolved = Style::from_yaml_str(
        r#"
info:
  id: springer-basic-author-date
extends: springer-basic-author-date-core
options:
  contributors: springer
"#,
    )
    .unwrap()
    .try_into_resolved()
    .expect("top-level contributor preset should resolve");

    let contributors = resolved
        .options
        .as_ref()
        .and_then(|options| options.contributors.as_ref())
        .expect("resolved style should include contributor config");
    assert_eq!(contributors.name_form, Some(options::NameForm::Initials));
    assert_eq!(
        contributors.demote_non_dropping_particle,
        Some(options::DemoteNonDroppingParticle::Never)
    );
}

#[test]
fn citation_superscript_wrap_applies_vertical_align() {
    let resolved = Style::from_yaml_str(
        r#"
info:
  id: elsevier-vancouver
extends: elsevier-vancouver-core
citation:
  options:
    label-wrap: superscript
"#,
    )
    .unwrap()
    .try_into_resolved()
    .expect("superscript citation wrap should resolve");

    let citation_number_rendering = resolved
        .citation
        .as_ref()
        .and_then(|citation| citation.resolve_template())
        .and_then(|template| {
            template.iter().find_map(|component| match component {
                template::TemplateComponent::Number(number)
                    if matches!(
                        number.number,
                        template::NumberVariable::CitationNumber
                            | template::NumberVariable::CitationLabel
                    ) =>
                {
                    Some(number.rendering.clone())
                }
                _ => None,
            })
        })
        .expect("numeric citation template should include a citation label");

    assert_eq!(
        citation_number_rendering.vertical_align,
        Some(VerticalAlign::Superscript)
    );
    assert_eq!(citation_number_rendering.wrap, None);
}

#[test]
fn bibliography_rejects_superscript_label_wrap_at_parse_time() {
    let yaml = r#"
bibliography:
  options:
    label-wrap: superscript
"#;
    let err = Style::from_yaml_str(yaml).expect_err("bibliography superscript wrap must fail");
    assert!(err.to_string().contains("unknown variant `superscript`"));
}

#[test]
fn standalone_styles_can_use_scoped_options() {
    let resolved = Style::from_yaml_str(
        r#"
citation:
  template-ref: numeric-citation
  options:
    label-wrap: superscript
    group-delimiter: comma
bibliography:
  template-ref: vancouver
  options:
    label-mode: numeric
    title-terminator: comma
    repeated-author-rendering: dash-with-space
"#,
    )
    .unwrap()
    .try_into_resolved()
    .expect("standalone scoped options should resolve");

    assert_eq!(
        resolved
            .citation
            .as_ref()
            .and_then(|citation| citation.multi_cite_delimiter.as_deref()),
        Some(", ")
    );
    assert_eq!(
        resolved
            .citation
            .as_ref()
            .and_then(|citation| citation.resolve_template())
            .and_then(|template| {
                template.iter().find_map(|component| match component {
                    template::TemplateComponent::Number(number)
                        if matches!(
                            number.number,
                            template::NumberVariable::CitationNumber
                                | template::NumberVariable::CitationLabel
                        ) =>
                    {
                        Some(number.rendering.vertical_align.clone())
                    }
                    _ => None,
                })
            }),
        Some(Some(VerticalAlign::Superscript))
    );
    assert_eq!(
        resolved
            .bibliography
            .as_ref()
            .and_then(|bib| bib.options.as_ref())
            .and_then(|options| options.subsequent_author_substitute.as_deref()),
        Some("——— ")
    );
}

#[test]
fn non_registry_extends_styles_do_not_use_profile_contract() {
    let yaml = r#"
info:
  id: local-custom-profile
extends: elsevier-vancouver-core
citation:
  template:
    - number: citation-number
"#;
    let resolved = Style::from_yaml_str(yaml)
        .unwrap()
        .try_into_resolved()
        .expect("non-registry extends styles should retain merge semantics");
    assert!(resolved.citation.is_some());
}

#[test]
fn uri_extends_file_resolves_yaml() {
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = std::env::temp_dir().join(format!("citum_test_{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    let parent_path = dir.join("parent.yaml");
    let mut f = std::fs::File::create(&parent_path).unwrap();
    f.write_all(b"info:\n  title: Parent\ncitation:\n  template: []\n")
        .unwrap();
    let child_yaml = format!(
        "info:\n  title: Child\nextends: \"file://{}\"\n",
        parent_path.display()
    );
    let child = Style::from_yaml_str(&child_yaml).unwrap();
    let resolved = child.try_into_resolved().unwrap();
    std::fs::remove_dir_all(&dir).ok();
    assert!(
        resolved.citation.is_some(),
        "should inherit citation from file-backed parent"
    );
}

#[test]
fn uri_extends_missing_file_returns_error() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = std::env::temp_dir().join(format!("citum_test_{nanos}_missing"));
    std::fs::create_dir_all(&dir).unwrap();
    let missing = dir.join("does_not_exist.yaml");
    let yaml = format!(
        "info:\n  title: Child\nextends: \"file://{}\"\n",
        missing.display()
    );
    let child = Style::from_yaml_str(&yaml).unwrap();
    let err = child.try_into_resolved().unwrap_err();
    std::fs::remove_dir_all(&dir).ok();
    assert!(
        matches!(err, ResolutionError::UriResolutionFailed { .. }),
        "expected UriResolutionFailed, got {err:?}"
    );
}

#[test]
fn uri_extends_unsupported_scheme_returns_error() {
    let yaml = "info:\n  title: Child\nextends: \"https://example.com/style.yaml\"\n";
    let child = Style::from_yaml_str(yaml).unwrap();
    let err = child.try_into_resolved().unwrap_err();
    assert!(
        matches!(err, ResolutionError::UriResolutionFailed { .. }),
        "expected UriResolutionFailed for unsupported scheme, got {err:?}"
    );
}

#[test]
fn style_loader_reports_components_typo_in_add_operation() {
    let yaml = r#"
bibliography:
  template:
  - title: primary
  type-variants:
    chapter:
      add:
      - after: { title: primary }
        components:
          variable: doi
"#;
    let err = Style::from_yaml_str(yaml).expect_err("components typo should fail");
    let message = err.to_string();

    assert!(
        message.contains("bibliography.type-variants.chapter.add[0]"),
        "message should include precise operation path: {message}"
    );
    assert!(
        message.contains("unknown property \"components\" in TemplateAddOperation"),
        "message should name the rejected property and operation type: {message}"
    );
    assert!(
        message.contains("did you mean \"component\""),
        "message should suggest the singular field: {message}"
    );
}

#[test]
fn style_loader_reports_invalid_nested_component_body_path() {
    let yaml = r#"
bibliography:
  template:
  - title: primary
  type-variants:
    chapter:
      add:
      - after: { title: primary }
        component:
          components:
          - variable: doi
"#;
    let err = Style::from_yaml_str(yaml).expect_err("nested component body should fail");
    let message = err.to_string();

    assert!(
        message.contains("bibliography.type-variants.chapter.add[0].component"),
        "message should include precise nested component path: {message}"
    );
    assert!(
        message.contains("unknown template component property \"components\""),
        "message should name the invalid component property: {message}"
    );
    assert!(
        message.contains("component must contain exactly one of contributor/date/title/number/variable/group/term"),
        "message should enumerate valid component kinds: {message}"
    );
}

#[test]
fn style_loader_reports_unknown_template_component_key() {
    let yaml = r#"
bibliography:
  template:
  - components:
    - variable: doi
"#;
    let err = Style::from_yaml_str(yaml).expect_err("unknown component key should fail");
    let message = err.to_string();

    assert!(
        message.contains("bibliography.template[0]"),
        "message should include template item path: {message}"
    );
    assert!(
        message.contains("component must contain exactly one of contributor/date/title/number/variable/group/term"),
        "message should enumerate valid component kinds: {message}"
    );
}

#[test]
fn style_loader_accepts_valid_recursive_template_surfaces() {
    let yaml = r#"
templates:
  fallback-title:
  - title: primary
citation:
  template:
  - group:
    - contributor: author
      form: short
    - date: issued
      form: year
      fallback:
      - variable: doi
  locales:
  - locale: [en-US]
    template:
    - title: primary
  integral:
    template:
    - contributor: author
      form: short
bibliography:
  template:
  - title: primary
  type-variants:
    chapter:
    - title: parent-monograph
    article-journal:
      add:
      - after: { title: primary }
        component: { title: parent-serial, emph: true }
  groups:
  - id: cited
    selector:
      cited: visible
    template:
    - variable: doi
"#;
    Style::from_yaml_str(yaml).expect("valid recursive template surfaces should parse");
}
