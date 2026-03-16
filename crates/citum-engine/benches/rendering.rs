#![allow(missing_docs)]

use citum_engine::processor::disambiguation::Disambiguator;
use citum_engine::{
    Bibliography, Citation, CitationItem, Contributor, EdtfString, Locale, Monograph,
    MonographType, MultilingualString, Processor, Reference, StructuredName, Title,
};
use citum_schema::options::{
    Config, Disambiguation, Group, LabelConfig, LabelPreset, Processing, ProcessingCustom, Sort,
    SortEntry, SortKey, SortSpec,
};
use citum_schema::{InputBibliography, Style};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
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
}

fn bench_disambiguation(c: &mut Criterion) {
    let locale = Locale::en_us();
    let no_collision_bib = make_no_collision_bibliography();
    let givenname_bib = make_givenname_collision_bibliography();
    let partition_bib = make_partition_collision_bibliography();
    let label_bib = make_label_collision_bibliography();

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
    let title = format!("Title {id}");
    Reference::Monograph(Box::new(Monograph {
        id: Some(id.to_string()),
        r#type: MonographType::Book,
        title: Some(Title::Single(title)),
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

criterion_group!(benches, bench_rendering);
criterion_main!(benches);
