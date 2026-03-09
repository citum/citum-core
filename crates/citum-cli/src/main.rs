use citum_engine::{
    Bibliography, Citation, CitationItem, DocumentFormat, Processor,
    io::{
        AnnotationFormat, AnnotationStyle, LoadedBibliography, ParagraphBreak, load_annotations,
        load_bibliography, load_bibliography_with_sets, load_citations, validate_compound_sets,
    },
    processor::document::{djot::DjotParser, markdown::MarkdownParser},
    render::{djot::Djot, html::Html, latex::Latex, plain::PlainText, typst::Typst},
};
use citum_schema::locale::RawLocale;
use citum_schema::options::Processing;
use citum_schema::reference::InputReference;
use citum_schema::{InputBibliography, Locale, Style};
use citum_store::{StoreConfig, StoreResolver, platform_data_dir};
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{ArgAction, Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
#[cfg(feature = "schema")]
use schemars::schema_for;
use serde::Serialize;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

mod typst_pdf;

const CLAP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser)]
#[command(
    name = "citum",
    author,
    version,
    about = "Modern, performant, and multilingual citation, bibliography, and document processor",
    long_about = "Citum is a Rust-based, declarative citation styling system.\n\n\
                  Styles are expressed as YAML templates and options, then rendered\n\
                  by a type-safe processor.\n\n\
                  EXAMPLES:\n  \
                  Render a document:\n    \
                  citum render doc input.djot -b refs.json -s apa-7th\n\n  \
                  Render references (human-readable):\n    \
                  citum render refs -b refs.json -s apa-7th\n\n  \
                  Check style and bibliography:\n    \
                  citum check -s apa-7th -b refs.json\n\n  \
                  Convert a style to binary CBOR:\n    \
                  citum convert style.yaml -o style.cbor\n\n  \
                  List all builtin styles:\n    \
                  citum styles list\n\n\
                  Run 'citum <COMMAND> --help' for more detailed examples and options.",
    styles = CLAP_STYLES,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum DataType {
    Style,
    Bib,
    Locale,
    Citations,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum RenderMode {
    Bib,
    Cite,
    Both,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum InputFormat {
    Djot,
    Markdown,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum OutputFormat {
    Plain,
    Html,
    Djot,
    Latex,
    Typst,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Plain => write!(f, "plain"),
            OutputFormat::Html => write!(f, "html"),
            OutputFormat::Djot => write!(f, "djot"),
            OutputFormat::Latex => write!(f, "latex"),
            OutputFormat::Typst => write!(f, "typst"),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Render documents or references
    Render {
        #[command(subcommand)]
        command: RenderCommands,
    },

    /// Validate style, bibliography, and citations files
    #[command(
        about = "Validate style, bibliography, and citations files",
        long_about = "Perform schema validation on input files.\n\n\
                      Citum checks the syntax and structure of style (YAML/JSON/CBOR),\n\
                      bibliography (JSON/YAML), and citation files against their\n\
                      respective schemas. Use this to ensure your data is compatible\n\
                      before processing.\n\n\
                      EXAMPLES:\n  \
                      Validate a style and its bibliography:\n    \
                      citum check -s apa-7th -b refs.json\n\n  \
                      Validate and output detailed results as JSON:\n    \
                      citum check -s apa-7th -b refs.json --json"
    )]
    Check(CheckArgs),

    /// Convert between CSLN formats (YAML, JSON, CBOR)
    #[command(
        about = "Convert between CSLN formats (YAML, JSON, CBOR)",
        long_about = "Convert between CSLN formats (YAML, JSON, CBOR).\n\n\
                      The tool automatically detects the data type (style, bib, locale,\n\
                      or citations) based on file stems and extensions, but this can\n\
                      be explicitly overridden with the --type flag.\n\n\
                      EXAMPLES:\n  \
                      Convert a style from YAML to binary CBOR:\n    \
                      citum convert style.yaml -o style.cbor\n\n  \
                      Convert a bibliography from JSON to YAML:\n    \
                      citum convert refs.json -o refs.yaml\n\n  \
                      Convert citations with explicit type override:\n    \
                      citum convert input.data -o citations.json -t citations"
    )]
    Convert(ConvertArgs),

    /// List and inspect embedded (builtin) citation styles
    #[command(
        about = "List and inspect embedded (builtin) citation styles",
        long_about = "Browse and inspect Citum's library of embedded citation styles.\n\n\
                      Citum includes several standard styles (APA, MLA, Chicago, etc.)\n\
                      built directly into the binary. You can reference these styles\n\
                      by their alias (e.g., 'apa-7th') instead of a file path.\n\n\
                      EXAMPLES:\n  \
                      List all builtin styles and their aliases:\n    \
                      citum styles list"
    )]
    Styles {
        #[command(subcommand)]
        command: Option<StylesCommands>,
    },

    /// Manage user-installed styles and locales
    #[command(
        about = "Manage user-installed styles and locales",
        long_about = "Install, remove, and list user-owned styles and locales.\n\n\
                      Stored in the platform-specific user data directory (for example,\n\
                      ~/.local/share/citum/ on Linux, ~/Library/Application Support/citum/ on\n\
                      macOS, or %APPDATA%\\citum\\ on Windows) and checked before builtin styles\n\
                      when resolving names.\n\n\
                      EXAMPLES:\n  \
                      List all installed styles and locales:\n    \
                      citum store list\n\n  \
                      Install a style from a local file:\n    \
                      citum store install /path/to/my-style.yaml\n\n  \
                      Remove an installed style:\n    \
                      citum store remove my-style"
    )]
    Store {
        #[command(subcommand)]
        command: StoreCommands,
    },

    /// Generate JSON schema for CSLN models
    #[cfg(feature = "schema")]
    Schema(SchemaArgs),

    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for
        shell: Shell,
    },

    /// Legacy alias for `render doc`
    #[command(hide = true)]
    Doc(LegacyDocArgs),

    /// Legacy alias for `check --style`
    #[command(hide = true)]
    Validate(LegacyValidateArgs),
}

#[derive(Subcommand)]
#[command(
    about = "Render documents or references",
    long_about = "Render documents or references using a specified citation style.\n\n\
                  Citum supports two primary rendering modes:\n\
                  - doc: Process a full document (Djot or Markdown) with integrated citations.\n\
                  - refs: Direct rendering of a bibliography file for debugging\n\
                    or inspection.\n\n\
                  Run 'citum render <COMMAND> --help' for specific examples."
)]
enum RenderCommands {
    /// Render a full document with citations and bibliography
    #[command(
        about = "Render a full document with citations and bibliography",
        long_about = "Process a full document with citations and bibliography.\n\n\
                      Citum parses the input document (default: Djot) for citations,\n\
                      matches them against the provided bibliography, and renders\n\
                      the final output in various formats (Plain, HTML, Latex, etc.).\n\n\
                      EXAMPLES:\n  \
                      Render to HTML:\n    \
                      citum render doc manuscript.djot -b refs.json -s apa-7th -f html\n\n  \
                      Render Markdown with Pandoc-style citations:\n    \
                      citum render doc manuscript.md --input-format markdown -b refs.json -s apa-7th\n\n  \
                      Render to PDF (requires 'typst-pdf' feature):\n    \
                      citum render doc manuscript.djot -b refs.json -s apa-7th\n\
                      -f typst -o paper.pdf --pdf"
    )]
    Doc(RenderDocArgs),

    /// Render references/citations directly
    #[command(
        about = "Render references/citations directly",
        long_about = "Directly render a set of references and/or citations from files.\n\n\
                      This command is useful for inspecting how a style renders\n\
                      specific entries or testing bibliography grouping logic.\n\n\
                      EXAMPLES:\n  \
                      Render bibliography entries (APA 7th style):\n    \
                      citum render refs -b refs.json -s apa-7th\n\n  \
                      Render specific citations with keys:\n    \
                      citum render refs -b refs.json -s apa-7th -m cite\n\
                      -k Doe2020,Smith2021\n\n  \
                      Output as JSON with human-readable rendered text:\n    \
                      citum render refs -b refs.json -s apa-7th --json"
    )]
    Refs(RenderRefsArgs),
}

#[derive(Subcommand)]
enum StylesCommands {
    /// List all embedded (builtin) style names
    List,
}

#[derive(Subcommand)]
enum StoreCommands {
    /// List all installed user styles and locales
    #[command(
        about = "List all installed user styles and locales",
        long_about = "Display names of all styles and locales installed in the user store\n\
                      directory. Does not include embedded/builtin styles."
    )]
    List,

    /// Install a style or locale from a local file
    #[command(
        about = "Install a style or locale from a local file",
        long_about = "Copy a local style or locale file into the user store directory.\n\
                      The style/locale name is derived from the file stem (without extension).\n\
                      Supports YAML, JSON, and CBOR formats.\n\n\
                      EXAMPLES:\n  \
                      Install a style:\n    \
                      citum store install /path/to/my-custom-style.yaml\n\n  \
                      Install a locale:\n    \
                      citum store install /path/to/my-locale.yaml"
    )]
    Install {
        /// Path to the style or locale file to install
        #[arg(index = 1, required = true)]
        source: PathBuf,
    },

    /// Remove an installed style or locale
    #[command(
        about = "Remove an installed style or locale",
        long_about = "Delete a style or locale from the user store directory.\n\
                      Requires confirmation before deletion."
    )]
    Remove {
        /// Name of the style or locale to remove (without extension)
        #[arg(index = 1, required = true)]
        name: String,
    },
}

#[derive(Args, Debug)]
struct RenderDocArgs {
    /// Path to input document
    #[arg(index = 1)]
    input: PathBuf,

    /// Style file path or builtin name (apa, mla, ieee, etc.)
    #[arg(short, long, required = true)]
    style: String,

    /// Path(s) to bibliography input files (repeat for multiple)
    #[arg(short, long, required = true, action = ArgAction::Append)]
    bibliography: Vec<PathBuf>,
    #[arg(short = 'c', long, action = ArgAction::Append)]
    citations: Vec<PathBuf>,

    /// Input document format
    #[arg(short = 'I', long = "input-format", value_enum, default_value_t = InputFormat::Djot)]
    input_format: InputFormat,

    /// Output format
    #[arg(
        short,
        long,
        value_enum,
        default_value_t = OutputFormat::Plain
    )]
    format: OutputFormat,

    /// Write output to file (defaults to stdout)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Compile Typst output to PDF (requires `typst-pdf` feature)
    #[arg(long)]
    pdf: bool,

    /// Preserve generated Typst source next to the PDF output
    #[arg(long)]
    typst_keep_source: bool,

    /// Disable semantic classes (HTML spans, Djot attributes)
    #[arg(long)]
    no_semantics: bool,
}

/// Line break style for annotation paragraphs.
#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum ParagraphBreakArg {
    #[default]
    BlankLine,
    SingleLine,
}

#[derive(Args, Debug)]
struct RenderRefsArgs {
    /// Path(s) to bibliography input files (repeat for multiple)
    #[arg(short, long, required = true, action = ArgAction::Append)]
    bibliography: Vec<PathBuf>,

    /// Style file path or builtin name (apa, mla, ieee, etc.)
    #[arg(short, long, required = true)]
    style: String,

    /// Path(s) to citations input files (repeat for multiple)
    #[arg(short = 'c', long, action = ArgAction::Append)]
    citations: Vec<PathBuf>,

    /// Render mode
    #[arg(short = 'm', long, value_enum, default_value_t = RenderMode::Both)]
    mode: RenderMode,

    /// Specific reference keys to render (comma-separated)
    #[arg(short = 'k', long, value_delimiter = ',')]
    keys: Option<Vec<String>>,

    /// Show reference keys/IDs in human output
    #[arg(long)]
    show_keys: bool,

    /// Output as JSON
    #[arg(short = 'j', long)]
    json: bool,

    /// Output format
    #[arg(
        short,
        long,
        value_enum,
        default_value_t = OutputFormat::Plain
    )]
    format: OutputFormat,

    /// Write output to file (defaults to stdout)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Disable semantic classes (HTML spans, Djot attributes)
    #[arg(long)]
    no_semantics: bool,

    /// Path to annotations file (JSON or YAML mapping ref IDs to annotation text)
    #[arg(long, value_name = "FILE")]
    annotations: Option<PathBuf>,

    /// Render annotation text in italics
    #[arg(long)]
    annotation_italic: bool,

    /// Indent annotation paragraphs (default: true)
    #[arg(long, default_value_t = true)]
    annotation_indent: bool,

    /// Line break before annotation paragraph
    #[arg(long, value_enum, default_value_t = ParagraphBreakArg::BlankLine)]
    annotation_break: ParagraphBreakArg,
}

#[derive(Args, Debug)]
struct CheckArgs {
    /// Style file path or builtin name (apa, mla, ieee, etc.)
    #[arg(short, long)]
    style: Option<String>,

    /// Path(s) to bibliography input files (repeat for multiple)
    #[arg(short, long, action = ArgAction::Append)]
    bibliography: Vec<PathBuf>,

    /// Path(s) to citations input files (repeat for multiple)
    #[arg(short = 'c', long, action = ArgAction::Append)]
    citations: Vec<PathBuf>,

    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[cfg(feature = "schema")]
#[derive(Args, Debug)]
struct SchemaArgs {
    /// Data type (style, bib, locale, citations)
    #[arg(index = 1, value_enum)]
    r#type: Option<DataType>,

    /// Output directory to export all schemas
    #[arg(short, long)]
    out_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct ConvertArgs {
    /// Path to input file
    #[arg(index = 1)]
    input: PathBuf,

    /// Path to output file
    #[arg(short = 'o', long)]
    output: PathBuf,

    /// Data type (style, bib, locale, citations)
    #[arg(short = 't', long = "type", value_enum)]
    r#type: Option<DataType>,
}

#[derive(Args, Debug)]
struct LegacyDocArgs {
    /// Path to the document file
    #[arg(index = 1)]
    document: PathBuf,

    /// Path to the references file
    #[arg(index = 2)]
    references: PathBuf,

    /// Path to the style file
    #[arg(index = 3)]
    style: PathBuf,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Plain)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct LegacyValidateArgs {
    /// Path to style file
    path: PathBuf,
}

#[derive(Serialize)]
struct CheckItem {
    kind: &'static str,
    path: String,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("\nError: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render { command } => match command {
            RenderCommands::Doc(args) => run_render_doc(args),
            RenderCommands::Refs(args) => run_render_refs(args),
        },
        Commands::Check(args) => run_check(args),
        Commands::Convert(args) => run_convert(args),
        Commands::Styles { command } => match command.unwrap_or(StylesCommands::List) {
            StylesCommands::List => run_styles_list(),
        },
        Commands::Store { command } => match command {
            StoreCommands::List => run_store_list(),
            StoreCommands::Install { source } => run_store_install(&source),
            StoreCommands::Remove { name } => run_store_remove(&name),
        },
        #[cfg(feature = "schema")]
        Commands::Schema(args) => run_schema(args),
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
        let types = [
            (DataType::Style, "style.json"),
            (DataType::Bib, "bib.json"),
            (DataType::Locale, "locale.json"),
            (DataType::Citations, "citation.json"),
        ];
        for (t, filename) in types {
            let schema = match t {
                DataType::Style => schema_for!(Style),
                DataType::Bib => schema_for!(InputBibliography),
                DataType::Locale => schema_for!(RawLocale),
                DataType::Citations => schema_for!(citum_schema::Citations),
            };
            let path = dir.join(filename);
            fs::write(&path, serde_json::to_string_pretty(&schema)?)?;
        }
        println!("Schemas exported to {}", dir.display());
        return Ok(());
    }

    if let Some(t) = args.r#type {
        let schema = match t {
            DataType::Style => schema_for!(Style),
            DataType::Bib => schema_for!(InputBibliography),
            DataType::Locale => schema_for!(RawLocale),
            DataType::Citations => schema_for!(citum_schema::Citations),
        };
        println!("{}", serde_json::to_string_pretty(&schema)?);
        return Ok(());
    }

    Err("Specify a type (style, bib, locale, citation) or --out-dir".into())
}

fn run_styles_list() -> Result<(), Box<dyn Error>> {
    println!("Embedded (builtin) citation styles:");
    println!();
    println!("  {:<10} {:<40} {:<30}", "Alias", "Title", "Full Name");
    println!("  {}", "-".repeat(82));

    for name in citum_schema::embedded::EMBEDDED_STYLE_NAMES {
        let style = citum_schema::embedded::get_embedded_style(name)
            .ok_or_else(|| format!("failed to load builtin style: {}", name))??;

        let alias = citum_schema::embedded::EMBEDDED_STYLE_ALIASES
            .iter()
            .find(|(_, full)| *full == *name)
            .map(|(a, _)| *a)
            .unwrap_or("-");

        let title = style.info.title.as_deref().unwrap_or("-");

        println!("  {:<10} {:<40} {:<30}", alias, truncate(title, 38), name);
    }

    println!();
    println!("Usage:");
    println!("  citum render refs -s <alias|name> -b refs.json");
    println!("  citum render doc <doc.dj> -s <alias|name> -b refs.json");
    Ok(())
}

/// Truncate `s` to at most `max_len` characters.
///
/// If `s` is longer than `max_len`, the returned string ends with `"..."`
/// and has a total length of exactly `max_len`.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
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

    if !styles.is_empty() {
        println!("Installed styles ({}):", styles.len());
        for name in &styles {
            println!("  - {}", name);
        }
        println!();
    } else {
        println!("No installed styles.");
        println!();
    }

    if !locales.is_empty() {
        println!("Installed locales ({}):", locales.len());
        for name in &locales {
            println!("  - {}", name);
        }
        println!();
    } else {
        println!("No installed locales.");
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

    println!("Successfully installed: {}", name);
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
        return Err(format!("style or locale not found: {}", name).into());
    }

    // Ask for confirmation
    print!(
        "Are you sure you want to remove '{}'? This cannot be undone. [y/N] ",
        name
    );
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

    println!("Successfully removed: {}", name);
    Ok(())
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

    let processor = create_processor(style_obj, bibliography, &args.style)?;

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
        indent: args.annotation_indent,
        paragraph_break: match args.annotation_break {
            ParagraphBreakArg::BlankLine => ParagraphBreak::BlankLine,
            ParagraphBreakArg::SingleLine => ParagraphBreak::SingleLine,
        },
        format: AnnotationFormat::Djot,
    };

    let processor = create_processor(style_obj, bibliography, &args.style)?;

    let style_name = {
        let path = Path::new(&args.style);
        if path.exists() {
            path.file_name()
                .map(|s: &std::ffi::OsStr| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            args.style.clone()
        }
    };

    let output = if args.json {
        render_refs_json(
            &processor,
            &style_name,
            args.mode,
            &item_ids,
            input_citations,
            args.format,
            annotations.as_ref(),
            &annotation_style,
        )?
    } else {
        render_refs_human(
            &processor,
            &style_name,
            args.mode,
            &item_ids,
            input_citations,
            args.show_keys,
            args.format,
            annotations.as_ref(),
            &annotation_style,
        )?
    };

    write_output(&output, args.output.as_ref())
}

/// Construct a [`Processor`] from a style, bibliography, and optional locale.
///
/// When the style declares a `default_locale`, the locale is resolved first
/// from disk (for file-based styles) and then from embedded data, falling back
/// to the hardcoded `en-US` defaults.
fn create_processor(
    style: Style,
    loaded: LoadedBibliography,
    style_input: &str,
) -> Result<Processor, Box<dyn Error>> {
    let LoadedBibliography { references, sets } = loaded;
    let compound_sets = sets.unwrap_or_default();
    if let Some(ref locale_id) = style.info.default_locale {
        let path = Path::new(style_input);
        let locale = if path.exists() && path.is_file() {
            // File-based style: search for locale on disk, fall back to embedded.
            let locales_dir = find_locales_dir(style_input);
            let disk_locale = Locale::load(locale_id, &locales_dir);
            if disk_locale.locale == *locale_id || locale_id == "en-US" {
                disk_locale
            } else {
                load_locale_builtin(locale_id)
            }
        } else {
            // Builtin style: use embedded locale directly.
            load_locale_builtin(locale_id)
        };
        Processor::try_with_locale_and_compound_sets(style, references, locale, compound_sets)
            .map_err(|e| e.into())
    } else {
        Processor::try_with_compound_sets(style, references, compound_sets).map_err(|e| e.into())
    }
}

/// Load a style from a file path, user store, or fallback to builtin name/alias.
fn load_any_style(style_input: &str, no_semantics: bool) -> Result<Style, Box<dyn Error>> {
    let path = Path::new(style_input);
    if path.exists() && path.is_file() {
        return load_style(path, no_semantics);
    }

    // Try user store first
    if let Some(data_dir) = platform_data_dir()
        && data_dir.exists()
    {
        let config = StoreConfig::load().unwrap_or_default();
        let resolver = StoreResolver::new(data_dir, config.store_format());
        if let Ok(style) = resolver.resolve_style(style_input) {
            let mut style_obj = style;
            if no_semantics {
                if let Some(ref mut options) = style_obj.options {
                    options.semantic_classes = Some(false);
                } else {
                    style_obj.options = Some(citum_schema::options::Config {
                        semantic_classes: Some(false),
                        ..Default::default()
                    });
                }
            }
            return Ok(style_obj);
        }
    }

    if let Some(res) = citum_schema::embedded::get_embedded_style(style_input) {
        return res.map_err(|e| e.into());
    }

    // Fuzzy matching suggestion
    let suggestions: Vec<_> = citum_schema::embedded::EMBEDDED_STYLE_NAMES
        .iter()
        .chain(
            citum_schema::embedded::EMBEDDED_STYLE_ALIASES
                .iter()
                .map(|(a, _)| a),
        )
        .filter(|&&name| strsim::jaro_winkler(style_input, name) > 0.8)
        .collect();

    let mut msg = format!("style not found: '{}'", style_input);
    if !suggestions.is_empty() {
        msg.push_str("\n\nDid you mean one of these?");
        for s in suggestions {
            msg.push_str(&format!("\n  - {}", s));
        }
    } else {
        msg.push_str("\n\nUse `citum styles list` to see all available builtin styles.");
    }

    Err(msg.into())
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
                    println!("  -> {}", err);
                }
            }
        }
    }

    if checks.iter().any(|c| !c.ok) {
        return Err("One or more checks failed.".into());
    }

    Ok(())
}

/// Execute the `convert` subcommand.
///
/// Deserialises the input file (YAML, JSON, or CBOR), then re-serialises it
/// to the output format inferred from the output file extension.  The data type
/// (style, bib, locale, citations) is auto-detected from the filename stem
/// unless overridden with `--type`.
fn run_convert(args: ConvertArgs) -> Result<(), Box<dyn Error>> {
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

    let data_type = args.r#type.unwrap_or_else(|| {
        let stem = args
            .input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if stem.contains("bib") || stem.contains("ref") {
            DataType::Bib
        } else if stem.contains("cite") || stem.contains("citation") {
            DataType::Citations
        } else if stem.len() == 5 && stem.contains('-') {
            DataType::Locale
        } else {
            DataType::Style
        }
    });

    match data_type {
        DataType::Style => {
            let style: Style = deserialize_any(&input_bytes, input_ext)?;
            let out_bytes = serialize_any(&style, output_ext)?;
            fs::write(&args.output, out_bytes)?;
        }
        DataType::Bib => {
            let bib_obj = load_bibliography(&args.input)?;
            let references: Vec<InputReference> = bib_obj.into_iter().map(|(_, r)| r).collect();
            let input_bib = InputBibliography {
                references,
                ..Default::default()
            };
            let out_bytes = serialize_any(&input_bib, output_ext)?;
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
    }

    println!(
        "Converted {} to {}",
        args.input.display(),
        args.output.display()
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

/// Render bibliography/citation output as a human-readable string.
///
/// Dispatches to the correct monomorphised format renderer based on `output_format`.
#[allow(clippy::too_many_arguments)]
fn render_refs_human(
    processor: &Processor,
    style_name: &str,
    mode: RenderMode,
    item_ids: &[String],
    citations: Option<Vec<Citation>>,
    show_keys: bool,
    output_format: OutputFormat,
    annotations: Option<&std::collections::HashMap<String, String>>,
    annotation_style: &AnnotationStyle,
) -> Result<String, Box<dyn Error>> {
    let show_cite = matches!(mode, RenderMode::Cite | RenderMode::Both);
    let show_bib = matches!(mode, RenderMode::Bib | RenderMode::Both);
    match output_format {
        OutputFormat::Plain => print_human_safe::<PlainText>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            show_keys,
            annotations,
            annotation_style,
        )
        .map_err(|e| e.into()),
        OutputFormat::Html => print_human_safe::<Html>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            show_keys,
            annotations,
            annotation_style,
        )
        .map_err(|e| e.into()),
        OutputFormat::Djot => print_human_safe::<Djot>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            show_keys,
            annotations,
            annotation_style,
        )
        .map_err(|e| e.into()),
        OutputFormat::Latex => print_human_safe::<Latex>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            show_keys,
            annotations,
            annotation_style,
        )
        .map_err(|e| e.into()),
        OutputFormat::Typst => print_human_safe::<Typst>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            show_keys,
            annotations,
            annotation_style,
        )
        .map_err(|e| e.into()),
    }
}

/// Render bibliography/citation output as a JSON string.
///
/// Builds a JSON object containing rendered citation and/or bibliography entries,
/// keyed by reference ID.
#[allow(clippy::too_many_arguments)]
fn render_refs_json(
    processor: &Processor,
    style_name: &str,
    mode: RenderMode,
    item_ids: &[String],
    citations: Option<Vec<Citation>>,
    output_format: OutputFormat,
    annotations: Option<&std::collections::HashMap<String, String>>,
    annotation_style: &AnnotationStyle,
) -> Result<String, Box<dyn Error>> {
    let show_cite = matches!(mode, RenderMode::Cite | RenderMode::Both);
    let show_bib = matches!(mode, RenderMode::Bib | RenderMode::Both);
    match output_format {
        OutputFormat::Plain => print_json_with_format::<PlainText>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            annotations,
            annotation_style,
        ),
        OutputFormat::Html => print_json_with_format::<Html>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            annotations,
            annotation_style,
        ),
        OutputFormat::Djot => print_json_with_format::<Djot>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            annotations,
            annotation_style,
        ),
        OutputFormat::Latex => print_json_with_format::<Latex>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            annotations,
            annotation_style,
        ),
        OutputFormat::Typst => print_json_with_format::<Typst>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            annotations,
            annotation_style,
        ),
    }
}

/// Heuristically locate the `locales/` directory relative to a style file.
///
/// Checks the style's own directory and up to two parent directories, then falls
/// back to a `locales/` folder in the current working directory.  Returns `"."`
/// if no matching directory is found.
fn find_locales_dir(style_path: &str) -> PathBuf {
    let style_dir = Path::new(style_path).parent().unwrap_or(Path::new("."));
    let candidates = [
        style_dir.join("locales"),
        style_dir.join("../locales"),
        style_dir.join("../../locales"),
        PathBuf::from("locales"),
    ];

    for candidate in &candidates {
        if candidate.exists() && candidate.is_dir() {
            return candidate.clone();
        }
    }

    PathBuf::from(".")
}

/// Load a CSLN style from a file path.
///
/// Selects the deserialiser based on the file extension (`cbor`, `json`, or YAML
/// for anything else).  When `no_semantics` is `true`, the `semantic_classes`
/// option is forced to `false` before returning.
fn load_style(path: &Path, no_semantics: bool) -> Result<Style, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    let mut style_obj: Style = match ext {
        "cbor" => ciborium::de::from_reader(std::io::Cursor::new(&bytes))?,
        "json" => serde_json::from_slice(&bytes)?,
        _ => serde_yaml::from_slice(&bytes)?,
    };

    if no_semantics {
        if let Some(ref mut options) = style_obj.options {
            options.semantic_classes = Some(false);
        } else {
            style_obj.options = Some(citum_schema::options::Config {
                semantic_classes: Some(false),
                ..Default::default()
            });
        }
    }

    Ok(style_obj)
}

/// Load a locale from embedded bytes, falling back to en-US.
fn load_locale_builtin(locale_id: &str) -> Locale {
    if let Some(bytes) = citum_schema::embedded::get_locale_bytes(locale_id) {
        let content = String::from_utf8_lossy(bytes);
        Locale::from_yaml_str(&content).unwrap_or_else(|_| Locale::en_us())
    } else {
        // Locale not bundled — fall back to the hardcoded en-US default.
        Locale::en_us()
    }
}

/// Load and merge one or more bibliography files into a single [`Bibliography`].
///
/// Entries from later files overwrite entries with the same ID from earlier files.
///
/// # Errors
/// Returns `Err` when `paths` is empty or any file fails to parse.
fn load_merged_bibliography(paths: &[PathBuf]) -> Result<LoadedBibliography, Box<dyn Error>> {
    if paths.is_empty() {
        return Err("At least one --bibliography file is required.".into());
    }

    let mut merged = Bibliography::new();
    let mut merged_sets = indexmap::IndexMap::<String, Vec<String>>::new();
    for path in paths {
        let loaded = load_bibliography_with_sets(path)?;
        for (id, reference) in loaded.references {
            merged.insert(id, reference);
        }
        if let Some(sets) = loaded.sets {
            for (set_id, members) in sets {
                if merged_sets.insert(set_id.clone(), members).is_some() {
                    return Err(
                        format!("Duplicate compound set id while merging: {}", set_id).into(),
                    );
                }
            }
        }
    }

    let validated_sets = validate_compound_sets(
        if merged_sets.is_empty() {
            None
        } else {
            Some(merged_sets)
        },
        &merged,
    )?;

    Ok(LoadedBibliography {
        references: merged,
        sets: validated_sets,
    })
}

/// Load and concatenate one or more citations files into a single list.
fn load_merged_citations(paths: &[PathBuf]) -> Result<Vec<Citation>, Box<dyn Error>> {
    let mut merged = Vec::new();
    for path in paths {
        let loaded = load_citations(path)?;
        merged.extend(loaded);
    }
    Ok(merged)
}

/// Write `output` to a file at `path`, or to stdout when `path` is `None`.
fn write_output(output: &str, path: Option<&PathBuf>) -> Result<(), Box<dyn Error>> {
    if let Some(file) = path {
        fs::write(file, output)?;
    } else {
        println!("{}", output);
    }
    Ok(())
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
#[allow(clippy::too_many_arguments)]
fn print_human_safe<F>(
    processor: &Processor,
    style_name: &str,
    show_cite: bool,
    show_bib: bool,
    item_ids: &[String],
    citations: Option<Vec<Citation>>,
    show_keys: bool,
    annotations: Option<&std::collections::HashMap<String, String>>,
    annotation_style: &AnnotationStyle,
) -> Result<String, String>
where
    F: citum_engine::render::format::OutputFormat<Output = String> + Send + Sync + 'static,
{
    use std::panic;

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        print_human::<F>(
            processor,
            style_name,
            show_cite,
            show_bib,
            item_ids,
            citations,
            show_keys,
            annotations,
            annotation_style,
        )
    }));

    match result {
        Ok(output) => Ok(output),
        Err(_) => Err(
            "The processor encountered a critical error during rendering. Please report this issue with the style and data used."
                .to_string(),
        ),
    }
}

/// Core human-readable renderer for references and citations.
///
/// Builds a formatted string containing citation clusters and/or bibliography
/// entries, optionally prefixed with reference IDs when `show_keys` is `true`.
#[allow(clippy::too_many_arguments)]
fn print_human<F>(
    processor: &Processor,
    style_name: &str,
    show_cite: bool,
    show_bib: bool,
    item_ids: &[String],
    citations: Option<Vec<Citation>>,
    show_keys: bool,
    annotations: Option<&std::collections::HashMap<String, String>>,
    annotation_style: &AnnotationStyle,
) -> String
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    let mut output = String::new();
    let _ = writeln!(output, "\n=== {} ===\n", style_name);

    if show_cite {
        if let Some(cite_list) = citations {
            let _ = writeln!(output, "CITATIONS (From file):");
            for (i, (citation, text)) in cite_list
                .iter()
                .zip(render_citation_file_entries::<F>(processor, &cite_list))
                .enumerate()
            {
                if show_keys {
                    let _ = writeln!(
                        output,
                        "  [{}] {}",
                        citation.id.as_deref().unwrap_or(&format!("{}", i)),
                        text
                    );
                } else {
                    let _ = writeln!(output, "  {}", text);
                }
            }
        } else {
            let _ = writeln!(output, "CITATIONS (Non-Integral):");
            for id in item_ids {
                let citation = Citation {
                    id: Some(id.to_string()),
                    items: vec![CitationItem {
                        id: id.to_string(),
                        ..Default::default()
                    }],
                    mode: citum_schema::citation::CitationMode::NonIntegral,
                    ..Default::default()
                };
                match processor.process_citation_with_format::<F>(&citation) {
                    Ok(text) => {
                        if show_keys {
                            let _ = writeln!(output, "  [{}] {}", id, text);
                        } else {
                            let _ = writeln!(output, "  {}", text);
                        }
                    }
                    Err(e) => {
                        let _ = writeln!(output, "  [{}] ERROR: {}", id, e);
                    }
                }
            }
            let _ = writeln!(output);

            let _ = writeln!(output, "CITATIONS (Integral):");
            for id in item_ids {
                let citation = Citation {
                    id: Some(id.to_string()),
                    items: vec![CitationItem {
                        id: id.to_string(),
                        ..Default::default()
                    }],
                    mode: citum_schema::citation::CitationMode::Integral,
                    ..Default::default()
                };
                match processor.process_citation_with_format::<F>(&citation) {
                    Ok(text) => {
                        if show_keys {
                            let _ = writeln!(output, "  [{}] {}", id, text);
                        } else {
                            let _ = writeln!(output, "  {}", text);
                        }
                    }
                    Err(e) => {
                        let _ = writeln!(output, "  [{}] ERROR: {}", id, e);
                    }
                }
            }
        }
        let _ = writeln!(output);
    }

    if show_bib {
        // Check if the style has bibliography groups defined
        if processor
            .style
            .bibliography
            .as_ref()
            .and_then(|b| b.groups.as_ref())
            .is_some()
        {
            let _ = writeln!(output, "BIBLIOGRAPHY:");
            if show_keys {
                // When show_keys is requested, render each entry with its ID prefix so the
                // oracle parser can match entries by key. Group headings are omitted in this
                // mode because the oracle only looks for `[id] text` patterns.
                let filter: HashSet<&str> = item_ids.iter().map(|id| id.as_str()).collect();
                let processed = processor.process_references();
                for entry in processed.bibliography {
                    if filter.contains(entry.id.as_str()) {
                        let text = citum_engine::render::refs_to_string_with_format::<F>(
                            vec![entry.clone()],
                            annotations,
                            Some(annotation_style),
                        );
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            let _ = writeln!(output, "  [{}] {}", entry.id, trimmed);
                        }
                    }
                }
            } else {
                // Use grouped renderer for human-readable output (preserves group headings)
                let grouped = processor.render_grouped_bibliography_with_format::<F>();
                output.push_str(&grouped);
            }
        } else {
            let _ = writeln!(output, "BIBLIOGRAPHY:");
            if show_keys {
                // Oracle/show_keys path: render each entry individually so entries
                // can be matched by reference ID. Compound merging is skipped here
                // because the oracle addresses each ref independently.
                let filter: HashSet<&str> = item_ids.iter().map(|id| id.as_str()).collect();
                let processed = processor.process_references();
                for entry in processed.bibliography {
                    if filter.contains(entry.id.as_str()) {
                        let text = citum_engine::render::refs_to_string_with_format::<F>(
                            vec![entry.clone()],
                            annotations,
                            Some(annotation_style),
                        );
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            let _ = writeln!(output, "  [{}] {}", entry.id, trimmed);
                        }
                    }
                }
            } else {
                // Human-readable path: use the engine bibliography renderer so
                // compound numeric groups are merged while still honoring keys.
                let bib =
                    processor.render_selected_bibliography_with_format::<F, _>(item_ids.to_vec());
                let _ = writeln!(output, "{}", bib);
            }
        }
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
                .unwrap_or_else(|error| error.to_string())
        })
        .collect()
}

/// Core JSON renderer for references and citations.
///
/// Returns a pretty-printed JSON object with `style`, `items`, and optionally
/// `citations` and `bibliography` keys.
#[allow(clippy::too_many_arguments)]
fn print_json_with_format<F>(
    processor: &Processor,
    style_name: &str,
    show_cite: bool,
    show_bib: bool,
    item_ids: &[String],
    citations: Option<Vec<Citation>>,
    annotations: Option<&std::collections::HashMap<String, String>>,
    annotation_style: &AnnotationStyle,
) -> Result<String, Box<dyn Error>>
where
    F: citum_engine::render::format::OutputFormat<Output = String>,
{
    use serde_json::json;

    let mut result = json!({
        "style": style_name,
        "items": item_ids.len()
    });

    if show_cite {
        if let Some(cite_list) = citations {
            let rendered: Vec<_> = cite_list
                .iter()
                .zip(render_citation_file_entries::<F>(processor, &cite_list))
                .map(|(citation, text)| {
                    json!({
                        "id": citation.id,
                        "text": text
                    })
                })
                .collect();
            result["citations"] = json!(rendered);
        } else {
            let non_integral: Vec<_> = item_ids
                .iter()
                .map(|id| {
                    let citation = Citation {
                        id: Some(id.to_string()),
                        items: vec![CitationItem {
                            id: id.to_string(),
                            ..Default::default()
                        }],
                        mode: citum_schema::citation::CitationMode::NonIntegral,
                        ..Default::default()
                    };
                    json!({
                        "id": id,
                        "text": processor
                            .process_citation_with_format::<F>(&citation)
                            .unwrap_or_else(|e| e.to_string())
                    })
                })
                .collect();

            let integral: Vec<_> = item_ids
                .iter()
                .map(|id| {
                    let citation = Citation {
                        id: Some(id.to_string()),
                        items: vec![CitationItem {
                            id: id.to_string(),
                            ..Default::default()
                        }],
                        mode: citum_schema::citation::CitationMode::Integral,
                        ..Default::default()
                    };
                    json!({
                        "id": id,
                        "text": processor
                            .process_citation_with_format::<F>(&citation)
                            .unwrap_or_else(|e| e.to_string())
                    })
                })
                .collect();

            result["citations"] = json!({
                "non-integral": non_integral,
                "integral": integral
            });
        }
    }

    if show_bib {
        let filter: HashSet<&str> = item_ids.iter().map(|id| id.as_str()).collect();
        let processed = processor.process_references();
        let entries: Vec<_> = processed
            .bibliography
            .into_iter()
            .filter(|entry| filter.contains(entry.id.as_str()))
            .map(|entry| {
                let text = citum_engine::render::refs_to_string_with_format::<F>(
                    vec![entry.clone()],
                    annotations,
                    Some(annotation_style),
                );
                json!({
                    "id": entry.id,
                    "text": text.trim()
                })
            })
            .collect();

        result["bibliography"] = json!({ "entries": entries });
    }

    Ok(serde_json::to_string_pretty(&result)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use citum_schema::citation::CitationMode;
    use citum_schema::grouping::{GroupSort, GroupSortEntry, GroupSortKey, SortKey};
    use citum_schema::options::{Config, Processing};
    use citum_schema::template::{
        NumberVariable, TemplateComponent, TemplateNumber, WrapPunctuation,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    // ------------------------------------------------------------------
    // truncate
    // ------------------------------------------------------------------

    #[test]
    fn test_truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_exact_length_unchanged() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_long_string_ends_with_ellipsis() {
        let result = truncate("hello world", 8);
        assert_eq!(result, "hello...");
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_truncate_empty_string() {
        assert_eq!(truncate("", 5), "");
    }

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
    // run_convert data-type inference (via stem heuristic)
    // The heuristic lives inline in run_convert; we replicate it here so
    // we can test it without spawning a process.
    // ------------------------------------------------------------------

    fn infer_data_type(stem: &str) -> DataType {
        if stem.contains("bib") || stem.contains("ref") {
            DataType::Bib
        } else if stem.contains("cite") || stem.contains("citation") {
            DataType::Citations
        } else if stem.len() == 5 && stem.contains('-') {
            DataType::Locale
        } else {
            DataType::Style
        }
    }

    #[test]
    fn test_infer_data_type_bib_stem() {
        assert!(matches!(infer_data_type("bibliography"), DataType::Bib));
        assert!(matches!(infer_data_type("refs"), DataType::Bib));
        assert!(matches!(infer_data_type("my-bib"), DataType::Bib));
    }

    #[test]
    fn test_infer_data_type_citations_stem() {
        assert!(matches!(infer_data_type("citations"), DataType::Citations));
        assert!(matches!(infer_data_type("cite-list"), DataType::Citations));
    }

    #[test]
    fn test_infer_data_type_locale_stem() {
        // Locale stems are exactly 5 chars and contain a hyphen (e.g. "en-US")
        assert!(matches!(infer_data_type("en-US"), DataType::Locale));
        assert!(matches!(infer_data_type("de-DE"), DataType::Locale));
    }

    #[test]
    fn test_infer_data_type_style_stem() {
        assert!(matches!(infer_data_type("apa-7th"), DataType::Style));
        assert!(matches!(infer_data_type("my-style"), DataType::Style));
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
    fn test_print_json_batches_numeric_citation_files() {
        let style = Style {
            citation: Some(citum_schema::CitationSpec {
                template: Some(vec![TemplateComponent::Number(TemplateNumber {
                    number: NumberVariable::CitationNumber,
                    ..Default::default()
                })]),
                wrap: Some(WrapPunctuation::Brackets),
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
            id: Some("c1".to_string()),
            items: vec![CitationItem {
                id: "smith2020".to_string(),
                ..Default::default()
            }],
            mode: CitationMode::NonIntegral,
            ..Default::default()
        }];

        let output = print_json_with_format::<PlainText>(
            &processor,
            "numeric-test",
            true,
            false,
            &["smith2020".to_string(), "adams2021".to_string()],
            Some(citations),
            None,
            &AnnotationStyle::default(),
        )
        .expect("json rendering should succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("output should be valid JSON");

        assert_eq!(parsed["citations"][0]["text"], "[2]");

        let _ = std::fs::remove_file(bib_path);
        let _ = std::fs::remove_dir(base);
    }
}
