#![allow(missing_docs, reason = "bin")]

use citum_engine::{
    Bibliography, Citation, CitationItem, DocumentFormat, Processor,
    io::{
        AnnotationFormat, AnnotationStyle, LoadedBibliography, ParagraphBreak, load_annotations,
        load_bibliography, load_bibliography_with_sets, load_citations, validate_compound_sets,
    },
    processor::document::{djot::DjotParser, markdown::MarkdownParser},
    render::{djot::Djot, html::Html, latex::Latex, plain::PlainText, typst::Typst},
};
use citum_schema::locale::{GeneralTerm, RawLocale, TermForm, types::LocaleOverride};
use citum_schema::options::{Config, Processing};
use citum_schema::reference::InputReference;
use citum_schema::template::{
    ContributorForm, ContributorRole, LabelForm as TemplateLabelForm, NumberVariable,
    RoleLabelForm, TemplateComponent,
};
use citum_schema::{BibliographySpec, CitationSpec, InputBibliography, Locale, Style, Template};
use citum_store::{StoreConfig, StoreResolver, platform_data_dir};
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{ArgAction, Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
#[cfg(feature = "schema")]
use schemars::schema_for;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
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
                  citum convert style style.yaml -o style.cbor\n\n  \
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

/// Valid target types for JSON schema export.
#[cfg(feature = "schema")]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum SchemaType {
    /// Citation style schema
    Style,
    /// Bibliography input schema
    Bib,
    /// Locale schema
    Locale,
    /// Citation input schema
    Citation,
    /// Style registry schema
    Registry,
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

    /// Convert styles, references, locales, and citations
    #[command(
        about = "Convert styles, references, locales, and citations",
        long_about = "Convert between native Citum formats and legacy bibliography formats.\n\n\
                      Use subcommands to make conversion intent explicit.\n\n\
                      EXAMPLES:\n  \
                      Convert references from BibLaTeX to native YAML:\n    \
                      citum convert refs refs.bib -o refs.yaml\n\n  \
                      Convert references from native YAML to RIS:\n    \
                      citum convert refs refs.yaml -o refs.ris\n\n  \
                      Convert a style from YAML to binary CBOR:\n    \
                      citum convert style style.yaml -o style.cbor"
    )]
    Convert {
        #[command(subcommand)]
        command: ConvertCommands,
    },

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

    /// Manage and inspect the style registry
    #[command(
        about = "Manage and inspect the style registry",
        long_about = "Inspect and manage the citation style registry.\n\n\
                      The registry maps style names and aliases to available styles.\n\n\
                      EXAMPLES:\n  \
                      List all styles in the registry:\n    \
                      citum registry list\n\n  \
                      Resolve a style name or alias:\n    \
                      citum registry resolve apa"
    )]
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
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

    /// Validate a style against a locale file
    Style {
        #[command(subcommand)]
        command: StyleCommands,
    },

    /// Validate locale files and inspect locale-specific behavior
    Locale {
        #[command(subcommand)]
        command: LocaleCommands,
    },

    /// Generate JSON schema for Citum models
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
enum ConvertCommands {
    /// Convert bibliography/reference files
    Refs(ConvertRefsArgs),
    /// Convert style files between YAML/JSON/CBOR
    Style(ConvertTypedArgs),
    /// Convert citations files between YAML/JSON/CBOR
    Citations(ConvertTypedArgs),
    /// Convert locale files between YAML/JSON/CBOR
    Locale(ConvertTypedArgs),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum RefsFormat {
    #[value(name = "citum-yaml")]
    CitumYaml,
    #[value(name = "citum-json")]
    CitumJson,
    #[value(name = "citum-cbor")]
    CitumCbor,
    #[value(name = "csl-json")]
    CslJson,
    #[value(name = "biblatex")]
    Biblatex,
    #[value(name = "ris")]
    Ris,
}

#[derive(Subcommand)]
enum StylesCommands {
    /// List all embedded (builtin) style names
    List,
}

#[derive(Subcommand)]
enum RegistryCommands {
    /// List all styles in the registry
    #[command(
        about = "List all styles in the registry",
        long_about = "Display all styles in the style registry with their\n\
                      aliases and descriptions."
    )]
    List {
        /// Output format
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// Resolve a style name or alias to its canonical ID
    #[command(
        about = "Resolve a style name or alias to its canonical ID",
        long_about = "Look up a style by name or alias in the registry.\n\
                      Returns the canonical style ID and source (builtin or path)."
    )]
    Resolve {
        /// Style name or alias to resolve
        name: String,
    },
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

#[derive(Subcommand)]
enum StyleCommands {
    /// Validate that a style's locale-driven features resolve against a locale file
    Lint(LintStyleArgs),
}

#[derive(Subcommand)]
enum LocaleCommands {
    /// Validate a locale file's message syntax and alias targets
    Lint(LintLocaleArgs),
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

#[derive(Args, Debug)]
struct LintLocaleArgs {
    /// Path to locale file
    #[arg(index = 1)]
    path: PathBuf,
}

#[derive(Args, Debug)]
struct LintStyleArgs {
    /// Style file path or builtin name
    #[arg(index = 1)]
    style: String,

    /// Locale file used for validation
    #[arg(long, required = true)]
    locale: PathBuf,
}

#[cfg(feature = "schema")]
#[derive(Args, Debug)]
struct SchemaArgs {
    /// Data type to export
    #[arg(index = 1, value_enum)]
    r#type: Option<SchemaType>,

    /// Output directory to export all schemas
    #[arg(short, long)]
    out_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct ConvertTypedArgs {
    /// Path to input file
    #[arg(index = 1)]
    input: PathBuf,

    /// Path to output file
    #[arg(short = 'o', long)]
    output: PathBuf,
}

#[derive(Args, Debug)]
struct ConvertRefsArgs {
    /// Path to input bibliography file
    #[arg(index = 1)]
    input: PathBuf,

    /// Path to output bibliography file
    #[arg(short = 'o', long)]
    output: PathBuf,

    /// Input format override
    #[arg(long, value_enum)]
    from: Option<RefsFormat>,

    /// Output format override
    #[arg(long, value_enum)]
    to: Option<RefsFormat>,
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
        eprintln!("\nError: {e}");
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

fn run_styles_list() -> Result<(), Box<dyn Error>> {
    println!("Embedded (builtin) citation styles:");
    println!();
    println!("  {:<10} {:<40} {:<30}", "Alias", "Title", "Full Name");
    println!("  {}", "-".repeat(82));

    for name in citum_schema::embedded::EMBEDDED_STYLE_NAMES {
        let style = citum_schema::embedded::get_embedded_style(name)
            .ok_or_else(|| format!("failed to load builtin style: {name}"))??;

        let alias = citum_schema::embedded::EMBEDDED_STYLE_ALIASES
            .iter()
            .find(|(_, full)| *full == *name)
            .map_or("-", |(a, _)| *a);

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

/// List all styles in the registry.
fn run_registry_list(format: &str) -> Result<(), Box<dyn Error>> {
    let registry = citum_schema::embedded::default_registry();

    if format == "json" {
        let json = serde_json::to_string_pretty(&registry)?;
        println!("{}", json);
    } else {
        // Table format (default)
        println!("Style Registry:");
        println!();
        println!("  {:<35} {:<30} {:<40}", "ID", "Aliases", "Description");
        println!("  {}", "-".repeat(110));

        for entry in &registry.styles {
            let aliases = if entry.aliases.is_empty() {
                "-".to_string()
            } else {
                entry.aliases.join(", ")
            };
            let description = entry.description.as_deref().unwrap_or("-");

            println!(
                "  {:<35} {:<30} {:<40}",
                entry.id,
                truncate(&aliases, 28),
                truncate(description, 38)
            );
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LintSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LintFinding {
    severity: LintSeverity,
    path: String,
    message: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LintReport {
    findings: Vec<LintFinding>,
}

impl LintReport {
    fn warning(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.findings.push(LintFinding {
            severity: LintSeverity::Warning,
            path: path.into(),
            message: message.into(),
        });
    }

    fn error(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.findings.push(LintFinding {
            severity: LintSeverity::Error,
            path: path.into(),
            message: message.into(),
        });
    }

    fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|finding| finding.severity == LintSeverity::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocaleRequirementKind {
    General {
        term: GeneralTerm,
        form: TermForm,
    },
    Role {
        role: ContributorRole,
        form: TermForm,
    },
    Locator {
        locator: citum_schema::citation::LocatorType,
        form: TermForm,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocaleRequirement {
    path: String,
    kind: LocaleRequirementKind,
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

fn print_lint_report(label: &str, report: &LintReport) {
    if report.findings.is_empty() {
        println!("{label}: ok");
        return;
    }

    println!("{label}:");
    for finding in &report.findings {
        let level = match finding.severity {
            LintSeverity::Warning => "warning",
            LintSeverity::Error => "error",
        };
        println!("  {level}: {}: {}", finding.path, finding.message);
    }
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

fn load_locale_file(path: &Path) -> Result<Locale, Box<dyn Error>> {
    Locale::from_file(path)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err).into())
}

fn lint_raw_locale(raw: &RawLocale) -> LintReport {
    let mut report = LintReport::default();
    let uses_mf1 = raw.evaluation.as_ref().is_some_and(|config| {
        config.message_syntax == citum_schema::locale::types::MessageSyntax::Mf1
    });

    for (message_id, message) in &raw.messages {
        if (uses_mf1 || message.contains('{'))
            && let Err(err) = lint_mf1_message(message)
        {
            report.error(
                format!("messages.{message_id}"),
                format!("invalid MF1 message: {err}"),
            );
        }
    }

    for (legacy_key, message_id) in &raw.legacy_term_aliases {
        if !raw.messages.contains_key(message_id) {
            report.error(
                format!("legacy-term-aliases.{legacy_key}"),
                format!("target '{message_id}' does not exist in messages"),
            );
        }
    }

    report
}

fn lint_style_against_locale(style: &Style, locale: &Locale) -> LintReport {
    let mut report = LintReport::default();
    let requirements = collect_style_locale_requirements(style);

    for requirement in requirements {
        match requirement.kind {
            LocaleRequirementKind::General { term, form } => {
                if locale.resolved_general_term(&term, form).is_none() {
                    report.warning(
                        requirement.path,
                        format!(
                            "locale does not resolve general term '{term:?}' in form '{form:?}'"
                        ),
                    );
                }
            }
            LocaleRequirementKind::Role { role, form } => {
                let singular = locale.resolved_role_term(&role, false, form);
                let plural = locale.resolved_role_term(&role, true, form);
                if singular.is_none() || plural.is_none() {
                    report.warning(
                        requirement.path,
                        format!(
                            "locale does not fully resolve role term '{role:?}' in form '{form:?}'"
                        ),
                    );
                }
            }
            LocaleRequirementKind::Locator { locator, form } => {
                let singular = locale.resolved_locator_term(&locator, false, form);
                let plural = locale.resolved_locator_term(&locator, true, form);
                if singular.is_none() || plural.is_none() {
                    report.warning(
                        requirement.path,
                        format!(
                            "locale does not fully resolve locator term '{locator:?}' in form '{form:?}'"
                        ),
                    );
                }
            }
        }
    }

    report
}

fn collect_style_locale_requirements(style: &Style) -> Vec<LocaleRequirement> {
    let mut requirements = Vec::new();
    let base_config = style.options.clone().unwrap_or_default();

    if let Some(template) = &style.templates {
        for (name, components) in template {
            collect_template_requirements(
                components,
                &format!("templates.{name}"),
                &base_config,
                &mut requirements,
            );
        }
    }

    if let Some(citation) = &style.citation {
        collect_citation_spec_requirements(citation, "citation", &base_config, &mut requirements);
    }

    if let Some(bibliography) = &style.bibliography {
        collect_bibliography_spec_requirements(
            bibliography,
            "bibliography",
            &base_config,
            &mut requirements,
        );
    }

    requirements
}

fn collect_citation_spec_requirements(
    spec: &CitationSpec,
    path: &str,
    base_config: &Config,
    requirements: &mut Vec<LocaleRequirement>,
) {
    let effective_config = spec.options.as_ref().map_or_else(
        || base_config.clone(),
        |options| Config::merged(base_config, options),
    );

    if let Some(template) = spec.resolve_template() {
        collect_template_requirements(
            &template,
            &format!("{path}.template"),
            &effective_config,
            requirements,
        );
    }
    if let Some(locales) = &spec.locales {
        for (index, localized) in locales.iter().enumerate() {
            collect_template_requirements(
                &localized.template,
                &format!("{path}.locales[{index}].template"),
                &effective_config,
                requirements,
            );
        }
    }
    if let Some(spec) = &spec.integral {
        collect_citation_spec_requirements(
            spec,
            &format!("{path}.integral"),
            &effective_config,
            requirements,
        );
    }
    if let Some(spec) = &spec.non_integral {
        collect_citation_spec_requirements(
            spec,
            &format!("{path}.non-integral"),
            &effective_config,
            requirements,
        );
    }
    if let Some(spec) = &spec.subsequent {
        collect_citation_spec_requirements(
            spec,
            &format!("{path}.subsequent"),
            &effective_config,
            requirements,
        );
    }
    if let Some(spec) = &spec.ibid {
        collect_citation_spec_requirements(
            spec,
            &format!("{path}.ibid"),
            &effective_config,
            requirements,
        );
    }
}

fn collect_bibliography_spec_requirements(
    spec: &BibliographySpec,
    path: &str,
    base_config: &Config,
    requirements: &mut Vec<LocaleRequirement>,
) {
    let effective_config = spec.options.as_ref().map_or_else(
        || base_config.clone(),
        |options| Config::merged(base_config, options),
    );

    if let Some(template) = spec.resolve_template() {
        collect_template_requirements(
            &template,
            &format!("{path}.template"),
            &effective_config,
            requirements,
        );
    }
    if let Some(locales) = &spec.locales {
        for (index, localized) in locales.iter().enumerate() {
            collect_template_requirements(
                &localized.template,
                &format!("{path}.locales[{index}].template"),
                &effective_config,
                requirements,
            );
        }
    }
    if let Some(type_templates) = &spec.type_templates {
        for (selector, template) in type_templates {
            collect_template_requirements(
                template,
                &format!("{path}.type-templates[{selector:?}]"),
                &effective_config,
                requirements,
            );
        }
    }
    if let Some(groups) = &spec.groups {
        for (index, group) in groups.iter().enumerate() {
            if let Some(heading) = &group.heading
                && let citum_schema::grouping::GroupHeading::Term { term, form } = heading
            {
                requirements.push(LocaleRequirement {
                    path: format!("{path}.groups[{index}].heading"),
                    kind: LocaleRequirementKind::General {
                        term: *term,
                        form: form.unwrap_or(TermForm::Long),
                    },
                });
            }
            if let Some(template) = &group.template {
                collect_template_requirements(
                    template,
                    &format!("{path}.groups[{index}].template"),
                    &effective_config,
                    requirements,
                );
            }
        }
    }
}

fn collect_template_requirements(
    template: &Template,
    path: &str,
    config: &Config,
    requirements: &mut Vec<LocaleRequirement>,
) {
    for (index, component) in template.iter().enumerate() {
        let component_path = format!("{path}[{index}]");
        match component {
            TemplateComponent::Term(term) => requirements.push(LocaleRequirement {
                path: component_path,
                kind: LocaleRequirementKind::General {
                    term: term.term,
                    form: term.form.unwrap_or(TermForm::Long),
                },
            }),
            TemplateComponent::Contributor(contributor) => {
                collect_contributor_requirements(
                    contributor,
                    &component_path,
                    config,
                    requirements,
                );
            }
            TemplateComponent::Number(number) => {
                if let Some(form) = number.label_form.clone()
                    && let Some(locator) = number_variable_to_locator(number.number.clone())
                {
                    let term_form = match form {
                        TemplateLabelForm::Short => TermForm::Short,
                        TemplateLabelForm::Long => TermForm::Long,
                        TemplateLabelForm::Symbol => TermForm::Symbol,
                    };
                    requirements.push(LocaleRequirement {
                        path: component_path,
                        kind: LocaleRequirementKind::Locator {
                            locator,
                            form: term_form,
                        },
                    });
                }
            }
            TemplateComponent::Date(date) => {
                if matches!(date.date, citum_schema::template::DateVariable::Issued) {
                    requirements.push(LocaleRequirement {
                        path: component_path.clone(),
                        kind: LocaleRequirementKind::General {
                            term: GeneralTerm::NoDate,
                            form: TermForm::Short,
                        },
                    });
                }
                if let Some(fallback) = &date.fallback {
                    collect_template_requirements(
                        fallback,
                        &format!("{component_path}.fallback"),
                        config,
                        requirements,
                    );
                }
            }
            TemplateComponent::List(list) => {
                collect_template_requirements(
                    &list.items,
                    &format!("{component_path}.items"),
                    config,
                    requirements,
                );
            }
            _ => {}
        }
    }
}

fn collect_contributor_requirements(
    contributor: &citum_schema::template::TemplateContributor,
    path: &str,
    config: &Config,
    requirements: &mut Vec<LocaleRequirement>,
) {
    if let Some(label) = &contributor.label {
        let role =
            role_label_term_to_role(&label.term).unwrap_or_else(|| contributor.contributor.clone());
        let form = match label.form {
            RoleLabelForm::Short => TermForm::Short,
            RoleLabelForm::Long => TermForm::Long,
        };
        requirements.push(LocaleRequirement {
            path: format!("{path}.label"),
            kind: LocaleRequirementKind::Role { role, form },
        });
        return;
    }

    let configured_preset = config.contributors.as_ref().and_then(|contributors| {
        contributors.effective_role_label_preset(&contributor.contributor)
    });
    if let Some(role_label_preset) = configured_preset {
        let form = match role_label_preset {
            citum_schema::options::RoleLabelPreset::None => return,
            citum_schema::options::RoleLabelPreset::VerbPrefix => TermForm::Verb,
            citum_schema::options::RoleLabelPreset::VerbShortPrefix => TermForm::VerbShort,
            citum_schema::options::RoleLabelPreset::ShortSuffix => TermForm::Short,
            citum_schema::options::RoleLabelPreset::LongSuffix => TermForm::Long,
        };
        requirements.push(LocaleRequirement {
            path: path.to_string(),
            kind: LocaleRequirementKind::Role {
                role: contributor.contributor.clone(),
                form,
            },
        });
        return;
    }

    match contributor.form {
        ContributorForm::Verb => requirements.push(LocaleRequirement {
            path: path.to_string(),
            kind: LocaleRequirementKind::Role {
                role: contributor.contributor.clone(),
                form: TermForm::Verb,
            },
        }),
        ContributorForm::VerbShort => requirements.push(LocaleRequirement {
            path: path.to_string(),
            kind: LocaleRequirementKind::Role {
                role: contributor.contributor.clone(),
                form: TermForm::VerbShort,
            },
        }),
        ContributorForm::Long
            if matches!(
                contributor.contributor,
                ContributorRole::Editor | ContributorRole::Translator
            ) =>
        {
            requirements.push(LocaleRequirement {
                path: path.to_string(),
                kind: LocaleRequirementKind::Role {
                    role: contributor.contributor.clone(),
                    form: TermForm::Short,
                },
            });
        }
        _ => {}
    }
}

fn role_label_term_to_role(term: &str) -> Option<ContributorRole> {
    match term {
        "editor" => Some(ContributorRole::Editor),
        "translator" => Some(ContributorRole::Translator),
        "director" => Some(ContributorRole::Director),
        "recipient" => Some(ContributorRole::Recipient),
        "interviewer" => Some(ContributorRole::Interviewer),
        _ => None,
    }
}

fn number_variable_to_locator(
    number: NumberVariable,
) -> Option<citum_schema::citation::LocatorType> {
    use citum_schema::citation::LocatorType;

    match number {
        NumberVariable::Volume | NumberVariable::NumberOfVolumes => Some(LocatorType::Volume),
        NumberVariable::Pages | NumberVariable::NumberOfPages => Some(LocatorType::Page),
        NumberVariable::ChapterNumber => Some(LocatorType::Chapter),
        NumberVariable::Issue => Some(LocatorType::Issue),
        NumberVariable::Number
        | NumberVariable::DocketNumber
        | NumberVariable::PatentNumber
        | NumberVariable::StandardNumber
        | NumberVariable::ReportNumber
        | NumberVariable::PrintingNumber
        | NumberVariable::CitationNumber
        | NumberVariable::CitationLabel => Some(LocatorType::Number),
        NumberVariable::PartNumber => Some(LocatorType::Part),
        NumberVariable::SupplementNumber => Some(LocatorType::Supplement),
        _ => None,
    }
}

fn lint_mf1_message(message: &str) -> Result<(), String> {
    lint_mf1_segment(message)
}

fn lint_mf1_segment(message: &str) -> Result<(), String> {
    let mut cursor = 0usize;
    while let Some(offset) = message[cursor..].find('{') {
        let open = cursor + offset;
        let close = find_matching_brace(message, open)
            .ok_or_else(|| format!("unmatched '{{' at byte offset {open}"))?;
        let inner = message
            .get(open + 1..close)
            .ok_or_else(|| "invalid brace range".to_string())?
            .trim();
        lint_mf1_placeholder(inner)?;
        cursor = close + 1;
    }
    Ok(())
}

fn lint_mf1_placeholder(inner: &str) -> Result<(), String> {
    let Some((variable, remainder)) = split_top_level_once(inner, ',') else {
        if inner.trim().is_empty() {
            return Err("empty placeholder".to_string());
        }
        return Ok(());
    };
    if variable.trim().is_empty() {
        return Err("placeholder variable name is empty".to_string());
    }
    let remainder = remainder.trim();
    let Some((kind, body)) = split_top_level_once(remainder, ',') else {
        return match remainder {
            "number" => Ok(()),
            _ => Err(format!("unsupported or malformed formatter '{remainder}'")),
        };
    };

    match kind.trim() {
        "plural" | "select" => {
            let selectors = parse_mf1_selectors(body.trim())?;
            if !selectors.contains_key("other") {
                return Err(format!("{} requires an 'other' branch", kind.trim()));
            }
            for branch in selectors.values() {
                lint_mf1_segment(branch)?;
            }
            Ok(())
        }
        "number" => Ok(()),
        other => Err(format!("unsupported formatter '{other}'")),
    }
}

fn split_top_level_once(input: &str, delimiter: char) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    for (index, ch) in input.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => {
                let len = ch.len_utf8();
                return Some((&input[..index], &input[index + len..]));
            }
            _ => {}
        }
    }
    None
}

fn parse_mf1_selectors(input: &str) -> Result<HashMap<String, String>, String> {
    let mut selectors = HashMap::new();
    let mut cursor = 0usize;

    while cursor < input.len() {
        let trimmed = input[cursor..].trim_start();
        cursor = input.len() - trimmed.len();
        if trimmed.is_empty() {
            break;
        }

        let key_end = trimmed
            .find('{')
            .ok_or_else(|| "selector branch is missing '{'".to_string())?;
        let key = trimmed[..key_end].trim();
        if key.is_empty() {
            return Err("selector branch key is empty".to_string());
        }
        let open = cursor + key_end;
        let close = find_matching_brace(input, open)
            .ok_or_else(|| format!("selector '{key}' is missing a closing '}}'"))?;
        selectors.insert(key.to_string(), input[open + 1..close].to_string());
        cursor = close + 1;
    }

    if selectors.is_empty() {
        return Err("selector has no branches".to_string());
    }

    Ok(selectors)
}

fn find_matching_brace(input: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0usize;

    for (index, ch) in input
        .char_indices()
        .skip_while(|(index, _)| *index < open_index)
    {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
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

    let processor = create_processor(style_obj, bibliography, &args.style, args.no_semantics)?;

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

    let processor = create_processor(style_obj, bibliography, &args.style, args.no_semantics)?;

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

/// Construct a [`Processor`] from a style, bibliography, and optional locale.
///
/// When the style declares a `default_locale`, the locale is resolved first
/// from disk (for file-based styles) and then from embedded data, falling back
/// to the hardcoded `en-US` defaults.
fn create_processor(
    style: Style,
    loaded: LoadedBibliography,
    style_input: &str,
    no_semantics: bool,
) -> Result<Processor, Box<dyn Error>> {
    let LoadedBibliography { references, sets } = loaded;
    let compound_sets = sets.unwrap_or_default();
    if let Some(ref locale_id) = style.info.default_locale {
        let path = Path::new(style_input);
        let mut locale = if path.exists() && path.is_file() {
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
        if let Some(override_id) = style
            .options
            .as_ref()
            .and_then(|options| options.locale_override.as_deref())
        {
            let locale_override = if path.exists() && path.is_file() {
                load_locale_override_for_file_style(override_id, style_input)?
                    .or_else(|| load_locale_override_builtin(override_id))
            } else {
                load_locale_override_builtin(override_id)
            };
            let locale_override = locale_override.ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                    "locale override not found: '{override_id}' (expected under locales/overrides/)"
                    ),
                )
            })?;
            locale.apply_override(&locale_override);
        }
        let mut processor =
            Processor::try_with_locale_and_compound_sets(style, references, locale, compound_sets)?;
        processor.show_semantics = !no_semantics;
        Ok(processor)
    } else {
        let mut processor = Processor::try_with_compound_sets(style, references, compound_sets)?;
        processor.show_semantics = !no_semantics;
        Ok(processor)
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
            return Ok(style);
        }
    }

    if let Some(res) = citum_schema::embedded::get_embedded_style(style_input) {
        return res.map_err(std::convert::Into::into);
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

    let mut msg = format!("style not found: '{style_input}'");
    if suggestions.is_empty() {
        msg.push_str("\n\nUse `citum styles list` to see all available builtin styles.");
    } else {
        msg.push_str("\n\nDid you mean one of these?");
        for s in suggestions {
            msg.push_str(&format!("\n  - {s}"));
        }
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
        f
    } else {
        infer_refs_input_format(&args.input)?
    };
    let output_format = args
        .to
        .unwrap_or_else(|| infer_refs_output_format(&args.output));

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

fn infer_refs_input_format(path: &Path) -> Result<RefsFormat, Box<dyn Error>> {
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

fn infer_refs_output_format(path: &Path) -> RefsFormat {
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

fn detect_json_refs_format(path: &Path) -> Result<RefsFormat, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    let is_csl_array = value.as_array().is_some_and(|items| {
        items.iter().any(|v| {
            v.get("id").is_some()
                && v.get("type").is_some()
                && (v.get("title").is_some() || v.get("author").is_some())
        })
    });
    let is_citum_object = value.get("references").is_some();
    if is_csl_array && !is_citum_object {
        Ok(RefsFormat::CslJson)
    } else {
        Ok(RefsFormat::CitumJson)
    }
}

fn load_input_bibliography(
    path: &Path,
    format: RefsFormat,
) -> Result<InputBibliography, Box<dyn Error>> {
    match format {
        RefsFormat::CitumYaml => {
            let bytes = fs::read(path)?;
            deserialize_any(&bytes, "yaml")
        }
        RefsFormat::CitumJson => {
            let bytes = fs::read(path)?;
            deserialize_any(&bytes, "json")
        }
        RefsFormat::CitumCbor => {
            let bytes = fs::read(path)?;
            deserialize_any(&bytes, "cbor")
        }
        RefsFormat::CslJson => load_csl_json_bibliography(path),
        RefsFormat::Biblatex => load_biblatex_bibliography(path),
        RefsFormat::Ris => load_ris_bibliography(path),
    }
}

fn write_output_bibliography(
    input: &InputBibliography,
    path: &Path,
    format: RefsFormat,
) -> Result<(), Box<dyn Error>> {
    match format {
        RefsFormat::CitumYaml => fs::write(path, serialize_any(input, "yaml")?)?,
        RefsFormat::CitumJson => fs::write(path, serialize_any(input, "json")?)?,
        RefsFormat::CitumCbor => fs::write(path, serialize_any(input, "cbor")?)?,
        RefsFormat::CslJson => {
            let refs: Vec<csl_legacy::csl_json::Reference> = input
                .references
                .iter()
                .map(input_reference_to_csl_json)
                .collect();
            fs::write(path, serde_json::to_string_pretty(&refs)?)?;
        }
        RefsFormat::Biblatex => {
            fs::write(path, render_biblatex(input))?;
        }
        RefsFormat::Ris => {
            fs::write(path, render_ris(input))?;
        }
    }
    Ok(())
}

fn load_csl_json_bibliography(path: &Path) -> Result<InputBibliography, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let refs: Vec<csl_legacy::csl_json::Reference> = serde_json::from_slice(&bytes)?;
    let references = refs.into_iter().map(InputReference::from).collect();
    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

fn load_biblatex_bibliography(path: &Path) -> Result<InputBibliography, Box<dyn Error>> {
    let src = fs::read_to_string(path)?;
    let bibliography =
        biblatex::Bibliography::parse(&src).map_err(|e| format!("BibLaTeX parse error: {e}"))?;
    let references = bibliography
        .iter()
        .map(|entry| InputReference::from(input_reference_from_biblatex(entry)))
        .collect();
    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

fn load_ris_bibliography(path: &Path) -> Result<InputBibliography, Box<dyn Error>> {
    let src = fs::read_to_string(path)?;
    parse_ris(&src)
}

fn input_reference_from_biblatex(entry: &biblatex::Entry) -> csl_legacy::csl_json::Reference {
    use csl_legacy::csl_json::{DateVariable, Name, Reference, StringOrNumber};

    let field_str = |key: &str| {
        entry.fields.get(key).map(|f| {
            f.iter()
                .map(|c| match &c.v {
                    biblatex::Chunk::Normal(s) | biblatex::Chunk::Verbatim(s) => s.as_str(),
                    _ => "",
                })
                .collect::<String>()
        })
    };
    let parse_names = |raw: Option<String>| -> Option<Vec<Name>> {
        raw.map(|s| {
            s.split(" and ")
                .map(|name| {
                    let parts: Vec<_> = name.split(',').map(str::trim).collect();
                    if parts.len() >= 2 {
                        Name::new(parts[0], parts[1])
                    } else {
                        Name::literal(parts.first().copied().unwrap_or(""))
                    }
                })
                .collect::<Vec<_>>()
        })
    };

    let mut r = Reference {
        id: entry.key.clone(),
        ref_type: "book".to_string(),
        ..Default::default()
    };

    let ty = entry.entry_type.to_string().to_lowercase();
    r.ref_type = if ty.contains("article") {
        "article-journal".to_string()
    } else if ty.contains("incollection") || ty.contains("inbook") {
        "chapter".to_string()
    } else {
        "book".to_string()
    };
    r.author = parse_names(field_str("author"));
    r.editor = parse_names(field_str("editor"));
    r.title = field_str("title");
    r.container_title = field_str("journaltitle").or_else(|| field_str("booktitle"));
    r.publisher = field_str("publisher");
    r.publisher_place = field_str("location");
    r.doi = field_str("doi");
    r.url = field_str("url");
    r.note = field_str("note");
    r.language = field_str("langid").or_else(|| field_str("language"));
    r.isbn = field_str("isbn");
    r.page = field_str("pages");
    r.volume = field_str("volume").map(StringOrNumber::String);
    r.issue = field_str("number").map(StringOrNumber::String);
    r.edition = field_str("edition").map(StringOrNumber::String);
    r.issued = field_str("date")
        .or_else(|| field_str("year"))
        .and_then(|date| {
            let year = date.get(0..4)?.parse::<i32>().ok()?;
            Some(DateVariable::year(year))
        });
    r
}

fn input_reference_to_csl_json(reference: &InputReference) -> csl_legacy::csl_json::Reference {
    use csl_legacy::csl_json::{DateVariable, Reference, StringOrNumber};

    let id = reference.id().unwrap_or_else(|| "item".to_string());
    let mut r = Reference {
        id,
        ..Default::default()
    };

    r.title = reference.title().map(|t| t.to_string());
    r.language = reference.language();
    r.note = reference.note();
    r.doi = reference.doi();
    r.issued = reference.issued().and_then(|d| {
        let s = d.0;
        let year = s.get(0..4)?.parse::<i32>().ok()?;
        Some(DateVariable::year(year))
    });
    r.author = reference.author().map(contributor_to_csl_names);
    r.editor = reference.editor().map(contributor_to_csl_names);
    r.translator = reference.translator().map(contributor_to_csl_names);
    r.publisher = reference.publisher().and_then(|c| c.name());

    match reference {
        InputReference::Monograph(m) => {
            r.ref_type = "book".to_string();
            r.isbn = m.isbn.clone();
            r.url = m.url.as_ref().map(std::string::ToString::to_string);
            r.edition = m.edition.clone().map(StringOrNumber::String);
        }
        InputReference::SerialComponent(s) => {
            r.ref_type = "article-journal".to_string();
            r.container_title = match &s.parent {
                citum_schema::reference::Parent::Embedded(parent) => {
                    parent.title.as_ref().map(std::string::ToString::to_string)
                }
                citum_schema::reference::Parent::Id(_) => None,
            };
            r.page = s.pages.clone();
            r.volume = s
                .volume
                .as_ref()
                .map(|v| StringOrNumber::String(v.to_string()));
            r.issue = s
                .issue
                .as_ref()
                .map(|v| StringOrNumber::String(v.to_string()));
            r.url = s.url.as_ref().map(std::string::ToString::to_string);
        }
        InputReference::CollectionComponent(c) => {
            r.ref_type = "chapter".to_string();
            r.container_title = match &c.parent {
                citum_schema::reference::Parent::Embedded(parent) => {
                    parent.title.as_ref().map(ToString::to_string)
                }
                citum_schema::reference::Parent::Id(_) => None,
            };
            r.page = c.pages.as_ref().map(std::string::ToString::to_string);
        }
        _ => {
            r.ref_type = "book".to_string();
        }
    }

    r
}

fn contributor_to_csl_names(
    contributor: citum_schema::reference::Contributor,
) -> Vec<csl_legacy::csl_json::Name> {
    let mut names = Vec::new();
    match contributor {
        citum_schema::reference::Contributor::SimpleName(n) => {
            names.push(csl_legacy::csl_json::Name::literal(&n.name.to_string()));
        }
        citum_schema::reference::Contributor::StructuredName(n) => {
            names.push(csl_legacy::csl_json::Name {
                family: Some(n.family.to_string()),
                given: Some(n.given.to_string()),
                suffix: n.suffix,
                dropping_particle: n.dropping_particle,
                non_dropping_particle: n.non_dropping_particle,
                literal: None,
            });
        }
        citum_schema::reference::Contributor::Multilingual(n) => {
            names.push(csl_legacy::csl_json::Name {
                family: Some(n.original.family.to_string()),
                given: Some(n.original.given.to_string()),
                suffix: n.original.suffix,
                dropping_particle: n.original.dropping_particle,
                non_dropping_particle: n.original.non_dropping_particle,
                literal: None,
            });
        }
        citum_schema::reference::Contributor::ContributorList(list) => {
            for member in list.0 {
                names.extend(contributor_to_csl_names(member));
            }
        }
    }
    names
}

fn render_biblatex(input: &InputBibliography) -> String {
    let mut out = String::new();
    for reference in &input.references {
        let id = reference.id().unwrap_or_else(|| "item".to_string());
        let entry_type = match reference {
            InputReference::SerialComponent(_) => "article",
            InputReference::CollectionComponent(_) => "incollection",
            _ => "book",
        };
        let _ = writeln!(&mut out, "@{entry_type}{{{id},");
        if let Some(title) = reference.title() {
            let _ = writeln!(&mut out, "  title = {{{title}}},");
        }
        if let Some(contributor) = reference.author() {
            let names: Vec<String> = contributor_to_biblatex_names(contributor);
            if !names.is_empty() {
                let _ = writeln!(&mut out, "  author = {{{}}},", names.join(" and "));
            }
        }
        if let Some(issued) = reference.issued()
            && let Some(year) = issued.0.get(0..4)
        {
            let _ = writeln!(&mut out, "  year = {{{year}}},");
        }
        if let Some(doi) = reference.doi() {
            let _ = writeln!(&mut out, "  doi = {{{doi}}},");
        }
        let _ = writeln!(&mut out, "}}\n");
    }
    out
}

fn contributor_to_biblatex_names(contributor: citum_schema::reference::Contributor) -> Vec<String> {
    match contributor {
        citum_schema::reference::Contributor::SimpleName(n) => vec![n.name.to_string()],
        citum_schema::reference::Contributor::StructuredName(n) => {
            vec![format!("{}, {}", n.family, n.given)]
        }
        citum_schema::reference::Contributor::Multilingual(n) => {
            vec![format!("{}, {}", n.original.family, n.original.given)]
        }
        citum_schema::reference::Contributor::ContributorList(list) => list
            .0
            .into_iter()
            .flat_map(contributor_to_biblatex_names)
            .collect(),
    }
}

fn parse_ris(input: &str) -> Result<InputBibliography, Box<dyn Error>> {
    let mut references = Vec::<InputReference>::new();
    let mut current = Vec::<(String, String)>::new();

    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.len() < 6 {
            continue;
        }
        let tag = line[0..2].to_string();
        let value = line[6..].trim().to_string();
        if tag == "ER" {
            if !current.is_empty() {
                references.push(InputReference::from(ris_record_to_reference(&current)));
            }
            current.clear();
            continue;
        }
        current.push((tag, value));
    }

    if !current.is_empty() {
        references.push(InputReference::from(ris_record_to_reference(&current)));
    }

    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

fn ris_record_to_reference(fields: &[(String, String)]) -> csl_legacy::csl_json::Reference {
    use csl_legacy::csl_json::{DateVariable, Name, Reference, StringOrNumber};

    let get = |tag: &str| -> Option<String> {
        fields
            .iter()
            .find_map(|(k, v)| (k == tag).then(|| v.clone()))
    };
    let get_all = |tag: &str| -> Vec<String> {
        fields
            .iter()
            .filter(|(k, _)| k == tag)
            .map(|(_, v)| v.clone())
            .collect()
    };

    let id = get("ID")
        .or_else(|| get("L1"))
        .or_else(|| get("M1"))
        .unwrap_or_else(|| "item".to_string());
    let title = get("TI").or_else(|| get("T1"));
    let ty = get("TY").unwrap_or_else(|| "BOOK".to_string());
    let author = {
        let authors = get_all("AU")
            .into_iter()
            .map(|n| {
                let parts: Vec<_> = n.split(',').map(str::trim).collect();
                if parts.len() >= 2 {
                    Name::new(parts[0], parts[1])
                } else {
                    Name::literal(parts.first().copied().unwrap_or(""))
                }
            })
            .collect::<Vec<_>>();
        (!authors.is_empty()).then_some(authors)
    };
    let issued = get("PY").or_else(|| get("Y1")).and_then(|s| {
        let year = s.chars().take(4).collect::<String>().parse::<i32>().ok()?;
        Some(DateVariable::year(year))
    });
    let doi = get("DO");
    let note = get("N1");
    let page = match (get("SP"), get("EP")) {
        (Some(sp), Some(ep)) => Some(format!("{sp}-{ep}")),
        (Some(sp), None) => Some(sp),
        _ => None,
    };
    let ref_type = if ty == "JOUR" || ty == "JFULL" {
        "article-journal".to_string()
    } else if ty == "CHAP" {
        "chapter".to_string()
    } else {
        "book".to_string()
    };

    Reference {
        id,
        ref_type,
        author,
        title,
        container_title: get("JO").or_else(|| get("JF")),
        issued,
        volume: get("VL").map(StringOrNumber::String),
        issue: get("IS").map(StringOrNumber::String),
        page,
        doi,
        url: get("UR"),
        isbn: get("SN"),
        publisher: get("PB"),
        publisher_place: get("CY"),
        language: get("LA"),
        note,
        ..Default::default()
    }
}

fn render_ris(input: &InputBibliography) -> String {
    let mut out = String::new();
    for reference in &input.references {
        let ty = match reference {
            InputReference::SerialComponent(_) => "JOUR",
            InputReference::CollectionComponent(_) => "CHAP",
            _ => "BOOK",
        };
        let _ = writeln!(&mut out, "TY  - {ty}");
        if let Some(id) = reference.id() {
            let _ = writeln!(&mut out, "ID  - {id}");
        }
        if let Some(title) = reference.title() {
            let _ = writeln!(&mut out, "TI  - {title}");
        }
        if let Some(contributor) = reference.author() {
            for name in contributor_to_biblatex_names(contributor) {
                let _ = writeln!(&mut out, "AU  - {name}");
            }
        }
        if let Some(issued) = reference.issued()
            && let Some(year) = issued.0.get(0..4)
        {
            let _ = writeln!(&mut out, "PY  - {year}");
        }
        if let Some(doi) = reference.doi() {
            let _ = writeln!(&mut out, "DO  - {doi}");
        }
        let _ = writeln!(&mut out, "ER  -\n");
    }
    out
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

fn load_locale_override_for_file_style(
    override_id: &str,
    style_path: &str,
) -> Result<Option<LocaleOverride>, Box<dyn Error>> {
    let overrides_dir = find_locales_dir(style_path).join("overrides");
    load_locale_override_from_dir(override_id, &overrides_dir)
}

/// Load a Citum style from a file path.
///
/// Selects the deserialiser based on the file extension (`cbor`, `json`, or YAML
/// for anything else).
fn load_style(path: &Path, _no_semantics: bool) -> Result<Style, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    let style_obj: Style = match ext {
        "cbor" => ciborium::de::from_reader(std::io::Cursor::new(&bytes))?,
        "json" => serde_json::from_slice(&bytes)?,
        _ => Style::from_yaml_bytes(&bytes)?,
    };

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

fn load_locale_override_from_dir(
    override_id: &str,
    overrides_dir: &Path,
) -> Result<Option<LocaleOverride>, Box<dyn Error>> {
    for ext in ["yaml", "yml", "json", "cbor"] {
        let path = overrides_dir.join(format!("{override_id}.{ext}"));
        if path.exists() && path.is_file() {
            return load_locale_override_file(&path).map(Some);
        }
    }
    Ok(None)
}

fn load_locale_override_file(path: &Path) -> Result<LocaleOverride, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");
    parse_locale_override_bytes(&bytes, ext)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err).into())
}

fn load_locale_override_builtin(override_id: &str) -> Option<LocaleOverride> {
    let bytes = citum_schema::embedded::get_locale_override_bytes(override_id)?;
    parse_locale_override_bytes(bytes, "yaml").ok()
}

fn parse_locale_override_bytes(bytes: &[u8], ext: &str) -> Result<LocaleOverride, String> {
    use citum_schema::locale::raw::RawLocaleOverride;

    match ext {
        "cbor" => ciborium::de::from_reader::<RawLocaleOverride, _>(std::io::Cursor::new(bytes))
            .map(Into::into)
            .map_err(|e| format!("Failed to parse CBOR locale override: {e}")),
        "json" => serde_json::from_slice::<RawLocaleOverride>(bytes)
            .map(Into::into)
            .map_err(|e| format!("Failed to parse JSON locale override: {e}")),
        _ => serde_yaml::from_slice::<RawLocaleOverride>(bytes)
            .map(Into::into)
            .map_err(|e| format!("Failed to parse YAML locale override: {e}")),
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
                    return Err(format!("Duplicate compound set id while merging: {set_id}").into());
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
        println!("{output}");
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
    // Check if the style has bibliography groups defined
    if ctx
        .processor
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
            // Use grouped renderer for human-readable output (preserves group headings)
            let grouped = ctx.processor.render_grouped_bibliography_with_format::<F>();
            output.push_str(&grouped);
        }
    } else {
        let _ = writeln!(output, "BIBLIOGRAPHY:");
        if show_keys {
            // Oracle/show_keys path: render each entry individually so entries
            // can be matched by reference ID. Compound merging is skipped here
            // because the oracle addresses each ref independently.
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
            // Human-readable path: use the engine bibliography renderer so
            // compound numeric groups are merged while still honoring keys.
            let bib = ctx
                .processor
                .render_selected_bibliography_with_format::<F, _>(ctx.item_ids.to_vec());
            let _ = writeln!(output, "{bib}");
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
            result["citations"] = json!(rendered);
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

            result["citations"] = json!({
                "non-integral": non_integral,
                "integral": integral
            });
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
    use citum_schema::locale::types::{EvaluationConfig, MessageSyntax};
    use citum_schema::options::{Config, Processing};
    use citum_schema::template::{
        NumberVariable, TemplateComponent, TemplateContributor, TemplateNumber, TemplateTerm,
        WrapPunctuation,
    };
    use std::collections::HashMap;
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
    // convert refs format inference
    // ------------------------------------------------------------------

    #[test]
    fn test_infer_refs_output_format_yaml() {
        assert!(matches!(
            infer_refs_output_format(Path::new("refs.yaml")),
            RefsFormat::CitumYaml
        ));
    }

    #[test]
    fn test_infer_refs_output_format_bib() {
        assert!(matches!(
            infer_refs_output_format(Path::new("refs.bib")),
            RefsFormat::Biblatex
        ));
    }

    #[test]
    fn test_infer_refs_output_format_ris() {
        assert!(matches!(
            infer_refs_output_format(Path::new("refs.ris")),
            RefsFormat::Ris
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
            create_processor(style, loaded, "chicago", false).expect("processor should load");

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

    #[test]
    fn test_lint_raw_locale_reports_invalid_mf1_syntax() {
        let raw = RawLocale {
            locale: "en-US".into(),
            evaluation: Some(EvaluationConfig {
                message_syntax: MessageSyntax::Mf1,
            }),
            messages: HashMap::from([(
                "term.page-label".into(),
                "{count, plural, one {p.} other {pp.}".into(),
            )]),
            ..Default::default()
        };

        let report = lint_raw_locale(&raw);

        assert!(report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.path == "messages.term.page-label"
                && finding.message.contains("invalid MF1 message")
        }));
    }

    #[test]
    fn test_lint_raw_locale_reports_missing_alias_target() {
        let raw = RawLocale {
            locale: "en-US".into(),
            messages: HashMap::from([("term.page-label".into(), "p.".into())]),
            legacy_term_aliases: HashMap::from([("page".into(), "term.page-label-long".into())]),
            ..Default::default()
        };

        let report = lint_raw_locale(&raw);

        assert!(report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.path == "legacy-term-aliases.page" && finding.message.contains("does not exist")
        }));
    }

    #[test]
    fn test_lint_style_against_locale_warns_for_missing_general_term() {
        let style = Style {
            citation: Some(citum_schema::CitationSpec {
                template: Some(vec![TemplateComponent::Term(TemplateTerm {
                    term: GeneralTerm::NoDate,
                    form: Some(TermForm::Short),
                    ..Default::default()
                })]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let locale = Locale::default();

        let report = lint_style_against_locale(&style, &locale);

        assert!(!report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.severity == LintSeverity::Warning
                && finding.path == "citation.template[0]"
                && finding.message.contains("general term")
        }));
    }

    #[test]
    fn test_lint_style_against_locale_warns_for_missing_role_term() {
        let style = Style {
            citation: Some(citum_schema::CitationSpec {
                template: Some(vec![TemplateComponent::Contributor(TemplateContributor {
                    contributor: ContributorRole::Editor,
                    form: ContributorForm::Verb,
                    ..Default::default()
                })]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let locale = Locale::default();

        let report = lint_style_against_locale(&style, &locale);

        assert!(!report.has_errors());
        assert!(report.findings.iter().any(|finding| {
            finding.severity == LintSeverity::Warning
                && finding.path == "citation.template[0]"
                && finding.message.contains("role term")
        }));
    }
}
