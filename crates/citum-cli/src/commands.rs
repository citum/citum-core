#[cfg(feature = "typescript")]
use crate::args::BindingsArgs;
use crate::args::{
    CheckArgs, CheckItem, Cli, Commands, ConvertCommands, ConvertRefsArgs, ConvertTypedArgs,
    DataType, InputFormat, LintLocaleArgs, LintStyleArgs, LocaleCommands, OutputFormat,
    ParagraphBreakArg, RefsFormat, RegistryCommands, RenderCommands, RenderDocArgs, RenderMode,
    RenderRefsArgs, StoreCommands, StyleCommands, StylesCommands,
};
#[cfg(feature = "schema")]
use crate::args::{SchemaArgs, SchemaType};
use crate::output::{print_lint_report, write_output};
use crate::style_resolver::{create_processor, load_any_style, load_locale_file};
use crate::typst_pdf;
use citum_engine::{
    Citation, CitationItem, DocumentFormat, Processor,
    io::{
        AnnotationFormat, AnnotationStyle, ParagraphBreak, RefsFormat as EngineRefsFormat,
        infer_refs_input_format as infer_engine_refs_input_format,
        infer_refs_output_format as infer_engine_refs_output_format, load_annotations,
        load_bibliography, load_citations, load_input_bibliography, load_merged_bibliography,
        load_merged_citations, write_output_bibliography,
    },
    processor::document::{djot::DjotParser, markdown::MarkdownParser},
    render::{djot::Djot, html::Html, latex::Latex, plain::PlainText, typst::Typst},
};
#[cfg(feature = "schema")]
use citum_schema::InputBibliography;
use citum_schema::Style;
use citum_schema::lint::{lint_raw_locale, lint_style_against_locale};
use citum_schema::locale::RawLocale;
use citum_schema::options::Processing;
use citum_store::{StoreConfig, StoreResolver, platform_data_dir};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
#[cfg(feature = "schema")]
use schemars::schema_for;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

impl From<RefsFormat> for EngineRefsFormat {
    fn from(format: RefsFormat) -> Self {
        match format {
            RefsFormat::CitumYaml => EngineRefsFormat::CitumYaml,
            RefsFormat::CitumJson => EngineRefsFormat::CitumJson,
            RefsFormat::CitumCbor => EngineRefsFormat::CitumCbor,
            RefsFormat::CslJson => EngineRefsFormat::CslJson,
            RefsFormat::Biblatex => EngineRefsFormat::Biblatex,
            RefsFormat::Ris => EngineRefsFormat::Ris,
        }
    }
}

pub(crate) fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render { command } => match command {
            RenderCommands::Doc(args) => run_render_doc(args),
            RenderCommands::Refs(args) => run_render_refs(args),
        },
        Commands::Check(args) => run_check(args),
        Commands::Convert { command } => match command {
            ConvertCommands::Refs(args) => run_convert_refs(args),
            ConvertCommands::Style(args) => run_convert_typed(args, DataType::Style),
            ConvertCommands::Citations(args) => run_convert_typed(args, DataType::Citations),
            ConvertCommands::Locale(args) => run_convert_typed(args, DataType::Locale),
        },
        Commands::Styles { command } => match command.unwrap_or(StylesCommands::List) {
            StylesCommands::List => run_styles_list(),
        },
        Commands::Registry { command } => match command {
            RegistryCommands::List { format } => run_registry_list(&format),
            RegistryCommands::Resolve { name } => run_registry_resolve(&name),
        },
        Commands::Store { command } => match command {
            StoreCommands::List => run_store_list(),
            StoreCommands::Install { source } => run_store_install(&source),
            StoreCommands::Remove { name } => run_store_remove(&name),
        },
        Commands::Style { command } => match command {
            StyleCommands::Lint(args) => run_lint_style(args),
        },
        Commands::Locale { command } => match command {
            LocaleCommands::Lint(args) => run_lint_locale(args),
        },
        #[cfg(feature = "schema")]
        Commands::Schema(args) => run_schema(args),
        #[cfg(feature = "typescript")]
        Commands::Bindings(args) => run_bindings(args),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
            Ok(())
        }
        Commands::Doc(args) => {
            eprintln!(
                "Warning: `citum doc` is deprecated. Use `citum render doc` with positional input."
            );
            let doc_args = RenderDocArgs {
                input: args.document,
                style: args.style.display().to_string(),
                bibliography: vec![args.references],
                citations: Vec::new(),
                input_format: InputFormat::Djot,
                format: args.format,
                output: None,
                pdf: false,
                typst_keep_source: false,
                no_semantics: false,
            };
            run_render_doc(doc_args)
        }
        Commands::Validate(args) => {
            eprintln!("Warning: `citum validate` is deprecated. Use `citum check --style`.");
            run_check(CheckArgs {
                style: Some(args.path.display().to_string()),
                bibliography: Vec::new(),
                citations: Vec::new(),
                json: false,
            })
        }
    }
}

#[cfg(feature = "schema")]
fn run_schema(args: SchemaArgs) -> Result<(), Box<dyn Error>> {
    if let Some(dir) = args.out_dir {
        fs::create_dir_all(&dir)?;
        let schemas = [
            ("style", schema_for!(Style)),
            ("bib", schema_for!(InputBibliography)),
            ("locale", schema_for!(RawLocale)),
            ("citation", schema_for!(citum_schema::Citations)),
            ("registry", schema_for!(citum_schema::StyleRegistry)),
        ];
        for (name, schema) in schemas {
            let filename = format!("{name}.json");
            let path = dir.join(&filename);
            fs::write(&path, serde_json::to_string_pretty(&schema)?)?;
        }
        println!("Schemas exported to {}", dir.display());
        return Ok(());
    }

    if let Some(t) = args.r#type {
        let schema = match t {
            SchemaType::Style => schema_for!(Style),
            SchemaType::Bib => schema_for!(InputBibliography),
            SchemaType::Locale => schema_for!(RawLocale),
            SchemaType::Citation => schema_for!(citum_schema::Citations),
            SchemaType::Registry => schema_for!(citum_schema::StyleRegistry),
        };
        println!("{}", serde_json::to_string_pretty(&schema)?);
        return Ok(());
    }

    Err("Specify a schema type (style, bib, locale, citation, registry) or --out-dir".into())
}

#[cfg(feature = "typescript")]
fn run_bindings(args: BindingsArgs) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&args.out_dir)?;
    let out_path = args.out_dir.join("citum.d.ts");
    citum_bindings::export_typescript(&out_path).map_err(|e| format!("{e}"))?;
    println!("TypeScript bindings exported to {}", out_path.display());
    Ok(())
}

fn run_styles_list() -> Result<(), Box<dyn Error>> {
    println!("Embedded (builtin) citation styles:");
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Alias").fg(Color::Cyan),
            Cell::new("Title").fg(Color::Cyan),
            Cell::new("Full Name").fg(Color::Cyan),
        ]);

    for name in citum_schema::embedded::EMBEDDED_STYLE_NAMES {
        let style = citum_schema::embedded::get_embedded_style(name)
            .ok_or_else(|| format!("failed to load builtin style: {name}"))??;

        let alias = citum_schema::embedded::EMBEDDED_STYLE_ALIASES
            .iter()
            .find(|(_, full)| *full == *name)
            .map_or("-", |(a, _)| *a);

        let title = style.info.title.as_deref().unwrap_or("-");

        table.add_row(vec![Cell::new(alias), Cell::new(title), Cell::new(name)]);
    }

    println!("{table}");

    println!();
    println!("Usage:");
    println!("  citum render refs -s <alias|name> -b refs.json");
    println!("  citum render doc <doc.dj> -s <alias|name> -b refs.json");
    Ok(())
}

/// List all styles in the registry.
fn run_registry_list(format: &str) -> Result<(), Box<dyn Error>> {
    let registry = citum_schema::embedded::default_registry();

    if format == "json" {
        let json = serde_json::to_string_pretty(&registry)?;
        println!("{}", json);
    } else {
        println!("Style Registry:");
        println!();

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("ID").fg(Color::Cyan),
                Cell::new("Aliases").fg(Color::Cyan),
                Cell::new("Description").fg(Color::Cyan),
            ]);

        for entry in &registry.styles {
            let aliases = if entry.aliases.is_empty() {
                "-".to_string()
            } else {
                entry.aliases.join(", ")
            };
            let description = entry.description.as_deref().unwrap_or("-");

            table.add_row(vec![
                Cell::new(&entry.id),
                Cell::new(aliases),
                Cell::new(description),
            ]);
        }

        println!("{table}");
        println!();
    }

    Ok(())
}

/// Resolve a style name or alias in the registry.
fn run_registry_resolve(name: &str) -> Result<(), Box<dyn Error>> {
    let registry = citum_schema::embedded::default_registry();

    if let Some(entry) = registry.resolve(name) {
        let source = if entry.builtin.is_some() {
            "builtin"
        } else if entry.path.is_some() {
            "path"
        } else {
            "unknown"
        };
        println!("{} ({})", entry.id, source);
        Ok(())
    } else {
        eprintln!("Error: style not found: {name}");
        std::process::exit(1);
    }
}

/// List all installed user styles and locales.
fn run_store_list() -> Result<(), Box<dyn Error>> {
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };

    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir.clone(), config.store_format());

    let styles = resolver.list_styles().unwrap_or_default();
    let locales = resolver.list_locales().unwrap_or_default();

    println!("User store location: {}", data_dir.display());
    println!("Configured format: {}", config.store_format());
    println!();

    if styles.is_empty() {
        println!("No installed styles.");
        println!();
    } else {
        println!("Installed styles ({}):", styles.len());
        for name in &styles {
            println!("  - {name}");
        }
        println!();
    }

    if locales.is_empty() {
        println!("No installed locales.");
        println!();
    } else {
        println!("Installed locales ({}):", locales.len());
        for name in &locales {
            println!("  - {name}");
        }
        println!();
    }

    println!("Usage:");
    println!("  Install a style:  citum store install <path>");
    println!("  Remove a style:   citum store remove <name>");

    Ok(())
}

/// Install a style or locale from a local file.
fn run_store_install(source: &Path) -> Result<(), Box<dyn Error>> {
    if !source.exists() || !source.is_file() {
        return Err(format!("file not found: {}", source.display()).into());
    }

    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };

    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());

    let name = resolver
        .install_style(source)
        .or_else(|_| resolver.install_locale(source))?;

    println!("Successfully installed: {name}");
    Ok(())
}

/// Remove an installed style or locale.
fn run_store_remove(name: &str) -> Result<(), Box<dyn Error>> {
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };

    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());

    // Check if style or locale exists
    let styles = resolver.list_styles().unwrap_or_default();
    let locales = resolver.list_locales().unwrap_or_default();

    if !styles.contains(&name.to_string()) && !locales.contains(&name.to_string()) {
        return Err(format!("style or locale not found: {name}").into());
    }

    // Ask for confirmation
    print!("Are you sure you want to remove '{name}'? This cannot be undone. [y/N] ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" && response.trim().to_lowercase() != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    // Try removing as style first, then as locale
    resolver
        .remove_style(name)
        .or_else(|_| resolver.remove_locale(name))?;

    println!("Successfully removed: {name}");
    Ok(())
}

fn run_lint_locale(args: LintLocaleArgs) -> Result<(), Box<dyn Error>> {
    let raw = load_raw_locale(&args.path)?;
    let report = lint_raw_locale(&raw);
    print_lint_report("locale lint", &report);
    if report.has_errors() {
        return Err(format!("Locale lint failed: {}", args.path.display()).into());
    }
    Ok(())
}

fn run_lint_style(args: LintStyleArgs) -> Result<(), Box<dyn Error>> {
    let style = load_any_style(&args.style, false)?;
    let locale = load_locale_file(&args.locale)?;
    let report = lint_style_against_locale(&style, &locale);
    print_lint_report("style lint", &report);
    Ok(())
}

fn load_raw_locale(path: &Path) -> Result<RawLocale, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    let raw = match ext {
        "cbor" => ciborium::de::from_reader::<RawLocale, _>(std::io::Cursor::new(&bytes))?,
        "json" => serde_json::from_slice::<RawLocale>(&bytes)?,
        _ => serde_yaml::from_slice::<RawLocale>(&bytes)?,
    };

    Ok(raw)
}

/// Execute the `render doc` subcommand.
///
/// Reads a document, resolves citations against the provided bibliography,
/// and writes the rendered output to stdout or a file.
fn run_render_doc(args: RenderDocArgs) -> Result<(), Box<dyn Error>> {
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
fn run_render_refs(args: RenderRefsArgs) -> Result<(), Box<dyn Error>> {
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
        italic: args.annotation_italic,
        paragraph_break: match args.annotation_break {
            ParagraphBreakArg::BlankLine => ParagraphBreak::BlankLine,
            ParagraphBreakArg::SingleLine => ParagraphBreak::SingleLine,
        },
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

/// Execute the `check` subcommand.
///
/// Attempts to load each provided style, bibliography, and citations file,
/// reporting per-item pass/fail results.  Exits with an error when any check fails.
fn run_check(args: CheckArgs) -> Result<(), Box<dyn Error>> {
    let mut checks = Vec::<CheckItem>::new();

    if let Some(style_input) = args.style {
        let status = match load_any_style(&style_input, false) {
            Ok(_) => CheckItem {
                kind: "style",
                path: style_input,
                ok: true,
                error: None,
            },
            Err(e) => CheckItem {
                kind: "style",
                path: style_input,
                ok: false,
                error: Some(e.to_string()),
            },
        };
        checks.push(status);
    }

    for path in args.bibliography {
        let display = path.display().to_string();
        let status = match load_bibliography(&path) {
            Ok(_) => CheckItem {
                kind: "bibliography",
                path: display,
                ok: true,
                error: None,
            },
            Err(e) => CheckItem {
                kind: "bibliography",
                path: display,
                ok: false,
                error: Some(e.to_string()),
            },
        };
        checks.push(status);
    }

    for path in args.citations {
        let display = path.display().to_string();
        let status = match load_citations(&path) {
            Ok(_) => CheckItem {
                kind: "citations",
                path: display,
                ok: true,
                error: None,
            },
            Err(e) => CheckItem {
                kind: "citations",
                path: display,
                ok: false,
                error: Some(e.to_string()),
            },
        };
        checks.push(status);
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&checks)?);
    } else {
        for check in &checks {
            if check.ok {
                println!("OK   {:<12} {}", check.kind, check.path);
            } else {
                println!("FAIL {:<12} {}", check.kind, check.path);
                if let Some(err) = &check.error {
                    println!("  -> {err}");
                }
            }
        }
    }

    if checks.iter().any(|c| !c.ok) {
        return Err("One or more checks failed.".into());
    }

    Ok(())
}

/// Execute typed conversion subcommands (`style`, `locale`, `citations`).
fn run_convert_typed(args: ConvertTypedArgs, data_type: DataType) -> Result<(), Box<dyn Error>> {
    let input_bytes = fs::read(&args.input)?;
    let input_ext = args
        .input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("yaml");
    let output_ext = args
        .output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("yaml");

    match data_type {
        DataType::Style => {
            let style: Style = deserialize_any(&input_bytes, input_ext)?;
            let out_bytes = serialize_any(&style, output_ext)?;
            fs::write(&args.output, out_bytes)?;
        }
        DataType::Locale => {
            let locale: RawLocale = deserialize_any(&input_bytes, input_ext)?;
            let out_bytes = serialize_any(&locale, output_ext)?;
            fs::write(&args.output, out_bytes)?;
        }
        DataType::Citations => {
            let citations: citum_schema::citation::Citations =
                deserialize_any(&input_bytes, input_ext)?;
            let out_bytes = serialize_any(&citations, output_ext)?;
            fs::write(&args.output, out_bytes)?;
        }
        DataType::Bib => {
            return Err("`convert bib` was replaced by `convert refs`.".into());
        }
    }

    println!(
        "Converted {} to {}",
        args.input.display(),
        args.output.display()
    );
    Ok(())
}

/// Execute `convert refs` for native and legacy bibliography formats.
fn run_convert_refs(args: ConvertRefsArgs) -> Result<(), Box<dyn Error>> {
    let input_format = if let Some(f) = args.from {
        f.into()
    } else {
        infer_engine_refs_input_format(&args.input)?
    };
    let output_format = args
        .to
        .map(Into::into)
        .unwrap_or_else(|| infer_engine_refs_output_format(&args.output));

    let bibliography = load_input_bibliography(&args.input, input_format)?;
    write_output_bibliography(&bibliography, &args.output, output_format)?;

    println!(
        "Converted {} ({:?}) to {} ({:?})",
        args.input.display(),
        input_format,
        args.output.display(),
        output_format
    );
    Ok(())
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
struct RenderContext<'a> {
    /// The configured citation processor.
    processor: &'a citum_engine::Processor,
    /// Display name of the active style.
    style_name: &'a str,
    /// Reference IDs to render.
    item_ids: &'a [String],
    /// Optional annotation map (reference ID → annotation text).
    annotations: Option<&'a HashMap<String, String>>,
    /// Formatting style for annotations.
    annotation_style: &'a citum_engine::io::AnnotationStyle,
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

/// Deserialise bytes into `T` using the format indicated by `ext`.
///
/// Recognised extensions: `yaml` / `yml`, `json`, `cbor`.  All other values
/// fall back to YAML.
fn deserialize_any<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    ext: &str,
) -> Result<T, Box<dyn Error>> {
    match ext {
        "yaml" | "yml" => Ok(serde_yaml::from_slice(bytes)?),
        "json" => Ok(serde_json::from_slice(bytes)?),
        "cbor" => Ok(ciborium::de::from_reader(std::io::Cursor::new(bytes))?),
        _ => Ok(serde_yaml::from_slice(bytes)?),
    }
}

/// Serialise `obj` to bytes using the format indicated by `ext`.
///
/// Recognised extensions: `yaml` / `yml` (default), `json`, `cbor`.
fn serialize_any<T: Serialize>(obj: &T, ext: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    match ext {
        "yaml" | "yml" => Ok(serde_yaml::to_string(obj)?.into_bytes()),
        "json" => Ok(serde_json::to_string_pretty(obj)?.into_bytes()),
        "cbor" => {
            let mut buf = Vec::new();
            ciborium::ser::into_writer(obj, &mut buf)?;
            Ok(buf)
        }
        _ => Ok(serde_yaml::to_string(obj)?.into_bytes()),
    }
}

/// Panic-safe wrapper around [`print_human`].
///
/// Catches any Rust panics that escape the processor and converts them into an
/// `Err` with a user-friendly message, preventing the CLI from crashing.
fn print_human_safe<F>(
    ctx: &RenderContext<'_>,
    show_cite: bool,
    show_bib: bool,
    citations: Option<Vec<Citation>>,
    show_keys: bool,
) -> Result<String, String>
where
    F: citum_engine::render::format::OutputFormat<Output = String> + Send + Sync + 'static,
{
    use std::panic;

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        print_human::<F>(ctx, show_cite, show_bib, citations, show_keys)
    }));

    match result {
        Ok(output) => Ok(output),
        Err(_) => Err(
            "The processor encountered a critical error during rendering. Please report this issue with the style and data used."
                .to_string(),
        ),
    }
}

/// Render the citations section into output.
fn render_citations_section<F>(
    ctx: &RenderContext<'_>,
    citations: Option<Vec<Citation>>,
    show_keys: bool,
    output: &mut String,
) where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    if let Some(cite_list) = citations {
        let _ = writeln!(output, "CITATIONS (From file):");
        for (i, (citation, text)) in cite_list
            .iter()
            .zip(render_citation_file_entries::<F>(ctx.processor, &cite_list))
            .enumerate()
        {
            if show_keys {
                let _ = writeln!(
                    output,
                    "  [{}] {}",
                    citation.id.as_deref().unwrap_or(&format!("{i}")),
                    text
                );
            } else {
                let _ = writeln!(output, "  {text}");
            }
        }
    } else {
        let _ = writeln!(output, "CITATIONS (Non-Integral):");
        for id in ctx.item_ids {
            let citation = Citation {
                id: Some(id.clone()),
                items: vec![CitationItem {
                    id: id.clone(),
                    ..Default::default()
                }],
                mode: citum_schema::citation::CitationMode::NonIntegral,
                ..Default::default()
            };
            match ctx.processor.process_citation_with_format::<F>(&citation) {
                Ok(text) => {
                    if show_keys {
                        let _ = writeln!(output, "  [{id}] {text}");
                    } else {
                        let _ = writeln!(output, "  {text}");
                    }
                }
                Err(e) => {
                    let _ = writeln!(output, "  [{id}] ERROR: {e}");
                }
            }
        }
        let _ = writeln!(output);

        let _ = writeln!(output, "CITATIONS (Integral):");
        for id in ctx.item_ids {
            let citation = Citation {
                id: Some(id.clone()),
                items: vec![CitationItem {
                    id: id.clone(),
                    ..Default::default()
                }],
                mode: citum_schema::citation::CitationMode::Integral,
                ..Default::default()
            };
            match ctx.processor.process_citation_with_format::<F>(&citation) {
                Ok(text) => {
                    if show_keys {
                        let _ = writeln!(output, "  [{id}] {text}");
                    } else {
                        let _ = writeln!(output, "  {text}");
                    }
                }
                Err(e) => {
                    let _ = writeln!(output, "  [{id}] ERROR: {e}");
                }
            }
        }
    }
    let _ = writeln!(output);
}

/// Render the bibliography section into output.
fn render_bibliography_section<F>(ctx: &RenderContext<'_>, show_keys: bool, output: &mut String)
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    let _ = writeln!(output, "BIBLIOGRAPHY:");
    if show_keys {
        // When show_keys is requested, render each entry with its ID prefix so the
        // oracle parser can match entries by key. Group headings are omitted in this
        // mode because the oracle only looks for `[id] text` patterns.
        let filter: HashSet<&str> = ctx
            .item_ids
            .iter()
            .map(std::string::String::as_str)
            .collect();
        let processed = ctx.processor.process_references();
        for entry in processed.bibliography {
            if filter.contains(entry.id.as_str()) {
                let text = citum_engine::render::refs_to_string_with_format::<F>(
                    vec![entry.clone()],
                    ctx.annotations,
                    Some(ctx.annotation_style),
                );
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let _ = writeln!(output, "  [{}] {}", entry.id, trimmed);
                }
            }
        }
    } else {
        // Use engine's built-in bibliography renderer which handles grouping/partitioning.
        // We use render_selected_bibliography_with_format_and_annotations to respect the CLI's item_ids filter and propagate annotations.
        let rendered = ctx
            .processor
            .render_selected_bibliography_with_format_and_annotations::<F, _>(
                ctx.item_ids.to_vec(),
                ctx.annotations,
                Some(ctx.annotation_style),
            );
        output.push_str(&rendered);
        if !rendered.is_empty() && !rendered.ends_with('\n') {
            output.push('\n');
        }
    }
}

/// Core human-readable renderer for references and citations.
///
/// Builds a formatted string containing citation clusters and/or bibliography
/// entries, optionally prefixed with reference IDs when `show_keys` is `true`.
fn print_human<F>(
    ctx: &RenderContext<'_>,
    show_cite: bool,
    show_bib: bool,
    citations: Option<Vec<Citation>>,
    show_keys: bool,
) -> String
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    let mut output = String::new();
    let _ = writeln!(output, "\n=== {} ===\n", ctx.style_name);

    if show_cite {
        render_citations_section::<F>(ctx, citations, show_keys, &mut output);
    }

    if show_bib {
        render_bibliography_section::<F>(ctx, show_keys, &mut output);
    }

    output
}

/// Render citation-file inputs in batch mode, falling back to per-citation rendering on error.
fn render_citation_file_entries<F>(processor: &Processor, citations: &[Citation]) -> Vec<String>
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    let is_numeric = processor
        .style
        .options
        .as_ref()
        .and_then(|config| config.processing.as_ref())
        .is_some_and(|processing| matches!(processing, Processing::Numeric));

    if is_numeric {
        processor
            .process_citations_with_format::<F>(citations)
            .unwrap_or_else(|_| render_citation_file_entries_one_by_one::<F>(processor, citations))
    } else {
        render_citation_file_entries_one_by_one::<F>(processor, citations)
    }
}

fn format_citation_file_render_error(error: impl std::fmt::Display) -> String {
    format!("ERROR: {error}")
}

/// Render citation-file inputs independently to avoid cross-citation context.
fn render_citation_file_entries_one_by_one<F>(
    processor: &Processor,
    citations: &[Citation],
) -> Vec<String>
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    citations
        .iter()
        .map(|citation| {
            processor
                .process_citation_with_format::<F>(citation)
                .unwrap_or_else(format_citation_file_render_error)
        })
        .collect()
}

/// Core JSON renderer for references and citations.
///
/// Returns a pretty-printed JSON object with `style`, `items`, and optionally
/// `citations` and `bibliography` keys.
#[allow(
    clippy::too_many_lines,
    reason = "JSON output construction is vertically long"
)]
fn print_json_with_format<F>(
    ctx: &RenderContext<'_>,
    show_cite: bool,
    show_bib: bool,
    citations: Option<Vec<Citation>>,
) -> Result<String, Box<dyn Error>>
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    use serde_json::json;

    let mut result = json!({
        "style": ctx.style_name,
        "items": ctx.item_ids.len()
    });

    if show_cite {
        if let Some(cite_list) = citations {
            let rendered: Vec<_> = cite_list
                .iter()
                .zip(render_citation_file_entries::<F>(ctx.processor, &cite_list))
                .map(|(citation, text)| {
                    json!({
                        "id": citation.id,
                        "text": text
                    })
                })
                .collect();
            #[allow(clippy::indexing_slicing, reason = "JSON object insertion")]
            {
                result["citations"] = json!(rendered);
            }
        } else {
            let non_integral: Vec<_> = ctx
                .item_ids
                .iter()
                .map(|id| {
                    let citation = Citation {
                        id: Some(id.clone()),
                        items: vec![CitationItem {
                            id: id.clone(),
                            ..Default::default()
                        }],
                        mode: citum_schema::citation::CitationMode::NonIntegral,
                        ..Default::default()
                    };
                    json!({
                        "id": id,
                        "text": ctx.processor
                            .process_citation_with_format::<F>(&citation)
                            .unwrap_or_else(|e| e.to_string())
                    })
                })
                .collect();

            let integral: Vec<_> = ctx
                .item_ids
                .iter()
                .map(|id| {
                    let citation = Citation {
                        id: Some(id.clone()),
                        items: vec![CitationItem {
                            id: id.clone(),
                            ..Default::default()
                        }],
                        mode: citum_schema::citation::CitationMode::Integral,
                        ..Default::default()
                    };
                    json!({
                        "id": id,
                        "text": ctx.processor
                            .process_citation_with_format::<F>(&citation)
                            .unwrap_or_else(|e| e.to_string())
                    })
                })
                .collect();

            #[allow(clippy::indexing_slicing, reason = "JSON object insertion")]
            {
                result["citations"] = json!({
                    "non-integral": non_integral,
                    "integral": integral
                });
            }
        }
    }

    if show_bib {
        let filter: HashSet<&str> = ctx
            .item_ids
            .iter()
            .map(std::string::String::as_str)
            .collect();
        let processed = ctx.processor.process_references();
        let entries: Vec<_> = processed
            .bibliography
            .into_iter()
            .filter(|entry| filter.contains(entry.id.as_str()))
            .map(|entry| {
                let text = citum_engine::render::refs_to_string_with_format::<F>(
                    vec![entry.clone()],
                    ctx.annotations,
                    Some(ctx.annotation_style),
                );
                json!({
                    "id": entry.id,
                    "text": text.trim()
                })
            })
            .collect();

        #[allow(clippy::indexing_slicing, reason = "JSON object insertion")]
        {
            result["bibliography"] = json!({ "entries": entries });
        }
    }

    Ok(serde_json::to_string_pretty(&result)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
    use crate::style_resolver::parse_locale_override_bytes;
    use citum_engine::{Bibliography, io::LoadedBibliography};
    use citum_schema::citation::CitationMode;
    use citum_schema::grouping::{GroupSort, GroupSortEntry, GroupSortKey, SortKey};
    use citum_schema::options::{Config, Processing};
    use citum_schema::template::{
        NumberVariable, TemplateComponent, TemplateNumber, WrapPunctuation,
    };
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    // ------------------------------------------------------------------
    // OutputFormat Display
    // ------------------------------------------------------------------

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Plain.to_string(), "plain");
        assert_eq!(OutputFormat::Html.to_string(), "html");
        assert_eq!(OutputFormat::Djot.to_string(), "djot");
        assert_eq!(OutputFormat::Latex.to_string(), "latex");
        assert_eq!(OutputFormat::Typst.to_string(), "typst");
    }

    // ------------------------------------------------------------------
    // convert refs format inference
    // ------------------------------------------------------------------

    #[test]
    fn test_infer_refs_output_format_yaml() {
        assert!(matches!(
            infer_engine_refs_output_format(Path::new("refs.yaml")),
            EngineRefsFormat::CitumYaml
        ));
    }

    #[test]
    fn test_infer_refs_output_format_bib() {
        assert!(matches!(
            infer_engine_refs_output_format(Path::new("refs.bib")),
            EngineRefsFormat::Biblatex
        ));
    }

    #[test]
    fn test_infer_refs_output_format_ris() {
        assert!(matches!(
            infer_engine_refs_output_format(Path::new("refs.ris")),
            EngineRefsFormat::Ris
        ));
    }

    #[test]
    fn test_format_citation_file_render_error_prefixes_message() {
        assert_eq!(format_citation_file_render_error("boom"), "ERROR: boom");
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
    fn test_load_citum_json_bibliography_keeps_standalone_edited_book_as_book_type() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/references-humanities-note.json");

        let bibliography = load_input_bibliography(&fixture, EngineRefsFormat::CitumJson)
            .expect("fixture should parse as native Citum");
        let edited_book = bibliography
            .references
            .iter()
            .find(|reference| reference.id().as_deref() == Some("burke2010-ed"))
            .expect("edited book fixture should exist");

        assert_eq!(edited_book.ref_type(), "book");
    }

    #[test]
    fn test_load_citum_json_bibliography_preserves_hybrid_edited_book_url() {
        let bytes = br#"[
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

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("citum-hybrid-json-{now}.json"));
        std::fs::write(&path, bytes).expect("hybrid fixture should write");

        let bibliography = load_input_bibliography(&path, EngineRefsFormat::CitumJson)
            .expect("hybrid JSON should parse as native Citum");
        let edited_book = bibliography
            .references
            .iter()
            .find(|reference| reference.id().as_deref() == Some("edited-book-1"))
            .expect("edited book fixture should exist");

        assert_eq!(
            edited_book.url().as_ref().map(url::Url::as_str),
            Some("https://example.com/edited-book")
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_print_json_batches_numeric_citation_files() {
        let style = Style {
            citation: Some(citum_schema::CitationSpec {
                template: Some(vec![TemplateComponent::Number(TemplateNumber {
                    number: NumberVariable::CitationNumber,
                    ..Default::default()
                })]),
                wrap: Some(WrapPunctuation::Brackets.into()),
                ..Default::default()
            }),
            bibliography: Some(citum_schema::BibliographySpec {
                sort: Some(GroupSortEntry::Explicit(GroupSort {
                    template: vec![GroupSortKey {
                        key: SortKey::Author,
                        ascending: true,
                        order: None,
                        sort_order: None,
                    }],
                })),
                ..Default::default()
            }),
            options: Some(Config {
                processing: Some(Processing::Numeric),
                ..Default::default()
            }),
            ..Default::default()
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("citum-batch-citations-{now}"));
        std::fs::create_dir_all(&base).expect("temp dir should be created");
        let bib_path = base.join("refs.yaml");
        std::fs::write(
            &bib_path,
            r#"
references:
  - class: monograph
    id: smith2020
    type: book
    title: Smith Book
    author:
      - family: Smith
        given: Jane
    issued: "2020"
  - class: monograph
    id: adams2021
    type: book
    title: Adams Book
    author:
      - family: Adams
        given: Amy
    issued: "2021"
"#,
        )
        .expect("fixture should write");

        let loaded = load_merged_bibliography(std::slice::from_ref(&bib_path))
            .expect("bibliography should load");
        let processor = Processor::new(style, loaded.references);
        let citations = vec![Citation {
            id: Some("c1".into()),
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                ..Default::default()
            }],
            mode: CitationMode::NonIntegral,
            ..Default::default()
        }];

        let item_ids = vec!["smith2020".to_string(), "adams2021".to_string()];
        let annotation_style = AnnotationStyle::default();
        let render_ctx = RenderContext {
            processor: &processor,
            style_name: "numeric-test",
            item_ids: &item_ids,
            annotations: None,
            annotation_style: &annotation_style,
        };
        let output = print_json_with_format::<PlainText>(&render_ctx, true, false, Some(citations))
            .expect("json rendering should succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("output should be valid JSON");

        assert_eq!(parsed["citations"][0]["text"], "[2]");

        let _ = std::fs::remove_file(bib_path);
        let _ = std::fs::remove_dir(base);
    }

    #[test]
    fn test_create_processor_applies_locale_override_from_file_style() {
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
