use super::*;
use crate::reference::Reference;
use crate::render::plain::PlainText;
use citum_schema::locale::{GeneralTerm, GrammaticalGender, Locale, TermForm};
use citum_schema::options::contributors::NameForm;
use citum_schema::options::*;
use citum_schema::reference::{
    Contributor, ContributorEntry, ContributorGender, EdtfString, FlatName, InputReference,
    Monograph, MonographType, StructuredName,
};
use citum_schema::template::DateVariable as TemplateDateVar;
use citum_schema::template::*;
use csl_legacy::csl_json::{DateVariable, Name, Reference as LegacyReference};

fn make_config() -> Config {
    Config {
        processing: Some(citum_schema::options::Processing::AuthorDate),
        contributors: Some(ContributorConfig {
            shorten: Some(ShortenListOptions {
                min: 3,
                use_first: 1,
                ..Default::default()
            }),
            and: Some(AndOptions::Symbol),
            display_as_sort: Some(DisplayAsSort::First),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_locale() -> Locale {
    Locale::en_us()
}

fn make_reference() -> Reference {
    Reference::from(LegacyReference {
        id: "kuhn1962".to_string(),
        ref_type: "book".to_string(),
        author: Some(vec![Name::new("Kuhn", "Thomas S.")]),
        title: Some("The Structure of Scientific Revolutions".to_string()),
        issued: Some(DateVariable::year(1962)),
        publisher: Some("University of Chicago Press".to_string()),
        ..Default::default()
    })
}

fn make_spanish_gendered_locale() -> Locale {
    Locale::from_yaml_str(include_str!("../../../../locales/es-ES.yaml"))
        .expect("spanish locale should parse")
}

fn make_french_gendered_locale() -> Locale {
    Locale::from_yaml_str(include_str!("../../../../locales/fr-FR.yaml"))
        .expect("french locale should parse")
}

fn make_arabic_gendered_locale() -> Locale {
    Locale::from_yaml_str(include_str!("../../../../locales/ar-AR.yaml"))
        .expect("arabic locale should parse")
}

fn make_editor_reference(genders: &[ContributorGender]) -> Reference {
    let contributors = genders
        .iter()
        .enumerate()
        .map(|(idx, gender)| ContributorEntry {
            role: citum_schema::reference::ContributorRole::Editor,
            contributor: Contributor::StructuredName(StructuredName {
                family: format!("Editor{idx}").into(),
                given: format!("Nombre{idx}").into(),
                ..Default::default()
            }),
            gender: Some(*gender),
        })
        .collect();

    InputReference::Monograph(Box::new(Monograph {
        id: Some("editor-role-ref".into()),
        r#type: MonographType::Book,
        title: Some(Title::Single("Obra".to_string())),
        contributors,
        issued: EdtfString("2024".to_string()),
        ..Default::default()
    }))
}

fn make_custom_role_reference(
    role: citum_schema::reference::ContributorRole,
    genders: &[ContributorGender],
) -> Reference {
    let contributors = genders
        .iter()
        .enumerate()
        .map(|(idx, gender)| ContributorEntry {
            role: role.clone(),
            contributor: Contributor::StructuredName(StructuredName {
                family: format!("Persona{idx}").into(),
                given: format!("Nombre{idx}").into(),
                ..Default::default()
            }),
            gender: Some(*gender),
        })
        .collect();

    InputReference::Monograph(Box::new(Monograph {
        id: Some("custom-role-ref".into()),
        r#type: MonographType::Book,
        title: Some(Title::Single("Obra".to_string())),
        contributors,
        issued: EdtfString("2024".to_string()),
        ..Default::default()
    }))
}

fn make_gendered_locator_locale() -> Locale {
    Locale::from_yaml_str(
        r#"
locale: es-ES
locators:
  volume:
    short:
      singular:
        masculine: tomo
        feminine: entrega
      plural:
        masculine: tomos
        feminine: entregas
"#,
    )
    .expect("gendered locator locale should parse")
}

/// Helper to create `NameFormatContext` for tests.
fn make_name_format_context<'a>(
    display_as_sort: Option<DisplayAsSort>,
    name_order: Option<&'a NameOrder>,
    initialize_with: Option<&'a String>,
    initialize_with_hyphen: Option<bool>,
    name_form: Option<NameForm>,
    demote_ndp: Option<&'a DemoteNonDroppingParticle>,
    sort_separator: Option<&'a String>,
) -> super::contributor::NameFormatContext<'a> {
    super::contributor::NameFormatContext {
        display_as_sort,
        name_order,
        initialize_with,
        initialize_with_hyphen,
        name_form,
        demote_ndp,
        sort_separator,
    }
}

/// Tests the behavior of `test_contributor_values`.
#[test]
fn test_contributor_values() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = make_reference();
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Author,
        form: ContributorForm::Short,
        label: None,
        name_order: None,
        name_form: None,
        delimiter: None,
        sort_separator: None,
        shorten: None,
        and: None,
        rendering: Default::default(),
        links: None,
        gender: None,
        custom: None,
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(values.value, "Kuhn");
}

#[test]
fn test_spanish_role_label_uses_feminine_form_for_single_editor() {
    let config = make_config();
    let locale = make_spanish_gendered_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = make_editor_reference(&[ContributorGender::Feminine]);
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("editor should render");

    assert_eq!(values.suffix, Some(", editora".to_string()));
}

#[test]
fn test_spanish_role_label_uses_plural_feminine_form_for_matching_group() {
    let config = make_config();
    let locale = make_spanish_gendered_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference =
        make_editor_reference(&[ContributorGender::Feminine, ContributorGender::Feminine]);
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("editors should render");

    assert_eq!(values.suffix, Some(", editoras".to_string()));
}

#[test]
fn test_spanish_role_label_prefers_common_form_for_mixed_group() {
    let config = make_config();
    let locale = make_spanish_gendered_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference =
        make_editor_reference(&[ContributorGender::Feminine, ContributorGender::Masculine]);
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("mixed editors should render");

    assert_eq!(values.suffix, Some(", equipo editorial".to_string()));
}

#[test]
fn test_spanish_role_label_omits_gendered_label_for_mixed_group_without_common_form() {
    let config = make_config();
    let locale = Locale::from_yaml_str(
        r#"
locale: es-ES
roles:
  editor:
    long:
      singular:
        masculine: editor
        feminine: editora
      plural:
        masculine: editores
        feminine: editoras
"#,
    )
    .expect("locale should parse");
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference =
        make_editor_reference(&[ContributorGender::Feminine, ContributorGender::Masculine]);
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("editors should render");

    assert_eq!(values.suffix, None);
}

#[test]
fn test_french_role_label_uses_feminine_form_for_single_contributor() {
    let config = make_config();
    let locale = make_french_gendered_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = make_editor_reference(&[ContributorGender::Feminine]);
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("editor should render");

    assert_eq!(values.suffix, Some(", éditrice".to_string()));
}

#[test]
fn test_arabic_role_label_uses_feminine_form_for_single_contributor() {
    let config = make_config();
    let locale = make_arabic_gendered_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = make_editor_reference(&[ContributorGender::Feminine]);
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("editor should render");

    assert_eq!(values.suffix, Some(", مُحَرِّرَة".to_string()));
}

#[test]
fn test_french_role_label_falls_back_to_masculine_plural_for_mixed_group() {
    let config = make_config();
    let locale = make_french_gendered_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference =
        make_editor_reference(&[ContributorGender::Feminine, ContributorGender::Masculine]);
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("mixed editors should render");

    assert_eq!(values.suffix, Some(", éditeurs".to_string()));
}

#[test]
fn test_arabic_role_label_falls_back_to_verbal_noun_for_mixed_group() {
    let config = make_config();
    let locale = make_arabic_gendered_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference =
        make_editor_reference(&[ContributorGender::Feminine, ContributorGender::Masculine]);
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("mixed editors should render");

    assert_eq!(values.suffix, Some(", تحقيق".to_string()));
}

#[test]
fn test_arabic_role_label_falls_back_to_roles_common_when_gender_missing() {
    let config = make_config();
    let locale = make_arabic_gendered_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };

    // Reference with no gender info
    let contributors = vec![ContributorEntry {
        role: citum_schema::reference::ContributorRole::Editor,
        contributor: Contributor::StructuredName(StructuredName {
            family: "Editor".into(),
            given: "Name".into(),
            ..Default::default()
        }),
        gender: None,
    }];

    let reference = InputReference::Monograph(Box::new(Monograph {
        id: Some("no-gender-ref".into()),
        r#type: MonographType::Book,
        title: Some(Title::Single("Obra".to_string())),
        contributors,
        issued: EdtfString("2024".to_string()),
        ..Default::default()
    }));

    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("editor should render");

    // MF2 will return None because $gender is missing, should fall back to roles.editor.long.singular.common
    assert_eq!(values.suffix, Some(", تحقيق".to_string()));
}

#[test]
fn test_collection_editor_role_label_derives_gender_from_reference_data() {
    let config = make_config();
    let locale = make_spanish_gendered_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = make_custom_role_reference(
        citum_schema::reference::ContributorRole::Custom("collection-editor".to_string()),
        &[ContributorGender::Feminine],
    );
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::CollectionEditor,
        form: ContributorForm::Long,
        label: Some(RoleLabel {
            term: "collection-editor".to_string(),
            form: RoleLabelForm::Long,
            placement: LabelPlacement::Suffix,
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("collection editor should render");

    assert_eq!(values.suffix, Some(", directora".to_string()));
}

/// Tests the behavior of `test_date_values`.
#[test]
fn test_date_values() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = make_reference();
    let hints = ProcHints::default();

    let component = TemplateDate {
        date: TemplateDateVar::Issued,
        form: DateForm::Year,
        fallback: None,
        rendering: Default::default(),
        links: None,
        custom: None,
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(values.value, "1962");
}

#[test]
fn test_year_month_day_dates_inline_disambiguation_suffix_on_year() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = Reference::from(LegacyReference {
        id: "dated-2018".to_string(),
        ref_type: "article-magazine".to_string(),
        issued: Some(DateVariable::full(2018, 7, 14)),
        ..Default::default()
    });
    let hints = ProcHints {
        disamb_condition: true,
        group_index: 3,
        group_length: 4,
        ..Default::default()
    };

    let component = TemplateDate {
        date: TemplateDateVar::Issued,
        form: DateForm::YearMonthDay,
        fallback: None,
        rendering: Default::default(),
        links: None,
        custom: None,
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(values.value, "2018c, July 14");
    assert_eq!(values.suffix, None);
}

/// Tests the behavior of `test_et_al`.
#[test]
fn test_et_al() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "multi".to_string(),
        ref_type: "article-journal".to_string(),
        author: Some(vec![
            Name::new("LeCun", "Yann"),
            Name::new("Bengio", "Yoshua"),
            Name::new("Hinton", "Geoffrey"),
        ]),
        ..Default::default()
    });

    let component = TemplateContributor {
        contributor: ContributorRole::Author,
        form: ContributorForm::Short,
        label: None,
        name_order: None,
        name_form: None,
        delimiter: None,
        sort_separator: None,
        shorten: None,
        and: None,
        rendering: Default::default(),
        links: None,
        gender: None,
        custom: None,
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(values.value, "LeCun et al.");
}

/// Tests the behavior of `test_et_al_delimiter_never`.
#[test]
fn test_et_al_delimiter_never() {
    use citum_schema::options::DelimiterPrecedesLast;

    let mut config = make_config();
    if let Some(ref mut contributors) = config.contributors {
        contributors.shorten = Some(ShortenListOptions {
            min: 2,
            use_first: 1,
            ..Default::default()
        });
        contributors.delimiter_precedes_et_al = Some(DelimiterPrecedesLast::Never);
    }

    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "multi".to_string(),
        ref_type: "article-journal".to_string(),
        author: Some(vec![Name::new("Smith", "John"), Name::new("Jones", "Jane")]),
        ..Default::default()
    });

    let component = TemplateContributor {
        contributor: ContributorRole::Author,
        form: ContributorForm::Short,
        label: None,
        name_order: None,
        name_form: None,
        delimiter: None,
        sort_separator: None,
        shorten: None,
        and: None,
        rendering: Default::default(),
        links: None,
        gender: None,
        custom: None,
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // With "never", no comma before et al.
    assert_eq!(values.value, "Smith et al.");
}

#[test]
fn test_role_substitute_uses_custom_fallback_roles_without_silent_drop() {
    let mut config = make_config();
    config.substitute = Some(SubstituteConfig::Explicit(Substitute {
        role_substitute: std::collections::HashMap::from([(
            "editor".to_string(),
            vec!["compiler".to_string()],
        )]),
        ..Default::default()
    }));

    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = Reference::from(LegacyReference {
        id: "compiler-fallback".to_string(),
        ref_type: "book".to_string(),
        title: Some("Compiled Work".to_string()),
        extra: std::collections::HashMap::from([(
            "compiler".to_string(),
            serde_json::json!([{ "family": "Compiler", "given": "Casey" }]),
        )]),
        ..Default::default()
    });
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("custom compiler fallback should render");
    assert_eq!(values.value, "Compiler, Casey");
}

/// Tests the behavior of `test_et_al_delimiter_always`.
#[test]
fn test_et_al_delimiter_always() {
    use citum_schema::options::DelimiterPrecedesLast;

    let mut config = make_config();
    if let Some(ref mut contributors) = config.contributors {
        contributors.shorten = Some(ShortenListOptions {
            min: 2,
            use_first: 1,
            ..Default::default()
        });
        contributors.delimiter_precedes_et_al = Some(DelimiterPrecedesLast::Always);
    }

    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "multi".to_string(),
        ref_type: "article-journal".to_string(),
        author: Some(vec![Name::new("Smith", "John"), Name::new("Jones", "Jane")]),
        ..Default::default()
    });

    let component = TemplateContributor {
        contributor: ContributorRole::Author,
        form: ContributorForm::Short,
        label: None,
        name_order: None,
        name_form: None,
        delimiter: None,
        sort_separator: None,
        shorten: None,
        and: None,
        rendering: Default::default(),
        links: None,
        gender: None,
        custom: None,
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // With "always", comma before et al.
    assert_eq!(values.value, "Smith, et al.");
}

/// Tests the behavior of `test_demote_non_dropping_particle`.
#[test]
fn test_demote_non_dropping_particle() {
    use citum_schema::options::DemoteNonDroppingParticle;

    // Name: Ludwig van Beethoven
    let name = FlatName {
        family: Some("Beethoven".to_string()),
        given: Some("Ludwig".to_string()),
        non_dropping_particle: Some("van".to_string()),
        ..Default::default()
    };

    // Case 1: Never demote (default CSL behavior for display)
    // Inverted: "van Beethoven, Ludwig"
    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        None,
        None,
        None,
        Some(&DemoteNonDroppingParticle::Never),
        None,
    );
    let res_never = contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(res_never, "van Beethoven, Ludwig");

    // Case 2: Display-and-sort (demote)
    // Inverted: "Beethoven, Ludwig van"
    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        None,
        None,
        None,
        Some(&DemoteNonDroppingParticle::DisplayAndSort),
        None,
    );
    let res_demote = contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(res_demote, "Beethoven, Ludwig van");

    // Case 3: Sort-only (same as Never for display)
    // Inverted: "van Beethoven, Ludwig"
    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        None,
        None,
        None,
        Some(&DemoteNonDroppingParticle::SortOnly),
        None,
    );
    let res_sort_only =
        contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(res_sort_only, "van Beethoven, Ludwig");

    // Case 4: Not inverted (should be same for all)
    // "Ludwig van Beethoven"
    let ctx = make_name_format_context(
        Some(DisplayAsSort::None),
        None,
        None,
        None,
        None,
        Some(&DemoteNonDroppingParticle::DisplayAndSort),
        None,
    );
    let res_straight =
        contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(res_straight, "Ludwig van Beethoven");
}

/// Tests the behavior of `test_initialize_with_variants_for_multi_part_given_names`.
#[test]
fn test_initialize_with_variants_for_multi_part_given_names() {
    let name = FlatName {
        family: Some("Kuhn".to_string()),
        given: Some("Thomas Samuel".to_string()),
        ..Default::default()
    };

    let init_compact = String::new();
    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        Some(&init_compact),
        None,
        Some(NameForm::Initials),
        None,
        None,
    );
    let compact = contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(compact, "Kuhn, TS");

    let init_space = " ".to_string();
    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        Some(&init_space),
        None,
        Some(NameForm::Initials),
        None,
        None,
    );
    let space = contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(space, "Kuhn, T S");

    let init_dot = ".".to_string();
    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        Some(&init_dot),
        None,
        Some(NameForm::Initials),
        None,
        None,
    );
    let dot = contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(dot, "Kuhn, T.S.");

    let init_dot_space = ". ".to_string();
    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        Some(&init_dot_space),
        None,
        Some(NameForm::Initials),
        None,
        None,
    );
    let dot_space = contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(dot_space, "Kuhn, T. S.");
}

/// Tests the behavior of `test_initialize_with_hyphen_guard`.
#[test]
fn test_initialize_with_hyphen_guard() {
    let name = FlatName {
        family: Some("Kuhn".to_string()),
        given: Some("Jean-Paul".to_string()),
        ..Default::default()
    };
    let init_dot = ".".to_string();

    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        Some(&init_dot),
        None,
        Some(NameForm::Initials),
        None,
        None,
    );
    let hyphen_default =
        contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(hyphen_default, "Kuhn, J.-P.");

    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        Some(&init_dot),
        Some(false),
        Some(NameForm::Initials),
        None,
        None,
    );
    let hyphen_disabled =
        contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(hyphen_disabled, "Kuhn, J.");
}

/// Tests `NameForm` variants: Full, `FamilyOnly`, Initials, and backward-compat defaulting.
#[test]
fn test_name_form_variants() {
    use citum_schema::options::contributors::NameForm;

    let name = FlatName {
        family: Some("Smith".to_string()),
        given: Some("John David".to_string()),
        ..Default::default()
    };

    // Full: render complete given names
    let ctx = make_name_format_context(None, None, None, None, Some(NameForm::Full), None, None);
    let full = contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(full, "John David Smith");

    // FamilyOnly: suppress given names entirely
    let ctx = make_name_format_context(
        None,
        None,
        None,
        None,
        Some(NameForm::FamilyOnly),
        None,
        None,
    );
    let family_only =
        contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(family_only, "Smith");

    // Initials with explicit initialize_with
    let init_str = ". ".to_string();
    let ctx = make_name_format_context(
        None,
        None,
        Some(&init_str),
        None,
        Some(NameForm::Initials),
        None,
        None,
    );
    let initials = contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(initials, "J. D. Smith");

    // Initials with defaulted initialize_with (None → ". ")
    let ctx =
        make_name_format_context(None, None, None, None, Some(NameForm::Initials), None, None);
    let initials_default =
        contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(initials_default, "J. D. Smith");

    // Semantic split: name_form=None + initialize_with=Some → Full (not Initials).
    // initialize_with only controls the separator, not the form activation.
    // The migrator is responsible for co-emitting name_form: Initials with initialize_with.
    let ctx = make_name_format_context(None, None, Some(&init_str), None, None, None, None);
    let semantic_split =
        contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(semantic_split, "John David Smith");
}

/// Tests that Initials + defaulted separator produces correct hyphenated output.
#[test]
fn test_name_form_initials_hyphen_default_separator() {
    use citum_schema::options::contributors::NameForm;

    let name = FlatName {
        family: Some("Sartre".to_string()),
        given: Some("Jean-Paul".to_string()),
        ..Default::default()
    };

    // Default separator ". " should produce "J.-P." not "J. -P."
    let ctx =
        make_name_format_context(None, None, None, None, Some(NameForm::Initials), None, None);
    let result = contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(result, "J.-P. Sartre");
}

/// Tests the behavior of `test_template_list_suppression`.
#[test]
fn test_template_list_suppression() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = Reference::from(LegacyReference {
        id: "multi".to_string(),
        ..Default::default()
    });
    let hints = ProcHints::default();

    let component = TemplateGroup {
        group: vec![
            TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Doi,
                ..Default::default()
            }),
            TemplateComponent::Variable(TemplateVariable {
                variable: SimpleVariable::Url,
                ..Default::default()
            }),
        ],
        delimiter: Some(DelimiterPunctuation::Comma),
        ..Default::default()
    };

    let values = component.values::<PlainText>(&reference, &hints, &options);
    assert!(values.is_none());
}

/// Tests the behavior of `test_et_al_use_last`.
#[test]
fn test_et_al_use_last() {
    let mut config = make_config();
    if let Some(ref mut contributors) = config.contributors {
        contributors.shorten = Some(ShortenListOptions {
            min: 3,
            use_first: 1,
            use_last: Some(1),
            ..Default::default()
        });
    }

    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "multi".to_string(),
        ref_type: "article-journal".to_string(),
        author: Some(vec![
            Name::new("LeCun", "Yann"),
            Name::new("Bengio", "Yoshua"),
            Name::new("Hinton", "Geoffrey"),
        ]),
        ..Default::default()
    });

    let component = TemplateContributor {
        contributor: ContributorRole::Author,
        form: ContributorForm::Short,
        links: None,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // first name (LeCun) + ellipsis + last name (Hinton)
    assert_eq!(values.value, "LeCun … Hinton");
}

/// Tests the behavior of `test_et_al_use_last_overlap`.
#[test]
fn test_et_al_use_last_overlap() {
    // Edge case: use_first + use_last >= names.len() should show all names
    let mut config = make_config();
    if let Some(ref mut contributors) = config.contributors {
        contributors.shorten = Some(ShortenListOptions {
            min: 3,
            use_first: 2,
            use_last: Some(2),
            ..Default::default()
        });
    }

    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "overlap".to_string(),
        ref_type: "article-journal".to_string(),
        author: Some(vec![
            Name::new("Alpha", "A."),
            Name::new("Beta", "B."),
            Name::new("Gamma", "C."),
        ]),
        ..Default::default()
    });

    let component = TemplateContributor {
        contributor: ContributorRole::Author,
        form: ContributorForm::Short,
        links: None,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // use_first(2) + use_last(2) = 4 >= 3 names, so show first 2 + ellipsis + last 1
    // Alpha & Beta … Gamma (skip=max(2, 3-2)=2, so last 1 name)
    assert_eq!(values.value, "Alpha & Beta … Gamma");
}

/// Tests the behavior of `test_title_hyperlink`.
#[test]
fn test_title_hyperlink() {
    use citum_schema::options::LinksConfig;

    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "kuhn1962".to_string(),
        title: Some("The Structure of Scientific Revolutions".to_string()),
        doi: Some("10.1001/example".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        links: Some(LinksConfig {
            doi: Some(true),
            target: Some(LinkTarget::Doi),
            anchor: Some(LinkAnchor::Title),
            ..Default::default()
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(
        values.url,
        Some("https://doi.org/10.1001/example".to_string())
    );
}

/// Tests the behavior of `test_title_hyperlink_url_fallback`.
#[test]
fn test_title_hyperlink_url_fallback() {
    use citum_schema::options::LinksConfig;

    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    // Reference with URL but no DOI
    let reference = Reference::from(LegacyReference {
        id: "web2024".to_string(),
        title: Some("A Web Resource".to_string()),
        url: Some("https://example.com/resource".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        links: Some(LinksConfig {
            doi: Some(true),
            url: Some(true),
            target: Some(LinkTarget::UrlOrDoi),
            anchor: Some(LinkAnchor::Title),
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // Falls back to URL when DOI is absent
    assert_eq!(values.url, Some("https://example.com/resource".to_string()));
}

/// Tests the behavior of `test_title_values_smarten_leading_single_quotes`.
#[test]
fn test_title_values_smarten_leading_single_quotes() {
    // Upstream provenance: CSL fixture `flipflop_LeadingSingleQuote`.
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "parmenides".to_string(),
        ref_type: "book".to_string(),
        title: Some("'Parmenides' 132c-133a and the development of Plato's thought".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(
        values.value,
        "‘Parmenides’ 132c-133a and the development of Plato’s thought"
    );
}

/// Tests the behavior of `test_title_values_smarten_starting_apostrophe`.
#[test]
fn test_title_values_smarten_starting_apostrophe() {
    // Upstream provenance: CSL fixture `flipflop_StartingApostrophe`.
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "etfa-09".to_string(),
        ref_type: "book".to_string(),
        title: Some(
            "IEEE Conference on Emerging Technologies and Factory Automation (ETFA '09)"
                .to_string(),
        ),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(
        values.value,
        "IEEE Conference on Emerging Technologies and Factory Automation (ETFA ’09)"
    );
}

/// Tests the behavior of `test_title_values_smarten_french_apostrophes`.
#[test]
fn test_title_values_smarten_french_apostrophes() {
    // Upstream provenance: adapted from CSL fixture `flipflop_Apostrophes`.
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "egypt".to_string(),
        ref_type: "book".to_string(),
        title: Some(
            "Supplément aux annales du Service des Antiquités de l'Egypte, Cahier".to_string(),
        ),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(
        values.value,
        "Supplément aux annales du Service des Antiquités de l’Egypte, Cahier"
    );
}

/// Tests straight double quotes being smartened in plain title values.
#[test]
fn test_title_values_smarten_double_quotes() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "quoted-title".to_string(),
        ref_type: "book".to_string(),
        title: Some("The \"Parmenides\" dialogue".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(values.value, "The “Parmenides” dialogue");
}

/// Tests the behavior of mixed outer single and inner double title quotes.
#[test]
fn test_title_values_flip_flop_outer_single_inner_double_quotes() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "flip-flop-single-double".to_string(),
        ref_type: "book".to_string(),
        title: Some("'Some Title \"with something\"'".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(values.value, "‘Some Title “with something”’");
}

/// Tests the behavior of mixed outer double and inner single title quotes.
#[test]
fn test_title_values_flip_flop_outer_double_inner_single_quotes() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "flip-flop-double-single".to_string(),
        ref_type: "book".to_string(),
        title: Some("\"Some title 'with something'\"".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(values.value, "“Some title ‘with something’”");
}

/// Tests the behavior of preserving ambiguous double quotes in title values.
#[test]
fn test_title_values_preserve_ambiguous_double_quotes() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "record-title".to_string(),
        ref_type: "book".to_string(),
        title: Some("The 12\" record".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(values.value, "The 12\" record");
}

/// Tests the behavior of djot-marked title values.
#[test]
fn test_title_values_render_djot_markup_as_preformatted() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "djot-title".to_string(),
        ref_type: "book".to_string(),
        title: Some("_Homo sapiens_ and *modern* world".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(values.value, "_Homo sapiens_ and **modern** world");
    assert!(values.pre_formatted);
}

/// Tests the behavior of djot-marked title smart apostrophes.
#[test]
fn test_title_values_smarten_djot_text_leaves() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "djot-apostrophe".to_string(),
        ref_type: "book".to_string(),
        title: Some("_Plato's dialogue_".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(values.value, "_Plato’s dialogue_");
    assert!(values.pre_formatted);
}

/// Tests the behavior of djot-marked title smart double quotes.
#[test]
fn test_title_values_smarten_djot_double_quotes() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Citation,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "djot-double-quotes".to_string(),
        ref_type: "book".to_string(),
        title: Some("_\"Parmenides\" dialogue_".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(values.value, "_“Parmenides” dialogue_");
    assert!(values.pre_formatted);
}

/// Tests the behavior of inline title links suppressing whole-title autolinks.
#[test]
fn test_title_values_inline_link_suppresses_outer_title_link() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "djot-link".to_string(),
        ref_type: "book".to_string(),
        title: Some("[Linked title](https://example.com)".to_string()),
        doi: Some("10.1001/test".to_string()),
        ..Default::default()
    });

    let component = TemplateTitle {
        title: TitleType::Primary,
        links: Some(LinksConfig {
            doi: Some(true),
            target: Some(LinkTarget::Doi),
            anchor: Some(LinkAnchor::Title),
            ..Default::default()
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .expect("title value should render");
    assert_eq!(values.value, "Linked title");
    assert!(values.pre_formatted);
    assert_eq!(values.url, None);
}

/// Tests the behavior of `test_variable_hyperlink`.
#[test]
fn test_variable_hyperlink() {
    use citum_schema::options::LinksConfig;

    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "pub2024".to_string(),
        publisher: Some("MIT Press".to_string()),
        doi: Some("10.1234/pub".to_string()),
        ..Default::default()
    });

    let component = TemplateVariable {
        variable: SimpleVariable::Publisher,
        links: Some(LinksConfig {
            doi: Some(true),
            target: Some(LinkTarget::Doi),
            anchor: Some(LinkAnchor::Component),
            ..Default::default()
        }),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(values.value, "MIT Press");
    assert_eq!(values.url, Some("https://doi.org/10.1234/pub".to_string()));
}

#[test]
fn test_report_number_variable_uses_report_number_accessor() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "report-1".to_string(),
        ref_type: "report".to_string(),
        title: Some("Report".to_string()),
        issued: Some(DateVariable::year(2024)),
        number: Some("TR-7".to_string()),
        ..Default::default()
    });

    let number_component = TemplateNumber {
        number: NumberVariable::ReportNumber,
        ..Default::default()
    };
    let variable_component = TemplateVariable {
        variable: SimpleVariable::ReportNumber,
        ..Default::default()
    };

    assert_eq!(
        number_component
            .values::<PlainText>(&reference, &hints, &options)
            .expect("report number should render")
            .value,
        "TR-7"
    );
    assert_eq!(
        variable_component
            .values::<PlainText>(&reference, &hints, &options)
            .expect("report variable should render")
            .value,
        "TR-7"
    );
}

#[test]
fn test_number_variable_excludes_report_number_accessor() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "report-1".to_string(),
        ref_type: "report".to_string(),
        title: Some("Report".to_string()),
        issued: Some(DateVariable::year(2024)),
        number: Some("TR-7".to_string()),
        ..Default::default()
    });

    let number_component = TemplateNumber {
        number: NumberVariable::Number,
        ..Default::default()
    };
    let variable_component = TemplateVariable {
        variable: SimpleVariable::Number,
        ..Default::default()
    };

    assert!(
        number_component
            .values::<PlainText>(&reference, &hints, &options)
            .is_none()
    );
    assert!(
        variable_component
            .values::<PlainText>(&reference, &hints, &options)
            .is_none()
    );
}

#[test]
fn test_custom_number_variable_renders_from_custom_numbering_kind() {
    let config = make_config();
    let locale = citum_schema::Locale::from_yaml_str(
        r#"
locale: en-US
locators:
  reel:
    short:
      singular: "reel"
      plural: "reels"
"#,
    )
    .expect("custom locale should parse");
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference: Reference = serde_json::from_str(
        r#"{
            "class": "monograph",
            "type": "book",
            "title": "Film",
            "issued": "2024",
            "numbering": [
                { "type": "reel", "value": "3" }
            ]
        }"#,
    )
    .expect("reference should parse");

    let unlabeled = TemplateNumber {
        number: NumberVariable::Custom("reel".to_string()),
        ..Default::default()
    };
    let labeled = TemplateNumber {
        number: NumberVariable::Custom("reel".to_string()),
        label_form: Some(citum_schema::template::LabelForm::Short),
        ..Default::default()
    };

    assert_eq!(
        unlabeled
            .values::<PlainText>(&reference, &hints, &options)
            .expect("custom number should render")
            .value,
        "3"
    );
    let labeled_values = labeled
        .values::<PlainText>(&reference, &hints, &options)
        .expect("custom labeled number should render");
    assert_eq!(labeled_values.value, "3");
    assert_eq!(labeled_values.prefix, Some("reel ".to_string()));
}

#[test]
fn test_custom_number_variable_normalizes_manual_custom_key() {
    let config = make_config();
    let locale = Locale::from_yaml_str(
        r#"
locale: en-US
locators:
  reel:
    short:
      singular: "reel"
      plural: "reels"
"#,
    )
    .expect("custom locale should parse");
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };

    let reference: Reference = serde_json::from_str(
        r#"{
            "class": "monograph",
            "type": "book",
            "title": "Film",
            "issued": "2024",
            "numbering": [
                { "type": "reel", "value": "3" }
            ]
        }"#,
    )
    .expect("reference should parse");

    let number = TemplateNumber {
        number: NumberVariable::Custom("Reel".to_string()),
        label_form: Some(citum_schema::template::LabelForm::Short),
        ..Default::default()
    };

    let values = number
        .values::<PlainText>(&reference, &ProcHints::default(), &options)
        .expect("custom number should render");

    assert_eq!(values.value, "3");
    assert_eq!(values.prefix, Some("reel ".to_string()));
}

#[test]
fn test_template_number_gender_overrides_locator_label_resolution() {
    let config = make_config();
    let locale = make_gendered_locator_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "book-1".to_string(),
        ref_type: "book".to_string(),
        title: Some("Libro".to_string()),
        volume: Some(csl_legacy::csl_json::StringOrNumber::String(
            "1".to_string(),
        )),
        issued: Some(DateVariable::year(2024)),
        ..Default::default()
    });

    let masculine = TemplateNumber {
        number: NumberVariable::Volume,
        label_form: Some(citum_schema::template::LabelForm::Short),
        gender: Some(GrammaticalGender::Masculine),
        ..Default::default()
    };
    let feminine = TemplateNumber {
        number: NumberVariable::Volume,
        label_form: Some(citum_schema::template::LabelForm::Short),
        gender: Some(GrammaticalGender::Feminine),
        ..Default::default()
    };

    let masculine_values = masculine
        .values::<PlainText>(&reference, &hints, &options)
        .expect("masculine volume should render");
    let feminine_values = feminine
        .values::<PlainText>(&reference, &hints, &options)
        .expect("feminine volume should render");

    assert_eq!(masculine_values.prefix, Some("tomo ".to_string()));
    assert_eq!(feminine_values.prefix, Some("entrega ".to_string()));
}

#[test]
fn test_role_label_preset_applies_to_translator_component() {
    let mut config = make_config();
    let locale = make_locale();
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "translator-test".to_string(),
        ref_type: "book".to_string(),
        translator: Some(vec![Name::new("Muller", "Anna")]),
        ..Default::default()
    });

    if let Some(ref mut contributors) = config.contributors {
        contributors.role = Some(RoleOptions {
            preset: Some(RoleLabelPreset::LongSuffix),
            ..Default::default()
        });
    }

    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let component = TemplateContributor {
        contributor: ContributorRole::Translator,
        form: ContributorForm::Long,
        links: None,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();

    assert_eq!(values.value, "Muller, Anna");
    assert_eq!(values.suffix, Some(", translator".to_string()));
}

#[test]
fn test_translator_substitute_uses_locale_aware_role_label() {
    let mut config = make_config();
    let locale = make_locale();
    let hints = ProcHints::default();

    config.substitute = Some(SubstituteConfig::Explicit(Substitute {
        contributor_role_form: Some("long".to_string()),
        template: vec![SubstituteKey::Translator],
        overrides: std::collections::HashMap::new(),
        role_substitute: std::collections::HashMap::new(),
    }));

    let reference = Reference::from(LegacyReference {
        id: "translator-substitute".to_string(),
        ref_type: "book".to_string(),
        translator: Some(vec![Name::new("Muller", "Anna")]),
        ..Default::default()
    });

    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let component = TemplateContributor {
        contributor: ContributorRole::Author,
        form: ContributorForm::Long,
        links: None,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();

    assert_eq!(values.value, "Muller, Anna");
    assert_eq!(values.suffix, Some(", translator".to_string()));
    assert_eq!(
        values.substituted_key,
        Some("contributor:Translator".to_string())
    );
}

#[test]
fn test_editor_substitute_suppresses_verb_prefix_role_label() {
    let mut config = make_config();
    let locale = make_locale();
    let hints = ProcHints::default();

    config.substitute = Some(SubstituteConfig::Preset(
        citum_schema::presets::SubstitutePreset::Standard,
    ));

    if let Some(ref mut contributors) = config.contributors {
        contributors.role = Some(RoleOptions {
            preset: Some(RoleLabelPreset::VerbPrefix),
            ..Default::default()
        });
    }

    let reference = Reference::from(LegacyReference {
        id: "editor-substitute".to_string(),
        ref_type: "book".to_string(),
        editor: Some(vec![Name::new("Grimm", "Jacob")]),
        ..Default::default()
    });

    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let component = TemplateContributor {
        contributor: ContributorRole::Author,
        form: ContributorForm::Long,
        links: None,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();

    assert_eq!(values.value, "Grimm, Jacob");
    assert_eq!(values.prefix, None);
    assert_eq!(values.suffix, None);
    assert_eq!(
        values.substituted_key,
        Some("contributor:Editor".to_string())
    );
}

#[test]
fn test_editor_component_keeps_verb_prefix_role_label() {
    let mut config = make_config();
    let locale = make_locale();
    let hints = ProcHints::default();

    if let Some(ref mut contributors) = config.contributors {
        contributors.role = Some(RoleOptions {
            preset: Some(RoleLabelPreset::VerbPrefix),
            ..Default::default()
        });
    }

    let reference = Reference::from(LegacyReference {
        id: "editor-component".to_string(),
        ref_type: "book".to_string(),
        editor: Some(vec![Name::new("Grimm", "Jacob")]),
        ..Default::default()
    });

    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        links: None,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();

    assert_eq!(values.value, "Grimm, Jacob");
    assert_eq!(values.prefix, Some("edited by ".to_string()));
    assert_eq!(values.suffix, None);
}

#[test]
fn test_role_substitute_normalizes_primary_role_lookup_keys() {
    let mut config = make_config();
    config.substitute = Some(SubstituteConfig::Explicit(Substitute {
        role_substitute: std::collections::HashMap::from([(
            "container_author".to_string(),
            vec!["Editor".to_string()],
        )]),
        ..Default::default()
    }));

    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = Reference::from(LegacyReference {
        id: "normalized-role-key".to_string(),
        ref_type: "chapter".to_string(),
        title: Some("Collected Work".to_string()),
        editor: Some(vec![Name::new("Editor", "Avery")]),
        ..Default::default()
    });
    let hints = ProcHints::default();

    let component = TemplateContributor {
        contributor: ContributorRole::ContainerAuthor,
        form: ContributorForm::Long,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();

    assert_eq!(values.value, "Editor, Avery");
    assert_eq!(values.suffix, None);
}

#[test]
fn test_role_specific_name_order_applies_in_substitute_path() {
    let mut config = make_config();
    let locale = make_locale();
    let hints = ProcHints::default();

    config.substitute = Some(SubstituteConfig::Explicit(Substitute {
        contributor_role_form: Some("short".to_string()),
        template: vec![SubstituteKey::Translator],
        overrides: std::collections::HashMap::new(),
        role_substitute: std::collections::HashMap::new(),
    }));

    if let Some(ref mut contributors) = config.contributors {
        contributors.role = Some(RoleOptions {
            roles: Some({
                let mut roles = std::collections::HashMap::new();
                roles.insert(
                    "translator".to_string(),
                    RoleRendering {
                        name_order: Some(NameOrder::GivenFirst),
                        ..Default::default()
                    },
                );
                roles
            }),
            ..Default::default()
        });
    }

    let reference = Reference::from(LegacyReference {
        id: "translator-name-order".to_string(),
        ref_type: "book".to_string(),
        translator: Some(vec![Name::new("Muller", "Anna")]),
        ..Default::default()
    });

    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let component = TemplateContributor {
        contributor: ContributorRole::Author,
        form: ContributorForm::Long,
        links: None,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();

    assert_eq!(values.value, "Anna Muller");
    assert_eq!(values.suffix, Some(" (Trans.)".to_string()));
}

/// Tests the behavior of `test_term_values`.
#[test]
fn test_term_values() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let reference = make_reference();
    let hints = ProcHints::default();

    let component = TemplateTerm {
        term: GeneralTerm::In,
        form: Some(TermForm::Long),
        custom: None,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(values.value, "in");
}

/// Tests the behavior of `test_template_list_term_suppression`.
#[test]
fn test_template_list_term_suppression() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    // Reference with no editor
    let reference = make_reference();
    let hints = ProcHints::default();

    let component = TemplateGroup {
        group: vec![
            TemplateComponent::Term(TemplateTerm {
                term: GeneralTerm::In,
                custom: None,
                ..Default::default()
            }),
            TemplateComponent::Contributor(TemplateContributor {
                contributor: ContributorRole::Editor,
                ..Default::default()
            }),
        ],
        delimiter: Some(DelimiterPunctuation::Space),
        ..Default::default()
    };

    let values = component.values::<PlainText>(&reference, &hints, &options);
    // Should be None because only the term "In" would render, and it's suppressed if no content-bearing items are present
    assert!(values.is_none());
}

/// Tests the behavior of `test_date_fallback`.
#[test]
fn test_date_fallback() {
    let config = make_config();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    // Reference with NO issued date
    let reference = Reference::from(LegacyReference {
        id: "no-date".to_string(),
        ref_type: "book".to_string(),
        author: Some(vec![Name::new("Aristotle", "Ancient")]),
        title: Some("Poetics".to_string()),
        ..Default::default()
    });
    let hints = ProcHints::default();

    let component = TemplateDate {
        date: TemplateDateVar::Issued,
        form: DateForm::Year,
        fallback: Some(vec![TemplateComponent::Term(TemplateTerm {
            term: GeneralTerm::NoDate,
            form: Some(TermForm::Short),
            ..Default::default()
        })]),
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(values.value, "n.d.");
}

/// Tests the behavior of `test_strip_periods_global_config`.
#[test]
fn test_strip_periods_global_config() {
    let mut config = make_config();
    config.strip_periods = Some(true);
    let locale = make_locale();
    let reference = Reference::from(LegacyReference {
        id: "editor1".to_string(),
        ref_type: "book".to_string(),
        editor: Some(vec![Name::new("Smith", "John")]),
        title: Some("A Book".to_string()),
        issued: Some(DateVariable::year(2020)),
        publisher: Some("Publisher".to_string()),
        ..Default::default()
    });
    let hints = ProcHints::default();

    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // Should have "(ed)" instead of "(ed.)" due to strip_periods
    assert!(values.suffix.is_some());
    assert_eq!(values.suffix.as_ref().unwrap(), " (ed)");
}

/// Tests the behavior of `test_strip_periods_component_override`.
#[test]
fn test_strip_periods_component_override() {
    let mut config = make_config();
    config.strip_periods = Some(false); // Global is false
    let locale = make_locale();
    let reference = Reference::from(LegacyReference {
        id: "editor1".to_string(),
        ref_type: "book".to_string(),
        editor: Some(vec![Name::new("Smith", "John")]),
        title: Some("A Book".to_string()),
        issued: Some(DateVariable::year(2020)),
        publisher: Some("Publisher".to_string()),
        ..Default::default()
    });
    let hints = ProcHints::default();

    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };

    // Component overrides global setting
    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        rendering: Rendering {
            strip_periods: Some(true),
            ..Default::default()
        },
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // Should strip periods because component overrides global
    assert!(values.suffix.is_some());
    assert_eq!(values.suffix.as_ref().unwrap(), " (ed)");
}

/// Tests the behavior of `test_strip_periods_no_strip_by_default`.
#[test]
fn test_strip_periods_no_strip_by_default() {
    let config = make_config();
    let locale = make_locale();
    let reference = Reference::from(LegacyReference {
        id: "editor1".to_string(),
        ref_type: "book".to_string(),
        editor: Some(vec![Name::new("Smith", "John")]),
        title: Some("A Book".to_string()),
        issued: Some(DateVariable::year(2020)),
        publisher: Some("Publisher".to_string()),
        ..Default::default()
    });
    let hints = ProcHints::default();

    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };

    let component = TemplateContributor {
        contributor: ContributorRole::Editor,
        form: ContributorForm::Long,
        ..Default::default()
    };

    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // Should preserve periods by default
    assert!(values.suffix.is_some());
    assert_eq!(values.suffix.as_ref().unwrap(), " (ed.)");
}

/// Tests the behavior of `test_strip_trailing_periods`.
#[test]
fn test_strip_trailing_periods() {
    assert_eq!(strip_trailing_periods("test."), "test");
    assert_eq!(strip_trailing_periods("test"), "test");
    assert_eq!(strip_trailing_periods("Ph.D."), "Ph.D");
    assert_eq!(strip_trailing_periods("A.B.C."), "A.B.C");
    assert_eq!(strip_trailing_periods("..."), "");
}

/// Tests the behavior of `test_should_strip_periods_precedence`.
#[test]
fn test_should_strip_periods_precedence() {
    let config = Config {
        strip_periods: Some(true),
        ..Default::default()
    };
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };

    // Component override takes precedence
    let rendering_override_true = Rendering {
        strip_periods: Some(true),
        ..Default::default()
    };
    assert!(should_strip_periods(&rendering_override_true, &options));

    let rendering_override_false = Rendering {
        strip_periods: Some(false),
        ..Default::default()
    };
    assert!(!should_strip_periods(&rendering_override_false, &options));

    // Falls back to config when component has None
    let rendering_default = Rendering::default();
    assert!(should_strip_periods(&rendering_default, &options));

    // Defaults to false when both are None
    let config_none = Config::default();
    let options_none = RenderOptions {
        config: &config_none,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    assert!(!should_strip_periods(&rendering_default, &options_none));
}

/// Tests the behavior of `test_sort_separator_space`.
#[test]
fn test_sort_separator_space() {
    use citum_schema::options::DisplayAsSort;

    // Test sort_separator directly via format_single_name with inverted display
    let name = FlatName {
        family: Some("Smith".to_string()),
        given: Some("John".to_string()),
        ..Default::default()
    };

    // Test with space separator: should produce "Smith J" (no comma)
    let sep_space = " ".to_string();
    let init_empty = String::new();
    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        Some(&init_empty),
        None,
        Some(NameForm::Initials),
        None,
        Some(&sep_space),
    );
    let result_space =
        contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(result_space, "Smith J");

    // Test with default (no sort_separator set): should produce "Smith, J" (with comma)
    let ctx = make_name_format_context(
        Some(DisplayAsSort::All),
        None,
        Some(&init_empty),
        None,
        Some(NameForm::Initials),
        None,
        None,
    );
    let result_default =
        contributor::format_single_name(&name, &ContributorForm::Long, 0, &ctx, false);
    assert_eq!(result_default, "Smith, J");
}

/// Tests the behavior of `preferred_transliteration_exact_match`.
#[test]
fn preferred_transliteration_exact_match() {
    use citum_schema::reference::types::{MultilingualComplex, MultilingualString};
    use std::collections::HashMap;

    let s = MultilingualString::Complex(MultilingualComplex {
        original: "战争".to_string(),
        lang: None,
        transliterations: vec![
            ("zh-Latn-wadegile".to_string(), "Chan-cheng".to_string()),
            ("zh-Latn-pinyin".to_string(), "Zhànzhēng".to_string()),
        ]
        .into_iter()
        .collect(),
        translations: HashMap::new(),
    });
    let result = super::resolve_multilingual_string(
        &s,
        Some(&citum_schema::options::MultilingualMode::Transliterated),
        Some(&["zh-Latn-wadegile".to_string()]),
        None,
        "en",
    );
    assert_eq!(result, "Chan-cheng");
}

/// Tests the behavior of `preferred_transliteration_substring_match`.
#[test]
fn preferred_transliteration_substring_match() {
    use citum_schema::reference::types::{MultilingualComplex, MultilingualString};
    use std::collections::HashMap;

    let s = MultilingualString::Complex(MultilingualComplex {
        original: "战争".to_string(),
        lang: None,
        transliterations: vec![("zh-Latn-pinyin".to_string(), "Zhànzhēng".to_string())]
            .into_iter()
            .collect(),
        translations: HashMap::new(),
    });
    let result = super::resolve_multilingual_string(
        &s,
        Some(&citum_schema::options::MultilingualMode::Transliterated),
        Some(&["zh-Latn".to_string()]),
        None,
        "en",
    );
    assert_eq!(result, "Zhànzhēng");
}

/// Tests the behavior of `preferred_transliteration_fallback_to_preferred_script`.
#[test]
fn preferred_transliteration_fallback_to_preferred_script() {
    use citum_schema::reference::types::{MultilingualComplex, MultilingualString};
    use std::collections::HashMap;

    let s = MultilingualString::Complex(MultilingualComplex {
        original: "战争".to_string(),
        lang: None,
        transliterations: vec![("zh-Latn-pinyin".to_string(), "Zhànzhēng".to_string())]
            .into_iter()
            .collect(),
        translations: HashMap::new(),
    });
    let script = "Latn".to_string();
    let result = super::resolve_multilingual_string(
        &s,
        Some(&citum_schema::options::MultilingualMode::Transliterated),
        None,
        Some(&script),
        "en",
    );
    assert_eq!(result, "Zhànzhēng");
}

/// Tests the behavior of `preferred_transliteration_fallback_to_original`.
#[test]
fn preferred_transliteration_fallback_to_original() {
    use citum_schema::reference::types::{MultilingualComplex, MultilingualString};
    use std::collections::HashMap;

    let s = MultilingualString::Complex(MultilingualComplex {
        original: "战争".to_string(),
        lang: None,
        transliterations: HashMap::new(),
        translations: HashMap::new(),
    });
    let result = super::resolve_multilingual_string(
        &s,
        Some(&citum_schema::options::MultilingualMode::Transliterated),
        None,
        None,
        "en",
    );
    assert_eq!(result, "战争");
}

/// Tests `int_to_letter` edge case: zero input returns None.
///
/// Verifies that `int_to_letter` correctly rejects invalid input (0).
#[test]
fn test_int_to_letter_zero_edge_case() {
    assert_eq!(int_to_letter(0), None);
}

/// Tests `int_to_letter` for large numbers (triple letters and beyond).
///
/// Verifies that `int_to_letter` correctly converts 27→aa, 52→az, 53→ba,
/// demonstrating base-26 conversion with wrapping.
#[test]
fn test_int_to_letter_large_number() {
    assert_eq!(int_to_letter(27), Some("aa".to_string()));
    assert_eq!(int_to_letter(52), Some("az".to_string()));
    assert_eq!(int_to_letter(53), Some("ba".to_string()));
}

/// Tests locator label selection with different value formats.
///
/// Verifies that `check_plural` correctly identifies plural locators:
/// ranges (hyphens), lists (commas), and conjunctions (ampersands).
#[test]
fn test_locator_label_selection_comprehensive() {
    use citum_schema::citation::LocatorType;

    // Single value: not plural
    assert!(!crate::values::number::check_plural(
        "15",
        &LocatorType::Page
    ));

    // Range with hyphen: plural
    assert!(crate::values::number::check_plural(
        "15-20",
        &LocatorType::Page
    ));

    // List with comma: plural
    assert!(crate::values::number::check_plural(
        "15, 18",
        &LocatorType::Page
    ));

    // Conjunction with ampersand: plural
    assert!(crate::values::number::check_plural(
        "15 & 18",
        &LocatorType::Page
    ));
}

// ── Title text-case tests ──────────────────────────────────────────────

fn make_config_with_titles(titles: citum_schema::options::TitlesConfig) -> Config {
    Config {
        titles: Some(titles),
        ..Default::default()
    }
}

fn title_value_with_config(title_str: &str, ref_type: &str, config: &Config) -> String {
    let locale = make_locale();
    let options = RenderOptions {
        config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();
    let reference = Reference::from(LegacyReference {
        id: "tc".to_string(),
        ref_type: ref_type.to_string(),
        title: Some(title_str.to_string()),
        ..Default::default()
    });
    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };
    component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap()
        .value
}

#[test]
fn test_text_case_sentence_apa_basic() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};
    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::SentenceApa),
            ..Default::default()
        }),
        ..Default::default()
    });
    let result =
        title_value_with_config("The Structure of Scientific Revolutions", "book", &config);
    assert_eq!(result, "The structure of scientific revolutions");
}

#[test]
fn test_text_case_sentence_nlm_basic() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};
    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::SentenceNlm),
            ..Default::default()
        }),
        ..Default::default()
    });
    let result =
        title_value_with_config("The Structure of Scientific Revolutions", "book", &config);
    assert_eq!(result, "The structure of scientific revolutions");
}

#[test]
fn test_text_case_title_case() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};
    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::Title),
            ..Default::default()
        }),
        ..Default::default()
    });
    let result = title_value_with_config("the quick brown fox", "book", &config);
    assert_eq!(result, "The Quick Brown Fox");
}

#[test]
fn test_text_case_as_is() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};
    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::AsIs),
            ..Default::default()
        }),
        ..Default::default()
    });
    let result =
        title_value_with_config("The Structure of Scientific Revolutions", "book", &config);
    assert_eq!(result, "The Structure of Scientific Revolutions");
}

#[test]
fn test_text_case_nocase_protection_in_djot() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};
    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::SentenceApa),
            ..Default::default()
        }),
        ..Default::default()
    });
    // [mRNA]{.nocase} should be preserved even under sentence case
    let result = title_value_with_config(
        "The Role of [mRNA]{.nocase} in Modern Science",
        "book",
        &config,
    );
    assert_eq!(result, "The role of mRNA in modern science");
}

#[test]
fn test_text_case_nocase_nested_in_emphasis() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};
    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::SentenceApa),
            ..Default::default()
        }),
        ..Default::default()
    });
    let result = title_value_with_config(
        "_Homo Sapiens_ and [DNA]{.nocase} Replication",
        "book",
        &config,
    );
    // Emphasis content gets sentence case (first word capitalized), DNA preserved
    assert_eq!(result, "_Homo sapiens_ and DNA replication");
}

#[test]
fn test_text_case_leading_nocase_advances_state() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};
    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::SentenceApa),
            ..Default::default()
        }),
        ..Default::default()
    });
    // Leading .nocase span: the first-word state should still advance,
    // so "replication" after the span is NOT capitalized.
    let result = title_value_with_config(
        "[DNA]{.nocase} Replication in Modern Science",
        "book",
        &config,
    );
    assert_eq!(result, "DNA replication in modern science");
}

#[test]
fn test_text_case_structured_title_sentence_apa() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};
    use citum_schema::reference::types::{StructuredTitle, Subtitle, Title};

    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::SentenceApa),
            ..Default::default()
        }),
        ..Default::default()
    });
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    // Build a native Citum structured title reference
    let mut reference = Reference::from(LegacyReference {
        id: "structured".to_string(),
        ref_type: "book".to_string(),
        title: Some("placeholder".to_string()),
        ..Default::default()
    });
    // Replace with structured title
    if let Reference::Monograph(ref mut m) = reference {
        m.title = Some(Title::Structured(StructuredTitle {
            full: None,
            main: "Understanding Citation Systems".to_string(),
            sub: Subtitle::Vector(vec![
                "History and Practice".to_string(),
                "A Comparative View".to_string(),
            ]),
        }));
    }

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };
    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // APA: first word of main + each subtitle capitalized
    assert_eq!(
        values.value,
        "Understanding citation systems: History and practice: A comparative view"
    );
}

#[test]
fn test_text_case_structured_title_sentence_nlm() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};
    use citum_schema::reference::types::{StructuredTitle, Subtitle, Title};

    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::SentenceNlm),
            ..Default::default()
        }),
        ..Default::default()
    });
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let mut reference = Reference::from(LegacyReference {
        id: "structured".to_string(),
        ref_type: "book".to_string(),
        title: Some("placeholder".to_string()),
        ..Default::default()
    });
    if let Reference::Monograph(ref mut m) = reference {
        m.title = Some(Title::Structured(StructuredTitle {
            full: None,
            main: "Understanding Citation Systems".to_string(),
            sub: Subtitle::String("History and Practice".to_string()),
        }));
    }

    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };
    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // NLM: main sentence-cased, subtitle lowercased only
    assert_eq!(
        values.value,
        "Understanding citation systems: history and practice"
    );
}

#[test]
fn test_text_case_non_english_falls_back_to_as_is() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};

    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::SentenceApa),
            ..Default::default()
        }),
        ..Default::default()
    });
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let reference = Reference::from(LegacyReference {
        id: "german".to_string(),
        ref_type: "book".to_string(),
        title: Some("Die Geschichte der Molekularbiologie".to_string()),
        language: Some("de".into()),
        ..Default::default()
    });
    let component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };
    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    // German: should be as-is (no English sentence case applied)
    assert_eq!(values.value, "Die Geschichte der Molekularbiologie");
}

#[test]
fn test_text_case_template_level_override() {
    use citum_schema::options::titles::{TextCase, TitleRendering, TitlesConfig};

    // Global config says as-is
    let config = make_config_with_titles(TitlesConfig {
        monograph: Some(TitleRendering {
            text_case: Some(TextCase::AsIs),
            ..Default::default()
        }),
        ..Default::default()
    });
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();
    let reference = Reference::from(LegacyReference {
        id: "override".to_string(),
        ref_type: "book".to_string(),
        title: Some("The Quick Brown Fox".to_string()),
        ..Default::default()
    });
    // Template-level rendering overrides to lowercase
    let component = TemplateTitle {
        title: TitleType::Primary,
        rendering: Rendering {
            text_case: Some(TextCase::Lowercase),
            ..Default::default()
        },
        ..Default::default()
    };
    let values = component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(values.value, "the quick brown fox");
}

#[test]
fn test_text_case_no_config_means_no_transform() {
    let config = Config::default();
    let result = title_value_with_config("The Quick Brown Fox", "book", &config);
    // No text-case configured: title rendered as-is (just smart quotes)
    assert_eq!(result, "The Quick Brown Fox");
}

/// `form: short` on a structured title returns only the main part, not the
/// full compound string.
#[test]
fn test_structured_title_form_short_returns_main_only() {
    use citum_schema::reference::types::{StructuredTitle, Subtitle, Title};

    let config = Config::default();
    let locale = make_locale();
    let options = RenderOptions {
        config: &config,
        bibliography_config: None,
        locale: &locale,
        context: RenderContext::Bibliography,
        mode: citum_schema::citation::CitationMode::NonIntegral,
        suppress_author: false,
        locator_raw: None,
        ref_type: None,
        show_semantics: true,
        current_template_index: None,
    };
    let hints = ProcHints::default();

    let mut reference = Reference::from(LegacyReference {
        id: "treaty".to_string(),
        ref_type: "treaty".to_string(),
        title: Some("placeholder".to_string()),
        ..Default::default()
    });
    if let Reference::Treaty(ref mut m) = reference {
        m.title = Some(Title::Structured(StructuredTitle {
            full: None,
            main: "Homeland Security Act of 2002".to_string(),
            sub: Subtitle::Vector(vec![
                "Hearings on H.R. 5005".to_string(),
                "Day 3".to_string(),
            ]),
        }));
    }

    // Without form: short — full compound title
    let full_component = TemplateTitle {
        title: TitleType::Primary,
        ..Default::default()
    };
    let full = full_component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(
        full.value,
        "Homeland Security Act of 2002: Hearings on H.R. 5005: Day 3"
    );

    // With form: short — main only, subtitles suppressed
    let short_component = TemplateTitle {
        title: TitleType::Primary,
        form: Some(TitleForm::Short),
        ..Default::default()
    };
    let short = short_component
        .values::<PlainText>(&reference, &hints, &options)
        .unwrap();
    assert_eq!(short.value, "Homeland Security Act of 2002");
}
