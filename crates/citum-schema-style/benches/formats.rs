/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

#![allow(missing_docs, reason = "test/bench/bin crate")]
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

use citum_schema_style::{InputBibliography, Style};
use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;

fn bench_formats(c: &mut Criterion) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.parent().unwrap().parent().unwrap();

    let style_path = root_dir.join("styles/embedded/apa-7th.yaml");
    let style_yaml = fs::read_to_string(&style_path).expect("failed to read apa-7th.yaml");
    let style: Style = serde_yaml::from_str(&style_yaml).expect("failed to parse style yaml");

    let style_json = serde_json::to_string(&style).expect("failed to serialize style to json");
    let mut style_cbor = Vec::new();
    ciborium::ser::into_writer(&style, &mut style_cbor).expect("failed to serialize style to cbor");

    let mut group = c.benchmark_group("Style Deserialization");

    group.bench_function("YAML", |b| {
        b.iter(|| {
            let _: Style = serde_yaml::from_str(black_box(&style_yaml)).unwrap();
        });
    });

    group.bench_function("JSON", |b| {
        b.iter(|| {
            let _: Style = serde_json::from_str(black_box(&style_json)).unwrap();
        });
    });

    group.bench_function("CBOR", |b| {
        b.iter(|| {
            let _: Style =
                ciborium::de::from_reader(std::io::Cursor::new(black_box(&style_cbor))).unwrap();
        });
    });

    group.finish();

    let bib_path = root_dir.join("examples/comprehensive.yaml");
    let bib_yaml = fs::read_to_string(&bib_path).expect("failed to read comprehensive.yaml");
    let bib: InputBibliography = serde_yaml::from_str(&bib_yaml).expect("failed to parse bib yaml");

    let bib_json = serde_json::to_string(&bib).expect("failed to serialize bib to json");
    let mut bib_cbor = Vec::new();
    ciborium::ser::into_writer(&bib, &mut bib_cbor).expect("failed to serialize bib to cbor");

    let mut group = c.benchmark_group("Bibliography Deserialization");

    group.bench_function("YAML", |b| {
        b.iter(|| {
            let _: InputBibliography = serde_yaml::from_str(black_box(&bib_yaml)).unwrap();
        });
    });

    group.bench_function("JSON", |b| {
        b.iter(|| {
            let _: InputBibliography = serde_json::from_str(black_box(&bib_json)).unwrap();
        });
    });

    group.bench_function("CBOR", |b| {
        b.iter(|| {
            let _: InputBibliography =
                ciborium::de::from_reader(std::io::Cursor::new(black_box(&bib_cbor))).unwrap();
        });
    });

    group.finish();
}

fn bench_style_resolution(c: &mut Criterion) {
    use citum_resolver_api::StyleResolver;
    use citum_schema_style::ResolverError;
    use citum_schema_style::locale::Locale;

    // Mirror the BDD test fixture: known-good template components only.
    let base_yaml = r#"
version: "0.44.0"
info:
  id: base-style
  title: Base Style
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
    article-journal:
      - title: primary
      - variable: doi
"#;

    let overlay_yaml = r#"
extends: base-style
info:
  id: overlay-style
  title: Overlay Style
bibliography:
  type-variants:
    book:
      modify:
        - match: { title: primary }
          emph: true
"#;

    let base = Style::from_yaml_str(base_yaml).expect("valid base");
    let overlay = Style::from_yaml_str(overlay_yaml).expect("valid overlay");

    struct MockResolver(Style);
    impl StyleResolver for MockResolver {
        type Style = Style;
        type Locale = Locale;
        fn resolve_style(&self, _uri: &str) -> Result<Style, ResolverError> {
            Ok(self.0.clone())
        }
        fn resolve_locale(&self, id: &str) -> Result<Locale, ResolverError> {
            Err(ResolverError::LocaleNotFound(std::borrow::Cow::Owned(
                id.to_string(),
            )))
        }
    }

    let resolver = MockResolver(base);

    let mut group = c.benchmark_group("Style Resolution");
    group.bench_function("merge_style_overlay", |b| {
        b.iter(|| {
            let style = black_box(overlay.clone());
            let mut visited = std::collections::HashSet::new();
            style
                .try_into_resolved_recursive_with(Some(&resolver), &mut visited)
                .unwrap()
        });
    });
    group.finish();
}

criterion_group!(benches, bench_formats, bench_style_resolution);
criterion_main!(benches);
