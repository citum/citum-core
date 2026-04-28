use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use serde::Serialize;
use std::path::PathBuf;

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
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum DataType {
    Style,
    Bib,
    Locale,
    Citations,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum RenderMode {
    Bib,
    Cite,
    Both,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum InputFormat {
    Djot,
    Markdown,
}

/// Valid target types for JSON schema export.
#[cfg(feature = "schema")]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum SchemaType {
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
pub(crate) enum OutputFormat {
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
pub(crate) enum Commands {
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

    /// Export language type bindings for Citum schema types
    #[cfg(feature = "typescript")]
    Bindings(BindingsArgs),

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
pub(crate) enum RenderCommands {
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
                      INPUT FORMATS (--bibliography):\n  \
                      The --bibliography flag accepts:\n    \
                      - Citum YAML (.yaml, .yml) — native Citum reference format\n    \
                      - Citum JSON (.json)        — native Citum reference format (auto-detected by content)\n    \
                      - Citum CBOR (.cbor)        — native Citum reference format (binary)\n    \
                      - CSL-JSON (.json)          — legacy CSL-JSON (auto-detected by content)\n    \
                      Use 'citum convert refs' to convert BibLaTeX or RIS files first.\n\n\
                      EXAMPLES:\n  \
                      Render bibliography entries (APA 7th style):\n    \
                      citum render refs -b refs.json -s apa-7th\n\n  \
                      Render specific citations with keys:\n    \
                      citum render refs -b refs.json -s apa-7th -m cite\n    \
                      -k Doe2020,Smith2021\n\n  \
                      Output as JSON with human-readable rendered text:\n    \
                      citum render refs -b refs.json -s apa-7th --json"
    )]
    Refs(RenderRefsArgs),
}

#[derive(Subcommand)]
pub(crate) enum ConvertCommands {
    /// Convert bibliography/reference files
    #[command(
        about = "Convert bibliography/reference files",
        long_about = "Convert bibliography/reference files between formats.\n\n\
                      INPUT FORMATS (--from):\n  \
                      citum-yaml    Citum native YAML (.yaml or .yml)\n  \
                      citum-json    Citum native JSON (.json; content-sniffed when --from is omitted)\n  \
                      citum-cbor    Citum native CBOR (.cbor)\n  \
                      csl-json      Legacy CSL-JSON (.json; content-sniffed when --from is omitted)\n  \
                      biblatex      BibLaTeX .bib file\n  \
                      ris           RIS (.ris) file\n\n\
                      OUTPUT FORMATS (--to):\n  \
                      Same variants as --from. Default output format is citum-yaml.\n\n\
                      EXAMPLES:\n  \
                      Convert BibLaTeX to Citum YAML:\n    \
                      citum convert refs thesis.bib -o refs.yaml\n\n  \
                      Convert RIS to Citum YAML:\n    \
                      citum convert refs export.ris -o refs.yaml\n\n  \
                      Convert CSL-JSON to Citum YAML:\n    \
                      citum convert refs legacy.json --from csl-json -o refs.yaml"
    )]
    Refs(ConvertRefsArgs),
    /// Convert style files between YAML/JSON/CBOR
    Style(ConvertTypedArgs),
    /// Convert citations files between YAML/JSON/CBOR
    Citations(ConvertTypedArgs),
    /// Convert locale files between YAML/JSON/CBOR
    Locale(ConvertTypedArgs),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum RefsFormat {
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
pub(crate) enum StylesCommands {
    /// List all embedded (builtin) style names
    List,
}

#[derive(Subcommand)]
pub(crate) enum RegistryCommands {
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
pub(crate) enum StoreCommands {
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
pub(crate) enum StyleCommands {
    /// Validate that a style's locale-driven features resolve against a locale file
    Lint(LintStyleArgs),
}

#[derive(Subcommand)]
pub(crate) enum LocaleCommands {
    /// Validate a locale file's message syntax and alias targets
    Lint(LintLocaleArgs),
}

#[derive(Args, Debug)]
pub(crate) struct RenderDocArgs {
    /// Path to input document
    #[arg(index = 1)]
    pub(crate) input: PathBuf,

    /// Style file path or builtin name (apa, mla, ieee, etc.)
    #[arg(short, long, required = true)]
    pub(crate) style: String,

    /// Path(s) to bibliography input files (repeat for multiple)
    #[arg(short, long, required = true, action = ArgAction::Append)]
    pub(crate) bibliography: Vec<PathBuf>,
    #[arg(short = 'c', long, action = ArgAction::Append)]
    pub(crate) citations: Vec<PathBuf>,

    /// Input document format
    #[arg(short = 'I', long = "input-format", value_enum, default_value_t = InputFormat::Djot)]
    pub(crate) input_format: InputFormat,

    /// Output format
    #[arg(
        short,
        long,
        value_enum,
        default_value_t = OutputFormat::Plain
    )]
    pub(crate) format: OutputFormat,

    /// Write output to file (defaults to stdout)
    #[arg(short = 'o', long)]
    pub(crate) output: Option<PathBuf>,

    /// Compile Typst output to PDF (requires `typst-pdf` feature)
    #[arg(long)]
    pub(crate) pdf: bool,

    /// Preserve generated Typst source next to the PDF output
    #[arg(long)]
    pub(crate) typst_keep_source: bool,

    /// Disable semantic classes (HTML spans, Djot attributes)
    #[arg(long)]
    pub(crate) no_semantics: bool,
}

/// Line break style for annotation paragraphs.
#[derive(Clone, Debug, Default, clap::ValueEnum)]
pub(crate) enum ParagraphBreakArg {
    #[default]
    BlankLine,
    SingleLine,
}

#[derive(Args, Debug)]
pub(crate) struct RenderRefsArgs {
    /// Path(s) to bibliography input files (repeat for multiple)
    #[arg(short, long, required = true, action = ArgAction::Append)]
    pub(crate) bibliography: Vec<PathBuf>,

    /// Style file path or builtin name (apa, mla, ieee, etc.)
    #[arg(short, long, required = true)]
    pub(crate) style: String,

    /// Locale ID (e.g. "es-ES", "fr-FR") to override the style's default locale
    #[arg(short = 'L', long)]
    pub(crate) locale: Option<String>,

    /// Path(s) to citations input files (repeat for multiple)
    #[arg(short = 'c', long, action = ArgAction::Append)]
    pub(crate) citations: Vec<PathBuf>,

    /// Render mode
    #[arg(short = 'm', long, value_enum, default_value_t = RenderMode::Both)]
    pub(crate) mode: RenderMode,

    /// Specific reference keys to render (comma-separated)
    #[arg(short = 'k', long, value_delimiter = ',')]
    pub(crate) keys: Option<Vec<String>>,

    /// Show reference keys/IDs in human output
    #[arg(long)]
    pub(crate) show_keys: bool,

    /// Output as JSON
    #[arg(short = 'j', long)]
    pub(crate) json: bool,

    /// Output format
    #[arg(
        short,
        long,
        value_enum,
        default_value_t = OutputFormat::Plain
    )]
    pub(crate) format: OutputFormat,

    /// Write output to file (defaults to stdout)
    #[arg(short = 'o', long)]
    pub(crate) output: Option<PathBuf>,

    /// Disable semantic classes (HTML spans, Djot attributes)
    #[arg(long)]
    pub(crate) no_semantics: bool,

    /// Path to annotations file (JSON or YAML mapping ref IDs to annotation text)
    #[arg(long, value_name = "FILE")]
    pub(crate) annotations: Option<PathBuf>,

    /// Render annotation text in italics
    #[arg(long)]
    pub(crate) annotation_italic: bool,

    /// Indent annotation paragraphs (default: true)
    #[arg(long, default_value_t = true)]
    pub(crate) annotation_indent: bool,

    /// Line break before annotation paragraph
    #[arg(long, value_enum, default_value_t = ParagraphBreakArg::BlankLine)]
    pub(crate) annotation_break: ParagraphBreakArg,
}

#[derive(Args, Debug)]
pub(crate) struct CheckArgs {
    /// Style file path or builtin name (apa, mla, ieee, etc.)
    #[arg(short, long)]
    pub(crate) style: Option<String>,

    /// Path(s) to bibliography input files (repeat for multiple)
    #[arg(short, long, action = ArgAction::Append)]
    pub(crate) bibliography: Vec<PathBuf>,

    /// Path(s) to citations input files (repeat for multiple)
    #[arg(short = 'c', long, action = ArgAction::Append)]
    pub(crate) citations: Vec<PathBuf>,

    /// Output as JSON
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Args, Debug)]
pub(crate) struct LintLocaleArgs {
    /// Path to locale file
    #[arg(index = 1)]
    pub(crate) path: PathBuf,
}

#[derive(Args, Debug)]
pub(crate) struct LintStyleArgs {
    /// Style file path or builtin name
    #[arg(index = 1)]
    pub(crate) style: String,

    /// Locale file used for validation
    #[arg(long, required = true)]
    pub(crate) locale: PathBuf,
}

#[cfg(feature = "schema")]
#[derive(Args, Debug)]
pub(crate) struct SchemaArgs {
    /// Data type to export
    #[arg(index = 1, value_enum)]
    pub(crate) r#type: Option<SchemaType>,

    /// Output directory to export all schemas
    #[arg(short, long)]
    pub(crate) out_dir: Option<PathBuf>,
}

#[cfg(feature = "typescript")]
#[derive(Args, Debug)]
pub(crate) struct BindingsArgs {
    /// Output directory for generated type definition files
    #[arg(short, long)]
    pub(crate) out_dir: PathBuf,
}

#[derive(Args, Debug)]
pub(crate) struct ConvertTypedArgs {
    /// Path to input file
    #[arg(index = 1)]
    pub(crate) input: PathBuf,

    /// Path to output file
    #[arg(short = 'o', long)]
    pub(crate) output: PathBuf,
}

#[derive(Args, Debug)]
pub(crate) struct ConvertRefsArgs {
    /// Path to input bibliography file
    #[arg(index = 1)]
    pub(crate) input: PathBuf,

    /// Path to output bibliography file
    #[arg(short = 'o', long)]
    pub(crate) output: PathBuf,

    /// Input format (auto-detected from extension; .json inputs are content-sniffed to distinguish citum-json from csl-json)
    #[arg(long, value_enum)]
    pub(crate) from: Option<RefsFormat>,

    /// Output format (auto-detected from extension if omitted; defaults to citum-yaml)
    #[arg(long, value_enum)]
    pub(crate) to: Option<RefsFormat>,
}

#[derive(Args, Debug)]
pub(crate) struct LegacyDocArgs {
    /// Path to the document file
    #[arg(index = 1)]
    pub(crate) document: PathBuf,

    /// Path to the references file
    #[arg(index = 2)]
    pub(crate) references: PathBuf,

    /// Path to the style file
    #[arg(index = 3)]
    pub(crate) style: PathBuf,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Plain)]
    pub(crate) format: OutputFormat,
}

#[derive(Args, Debug)]
pub(crate) struct LegacyValidateArgs {
    /// Path to style file
    pub(crate) path: PathBuf,
}

#[derive(Serialize)]
pub(crate) struct CheckItem {
    pub(crate) kind: &'static str,
    pub(crate) path: String,
    pub(crate) ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
}
