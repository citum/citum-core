/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! `render` subcommands: render a document (`render doc`) or render references
//! and citations directly from data files (`render refs`).

mod human;
mod json;

use super::CliResult;
use crate::args::{
    InputFormat, OutputFormat, RenderCommands, RenderDocArgs, RenderMode, RenderRefsArgs,
};
use crate::output::write_output;
use crate::style_resolver::{create_processor, load_any_style};
use crate::typst_pdf;
use citum_engine::processor::document::{djot::DjotParser, markdown::MarkdownParser};
use citum_engine::render::{djot::Djot, html::Html, latex::Latex, plain::PlainText, typst::Typst};
use citum_engine::{Citation, DocumentFormat, Processor};
use citum_io::{
    AnnotationFormat, AnnotationStyle, load_annotations, load_merged_bibliography,
    load_merged_citations,
};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

pub(super) fn dispatch(command: RenderCommands) -> CliResult {
    match command {
        RenderCommands::Doc(args) => run_render_doc(args),
        RenderCommands::Refs(args) => run_render_refs(args),
    }
}

/// Execute the `render doc` subcommand.
///
/// Reads a document, resolves citations against the provided bibliography,
/// and writes the rendered output to stdout or a file.
pub(super) fn run_render_doc(args: RenderDocArgs) -> CliResult {
    if args.pdf && args.format != OutputFormat::Typst {
        return Err("`--pdf` is only supported with `--format typst`.".into());
    }

    let style_obj = load_any_style(&args.style, args.no_semantics)?;
    let bibliography = load_merged_bibliography(&args.bibliography)?;

    if !args.citations.is_empty() {
        eprintln!(
            "Warning: --citations is currently ignored by `render doc`; citations are parsed from the input document."
        );
    }

    let processor = create_processor(
        style_obj,
        bibliography,
        &args.style,
        args.no_semantics,
        None,
    )?;

    let doc_content = fs::read_to_string(&args.input)?;
    let output = match args.input_format {
        InputFormat::Djot => render_doc_with_output_format(
            &processor,
            &doc_content,
            args.format,
            DocumentInput::Djot,
        )?,
        InputFormat::Markdown => render_doc_with_output_format(
            &processor,
            &doc_content,
            args.format,
            DocumentInput::Markdown,
        )?,
    };

    if args.pdf {
        let output_path = args
            .output
            .as_ref()
            .ok_or("`--pdf` requires `--output <file.pdf>`.")?;
        typst_pdf::compile_document_to_pdf(&output, output_path, args.typst_keep_source)?;
        return Ok(());
    }

    write_output(&output, args.output.as_ref())
}

/// Execute the `render refs` subcommand.
///
/// Renders bibliography entries and/or citations directly from data files
/// without requiring a full document.
pub(super) fn run_render_refs(args: RenderRefsArgs) -> CliResult {
    let style_obj = load_any_style(&args.style, args.no_semantics)?;
    let bibliography = load_merged_bibliography(&args.bibliography)?;

    let item_ids = if let Some(k) = args.keys.clone() {
        k
    } else {
        bibliography.references.keys().cloned().collect()
    };

    let input_citations = if args.citations.is_empty() {
        None
    } else {
        Some(load_merged_citations(&args.citations)?)
    };

    let annotations = if let Some(path) = &args.annotations {
        Some(load_annotations(path)?)
    } else {
        None
    };

    let annotation_style = AnnotationStyle {
        format: AnnotationFormat::Djot,
    };

    let processor = create_processor(
        style_obj,
        bibliography,
        &args.style,
        args.no_semantics,
        args.locale.as_deref(),
    )?;

    let style_name = {
        let path = Path::new(&args.style);
        if path.exists() {
            path.file_name().map_or_else(
                || "unknown".to_string(),
                |s: &std::ffi::OsStr| s.to_string_lossy().to_string(),
            )
        } else {
            args.style.clone()
        }
    };

    let render_ctx = RenderContext {
        processor: &processor,
        style_name: &style_name,
        item_ids: &item_ids,
        annotations: annotations.as_ref(),
        annotation_style: &annotation_style,
    };
    let output = if args.json {
        render_refs_json(&render_ctx, args.mode, input_citations, args.format)?
    } else {
        render_refs_human(
            &render_ctx,
            args.mode,
            input_citations,
            args.show_keys,
            args.format,
        )?
    };

    write_output(&output, args.output.as_ref())
}

enum DocumentInput {
    Djot,
    Markdown,
}

/// Render a full document through the processor using the given output format.
///
/// Dispatches to the monomorphised `process_document` call matching `output_format`.
fn render_doc_with_output_format(
    processor: &Processor,
    content: &str,
    output_format: OutputFormat,
    input_format: DocumentInput,
) -> Result<String, Box<dyn Error>> {
    let doc_format = to_document_format(output_format)?;

    match input_format {
        DocumentInput::Djot => {
            let parser = DjotParser;
            match output_format {
                OutputFormat::Plain => {
                    Ok(processor.process_document::<_, PlainText>(content, &parser, doc_format))
                }
                OutputFormat::Html => {
                    Ok(processor.process_document::<_, Html>(content, &parser, doc_format))
                }
                OutputFormat::Djot => {
                    Ok(processor.process_document::<_, Djot>(content, &parser, doc_format))
                }
                OutputFormat::Latex => {
                    Ok(processor.process_document::<_, Latex>(content, &parser, doc_format))
                }
                OutputFormat::Typst => {
                    Ok(processor.process_document::<_, Typst>(content, &parser, doc_format))
                }
            }
        }
        DocumentInput::Markdown => {
            let parser = MarkdownParser;
            match output_format {
                OutputFormat::Plain => {
                    Ok(processor.process_document::<_, PlainText>(content, &parser, doc_format))
                }
                OutputFormat::Html => {
                    Ok(processor.process_document::<_, Html>(content, &parser, doc_format))
                }
                OutputFormat::Djot => {
                    Ok(processor.process_document::<_, Djot>(content, &parser, doc_format))
                }
                OutputFormat::Latex => {
                    Ok(processor.process_document::<_, Latex>(content, &parser, doc_format))
                }
                OutputFormat::Typst => {
                    Ok(processor.process_document::<_, Typst>(content, &parser, doc_format))
                }
            }
        }
    }
}

/// Map the CLI [`OutputFormat`] enum to the engine's [`DocumentFormat`].
fn to_document_format(output_format: OutputFormat) -> Result<DocumentFormat, Box<dyn Error>> {
    match output_format {
        OutputFormat::Plain => Ok(DocumentFormat::Plain),
        OutputFormat::Html => Ok(DocumentFormat::Html),
        OutputFormat::Djot => Ok(DocumentFormat::Djot),
        OutputFormat::Latex => Ok(DocumentFormat::Latex),
        OutputFormat::Typst => Ok(DocumentFormat::Typst),
    }
}

/// Shared rendering context threaded through the reference-rendering call chain.
pub(super) struct RenderContext<'a> {
    /// The configured citation processor.
    pub(super) processor: &'a citum_engine::Processor,
    /// Display name of the active style.
    pub(super) style_name: &'a str,
    /// Reference IDs to render.
    pub(super) item_ids: &'a [String],
    /// Optional annotation map (reference ID → annotation text).
    pub(super) annotations: Option<&'a HashMap<String, String>>,
    /// Formatting style for annotations.
    pub(super) annotation_style: &'a citum_io::AnnotationStyle,
}

/// Render bibliography/citation output as a human-readable string.
///
/// Dispatches to the correct monomorphised format renderer based on `output_format`.
fn render_refs_human(
    ctx: &RenderContext<'_>,
    mode: RenderMode,
    citations: Option<Vec<Citation>>,
    show_keys: bool,
    output_format: OutputFormat,
) -> Result<String, Box<dyn Error>> {
    use human::print_human_safe;
    let show_cite = matches!(mode, RenderMode::Cite | RenderMode::Both);
    let show_bib = matches!(mode, RenderMode::Bib | RenderMode::Both);
    match output_format {
        OutputFormat::Plain => {
            print_human_safe::<PlainText>(ctx, show_cite, show_bib, citations, show_keys)
                .map_err(std::convert::Into::into)
        }
        OutputFormat::Html => {
            print_human_safe::<Html>(ctx, show_cite, show_bib, citations, show_keys)
                .map_err(std::convert::Into::into)
        }
        OutputFormat::Djot => {
            print_human_safe::<Djot>(ctx, show_cite, show_bib, citations, show_keys)
                .map_err(std::convert::Into::into)
        }
        OutputFormat::Latex => {
            print_human_safe::<Latex>(ctx, show_cite, show_bib, citations, show_keys)
                .map_err(std::convert::Into::into)
        }
        OutputFormat::Typst => {
            print_human_safe::<Typst>(ctx, show_cite, show_bib, citations, show_keys)
                .map_err(std::convert::Into::into)
        }
    }
}

/// Render bibliography/citation output as a JSON string.
///
/// Builds a JSON object containing rendered citation and/or bibliography entries,
/// keyed by reference ID.
fn render_refs_json(
    ctx: &RenderContext<'_>,
    mode: RenderMode,
    citations: Option<Vec<Citation>>,
    output_format: OutputFormat,
) -> Result<String, Box<dyn Error>> {
    use json::print_json_with_format;
    let show_cite = matches!(mode, RenderMode::Cite | RenderMode::Both);
    let show_bib = matches!(mode, RenderMode::Bib | RenderMode::Both);
    match output_format {
        OutputFormat::Plain => {
            print_json_with_format::<PlainText>(ctx, show_cite, show_bib, citations)
        }
        OutputFormat::Html => print_json_with_format::<Html>(ctx, show_cite, show_bib, citations),
        OutputFormat::Djot => print_json_with_format::<Djot>(ctx, show_cite, show_bib, citations),
        OutputFormat::Latex => print_json_with_format::<Latex>(ctx, show_cite, show_bib, citations),
        OutputFormat::Typst => print_json_with_format::<Typst>(ctx, show_cite, show_bib, citations),
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests"
)]
mod tests {
    use super::*;
    use crate::style_resolver::parse_locale_override_bytes;
    use citum_engine::Bibliography;
    use citum_io::LoadedBibliography;
    use citum_schema::Style;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Plain.to_string(), "plain");
        assert_eq!(OutputFormat::Html.to_string(), "html");
        assert_eq!(OutputFormat::Djot.to_string(), "djot");
        assert_eq!(OutputFormat::Latex.to_string(), "latex");
        assert_eq!(OutputFormat::Typst.to_string(), "typst");
    }

    #[test]
    fn test_load_merged_bibliography_rejects_cross_file_duplicate_membership() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("citum-merged-bib-{now}"));
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
        let msg = err.to_string();
        assert!(
            msg.contains("appears in both compound sets 'group-1' and 'group-2'"),
            "unexpected error: {msg}"
        );

        let _ = std::fs::remove_file(bib_a);
        let _ = std::fs::remove_file(bib_b);
        let _ = std::fs::remove_dir(base);
    }

    #[test]
    fn test_create_processor_applies_locale_override_from_file_style() {
        use citum_schema::options::Config;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("citum-locale-override-{now}"));
        let style_path = base.join("style.yaml");
        let overrides_dir = base.join("locales").join("overrides");
        std::fs::create_dir_all(&overrides_dir).expect("override dir should exist");
        std::fs::write(&style_path, "info: { title: Test Style }\n")
            .expect("style file should write");
        std::fs::write(
            overrides_dir.join("test-override.yaml"),
            r#"
grammar-options:
  punctuation-in-quote: false
  nbsp-before-colon: false
  open-quote: "<<"
  close-quote: ">>"
  open-inner-quote: "<"
  close-inner-quote: ">"
  serial-comma: false
  page-range-delimiter: "~"
"#,
        )
        .expect("override file should write");

        let style = Style {
            info: citum_schema::StyleInfo {
                title: Some("Test Style".into()),
                default_locale: Some("en-US".into()),
                ..Default::default()
            },
            options: Some(Config {
                locale_override: Some("test-override".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let loaded = LoadedBibliography {
            references: Bibliography::new(),
            sets: None,
        };

        let processor = create_processor(
            style,
            loaded,
            style_path.to_str().expect("utf-8 path"),
            false,
            None,
        )
        .expect("processor should apply locale override");

        assert!(!processor.locale.punctuation_in_quote);
        assert_eq!(processor.locale.grammar_options.open_quote, "<<");
        assert_eq!(processor.locale.grammar_options.page_range_delimiter, "~");

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn test_create_processor_applies_builtin_locale_override() {
        let style = citum_schema::embedded::get_embedded_style("chicago")
            .expect("embedded style should exist")
            .expect("embedded style should parse");
        let loaded = LoadedBibliography {
            references: Bibliography::new(),
            sets: None,
        };

        let processor =
            create_processor(style, loaded, "chicago", false, None).expect("processor should load");

        assert_eq!(processor.locale.locale, "en-US");
        assert_eq!(
            processor
                .style
                .options
                .as_ref()
                .and_then(|c| c.locale_override.as_deref()),
            Some("en-US-chicago")
        );
        assert_eq!(
            processor.locale.grammar_options.page_range_delimiter,
            "\u{2013}"
        );
    }

    #[test]
    fn test_create_processor_locale_arg_overrides_style_default() {
        let style = citum_schema::embedded::get_embedded_style("apa")
            .expect("embedded style should exist")
            .expect("embedded style should parse");
        let loaded = LoadedBibliography {
            references: Bibliography::new(),
            sets: None,
        };

        let processor = create_processor(style, loaded, "apa", false, Some("es-ES"))
            .expect("processor should load with locale override");

        assert_eq!(processor.locale.locale, "es-ES");
    }

    #[test]
    fn test_create_processor_locale_arg_skips_style_locale_override() {
        let style = citum_schema::embedded::get_embedded_style("chicago")
            .expect("embedded style should exist")
            .expect("embedded style should parse");
        let loaded = LoadedBibliography {
            references: Bibliography::new(),
            sets: None,
        };

        let processor = create_processor(style, loaded, "chicago", false, Some("es-ES"))
            .expect("processor should load requested locale");

        assert_eq!(processor.locale.locale, "es-ES");
        assert!(!processor.locale.grammar_options.serial_comma);
        assert_eq!(processor.locale.grammar_options.open_quote, "«");
    }

    #[test]
    fn test_create_processor_locale_arg_rejects_unknown_locale() {
        let style = citum_schema::embedded::get_embedded_style("apa")
            .expect("embedded style should exist")
            .expect("embedded style should parse");
        let loaded = LoadedBibliography {
            references: Bibliography::new(),
            sets: None,
        };

        let err = create_processor(style, loaded, "apa", false, Some("zz-ZZ"))
            .expect_err("unknown explicit locale should error");

        assert!(
            err.to_string().contains("locale not found: 'zz-ZZ'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_parse_locale_override_bytes_from_json() {
        let override_data = serde_json::to_vec(&serde_json::json!({
            "messages": { "term.page-label": "pg." },
            "legacy-term-aliases": { "page": "term.page-label" }
        }))
        .expect("json should serialize");

        let parsed = parse_locale_override_bytes(&override_data, "json")
            .expect("override json should parse");

        assert_eq!(
            parsed.messages,
            HashMap::from([(String::from("term.page-label"), String::from("pg."))])
        );
        assert_eq!(
            parsed.legacy_term_aliases.get("page").map(String::as_str),
            Some("term.page-label")
        );
    }
}
