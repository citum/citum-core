#[cfg(feature = "typescript")]
use crate::args::BindingsArgs;
use crate::args::{
    CheckArgs, CheckItem, Cli, Commands, ConvertCommands, ConvertRefsArgs, ConvertTypedArgs,
    DataType, InputFormat, LintLocaleArgs, LintStyleArgs, LocaleCommands, OutputFormat, RefsFormat,
    RegistryCommands, RenderCommands, RenderDocArgs, RenderMode, RenderRefsArgs,
    StyleCatalogFormat, StyleCommands,
};
#[cfg(feature = "schema")]
use crate::args::{SchemaArgs, SchemaType};
use crate::output::{print_lint_report, write_output};
use crate::style_resolver::{create_processor, load_any_style, load_locale_file};
use crate::table::build_table;
use crate::typst_pdf;
use citum_engine::{
    Citation, CitationItem, DocumentFormat, Processor,
    io::{
        AnnotationFormat, AnnotationStyle, RefsFormat as EngineRefsFormat,
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
use citum_schema::lint::{lint_raw_locale, lint_style_against_locale};
use citum_schema::locale::RawLocale;
use citum_schema::options::Processing;
use citum_schema::{RegistryEntry, Style};
use citum_store::{
    StoreConfig, StoreResolver, platform_cache_dir, platform_config_dir, platform_data_dir,
};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
#[cfg(feature = "schema")]
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

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
        Commands::Registry { command } => match command {
            RegistryCommands::List { format } => run_registry_list(&format),
            RegistryCommands::Add { source, name } => run_registry_add(&source, name.as_deref()),
            RegistryCommands::Remove { name, yes } => run_registry_remove(&name, yes),
            RegistryCommands::Update { name, all } => run_registry_update(name.as_deref(), all),
            RegistryCommands::Resolve { name } => run_registry_resolve(&name),
        },
        Commands::Style { command } => match command {
            StyleCommands::List {
                source,
                format,
                limit,
                offset,
            } => run_style_list(&source, format, limit, offset),
            StyleCommands::Search {
                query,
                source,
                format,
                limit,
                offset,
            } => run_style_search(&query, &source, format, limit, offset),
            StyleCommands::Info { name, format } => run_style_info(&name, format),
            StyleCommands::Browse { query, source } => run_style_browse(query.as_deref(), &source),
            StyleCommands::Add { query, yes } => run_style_add(&query, yes),
            StyleCommands::Remove { name, yes } => run_style_remove(&name, yes),
            StyleCommands::Lint(args) => run_lint_style(args),
        },
        Commands::Locale { command } => match command {
            LocaleCommands::List { source, format } => run_locale_list(&source, format),
            LocaleCommands::Add { path } => run_locale_add(&path),
            LocaleCommands::Remove { name, yes } => run_locale_remove(&name, yes),
            LocaleCommands::Lint(args) => run_lint_locale(args),
        },
        Commands::Doctor { json } => run_doctor(json),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RegistrySourceRecord {
    name: String,
    source: String,
}

#[derive(Clone, Debug)]
struct LoadedRegistry {
    name: String,
    source: String,
    registry: citum_schema::StyleRegistry,
}

#[derive(Serialize)]
struct RegistryInfo {
    name: String,
    source: String,
    version: String,
    styles: usize,
    status: String,
}

#[derive(Clone, Serialize)]
struct StyleCatalogRow {
    source: String,
    id: String,
    aliases: Vec<String>,
    title: Option<String>,
    description: Option<String>,
    fields: Vec<String>,
    url: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CatalogSourceFilter<'a> {
    All,
    Embedded,
    Installed,
    Registry(&'a str),
}

impl<'a> CatalogSourceFilter<'a> {
    fn parse(source: &'a str) -> Result<Self, Box<dyn Error>> {
        match source {
            "all" => Ok(Self::All),
            "embedded" => Ok(Self::Embedded),
            "installed" => Ok(Self::Installed),
            s if s.starts_with("registry:") => {
                let name = s.trim_start_matches("registry:");
                if name.is_empty() {
                    Err("registry source filter requires a name: registry:<name>".into())
                } else {
                    Ok(Self::Registry(name))
                }
            }
            _ => Err(format!(
                "unknown source '{source}' (expected all, embedded, installed, or registry:<name>)"
            )
            .into()),
        }
    }

    fn label(self) -> String {
        match self {
            Self::All => "all".to_string(),
            Self::Embedded => "embedded".to_string(),
            Self::Installed => "installed".to_string(),
            Self::Registry(name) => format!("registry:{name}"),
        }
    }
}

fn style_entry_kind(entry: &RegistryEntry) -> &'static str {
    if entry.builtin.is_some() {
        "embedded"
    } else if entry.url.is_some() {
        "url"
    } else if entry.path.is_some() {
        "path"
    } else {
        "unknown"
    }
}

fn style_entry_matches_source(source_name: &str, source: CatalogSourceFilter<'_>) -> bool {
    match source {
        CatalogSourceFilter::All => true,
        CatalogSourceFilter::Embedded => source_name == "embedded",
        CatalogSourceFilter::Installed => source_name == "installed",
        CatalogSourceFilter::Registry(name) => source_name == format!("registry:{name}"),
    }
}

fn style_catalog_row(source: &str, entry: &RegistryEntry) -> StyleCatalogRow {
    let title = entry.title.clone().or_else(|| {
        entry.builtin.as_ref().and_then(|builtin| {
            citum_schema::embedded::get_embedded_style(builtin)
                .and_then(Result::ok)
                .and_then(|style| style.info.title)
        })
    });

    StyleCatalogRow {
        source: source.to_string(),
        id: entry.id.clone(),
        aliases: entry.aliases.clone(),
        title,
        description: entry.description.clone(),
        fields: entry.fields.clone(),
        url: entry.url.clone(),
    }
}

fn installed_style_catalog_row(id: String) -> StyleCatalogRow {
    StyleCatalogRow {
        source: "installed".to_string(),
        id,
        aliases: Vec::new(),
        title: None,
        description: None,
        fields: Vec::new(),
        url: None,
    }
}

#[derive(Debug, Clone, Copy)]
struct StyleCatalogPage {
    limit: Option<usize>,
    offset: usize,
}

fn paginate_style_catalog_rows(
    mut rows: Vec<StyleCatalogRow>,
    page: StyleCatalogPage,
) -> (usize, Vec<StyleCatalogRow>) {
    let total = rows.len();
    if page.offset >= total {
        return (total, Vec::new());
    }
    rows.drain(..page.offset);
    if let Some(limit) = page.limit {
        rows.truncate(limit);
    }
    (total, rows)
}

fn print_style_catalog_rows(
    rows: &[StyleCatalogRow],
    total: usize,
    source: &str,
    format: StyleCatalogFormat,
) -> Result<(), Box<dyn Error>> {
    if format == StyleCatalogFormat::Json {
        println!("{}", serde_json::to_string_pretty(rows)?);
        return Ok(());
    }

    print!("{}", format_style_catalog_text(rows, total, source));
    Ok(())
}

fn format_style_catalog_text(rows: &[StyleCatalogRow], total: usize, source: &str) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "{total} {source} styles");
    if rows.len() != total {
        let _ = writeln!(output, "showing {}", rows.len());
    }
    output.push('\n');

    let table_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            vec![
                row.source.clone(),
                row.id.clone(),
                row.title.as_deref().unwrap_or("-").to_string(),
            ]
        })
        .collect();

    let table = build_table(&["Source", "ID", "Title"], table_rows);
    output.push_str(&table);
    output
}

fn validate_resource_name(name: &str) -> Result<(), Box<dyn Error>> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
    {
        Ok(())
    } else {
        Err(
            format!("Invalid name: '{name}'. Names must be alphanumeric, hyphens, or underscores.")
                .into(),
        )
    }
}

fn configured_registry_dir() -> Result<PathBuf, Box<dyn Error>> {
    platform_config_dir()
        .map(|dir| dir.join("registries"))
        .ok_or_else(|| "Unable to determine Citum config directory".into())
}

fn registry_sources_path() -> Result<PathBuf, Box<dyn Error>> {
    platform_config_dir()
        .map(|dir| dir.join("registry-sources.json"))
        .ok_or_else(|| "Unable to determine Citum config directory".into())
}

fn read_registry_source_records() -> Result<Vec<RegistrySourceRecord>, Box<dyn Error>> {
    let path = registry_sources_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn write_registry_source_records(records: &[RegistrySourceRecord]) -> Result<(), Box<dyn Error>> {
    let path = registry_sources_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(records)?)?;
    Ok(())
}

fn registry_file_path(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    Ok(configured_registry_dir()?.join(format!("{name}.yaml")))
}

fn fetch_registry_bytes(source: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let resolver = citum_store::HttpResolver::from_platform_cache_dir()
            .ok_or("Unable to determine platform cache directory")?;
        return resolver.fetch_bytes(source);
    }
    Ok(fs::read(source)?)
}

fn parse_registry_bytes(bytes: &[u8]) -> Result<citum_schema::StyleRegistry, Box<dyn Error>> {
    let registry: citum_schema::StyleRegistry = serde_yaml::from_slice(bytes)?;
    registry.validate_sources()?;
    Ok(registry)
}

fn infer_registry_name(source: &str) -> Result<String, Box<dyn Error>> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let url = url::Url::parse(source)?;
        return url
            .host_str()
            .map(|host| host.replace('.', "-"))
            .ok_or_else(|| format!("URL has no host: {source}").into());
    }
    let path = Path::new(source);
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(ToString::to_string)
        .ok_or_else(|| format!("cannot infer registry name from {source}").into())
}

fn load_configured_registries() -> Result<Vec<LoadedRegistry>, Box<dyn Error>> {
    let records = read_registry_source_records()?;
    let mut registries = Vec::new();
    for record in records {
        let path = registry_file_path(&record.name)?;
        if !path.exists() {
            continue;
        }
        let registry = citum_schema::StyleRegistry::load_from_file(&path)?;
        registries.push(LoadedRegistry {
            name: record.name,
            source: record.source,
            registry,
        });
    }
    Ok(registries)
}

fn load_local_registry() -> Option<LoadedRegistry> {
    let path = Path::new("citum-registry.yaml");
    if !path.exists() {
        return None;
    }
    let registry = citum_schema::StyleRegistry::load_from_file(path).ok()?;
    Some(LoadedRegistry {
        name: "local".to_string(),
        source: path.display().to_string(),
        registry,
    })
}

fn load_registry_chain() -> Result<Vec<LoadedRegistry>, Box<dyn Error>> {
    let mut registries = Vec::new();
    if let Some(local) = load_local_registry() {
        registries.push(local);
    }
    registries.extend(load_configured_registries()?);
    registries.push(LoadedRegistry {
        name: "embedded".to_string(),
        source: "embedded".to_string(),
        registry: citum_schema::embedded::default_registry(),
    });
    Ok(registries)
}

fn style_catalog_entries(
    source: CatalogSourceFilter<'_>,
) -> Result<Vec<StyleCatalogRow>, Box<dyn Error>> {
    let mut rows = Vec::new();
    for loaded in load_registry_chain()? {
        for entry in &loaded.registry.styles {
            let actual_kind = style_entry_kind(entry);
            let row_source = if loaded.name == "embedded" {
                if actual_kind == "embedded" {
                    "embedded".to_string()
                } else {
                    "registry:default".to_string()
                }
            } else {
                format!("registry:{}", loaded.name)
            };

            if style_entry_matches_source(&row_source, source) {
                // Special case: if filtering for 'embedded', only show truly embedded entries
                if matches!(source, CatalogSourceFilter::Embedded) && actual_kind != "embedded" {
                    continue;
                }
                rows.push(style_catalog_row(&row_source, entry));
            }
        }
    }

    if style_entry_matches_source("installed", source)
        && let Some(data_dir) = platform_data_dir()
    {
        let config = StoreConfig::load().unwrap_or_default();
        let resolver = StoreResolver::new(data_dir, config.store_format());
        rows.extend(
            resolver
                .list_styles()?
                .into_iter()
                .map(installed_style_catalog_row),
        );
    }

    Ok(rows)
}

fn run_style_list(
    source: &str,
    format: StyleCatalogFormat,
    limit: Option<usize>,
    offset: usize,
) -> Result<(), Box<dyn Error>> {
    let source_filter = CatalogSourceFilter::parse(source)?;
    let rows = style_catalog_entries(source_filter)?;
    let (total, rows) = paginate_style_catalog_rows(rows, StyleCatalogPage { limit, offset });
    print_style_catalog_rows(&rows, total, &source_filter.label(), format)
}

fn style_row_matches_query(row: &StyleCatalogRow, query: &str) -> bool {
    let query = query.to_lowercase();
    row.id.to_lowercase().contains(&query)
        || row
            .aliases
            .iter()
            .any(|alias| alias.to_lowercase().contains(&query))
        || row
            .title
            .as_ref()
            .is_some_and(|title| title.to_lowercase().contains(&query))
        || row
            .description
            .as_ref()
            .is_some_and(|description| description.to_lowercase().contains(&query))
        || row
            .fields
            .iter()
            .any(|field| field.to_lowercase().contains(&query))
}

fn run_style_search(
    query: &str,
    source: &str,
    format: StyleCatalogFormat,
    limit: Option<usize>,
    offset: usize,
) -> Result<(), Box<dyn Error>> {
    let source_filter = CatalogSourceFilter::parse(source)?;
    let rows: Vec<_> = style_catalog_entries(source_filter)?
        .into_iter()
        .filter(|row| style_row_matches_query(row, query))
        .collect();
    let (total, rows) = paginate_style_catalog_rows(rows, StyleCatalogPage { limit, offset });
    print_style_catalog_rows(&rows, total, &source_filter.label(), format)
}

fn run_style_info(name: &str, format: StyleCatalogFormat) -> Result<(), Box<dyn Error>> {
    let rows = style_catalog_entries(CatalogSourceFilter::All)?;
    let row = rows
        .into_iter()
        .find(|row| row.id == name || row.aliases.iter().any(|alias| alias == name))
        .ok_or_else(|| format!("style not found: {name}"))?;

    if format == StyleCatalogFormat::Json {
        println!("{}", serde_json::to_string_pretty(&row)?);
        return Ok(());
    }

    println!("ID:       {}", row.id);
    println!("Title:    {}", row.title.as_deref().unwrap_or("-"));
    println!("Source:   {}", row.source);
    println!(
        "Aliases:  {}",
        if row.aliases.is_empty() {
            "-".to_string()
        } else {
            row.aliases.join(", ")
        }
    );
    if let Some(description) = row.description {
        println!("Summary:  {description}");
    }
    if !row.fields.is_empty() {
        println!("Fields:   {}", row.fields.join(", "));
    }
    if let Some(url) = row.url {
        println!("URL:      {url}");
    }
    Ok(())
}

fn run_style_browse(query: Option<&str>, source: &str) -> Result<(), Box<dyn Error>> {
    let source_filter = CatalogSourceFilter::parse(source)?;
    let all_rows = style_catalog_entries(source_filter)?;
    if !io::stdin().is_terminal() {
        let rows: Vec<_> = all_rows
            .into_iter()
            .filter(|row| query.is_none_or(|q| style_row_matches_query(row, q)))
            .take(20)
            .collect();
        print_style_catalog_rows(
            &rows,
            rows.len(),
            &source_filter.label(),
            StyleCatalogFormat::Text,
        )?;
        return Ok(());
    }

    let mut filter = query.unwrap_or("").to_string();
    let mut offset = 0usize;
    loop {
        let rows: Vec<_> = all_rows
            .iter()
            .filter(|row| filter.is_empty() || style_row_matches_query(row, &filter))
            .cloned()
            .collect();
        if rows.is_empty() {
            println!("No styles match '{filter}'.");
        } else {
            print_browse_page(&rows, offset, &filter);
        }
        println!();
        println!("Commands: /text filter, n next, p previous, i <n> info, a <n> add, q quit");
        print!("browse> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let command = input.trim();
        match command {
            "" => {}
            "q" | "quit" => break,
            "n" | "next" => {
                if offset + 10 < rows.len() {
                    offset += 10;
                }
            }
            "p" | "prev" | "previous" => {
                offset = offset.saturating_sub(10);
            }
            s if s.starts_with('/') => {
                filter = s.trim_start_matches('/').trim().to_string();
                offset = 0;
            }
            s if s.starts_with("i ") => {
                let row = browse_row_by_number(&rows, offset, s.trim_start_matches("i "))?;
                print_style_detail(&row);
            }
            s if s.starts_with("a ") => {
                let row = browse_row_by_number(&rows, offset, s.trim_start_matches("a "))?;
                let style = load_any_style(&row.id, false)?;
                write_installed_style(&row.id, &style)?;
                println!("Installed style: {}", row.id);
            }
            _ => {
                filter = command.to_string();
                offset = 0;
            }
        }
    }

    Ok(())
}

fn print_browse_page(rows: &[StyleCatalogRow], offset: usize, filter: &str) {
    let end = (offset + 10).min(rows.len());
    println!(
        "{} styles{}",
        rows.len(),
        if filter.is_empty() {
            String::new()
        } else {
            format!(" matching '{filter}'")
        }
    );
    for (idx, row) in rows.iter().enumerate().skip(offset).take(10) {
        println!(
            "{:>2}. {:<36} {}",
            idx - offset + 1,
            row.id,
            row.title.as_deref().unwrap_or("-")
        );
    }
    println!("Showing {}-{} of {}", offset + 1, end, rows.len());
}

fn browse_row_by_number(
    rows: &[StyleCatalogRow],
    offset: usize,
    input: &str,
) -> Result<StyleCatalogRow, Box<dyn Error>> {
    let choice = input.trim().parse::<usize>()?;
    if choice == 0 || choice > 10 {
        return Err("selection out of range".into());
    }
    rows.get(offset + choice - 1)
        .cloned()
        .ok_or_else(|| "selection out of range".into())
}

fn print_style_detail(row: &StyleCatalogRow) {
    println!("ID:       {}", row.id);
    println!("Title:    {}", row.title.as_deref().unwrap_or("-"));
    println!("Source:   {}", row.source);
    println!(
        "Aliases:  {}",
        if row.aliases.is_empty() {
            "-".to_string()
        } else {
            row.aliases.join(", ")
        }
    );
    if let Some(description) = &row.description {
        println!("Summary:  {description}");
    }
    if !row.fields.is_empty() {
        println!("Fields:   {}", row.fields.join(", "));
    }
}

fn style_install_name_from_url(input: &str) -> Result<String, Box<dyn Error>> {
    let url = url::Url::parse(input)?;
    url.path_segments()
        .and_then(Iterator::last)
        .and_then(|segment| segment.split('.').next())
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| format!("cannot infer style name from {input}").into())
}

fn write_installed_style(name: &str, style: &Style) -> Result<(), Box<dyn Error>> {
    validate_resource_name(name)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let format = config.store_format();
    let styles_dir = data_dir.join("styles");
    fs::create_dir_all(&styles_dir)?;
    let path = styles_dir.join(format!("{name}.{}", format.extension()));
    let bytes = match format {
        citum_store::StoreFormat::Yaml => serde_yaml::to_string(style)?.into_bytes(),
        citum_store::StoreFormat::Json => serde_json::to_string(style)?.into_bytes(),
        citum_store::StoreFormat::Cbor => {
            let mut bytes = Vec::new();
            ciborium::ser::into_writer(style, &mut bytes)?;
            bytes
        }
    };
    fs::write(path, bytes)?;
    Ok(())
}

fn install_style_from_file(path: &Path) -> Result<String, Box<dyn Error>> {
    if !path.exists() || !path.is_file() {
        return Err(format!("file not found: {}", path.display()).into());
    }
    let _ = load_any_style(&path.display().to_string(), false)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());
    Ok(resolver.install_style(path)?)
}

fn select_style_match(query: &str, yes: bool) -> Result<StyleCatalogRow, Box<dyn Error>> {
    let rows = style_catalog_entries(CatalogSourceFilter::All)?;
    let exact: Vec<_> = rows
        .iter()
        .filter(|row| row.id == query || row.aliases.iter().any(|alias| alias == query))
        .cloned()
        .collect();
    if let [row] = exact.as_slice() {
        return Ok(row.clone());
    }

    let mut matches: Vec<_> = rows
        .into_iter()
        .filter(|row| style_row_matches_query(row, query))
        .collect();
    matches.sort_by(|a, b| a.id.len().cmp(&b.id.len()).then_with(|| a.id.cmp(&b.id)));

    match matches.as_slice() {
        [] => Err(format!(
            "style not found: {query}\n\nSearch styles with: citum style search {query}"
        )
        .into()),
        [row] => Ok(row.clone()),
        _ if yes || !io::stdin().is_terminal() => {
            let mut msg = format!("style query is ambiguous: {query}\n\nMatches:");
            for row in matches.iter().take(10) {
                let _ = write!(
                    msg,
                    "\n  - {} ({})",
                    row.id,
                    row.title.as_deref().unwrap_or(&row.source)
                );
            }
            msg.push_str("\n\nRerun with an exact ID or alias.");
            Err(msg.into())
        }
        _ => {
            println!("Multiple styles match '{query}':");
            for (idx, row) in matches.iter().take(10).enumerate() {
                println!(
                    "  {}. {} - {}",
                    idx + 1,
                    row.id,
                    row.title.as_deref().unwrap_or("-")
                );
            }
            print!("Choose a style to install [1-{}]: ", matches.len().min(10));
            io::stdout().flush()?;
            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            let choice = response.trim().parse::<usize>()?;
            if choice == 0 || choice > matches.len().min(10) {
                return Err("selection out of range".into());
            }
            matches
                .get(choice - 1)
                .cloned()
                .ok_or_else(|| "selection out of range".into())
        }
    }
}

fn run_style_add(query: &str, yes: bool) -> Result<(), Box<dyn Error>> {
    let path = Path::new(query);
    let name = if path.exists() || query.starts_with("file://") {
        let raw_path = query.strip_prefix("file://").unwrap_or(query);
        install_style_from_file(Path::new(raw_path))?
    } else if query.starts_with("http://") || query.starts_with("https://") {
        let style = load_any_style(query, false)?;
        let name = style_install_name_from_url(query)?;
        write_installed_style(&name, &style)?;
        name
    } else {
        let row = select_style_match(query, yes)?;
        let style = load_any_style(&row.id, false)?;
        write_installed_style(&row.id, &style)?;
        row.id
    };

    println!("Installed style: {name}");
    Ok(())
}

fn run_style_remove(name: &str, yes: bool) -> Result<(), Box<dyn Error>> {
    validate_resource_name(name)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());
    let styles = resolver.list_styles()?;
    if !styles.contains(&name.to_string()) {
        return Err(format!("installed style not found: {name}").into());
    }
    if !yes && !confirm(&format!("Remove installed style '{name}'?"))? {
        return Ok(());
    }
    resolver.remove_style(name)?;
    println!("Removed style: {name}");
    Ok(())
}

fn confirm(prompt: &str) -> Result<bool, Box<dyn Error>> {
    if !io::stdin().is_terminal() {
        return Err(format!("{prompt} Use --yes to run non-interactively.").into());
    }
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    Ok(matches!(
        response.trim().to_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn run_registry_list(format: &str) -> Result<(), Box<dyn Error>> {
    let mut registries = Vec::new();
    let default_reg = citum_schema::embedded::default_registry();
    registries.push(RegistryInfo {
        name: "embedded".to_string(),
        source: "embedded".to_string(),
        version: default_reg.version.clone(),
        styles: default_reg.styles.len(),
        status: "ok".to_string(),
    });
    if let Some(local) = load_local_registry() {
        registries.push(RegistryInfo {
            name: local.name,
            source: local.source,
            version: local.registry.version,
            styles: local.registry.styles.len(),
            status: "ok".to_string(),
        });
    }
    for record in read_registry_source_records()? {
        let path = registry_file_path(&record.name)?;
        match citum_schema::StyleRegistry::load_from_file(&path) {
            Ok(registry) => registries.push(RegistryInfo {
                name: record.name,
                source: record.source,
                version: registry.version,
                styles: registry.styles.len(),
                status: "ok".to_string(),
            }),
            Err(err) => registries.push(RegistryInfo {
                name: record.name,
                source: record.source,
                version: "-".to_string(),
                styles: 0,
                status: err.to_string(),
            }),
        }
    }

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&registries)?);
    } else {
        let rows = registries
            .iter()
            .map(|reg| {
                vec![
                    reg.name.clone(),
                    reg.source.clone(),
                    reg.version.clone(),
                    reg.styles.to_string(),
                    reg.status.clone(),
                ]
            })
            .collect();
        println!(
            "{}",
            build_table(&["Name", "Source", "Version", "Styles", "Status"], rows)
        );
    }
    Ok(())
}

fn run_registry_add(source: &str, name: Option<&str>) -> Result<(), Box<dyn Error>> {
    let name = name.map_or_else(|| infer_registry_name(source), |name| Ok(name.to_string()))?;
    validate_resource_name(&name)?;
    let bytes = fetch_registry_bytes(source)?;
    let registry = parse_registry_bytes(&bytes)?;
    fs::create_dir_all(configured_registry_dir()?)?;
    fs::write(registry_file_path(&name)?, bytes)?;

    let mut records = read_registry_source_records()?;
    records.retain(|record| record.name != name);
    records.push(RegistrySourceRecord {
        name: name.clone(),
        source: source.to_string(),
    });
    write_registry_source_records(&records)?;

    println!(
        "Added registry '{name}' with {} styles.",
        registry.styles.len()
    );
    Ok(())
}

fn run_registry_remove(name: &str, yes: bool) -> Result<(), Box<dyn Error>> {
    let path = registry_file_path(name)?;
    if !path.exists() {
        return Err(format!("configured registry not found: {name}").into());
    }
    if !yes && !confirm(&format!("Remove registry '{name}'?"))? {
        return Ok(());
    }
    fs::remove_file(path)?;
    let mut records = read_registry_source_records()?;
    records.retain(|record| record.name != name);
    write_registry_source_records(&records)?;
    println!("Removed registry: {name}");
    Ok(())
}

fn run_registry_update(name: Option<&str>, all: bool) -> Result<(), Box<dyn Error>> {
    if name.is_some() == all {
        return Err("Specify either a registry name or --all.".into());
    }
    let records = read_registry_source_records()?;
    let selected: Vec<_> = records
        .iter()
        .filter(|record| all || Some(record.name.as_str()) == name)
        .cloned()
        .collect();
    if selected.is_empty() {
        return Err("No configured registries matched.".into());
    }
    for record in selected {
        let bytes = fetch_registry_bytes(&record.source)?;
        let registry = parse_registry_bytes(&bytes)?;
        fs::write(registry_file_path(&record.name)?, bytes)?;
        println!(
            "Updated registry '{}' ({} styles).",
            record.name,
            registry.styles.len()
        );
    }
    Ok(())
}

fn run_registry_resolve(name: &str) -> Result<(), Box<dyn Error>> {
    for loaded in load_registry_chain()? {
        if let Some(entry) = loaded.registry.resolve(name) {
            println!(
                "{} (registry:{}, {})",
                entry.id,
                loaded.name,
                style_entry_kind(entry)
            );
            return Ok(());
        }
    }
    Err(format!("style not found: {name}").into())
}

#[derive(Serialize)]
struct LocaleRow {
    source: String,
    id: String,
}

fn embedded_locale_ids() -> Vec<String> {
    citum_schema::embedded::EMBEDDED_LOCALE_IDS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn locale_rows(source: &str) -> Result<Vec<LocaleRow>, Box<dyn Error>> {
    let mut rows = Vec::new();
    if matches!(source, "all" | "embedded") {
        rows.extend(embedded_locale_ids().into_iter().map(|id| LocaleRow {
            source: "embedded".to_string(),
            id,
        }));
    }
    if matches!(source, "all" | "installed")
        && let Some(data_dir) = platform_data_dir()
    {
        let config = StoreConfig::load().unwrap_or_default();
        let resolver = StoreResolver::new(data_dir, config.store_format());
        rows.extend(
            resolver
                .list_locales()
                .unwrap_or_default()
                .into_iter()
                .map(|id| LocaleRow {
                    source: "installed".to_string(),
                    id,
                }),
        );
    }
    if !matches!(source, "all" | "embedded" | "installed") {
        return Err(
            format!("unknown source '{source}' (expected all, embedded, or installed)").into(),
        );
    }
    Ok(rows)
}

fn run_locale_list(source: &str, format: StyleCatalogFormat) -> Result<(), Box<dyn Error>> {
    let rows = locale_rows(source)?;
    if format == StyleCatalogFormat::Json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    let table_rows = rows
        .iter()
        .map(|row| vec![row.source.clone(), row.id.clone()])
        .collect();
    println!("{}", build_table(&["Source", "ID"], table_rows));
    Ok(())
}

fn run_locale_add(path: &Path) -> Result<(), Box<dyn Error>> {
    let _ = load_raw_locale(path)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());
    let name = resolver.install_locale(path)?;
    println!("Installed locale: {name}");
    Ok(())
}

fn run_locale_remove(name: &str, yes: bool) -> Result<(), Box<dyn Error>> {
    validate_resource_name(name)?;
    let Some(data_dir) = platform_data_dir() else {
        return Err("Unable to determine platform data directory".into());
    };
    let config = StoreConfig::load().unwrap_or_default();
    let resolver = StoreResolver::new(data_dir, config.store_format());
    let locales = resolver.list_locales()?;
    if !locales.contains(&name.to_string()) {
        return Err(format!("installed locale not found: {name}").into());
    }
    if !yes && !confirm(&format!("Remove installed locale '{name}'?"))? {
        return Ok(());
    }
    resolver.remove_locale(name)?;
    println!("Removed locale: {name}");
    Ok(())
}

#[derive(Serialize)]
struct DoctorReport {
    data_dir: Option<String>,
    config_dir: Option<String>,
    cache_dir: Option<String>,
    installed_styles: usize,
    installed_locales: usize,
    registries: Vec<RegistryInfo>,
}

fn run_doctor(json: bool) -> Result<(), Box<dyn Error>> {
    let data_dir = platform_data_dir();
    let config = StoreConfig::load().unwrap_or_default();
    let (installed_styles, installed_locales) = if let Some(dir) = data_dir.clone() {
        let resolver = StoreResolver::new(dir, config.store_format());
        (
            resolver.list_styles().unwrap_or_default().len(),
            resolver.list_locales().unwrap_or_default().len(),
        )
    } else {
        (0, 0)
    };
    let mut registries = Vec::new();
    let default_reg = citum_schema::embedded::default_registry();
    registries.push(RegistryInfo {
        name: "embedded".to_string(),
        source: "embedded".to_string(),
        version: default_reg.version,
        styles: default_reg.styles.len(),
        status: "ok".to_string(),
    });
    for record in read_registry_source_records().unwrap_or_default() {
        let path = registry_file_path(&record.name)?;
        let status = if path.exists() { "ok" } else { "missing" };
        registries.push(RegistryInfo {
            name: record.name,
            source: record.source,
            version: "-".to_string(),
            styles: 0,
            status: status.to_string(),
        });
    }
    let report = DoctorReport {
        data_dir: data_dir.map(|path| path.display().to_string()),
        config_dir: platform_config_dir().map(|path| path.display().to_string()),
        cache_dir: platform_cache_dir().map(|path| path.display().to_string()),
        installed_styles,
        installed_locales,
        registries,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Data dir:          {}",
            report.data_dir.as_deref().unwrap_or("-")
        );
        println!(
            "Config dir:        {}",
            report.config_dir.as_deref().unwrap_or("-")
        );
        println!(
            "Cache dir:         {}",
            report.cache_dir.as_deref().unwrap_or("-")
        );
        println!("Installed styles:  {}", report.installed_styles);
        println!("Installed locales: {}", report.installed_locales);
        println!("Registries:        {}", report.registries.len());
    }
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

    #[test]
    fn test_style_catalog_embedded_title_falls_back_to_style_metadata() {
        let registry = citum_schema::embedded::default_registry();
        let entry = registry.resolve("apa").expect("APA alias should resolve");

        let row = style_catalog_row("embedded", entry);

        assert_eq!(
            row.title.as_deref(),
            Some("American Psychological Association 7th edition")
        );
    }

    #[test]
    fn test_style_catalog_source_filter_and_pagination() {
        let rows = style_catalog_entries(CatalogSourceFilter::Embedded)
            .expect("embedded catalog should load");
        let (total, page) = paginate_style_catalog_rows(
            rows,
            StyleCatalogPage {
                limit: Some(2),
                offset: 1,
            },
        );

        assert!(total > 2);
        assert_eq!(page.len(), 2);
        assert!(page.iter().all(|row| row.source == "embedded"));
    }

    #[test]
    fn test_style_catalog_search_matches_embedded_title() {
        let registry = citum_schema::embedded::default_registry();
        let rows: Vec<_> = registry
            .styles
            .iter()
            .map(|entry| style_catalog_row("embedded", entry))
            .filter(|row| style_row_matches_query(row, "Psychological Association"))
            .collect();

        assert!(rows.iter().any(|row| row.id == "apa-7th"));
    }

    #[test]
    fn test_style_catalog_text_output_contains_table() {
        let rows = vec![StyleCatalogRow {
            source: "embedded".to_string(),
            id: "alpha".to_string(),
            aliases: Vec::new(),
            title: Some("Alpha (biblatex-alpha)".to_string()),
            description: None,
            fields: Vec::new(),
            url: None,
        }];

        let output = format_style_catalog_text(&rows, 3, "embedded");

        assert!(output.contains("3 embedded styles"));
        assert!(output.contains("showing 1"));
        assert!(output.contains("Source"));
        assert!(output.contains("ID"));
        assert!(output.contains("Title"));
        assert!(output.contains("embedded"));
        assert!(output.contains("alpha"));
        assert!(output.contains("Alpha (biblatex-alpha)"));
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

    #[test]
    fn test_validate_resource_name() {
        assert!(validate_resource_name("apa").is_ok());
        assert!(validate_resource_name("apa-7th").is_ok());
        assert!(validate_resource_name("chicago_fullnote").is_ok());
        assert!(validate_resource_name("").is_err());
        assert!(validate_resource_name("..").is_err());
        assert!(validate_resource_name("../../etc/passwd").is_err());
        assert!(validate_resource_name("styles/apa").is_err());
        assert!(validate_resource_name("apa.yaml").is_err()); // dots not allowed if we want to be strict
        assert!(validate_resource_name("my registry!").is_err());
    }

    #[test]
    fn test_registry_source_record_serialization() {
        let record = RegistrySourceRecord {
            name: "test".to_string(),
            source: "https://example.com/registry.yaml".to_string(),
        };
        let json = serde_json::to_string(&record).expect("should serialize");
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"source\":\"https://example.com/registry.yaml\""));
    }
}
