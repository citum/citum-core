#![allow(missing_docs, reason = "test")]

use citum_engine::grouping::GroupSorter;
use citum_engine::processor::disambiguation::Disambiguator;
use citum_engine::render::plain::PlainText;
use citum_engine::{
    Bibliography, Citation, CitationItem, Contributor, EdtfString, Locale, Monograph,
    MonographType, MultilingualString, Processor, Reference, StructuredName, Title,
};
use citum_schema::grouping::{GroupSort, GroupSortKey, NameSortOrder, SortKey as GroupSortKeyKind};
use citum_schema::options::{
    BibliographyOptions, Config, Disambiguation, Group, LabelConfig, LabelPreset, Processing,
    ProcessingCustom, Sort, SortEntry, SortKey, SortSpec, bibliography::CompoundNumericConfig,
};
use citum_schema::{
    BibliographySpec, InputBibliography, Style, StyleInfo,
    template::{
        ContributorRole, NumberVariable, Rendering, SimpleVariable, TemplateComponent,
        TemplateContributor, TemplateNumber, TemplateTitle, TemplateVariable, TitleType,
        WrapPunctuation,
    },
};
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use csl_legacy::csl_json::Reference as LegacyReference;
use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;

fn bench_rendering(c: &mut Criterion) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.parent().unwrap().parent().unwrap();

    // Load style
    let style_path = root_dir.join("styles/apa-7th.yaml");
    let style_yaml = fs::read_to_string(&style_path).expect("failed to read apa-7th.yaml");
    let style: Style = serde_yaml::from_str(&style_yaml).expect("failed to parse style yaml");

    // Load bibliography
    let bib_path = root_dir.join("examples/comprehensive.yaml");
    let bib_yaml = fs::read_to_string(&bib_path).expect("failed to read comprehensive.yaml");
    let input_bib: InputBibliography =
        serde_yaml::from_str(&bib_yaml).expect("failed to parse bib yaml");

    // Convert to processor bibliography
    let mut bib = Bibliography::new();
    for r in input_bib.references {
        if let Some(id) = r.id() {
            bib.insert(id.clone(), r);
        }
    }

    // Benchmark Citation Processing (single item)
    let first_id = bib.keys().next().unwrap().clone();
    let citation = Citation {
        items: vec![CitationItem {
            id: first_id,
            ..Default::default()
        }],
        ..Default::default()
    };

    c.bench_function("Process Citation (APA)", |b| {
        let processor = Processor::new(style.clone(), bib.clone());
        b.iter(|| {
            processor.process_citation(black_box(&citation)).unwrap();
        });
    });

    // Benchmark Bibliography Processing (full set)
    c.bench_function("Process Bibliography (APA, 10 items)", |b| {
        let processor = Processor::new(style.clone(), bib.clone());
        b.iter(|| {
            processor.process_references();
        });
    });

    bench_disambiguation(c);
    bench_group_sorting(c);
    bench_bibliography_type_variants(c);
    bench_compound_bibliography(c);
}

fn bench_disambiguation(c: &mut Criterion) {
    let locale = Locale::en_us();
    let no_collision_bib = make_no_collision_bibliography();
    let givenname_bib = make_givenname_collision_bibliography();
    let partition_bib = make_partition_collision_bibliography();
    let label_bib = make_label_collision_bibliography();
    let default_title_sort_bib = make_default_title_sort_collision_bibliography();

    let no_collision_config = Config::default();
    let givenname_config = make_custom_config(true, true, true);
    let partition_config = make_custom_config(true, false, true);
    let label_config = Config {
        processing: Some(Processing::Label(LabelConfig {
            preset: LabelPreset::Din,
            ..Default::default()
        })),
        ..Default::default()
    };
    let default_title_sort_config = Config {
        processing: Some(Processing::Custom(ProcessingCustom {
            disambiguate: Some(Disambiguation {
                names: false,
                add_givenname: false,
                year_suffix: true,
            }),
            ..Default::default()
        })),
        ..Default::default()
    };

    let mut bench_group = c.benchmark_group("Disambiguator::calculate_hints");
    bench_group.bench_function("No collisions", |b| {
        let disambiguator = Disambiguator::new(&no_collision_bib, &no_collision_config, &locale);
        b.iter(|| {
            black_box(disambiguator.calculate_hints());
        });
    });
    bench_group.bench_function("Given-name collisions", |b| {
        let disambiguator = Disambiguator::new(&givenname_bib, &givenname_config, &locale);
        b.iter(|| {
            black_box(disambiguator.calculate_hints());
        });
    });
    bench_group.bench_function("Name partition with suffix fallback", |b| {
        let disambiguator = Disambiguator::new(&partition_bib, &partition_config, &locale);
        b.iter(|| {
            black_box(disambiguator.calculate_hints());
        });
    });
    bench_group.bench_function("Label-mode suffix collisions", |b| {
        let disambiguator = Disambiguator::new(&label_bib, &label_config, &locale);
        b.iter(|| {
            black_box(disambiguator.calculate_hints());
        });
    });
    bench_group.bench_function("Default title-order suffix collisions", |b| {
        let disambiguator =
            Disambiguator::new(&default_title_sort_bib, &default_title_sort_config, &locale);
        b.iter(|| {
            black_box(disambiguator.calculate_hints());
        });
    });
    bench_group.finish();
}

fn make_custom_config(names: bool, add_givenname: bool, year_suffix: bool) -> Config {
    Config {
        processing: Some(Processing::Custom(ProcessingCustom {
            sort: Some(SortEntry::Explicit(Sort {
                shorten_names: false,
                render_substitutions: false,
                template: vec![
                    SortSpec {
                        key: SortKey::Author,
                        ascending: true,
                    },
                    SortSpec {
                        key: SortKey::Year,
                        ascending: true,
                    },
                ],
            })),
            group: Some(Group {
                template: vec![SortKey::Author, SortKey::Year],
            }),
            disambiguate: Some(Disambiguation {
                names,
                add_givenname,
                year_suffix,
            }),
        })),
        ..Default::default()
    }
}

fn make_ref(id: &str, family: &str, given: &str, year: i32) -> Reference {
    make_ref_with_title(id, family, given, year, &format!("Title {id}"))
}

fn make_ref_with_title(id: &str, family: &str, given: &str, year: i32, title: &str) -> Reference {
    Reference::Monograph(Box::new(Monograph {
        id: Some(id.to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single(title.to_string())),
        container_title: None,
        author: Some(Contributor::StructuredName(StructuredName {
            family: MultilingualString::Simple(family.to_string()),
            given: MultilingualString::Simple(given.to_string()),
            suffix: None,
            dropping_particle: None,
            non_dropping_particle: None,
        })),
        editor: None,
        translator: None,
        recipient: None,
        interviewer: None,
        issued: EdtfString(year.to_string()),
        publisher: None,
        url: None,
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        isbn: None,
        doi: None,
        edition: None,
        report_number: None,
        collection_number: None,
        genre: None,
        medium: None,
        archive: None,
        archive_location: None,
        keywords: None,
        original_date: None,
        original_title: None,
        ads_bibcode: None,
    }))
}

fn make_no_collision_bibliography() -> Bibliography {
    let mut bib = Bibliography::new();
    for (id, family, given, year) in [
        ("adams2020", "Adams", "Alice", 2020),
        ("baker2021", "Baker", "Bob", 2021),
        ("clark2022", "Clark", "Cara", 2022),
        ("davis2023", "Davis", "Drew", 2023),
    ] {
        bib.insert(id.to_string(), make_ref(id, family, given, year));
    }
    bib
}

fn make_givenname_collision_bibliography() -> Bibliography {
    let mut bib = Bibliography::new();
    for (id, given) in [("smith2020a", "John"), ("smith2020b", "Alice")] {
        bib.insert(id.to_string(), make_ref(id, "Smith", given, 2020));
    }
    bib
}

fn make_partition_collision_bibliography() -> Bibliography {
    let mut bib = Bibliography::new();
    for (id, family, given, year) in [
        ("smith-jones-2020", "Smith,Jones", "John,Peter", 2020),
        ("smith-brown-a-2020", "Smith,Brown", "John,Alice", 2020),
        ("smith-brown-b-2020", "Smith,Brown", "John,Adam", 2020),
    ] {
        bib.insert(
            id.to_string(),
            make_multi_author_ref(id, family.split(','), given.split(','), year),
        );
    }
    bib
}

fn make_label_collision_bibliography() -> Bibliography {
    let mut bib = Bibliography::new();
    for id in ["kuhn1962a", "kuhn1962b"] {
        bib.insert(id.to_string(), make_ref(id, "Kuhn", "Thomas", 1962));
    }
    bib
}

fn make_default_title_sort_collision_bibliography() -> Bibliography {
    let mut bib = Bibliography::new();
    for (id, title) in [
        ("smith2020-zeta", "Zeta title"),
        ("smith2020-alpha", "Alpha title"),
        ("smith2020-gamma", "Gamma title"),
    ] {
        bib.insert(
            id.to_string(),
            make_ref_with_title(id, "Smith", "John", 2020, title),
        );
    }
    bib
}

fn make_multi_author_ref<'a, I, J>(id: &str, families: I, givens: J, year: i32) -> Reference
where
    I: IntoIterator<Item = &'a str>,
    J: IntoIterator<Item = &'a str>,
{
    let authors = families
        .into_iter()
        .zip(givens)
        .map(|(family, given)| {
            Contributor::StructuredName(StructuredName {
                family: MultilingualString::Simple(family.to_string()),
                given: MultilingualString::Simple(given.to_string()),
                suffix: None,
                dropping_particle: None,
                non_dropping_particle: None,
            })
        })
        .collect();

    let title = format!("Title {id}");
    Reference::Monograph(Box::new(Monograph {
        id: Some(id.to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single(title)),
        container_title: None,
        author: Some(Contributor::ContributorList(citum_engine::ContributorList(
            authors,
        ))),
        editor: None,
        translator: None,
        recipient: None,
        interviewer: None,
        issued: EdtfString(year.to_string()),
        publisher: None,
        url: None,
        accessed: None,
        language: None,
        field_languages: Default::default(),
        note: None,
        isbn: None,
        doi: None,
        edition: None,
        report_number: None,
        collection_number: None,
        genre: None,
        medium: None,
        archive: None,
        archive_location: None,
        keywords: None,
        original_date: None,
        original_title: None,
        ads_bibcode: None,
    }))
}

fn bench_group_sorting(c: &mut Criterion) {
    let locale = Locale::en_us();
    let sorter = GroupSorter::new(&locale);
    let bibliography = make_group_sort_bibliography();
    let references: Vec<&Reference> = bibliography.values().collect();
    let sort_spec = GroupSort {
        template: vec![
            GroupSortKey {
                key: GroupSortKeyKind::RefType,
                ascending: true,
                order: Some(vec![
                    "legal-case".to_string(),
                    "article-journal".to_string(),
                    "book".to_string(),
                ]),
                sort_order: None,
            },
            GroupSortKey {
                key: GroupSortKeyKind::Author,
                ascending: true,
                order: None,
                sort_order: Some(NameSortOrder::FamilyGiven),
            },
            GroupSortKey {
                key: GroupSortKeyKind::Issued,
                ascending: false,
                order: None,
                sort_order: None,
            },
        ],
    };

    let mut bench_group = c.benchmark_group("GroupSorter::sort_references");
    bench_group.bench_function("Explicit type order + author", |b| {
        b.iter_batched(
            || references.clone(),
            |refs| black_box(sorter.sort_references(refs, &sort_spec)),
            BatchSize::SmallInput,
        );
    });
    bench_group.finish();
}

fn bench_bibliography_type_variants(c: &mut Criterion) {
    let processor = Processor::new(
        build_type_variant_bench_style(),
        make_type_variant_bibliography(),
    );
    let reference = processor
        .bibliography
        .get("article-no-pages")
        .expect("type-variant benchmark reference should exist");

    let mut bench_group = c.benchmark_group("Renderer::process_bibliography_entry");
    bench_group.bench_function("Type variant + article-journal fallback", |b| {
        b.iter(|| {
            black_box(processor.process_bibliography_entry_with_format::<PlainText>(reference, 1));
        });
    });
    bench_group.finish();
}

fn bench_compound_bibliography(c: &mut Criterion) {
    let processor = Processor::with_compound_sets(
        build_compound_bench_style(),
        make_compound_bibliography(),
        make_compound_sets(),
    );

    let mut bench_group = c.benchmark_group("Processor::render_bibliography_with_format");
    bench_group.bench_function("Compound bibliography merge", |b| {
        b.iter(|| {
            black_box(processor.render_bibliography_with_format::<PlainText>());
        });
    });
    bench_group.finish();
}

fn make_group_sort_bibliography() -> Bibliography {
    let mut bibliography = Bibliography::new();
    for (id, ref_type, family, title, year) in [
        ("legal-alpha", "legal-case", "Adams", "Alpha v Beta", 2001),
        (
            "journal-gamma",
            "article-journal",
            "Gamma",
            "Gamma Article",
            2022,
        ),
        ("book-epsilon", "book", "Epsilon", "Collected Essays", 1998),
        (
            "journal-beta",
            "article-journal",
            "Beta",
            "Beta Article",
            2021,
        ),
        ("legal-delta", "legal-case", "Delta", "Delta v State", 1999),
        ("book-zeta", "book", "Zeta", "Zeta Monograph", 2005),
    ] {
        bibliography.insert(
            id.to_string(),
            make_legacy_reference(id, ref_type, family, title, year, None, None, None),
        );
    }
    bibliography
}

fn build_type_variant_bench_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Type Variant Bench".to_string()),
            id: Some("type-variant-bench".to_string()),
            ..Default::default()
        },
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
                article_journal: Some(citum_schema::options::ArticleJournalBibliographyConfig {
                    no_page_fallback: Some(
                        citum_schema::options::ArticleJournalNoPageFallback::Doi,
                    ),
                }),
                ..Default::default()
            }),
            template: Some(vec![TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                form: None,
                rendering: Rendering {
                    prefix: Some("DEFAULT ".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            })]),
            type_variants: Some(IndexMap::from([(
                "article-journal"
                    .parse()
                    .expect("type selector should parse"),
                vec![
                    TemplateComponent::Contributor(TemplateContributor {
                        contributor: ContributorRole::Author,
                        form: citum_schema::template::ContributorForm::Long,
                        ..Default::default()
                    }),
                    TemplateComponent::Number(TemplateNumber {
                        number: NumberVariable::Volume,
                        rendering: Rendering {
                            prefix: Some(", ".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    TemplateComponent::Number(TemplateNumber {
                        number: NumberVariable::Pages,
                        rendering: Rendering {
                            prefix: Some(": ".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    TemplateComponent::Variable(TemplateVariable {
                        variable: SimpleVariable::Doi,
                        rendering: Rendering {
                            prefix: Some(" DOI: ".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
            )])),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_type_variant_bibliography() -> Bibliography {
    let mut bibliography = Bibliography::new();
    bibliography.insert(
        "article-no-pages".to_string(),
        make_legacy_reference(
            "article-no-pages",
            "article-journal",
            "Smith",
            "Article Without Pages",
            2020,
            Some("12"),
            None,
            Some("10.1000/no-pages"),
        ),
    );
    bibliography.insert(
        "article-with-pages".to_string(),
        make_legacy_reference(
            "article-with-pages",
            "article-journal",
            "Jones",
            "Article With Pages",
            2021,
            Some("18"),
            Some("33-40"),
            Some("10.1000/with-pages"),
        ),
    );
    bibliography
}

fn build_compound_bench_style() -> Style {
    Style {
        info: StyleInfo {
            title: Some("Compound Bench".to_string()),
            id: Some("compound-bench".to_string()),
            ..Default::default()
        },
        options: Some(Config {
            processing: Some(Processing::Numeric),
            ..Default::default()
        }),
        bibliography: Some(BibliographySpec {
            options: Some(BibliographyOptions {
                compound_numeric: Some(CompoundNumericConfig {
                    sub_label_suffix: ")".to_string(),
                    sub_delimiter: ", ".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            template: Some(vec![
                TemplateComponent::Number(TemplateNumber {
                    number: NumberVariable::CitationNumber,
                    rendering: Rendering {
                        wrap: Some(WrapPunctuation::Brackets),
                        suffix: Some(" ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Author,
                    form: citum_schema::template::ContributorForm::Long,
                    ..Default::default()
                }),
                TemplateComponent::Title(TemplateTitle {
                    title: TitleType::Primary,
                    form: None,
                    rendering: Rendering {
                        prefix: Some(". ".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_compound_bibliography() -> Bibliography {
    let mut bibliography = Bibliography::new();
    for (id, family, title, year) in [
        ("cmp-a", "Alpha", "Compound Alpha", 2020),
        ("cmp-b", "Beta", "Compound Beta", 2020),
        ("cmp-c", "Gamma", "Compound Gamma", 2020),
        ("solo-d", "Delta", "Standalone Delta", 2021),
    ] {
        bibliography.insert(
            id.to_string(),
            make_legacy_reference(id, "book", family, title, year, None, None, None),
        );
    }
    bibliography
}

fn make_compound_sets() -> IndexMap<String, Vec<String>> {
    IndexMap::from([(
        "cmp-group".to_string(),
        vec![
            "cmp-a".to_string(),
            "cmp-b".to_string(),
            "cmp-c".to_string(),
        ],
    )])
}

#[allow(
    clippy::too_many_arguments,
    reason = "Benchmark fixtures vary only by a small set of legacy reference fields."
)]
fn make_legacy_reference(
    id: &str,
    ref_type: &str,
    family: &str,
    title: &str,
    year: i32,
    volume: Option<&str>,
    pages: Option<&str>,
    doi: Option<&str>,
) -> Reference {
    let mut value = serde_json::json!({
        "id": id,
        "type": ref_type,
        "title": title,
        "author": [{"family": family, "given": "Test"}],
        "issued": {"date-parts": [[year]]},
    });

    if let Some(volume) = volume {
        value["volume"] = serde_json::json!(volume);
    }
    if let Some(pages) = pages {
        value["page"] = serde_json::json!(pages);
    }
    if let Some(doi) = doi {
        value["DOI"] = serde_json::json!(doi);
    }

    let legacy: LegacyReference =
        serde_json::from_value(value).expect("legacy benchmark reference should parse");
    legacy.into()
}

criterion_group!(benches, bench_rendering);
criterion_main!(benches);
