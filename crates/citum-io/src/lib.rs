/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Citum I/O and format conversion library.
//!
//! Provides functions to load bibliographies and citations from various formats
//! (Citum YAML/JSON/CBOR, CSL-JSON, BibLaTeX, RIS) into Citum's internal types.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use citum_engine::processor::validate_compound_sets;
use citum_schema::InputBibliography;
use csl_legacy::csl_json::Reference as LegacyReference;
use indexmap::IndexMap;

pub use citum_engine::api::{AnnotationFormat, AnnotationStyle};
use citum_engine::render::format::OutputFormat;
use citum_engine::render::rich_text::render_djot_inline;
use citum_engine::{Bibliography, Citation, ProcessorError};

pub mod biblatex;
pub(crate) mod formats;

// Re-export private format helpers so the inline test module can reach them via `use super::*`.
#[cfg(test)]
pub(crate) use formats::{
    loaded_from_input_bibliography, parse_json_bibliography, parse_yaml_bibliography,
};

/// Bibliography formats supported by reusable reference conversion helpers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RefsFormat {
    /// Native Citum bibliography encoded as YAML.
    CitumYaml,
    /// Native Citum bibliography encoded as JSON.
    CitumJson,
    /// Native Citum bibliography encoded as CBOR.
    CitumCbor,
    /// Legacy CSL-JSON bibliography.
    CslJson,
    /// BibLaTeX `.bib` bibliography.
    Biblatex,
    /// RIS bibliography.
    Ris,
}

/// Bibliography data loaded from input, including optional compound sets.
#[derive(Debug, Clone, Default)]
pub struct LoadedBibliography {
    /// Parsed bibliography references keyed by ID.
    pub references: Bibliography,
    /// Optional compound sets keyed by set ID.
    pub sets: Option<IndexMap<String, Vec<String>>>,
}

/// Render a free-text reference field with djot inline markup.
///
/// Applies `render_djot_inline` to the source text using the provided
/// `OutputFormat` for emphasis, links, and other inline markup.
pub fn render_rich_text_field<F: OutputFormat<Output = String>>(src: &str, fmt: &F) -> String {
    render_djot_inline(src, fmt)
}

/// Load a list of citations from a file.
///
/// Supports Citum YAML/JSON.
///
/// # Errors
///
/// Returns an error when the file cannot be read or its contents are not valid
/// citation data.
pub fn load_citations(path: &Path) -> Result<Vec<Citation>, ProcessorError> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    if ext == "json" {
        let _: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

        if let Ok(citations) = serde_json::from_slice::<Vec<Citation>>(&bytes) {
            return Ok(citations);
        }
        match serde_json::from_slice::<Citation>(&bytes) {
            Ok(citation) => Ok(vec![citation]),
            Err(e) => Err(ProcessorError::ParseError(
                "JSON".to_string(),
                e.to_string(),
            )),
        }
    } else {
        let content = String::from_utf8_lossy(&bytes);
        let _: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))?;

        if let Ok(citations) = serde_yaml::from_str::<Vec<Citation>>(&content) {
            return Ok(citations);
        }
        match serde_yaml::from_str::<Citation>(&content) {
            Ok(citation) => Ok(vec![citation]),
            Err(e) => Err(ProcessorError::ParseError(
                "YAML".to_string(),
                e.to_string(),
            )),
        }
    }
}

/// Load annotations from a file (YAML or JSON).
///
/// Returns a mapping from reference ID to annotation text.
///
/// # Errors
///
/// Returns an error when the file cannot be read or its contents are not valid
/// annotation data.
pub fn load_annotations(path: &Path) -> Result<HashMap<String, String>, ProcessorError> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    if ext == "json" {
        let _: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;

        serde_json::from_slice::<HashMap<String, String>>(&bytes)
            .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))
    } else {
        let content = String::from_utf8_lossy(&bytes);
        let _: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))?;

        serde_yaml::from_str::<HashMap<String, String>>(&content)
            .map_err(|e| ProcessorError::ParseError("YAML".to_string(), e.to_string()))
    }
}

/// Load bibliography data from a file, including optional compound sets.
///
/// Supports Citum YAML/JSON/CBOR and CSL-JSON.
///
/// # Errors
///
/// Returns an error when the file cannot be read, cannot be parsed, or
/// compound sets are invalid.
pub fn load_bibliography_with_sets(path: &Path) -> Result<LoadedBibliography, ProcessorError> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    match ext {
        "cbor" => formats::parse_cbor_bibliography(&bytes),
        "json" => formats::parse_json_bibliography(&bytes),
        _ => {
            let content = String::from_utf8_lossy(&bytes);
            formats::parse_yaml_bibliography(&content)
        }
    }
}

/// Load a bibliography from a file given its path.
///
/// Supports Citum YAML/JSON/CBOR and CSL-JSON.
///
/// # Errors
///
/// Returns an error when the file cannot be read, cannot be parsed, or
/// embedded compound-set metadata is invalid.
pub fn load_bibliography(path: &Path) -> Result<Bibliography, ProcessorError> {
    Ok(load_bibliography_with_sets(path)?.references)
}

/// Load and merge one or more bibliography files, preserving compound set metadata.
///
/// Entries from later files replace entries with the same ID from earlier files.
/// Compound set IDs must be unique across input files, and final membership is
/// validated against the merged bibliography.
///
/// # Errors
///
/// Returns an error when no paths are supplied, any file cannot be loaded, or
/// merged compound sets are invalid.
pub fn load_merged_bibliography(paths: &[PathBuf]) -> Result<LoadedBibliography, ProcessorError> {
    if paths.is_empty() {
        return Err(ProcessorError::ParseError(
            "BIBLIOGRAPHY".to_string(),
            "At least one bibliography path is required.".to_string(),
        ));
    }

    let mut merged = Bibliography::new();
    let mut merged_sets = IndexMap::<String, Vec<String>>::new();
    for path in paths {
        let loaded = load_bibliography_with_sets(path)?;
        for (id, reference) in loaded.references {
            merged.insert(id, reference);
        }
        if let Some(sets) = loaded.sets {
            for (set_id, members) in sets {
                if merged_sets.insert(set_id.clone(), members).is_some() {
                    return Err(ProcessorError::ParseError(
                        "BIBLIOGRAPHY".to_string(),
                        format!("Duplicate compound set id while merging: {set_id}"),
                    ));
                }
            }
        }
    }

    let validated_sets =
        validate_compound_sets((!merged_sets.is_empty()).then_some(merged_sets), &merged)?;

    Ok(LoadedBibliography {
        references: merged,
        sets: validated_sets,
    })
}

/// Load and concatenate one or more citation files.
///
/// # Errors
///
/// Returns an error when any citation file cannot be read or parsed.
pub fn load_merged_citations(paths: &[PathBuf]) -> Result<Vec<Citation>, ProcessorError> {
    let mut merged = Vec::new();
    for path in paths {
        merged.extend(load_citations(path)?);
    }
    Ok(merged)
}

/// Infer a bibliography input format from a path.
///
/// JSON inputs are content-sniffed to distinguish native Citum JSON from CSL-JSON.
///
/// # Errors
///
/// Returns an error when a JSON input cannot be read or parsed for detection.
pub fn infer_refs_input_format(path: &Path) -> Result<RefsFormat, ProcessorError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let fmt = match ext.to_ascii_lowercase().as_str() {
        "yaml" | "yml" => RefsFormat::CitumYaml,
        "cbor" => RefsFormat::CitumCbor,
        "bib" => RefsFormat::Biblatex,
        "ris" => RefsFormat::Ris,
        "json" => detect_json_refs_format(path)?,
        _ => RefsFormat::CitumYaml,
    };
    Ok(fmt)
}

/// Infer a bibliography output format from a path.
///
/// Unknown extensions default to native Citum YAML.
#[must_use]
pub fn infer_refs_output_format(path: &Path) -> RefsFormat {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_ascii_lowercase().as_str() {
        "yaml" | "yml" => RefsFormat::CitumYaml,
        "cbor" => RefsFormat::CitumCbor,
        "bib" => RefsFormat::Biblatex,
        "ris" => RefsFormat::Ris,
        "json" => RefsFormat::CitumJson,
        _ => RefsFormat::CitumYaml,
    }
}

fn detect_json_refs_format(path: &Path) -> Result<RefsFormat, ProcessorError> {
    let bytes = fs::read(path)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;
    let array = value.as_array();
    let is_citum_array = array.is_some_and(|items| items.iter().any(|v| v.get("class").is_some()));
    let is_csl_array = array.is_some_and(|items| {
        items.iter().any(|v| {
            v.get("id").is_some()
                && v.get("type").is_some()
                && (v.get("title").is_some() || v.get("author").is_some())
        })
    });
    let is_citum_object = value.get("references").is_some();
    if is_csl_array && !is_citum_array && !is_citum_object {
        Ok(RefsFormat::CslJson)
    } else {
        Ok(RefsFormat::CitumJson)
    }
}

/// Load bibliography input in a specified native or legacy reference format.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed as `format`.
pub fn load_input_bibliography(
    path: &Path,
    format: RefsFormat,
) -> Result<InputBibliography, ProcessorError> {
    match format {
        RefsFormat::CitumYaml => {
            let bytes = fs::read(path)?;
            formats::deserialize_any(&bytes, "yaml")
        }
        RefsFormat::CitumJson => {
            let bytes = fs::read(path)?;
            formats::load_citum_json_bibliography(&bytes)
        }
        RefsFormat::CitumCbor => {
            let bytes = fs::read(path)?;
            formats::deserialize_any(&bytes, "cbor")
        }
        RefsFormat::CslJson => formats::load_csl_json_bibliography(path),
        RefsFormat::Biblatex => formats::load_biblatex_bibliography(path),
        RefsFormat::Ris => formats::load_ris_bibliography(path),
    }
}

/// Write bibliography input to a specified native or legacy reference format.
///
/// # Errors
///
/// Returns an error when serialization fails or the output file cannot be written.
pub fn write_output_bibliography(
    input: &InputBibliography,
    path: &Path,
    format: RefsFormat,
) -> Result<(), ProcessorError> {
    match format {
        RefsFormat::CitumYaml => fs::write(path, formats::serialize_any(input, "yaml")?)?,
        RefsFormat::CitumJson => fs::write(path, formats::serialize_any(input, "json")?)?,
        RefsFormat::CitumCbor => fs::write(path, formats::serialize_any(input, "cbor")?)?,
        RefsFormat::CslJson => {
            let refs: Vec<LegacyReference> = input
                .references
                .iter()
                .map(formats::input_reference_to_csl_json)
                .collect();
            let json = serde_json::to_string_pretty(&refs)
                .map_err(|e| ProcessorError::ParseError("JSON".to_string(), e.to_string()))?;
            fs::write(path, json)?;
        }
        RefsFormat::Biblatex => {
            fs::write(path, formats::render_biblatex(input))?;
        }
        RefsFormat::Ris => {
            fs::write(path, formats::render_ris(input))?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(
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
mod tests {
    use super::*;
    use citum_engine::render::plain::PlainText;
    use citum_schema::reference::{ClassExtension, InputReference};
    use indexmap::IndexMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(stem: &str, ext: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{stem}-{now}.{ext}"))
    }

    #[test]
    fn render_rich_text_field_with_bold() {
        let fmt = PlainText;
        let result = render_rich_text_field("This is *bold* text", &fmt);
        assert_eq!(result, "This is **bold** text");
    }

    #[test]
    fn render_rich_text_field_with_link() {
        let fmt = PlainText;
        let result = render_rich_text_field("See [this](https://example.com) for details", &fmt);
        assert_eq!(result, "See this for details");
    }

    #[test]
    fn load_citations_preserves_locator_labels() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/citations-expanded.json");
        let citations = load_citations(&path).expect("citations fixture should parse");
        let with_locator = citations
            .iter()
            .find(|c| c.id.as_deref() == Some("with-locator"))
            .expect("with-locator citation should exist");

        assert_eq!(with_locator.items.len(), 1);
        assert_eq!(
            with_locator.items[0].locator,
            Some(citum_schema::citation::CitationLocator::single(
                citum_schema::citation::LocatorType::Page,
                "23",
            ))
        );
    }

    #[test]
    fn infer_refs_output_format_uses_supported_extensions() {
        assert!(matches!(
            infer_refs_output_format(Path::new("refs.yaml")),
            RefsFormat::CitumYaml
        ));
        assert!(matches!(
            infer_refs_output_format(Path::new("refs.bib")),
            RefsFormat::Biblatex
        ));
        assert!(matches!(
            infer_refs_output_format(Path::new("refs.ris")),
            RefsFormat::Ris
        ));
    }

    #[test]
    fn load_merged_bibliography_rejects_cross_file_duplicate_membership() {
        let base = temp_path("citum-merged-bib", "dir");
        std::fs::create_dir_all(&base).expect("temp dir should be created");
        let bib_a = base.join("a.yaml");
        let bib_b = base.join("b.yaml");

        std::fs::write(
            &bib_a,
            r#"
references:
  - class: monograph
    id: ref-a
    type: book
    title: Book A
    issued: "2020"
sets:
  group-1: [ref-a]
"#,
        )
        .expect("first fixture should write");
        std::fs::write(
            &bib_b,
            r#"
references:
  - class: monograph
    id: ref-a
    type: book
    title: Book A
    issued: "2020"
sets:
  group-2: [ref-a]
"#,
        )
        .expect("second fixture should write");

        let err = load_merged_bibliography(&[bib_a.clone(), bib_b.clone()])
            .expect_err("must reject cross-file duplicate membership");

        assert!(
            err.to_string()
                .contains("appears in both compound sets 'group-1' and 'group-2'"),
            "unexpected error: {err}"
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn load_input_bibliography_accepts_biblatex() {
        let path = temp_path("citum-biblatex", "bib");
        std::fs::write(
            &path,
            "@article{smith2020,\n  title = {Article},\n  author = {Smith, Jane},\n  date = {2020},\n  journaltitle = {Journal}\n}",
        )
        .expect("biblatex fixture should write");

        let bibliography =
            load_input_bibliography(&path, RefsFormat::Biblatex).expect("BibLaTeX should parse");

        let reference = bibliography
            .references
            .iter()
            .find(|reference| reference.id().as_deref() == Some("smith2020"))
            .expect("reference should be loaded");
        assert_eq!(reference.ref_type(), "article-journal");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn load_and_write_ris_bibliography() {
        let input = temp_path("citum-ris-in", "ris");
        let output = temp_path("citum-ris-out", "ris");
        std::fs::write(
            &input,
            "TY  - JOUR\nID  - smith2020\nTI  - Article\nAU  - Smith, Jane\nPY  - 2020\nDO  - 10.1000/example\nER  -\n",
        )
        .expect("RIS fixture should write");

        let bibliography =
            load_input_bibliography(&input, RefsFormat::Ris).expect("RIS should parse");
        write_output_bibliography(&bibliography, &output, RefsFormat::Ris)
            .expect("RIS should serialize");
        let rendered = std::fs::read_to_string(&output).expect("RIS output should read");

        assert_eq!(
            rendered,
            "TY  - JOUR\nID  - smith2020\nTI  - Article\nAU  - Smith, Jane\nPY  - 2020\nDO  - 10.1000/example\nER  -\n\n"
        );

        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn load_ris_bibliography_accepts_utf8_bom() {
        let input = temp_path("citum-ris-bom", "ris");
        std::fs::write(
            &input,
            "\u{feff}TY  - JOUR\nID  - smith2020\nTI  - Article\nER  -\n",
        )
        .expect("RIS fixture should write");

        let bibliography =
            load_input_bibliography(&input, RefsFormat::Ris).expect("RIS with BOM should parse");

        let reference = bibliography
            .references
            .iter()
            .find(|reference| reference.id().as_deref() == Some("smith2020"))
            .expect("reference should be loaded");
        assert_eq!(reference.ref_type(), "article-journal");

        let _ = std::fs::remove_file(input);
    }

    #[test]
    fn loaded_bibliography_rejects_unknown_set_member() {
        let input = InputBibliography {
            references: vec![
                serde_yaml::from_str::<InputReference>(
                    r#"
class: monograph
id: ref-a
type: book
title: Book A
issued: "2020"
"#,
                )
                .expect("reference should parse"),
            ],
            sets: Some(IndexMap::from([(
                "group-1".to_string(),
                vec!["missing-ref".to_string()],
            )])),
            ..Default::default()
        };

        let err = loaded_from_input_bibliography(input).expect_err("must reject unknown member");
        let msg = err.to_string();
        assert!(
            msg.contains("unknown id 'missing-ref'"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn loaded_bibliography_rejects_duplicate_set_membership() {
        let input = InputBibliography {
            references: vec![
                serde_yaml::from_str::<InputReference>(
                    r#"
class: monograph
id: ref-a
type: book
title: Book A
issued: "2020"
"#,
                )
                .expect("reference should parse"),
            ],
            sets: Some(IndexMap::from([
                ("group-1".to_string(), vec!["ref-a".to_string()]),
                ("group-2".to_string(), vec!["ref-a".to_string()]),
            ])),
            ..Default::default()
        };

        let err =
            loaded_from_input_bibliography(input).expect_err("must reject duplicate membership");
        let msg = err.to_string();
        assert!(
            msg.contains("appears in both compound sets 'group-1' and 'group-2'"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn loaded_bibliography_rejects_duplicate_within_same_set() {
        let input = InputBibliography {
            references: vec![
                serde_yaml::from_str::<InputReference>(
                    r#"
class: monograph
id: ref-a
type: book
title: Book A
issued: "2020"
"#,
                )
                .expect("reference should parse"),
            ],
            sets: Some(IndexMap::from([(
                "group-1".to_string(),
                vec!["ref-a".to_string(), "ref-a".to_string()],
            )])),
            ..Default::default()
        };

        let err = loaded_from_input_bibliography(input)
            .expect_err("must reject duplicate member in the same set");
        let msg = err.to_string();
        assert!(
            msg.contains("appears more than once in compound set 'group-1'"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn loaded_bibliography_accepts_empty_and_singleton_sets() {
        let input = InputBibliography {
            references: vec![
                serde_yaml::from_str::<InputReference>(
                    r#"
class: monograph
id: ref-a
type: book
title: Book A
issued: "2020"
"#,
                )
                .expect("reference should parse"),
            ],
            sets: Some(IndexMap::from([
                ("empty".to_string(), Vec::new()),
                ("single".to_string(), vec!["ref-a".to_string()]),
            ])),
            ..Default::default()
        };

        let loaded = loaded_from_input_bibliography(input).expect("sets should be accepted");
        let sets = loaded.sets.expect("sets should be present");
        assert!(sets.contains_key("empty"));
        assert_eq!(sets.get("single"), Some(&vec!["ref-a".to_string()]));
    }

    #[test]
    /// Parse a JSON array of native Citum references before falling back to CSL-JSON.
    fn parse_json_vec_input_references() {
        let json = r#"[
  {
    "class": "collection",
    "id": "edited-book-1",
    "type": "edited-book",
    "title": "Edited Book",
    "editor": [{"family": "Miller", "given": "Ruth"}],
    "issued": "2022"
  }
]"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse native Citum JSON vec");
        assert_eq!(result.references.len(), 1);
        let reference = result
            .references
            .get("edited-book-1")
            .expect("reference should be preserved");
        assert_eq!(reference.ref_type(), "book");
        assert!(matches!(
            reference.extension(),
            ClassExtension::Collection(_)
        ));
        assert!(result.sets.is_none());
    }

    #[test]
    /// Parse hybrid JSON fixtures that combine Citum `class` tags with CSL-JSON contributor/date shapes.
    fn parse_json_hybrid_edited_book_reference() {
        let json = r#"[
  {
    "id": "edited-book-1",
    "class": "collection",
    "type": "edited-book",
    "title": "Edited Book",
    "editor": [{"family": "Miller", "given": "Ruth"}],
    "issued": {"date-parts": [[2022]]},
    "publisher": "Example Press",
    "publisher-place": "Chicago"
  }
]"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse hybrid Citum/CSL JSON");
        let reference = result
            .references
            .get("edited-book-1")
            .expect("reference should be preserved");
        assert_eq!(reference.ref_type(), "book");
        assert!(matches!(
            reference.extension(),
            ClassExtension::Collection(_)
        ));
    }

    #[test]
    /// Preserve URLs when hybrid edited-book JSON falls back through the legacy loader.
    fn parse_json_hybrid_edited_book_preserves_url() {
        let json = r#"[
  {
    "id": "edited-book-1",
    "class": "collection",
    "type": "edited-book",
    "title": "Edited Book",
    "editor": [{"family": "Miller", "given": "Ruth"}],
    "issued": {"date-parts": [[2022]]},
    "publisher": "Example Press",
    "publisher-place": "Chicago",
    "URL": "https://example.com/edited-book"
  }
]"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse hybrid Citum/CSL JSON");
        let reference = result
            .references
            .get("edited-book-1")
            .expect("reference should be preserved");

        assert_eq!(
            reference.url().as_ref().map(url::Url::as_str),
            Some("https://example.com/edited-book")
        );
    }

    #[test]
    /// Parse a JSON array of CSL-JSON objects directly into `LoadedBibliography`.
    fn parse_json_csl_vec() {
        let json = r#"[
  {"id": "smith-2020", "type": "book", "title": "Test Book"},
  {"id": "doe-2021", "type": "journal-article", "title": "Test Article"}
]"#;
        let result = parse_json_bibliography(json.as_bytes()).expect("should parse CSL-JSON vec");
        assert_eq!(result.references.len(), 2);
        assert!(result.references.contains_key("smith-2020"));
        assert!(result.references.contains_key("doe-2021"));
        assert!(result.sets.is_none());
    }

    #[test]
    /// Parse a Citum `InputBibliography` from JSON with references and sets.
    fn parse_json_citum_input_bibliography() {
        let json = r#"{
  "references": [
    {
      "class": "monograph",
      "id": "ref-x",
      "type": "book",
      "title": "Citum Book",
      "issued": "2020"
    }
  ],
  "sets": {
    "group-a": ["ref-x"]
  }
}"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse Citum InputBibliography");
        assert_eq!(result.references.len(), 1);
        assert!(result.references.contains_key("ref-x"));
        let sets = result.sets.expect("sets should be present");
        assert_eq!(sets.get("group-a"), Some(&vec!["ref-x".to_string()]));
    }

    #[test]
    /// Parse a wrapped legacy JSON object with references and optional sets.
    fn parse_json_wrapped_legacy() {
        let json = r#"{
  "references": [
    {"id": "legacy-1", "type": "book", "title": "Legacy Book"}
  ],
  "sets": null
}"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse wrapped legacy format");
        assert_eq!(result.references.len(), 1);
        assert!(result.references.contains_key("legacy-1"));
        assert!(result.sets.is_none());
    }

    #[test]
    /// Parse an `IndexMap` of CSL-JSON objects keyed by id from JSON.
    fn parse_json_indexmap() {
        let json = r#"{
  "book-1": {"type": "book", "title": "First Book", "id": "book-1"},
  "article-2": {"type": "journal-article", "title": "First Article", "id": "article-2"}
}"#;
        let result =
            parse_json_bibliography(json.as_bytes()).expect("should parse IndexMap format");
        assert_eq!(result.references.len(), 2);
        assert!(result.references.contains_key("book-1"));
        assert!(result.references.contains_key("article-2"));
    }

    #[test]
    /// Parse a Citum YAML `InputBibliography` with references.
    fn parse_yaml_citum_input_bibliography() {
        let yaml = r#"
references:
  - class: monograph
    id: yaml-ref-1
    type: book
    title: YAML Book
    issued: "2021"
"#;
        let result = parse_yaml_bibliography(yaml).expect("should parse Citum YAML bibliography");
        assert_eq!(result.references.len(), 1);
        assert!(result.references.contains_key("yaml-ref-1"));
    }

    #[test]
    /// Parse a wrapped legacy YAML object with references and optional sets.
    fn parse_yaml_wrapped_legacy() {
        let yaml = r"
references:
  - id: yaml-legacy-1
    type: book
    title: YAML Legacy Book
sets: null
";
        let result = parse_yaml_bibliography(yaml).expect("should parse wrapped legacy YAML");
        assert_eq!(result.references.len(), 1);
        assert!(result.references.contains_key("yaml-legacy-1"));
        assert!(result.sets.is_none());
    }

    #[test]
    /// Parse an `IndexMap` of legacy references keyed by id from YAML.
    fn parse_yaml_indexmap() {
        let yaml = r"ref-yaml-1:
  id: ref-yaml-1
  type: book
  title: First Book
ref-yaml-2:
  id: ref-yaml-2
  type: journal-article
  title: Second Article
";
        let result = parse_yaml_bibliography(yaml).expect("should parse YAML IndexMap format");
        assert_eq!(result.references.len(), 2);
        assert!(result.references.contains_key("ref-yaml-1"));
        assert!(result.references.contains_key("ref-yaml-2"));
    }

    #[test]
    /// Parse a YAML sequence of `InputReference` objects.
    fn parse_yaml_vec_input_references() {
        let yaml = r#"
- class: monograph
  id: seq-ref-1
  type: book
  title: Sequential Book
  issued: "2024"
- class: monograph
  id: seq-ref-2
  type: book
  title: Another Sequential Book
  issued: "2025"
"#;
        let result =
            parse_yaml_bibliography(yaml).expect("should parse YAML sequence of references");
        assert_eq!(result.references.len(), 2);
        assert!(result.references.contains_key("seq-ref-1"));
        assert!(result.references.contains_key("seq-ref-2"));
    }
}
