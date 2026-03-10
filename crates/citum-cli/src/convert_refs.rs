//! Bibliography/reference conversion module.
//!
//! Supports conversion between multiple reference formats: Citum YAML/JSON/CBOR,
//! CSL-JSON, BibLaTeX, and RIS.

use citum_schema::InputBibliography;
use citum_schema::reference::{
    Contributor, EdtfString, InputReference, MonographType, MultilingualString, SerialType,
    SimpleName, StructuredName, Title,
};
use clap::{Args, ValueEnum};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Reference format for conversion.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum RefsFormat {
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

impl std::fmt::Display for RefsFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RefsFormat::CitumYaml => "citum-yaml",
            RefsFormat::CitumJson => "citum-json",
            RefsFormat::CitumCbor => "citum-cbor",
            RefsFormat::CslJson => "csl-json",
            RefsFormat::Biblatex => "biblatex",
            RefsFormat::Ris => "ris",
        };
        f.write_str(s)
    }
}

/// Arguments for the `convert refs` command.
#[derive(Args, Debug)]
pub struct ConvertRefsArgs {
    /// Path to input bibliography file
    #[arg(index = 1)]
    pub input: std::path::PathBuf,

    /// Path to output bibliography file
    #[arg(short = 'o', long)]
    pub output: std::path::PathBuf,

    /// Input format override
    #[arg(long, value_enum)]
    pub from: Option<RefsFormat>,

    /// Output format override
    #[arg(long, value_enum)]
    pub to: Option<RefsFormat>,
}

/// Execute the refs conversion.
pub fn run_convert_refs(args: ConvertRefsArgs) -> Result<(), Box<dyn Error>> {
    let (input_format, cached_bytes) = if let Some(f) = args.from {
        (f, None)
    } else {
        infer_refs_input_format(&args.input)?
    };
    let output_format = args
        .to
        .unwrap_or_else(|| infer_refs_output_format(&args.output));

    let bibliography = load_input_bibliography(&args.input, input_format, cached_bytes)?;
    write_output_bibliography(&bibliography, &args.output, output_format)?;

    println!(
        "Converted {} ({}) to {} ({})",
        args.input.display(),
        input_format,
        args.output.display(),
        output_format
    );
    Ok(())
}

/// Infer input format from file extension or JSON content.
#[allow(clippy::type_complexity)]
fn infer_refs_input_format(path: &Path) -> Result<(RefsFormat, Option<Vec<u8>>), Box<dyn Error>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let (fmt, cached) = match ext.to_ascii_lowercase().as_str() {
        "yaml" | "yml" => (RefsFormat::CitumYaml, None),
        "cbor" => (RefsFormat::CitumCbor, None),
        "bib" => (RefsFormat::Biblatex, None),
        "ris" => (RefsFormat::Ris, None),
        "json" => {
            let (fmt, bytes) = detect_json_refs_format(path)?;
            (fmt, Some(bytes))
        }
        other => {
            return Err(format!(
                "unrecognized input format '{other}'; use --from to specify explicitly"
            )
            .into());
        }
    };
    Ok((fmt, cached))
}

/// Infer output format from file extension (defaults to citum-yaml).
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

/// Detect whether a JSON file is CSL-JSON or Citum JSON format.
fn detect_json_refs_format(path: &Path) -> Result<(RefsFormat, Vec<u8>), Box<dyn Error>> {
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
    let fmt = if is_csl_array && !is_citum_object {
        RefsFormat::CslJson
    } else {
        RefsFormat::CitumJson
    };
    Ok((fmt, bytes))
}

/// Load a bibliography from file, inferring or using the specified format.
fn load_input_bibliography(
    path: &Path,
    format: RefsFormat,
    cached: Option<Vec<u8>>,
) -> Result<InputBibliography, Box<dyn Error>> {
    match format {
        RefsFormat::CitumYaml => {
            let bytes = cached
                .map(Ok::<_, Box<dyn Error>>)
                .unwrap_or_else(|| fs::read(path).map_err(|e| Box::new(e) as Box<dyn Error>))?;
            deserialize_any(&bytes, "yaml")
        }
        RefsFormat::CitumJson => {
            let bytes = cached
                .map(Ok::<_, Box<dyn Error>>)
                .unwrap_or_else(|| fs::read(path).map_err(|e| Box::new(e) as Box<dyn Error>))?;
            deserialize_any(&bytes, "json")
        }
        RefsFormat::CitumCbor => {
            let bytes = cached
                .map(Ok::<_, Box<dyn Error>>)
                .unwrap_or_else(|| fs::read(path).map_err(|e| Box::new(e) as Box<dyn Error>))?;
            deserialize_any(&bytes, "cbor")
        }
        RefsFormat::CslJson => load_csl_json_bibliography(path),
        RefsFormat::Biblatex => load_biblatex_bibliography(path),
        RefsFormat::Ris => load_ris_bibliography(path),
    }
}

/// Deserialize bibliography from bytes in specified format.
fn deserialize_any(bytes: &[u8], format: &str) -> Result<InputBibliography, Box<dyn Error>> {
    match format {
        "yaml" => serde_yaml::from_slice(bytes).map_err(Into::into),
        "json" => serde_json::from_slice(bytes).map_err(Into::into),
        "cbor" => ciborium::de::from_reader(std::io::Cursor::new(bytes)).map_err(Into::into),
        _ => Err(format!("unknown format: {format}").into()),
    }
}

/// Serialize bibliography to bytes in specified format.
fn serialize_any(bib: &InputBibliography, format: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    match format {
        "yaml" => serde_yaml::to_string(bib)
            .map(|s| s.into_bytes())
            .map_err(Into::into),
        "json" => serde_json::to_string_pretty(bib)
            .map(|s| s.into_bytes())
            .map_err(Into::into),
        "cbor" => {
            let mut buf = Vec::new();
            ciborium::ser::into_writer(bib, &mut buf)?;
            Ok(buf)
        }
        _ => Err(format!("unknown format: {format}").into()),
    }
}

/// Write bibliography to file in specified format.
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

/// Load CSL-JSON bibliography from file.
fn load_csl_json_bibliography(path: &Path) -> Result<InputBibliography, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let refs: Vec<csl_legacy::csl_json::Reference> = serde_json::from_slice(&bytes)?;
    let references = refs.into_iter().map(InputReference::from).collect();
    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

/// Load BibLaTeX bibliography from file.
fn load_biblatex_bibliography(path: &Path) -> Result<InputBibliography, Box<dyn Error>> {
    let src = fs::read_to_string(path)?;
    let bibliography =
        biblatex::Bibliography::parse(&src).map_err(|e| format!("BibLaTeX parse error: {e}"))?;
    let references = bibliography
        .iter()
        .map(input_reference_from_biblatex)
        .collect();
    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

/// Load RIS bibliography from file.
fn load_ris_bibliography(path: &Path) -> Result<InputBibliography, Box<dyn Error>> {
    let src = fs::read_to_string(path)?;
    parse_ris(&src)
}

/// Convert a BibLaTeX entry to an InputReference.
fn input_reference_from_biblatex(entry: &biblatex::Entry) -> InputReference {
    use citum_schema::reference::{
        CollectionComponent, Monograph, Parent, Serial, SerialComponent,
    };

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

    let parse_names = |raw: Option<String>| -> Option<Contributor> {
        raw.map(|s| {
            let names: Vec<_> = s
                .split(" and ")
                .map(|name| {
                    let parts: Vec<_> = name.split(',').map(str::trim).collect();
                    if parts.len() >= 2 {
                        Contributor::StructuredName(StructuredName {
                            family: MultilingualString::Simple(parts[0].to_string()),
                            given: MultilingualString::Simple(parts[1].to_string()),
                            suffix: None,
                            dropping_particle: None,
                            non_dropping_particle: None,
                        })
                    } else {
                        Contributor::SimpleName(SimpleName {
                            name: MultilingualString::Simple(
                                parts.first().copied().unwrap_or("").to_string(),
                            ),
                            location: None,
                        })
                    }
                })
                .collect();

            if names.len() == 1 {
                names.into_iter().next().unwrap()
            } else {
                use citum_schema::reference::ContributorList;
                Contributor::ContributorList(ContributorList(names))
            }
        })
    };

    let ty = entry.entry_type.to_string().to_lowercase();
    let id = Some(entry.key.clone());
    let title = field_str("title").map(Title::Single);
    let author = parse_names(field_str("author"));
    let issued = EdtfString(
        field_str("date")
            .or_else(|| field_str("year"))
            .unwrap_or_else(|| "".to_string()),
    );

    if ty.contains("article") {
        // SerialComponent
        let parent_title = field_str("journaltitle").or_else(|| field_str("booktitle"));
        let parent = Parent::Embedded(Serial {
            title: parent_title
                .map(Title::Single)
                .unwrap_or(Title::Single(String::new())),
            r#type: SerialType::AcademicJournal,
            short_title: None,
            editor: None,
            publisher: None,
            issn: None,
        });

        InputReference::SerialComponent(Box::new(SerialComponent {
            id,
            r#type: citum_schema::reference::SerialComponentType::Article,
            title,
            author,
            translator: parse_names(field_str("translator")),
            parent,
            pages: field_str("pages"),
            volume: field_str("volume").map(citum_schema::reference::NumOrStr::Str),
            issue: field_str("number").map(citum_schema::reference::NumOrStr::Str),
            url: field_str("url").and_then(|u| u.parse().ok()),
            accessed: None,
            language: field_str("langid").or_else(|| field_str("language")),
            field_languages: Default::default(),
            issued,
            doi: field_str("doi"),
            genre: None,
            keywords: None,
            medium: None,
            note: field_str("note"),
            ads_bibcode: None,
        }))
    } else if ty.contains("incollection") || ty.contains("inbook") {
        // CollectionComponent
        let parent_title = field_str("booktitle");
        let parent_editor = parse_names(field_str("editor"));
        let parent = Parent::Embedded(citum_schema::reference::Collection {
            id: None,
            r#type: citum_schema::reference::CollectionType::EditedBook,
            title: parent_title.map(Title::Single),
            short_title: None,
            editor: parent_editor,
            translator: None,
            issued: EdtfString("".to_string()),
            publisher: None,
            collection_number: None,
            url: None,
            accessed: None,
            language: None,
            field_languages: Default::default(),
            note: None,
            isbn: None,
            keywords: None,
        });

        InputReference::CollectionComponent(Box::new(CollectionComponent {
            id,
            title,
            author,
            translator: parse_names(field_str("translator")),
            parent,
            pages: field_str("pages").map(citum_schema::reference::NumOrStr::Str),
            url: field_str("url").and_then(|u| u.parse().ok()),
            accessed: None,
            language: field_str("langid").or_else(|| field_str("language")),
            field_languages: Default::default(),
            issued,
            doi: field_str("doi"),
            genre: None,
            keywords: None,
            medium: None,
            note: field_str("note"),
            r#type: citum_schema::reference::MonographComponentType::Chapter,
        }))
    } else {
        // Monograph (default)
        if !matches!(ty.as_str(), "book" | "collection" | "mvbook") {
            eprintln!(
                "Warning: unmapped BibLaTeX type '@{}' for '{}' treated as book",
                entry.entry_type, entry.key
            );
        }

        InputReference::Monograph(Box::new(Monograph {
            id,
            r#type: MonographType::Book,
            title: title.unwrap_or(Title::Single(String::new())),
            container_title: None,
            author,
            editor: parse_names(field_str("editor")),
            translator: parse_names(field_str("translator")),
            recipient: None,
            interviewer: None,
            issued,
            publisher: field_str("publisher").map(|p| {
                Contributor::SimpleName(SimpleName {
                    name: MultilingualString::Simple(p),
                    location: None,
                })
            }),
            url: field_str("url").and_then(|u| u.parse().ok()),
            accessed: None,
            language: field_str("langid").or_else(|| field_str("language")),
            field_languages: Default::default(),
            note: field_str("note"),
            isbn: field_str("isbn"),
            doi: field_str("doi"),
            ads_bibcode: None,
            edition: field_str("edition"),
            report_number: None,
            collection_number: None,
            genre: None,
            medium: None,
            archive: None,
            archive_location: None,
            keywords: None,
            original_date: None,
            original_title: None,
        }))
    }
}

/// Convert an InputReference to CSL-JSON format.
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
    r.issued = reference.issued().map(|d| {
        let s = d.0;
        if let Some(year_str) = s.get(0..4)
            && let Ok(year) = year_str.parse::<i32>()
        {
            return DateVariable::year(year);
        }
        DateVariable {
            literal: Some(s),
            ..Default::default()
        }
    });
    r.author = reference.author().map(contributor_to_csl_names);
    r.editor = reference.editor().map(contributor_to_csl_names);
    r.translator = reference.translator().map(contributor_to_csl_names);
    r.publisher = reference.publisher().and_then(|c| c.name());

    match reference {
        InputReference::Monograph(m) => {
            r.ref_type = "book".to_string();
            r.isbn = m.isbn.clone();
            r.url = m.url.as_ref().map(|u| u.to_string());
            r.edition = m.edition.clone().map(StringOrNumber::String);
        }
        InputReference::SerialComponent(s) => {
            r.ref_type = "article-journal".to_string();
            r.container_title = match &s.parent {
                citum_schema::reference::Parent::Embedded(parent) => Some(parent.title.to_string()),
                citum_schema::reference::Parent::Id(_) => None,
            };
            r.page = s.pages.as_ref().map(|p| p.to_string());
            r.volume = s
                .volume
                .as_ref()
                .map(|v| StringOrNumber::String(v.to_string()));
            r.issue = s
                .issue
                .as_ref()
                .map(|v| StringOrNumber::String(v.to_string()));
            r.url = s.url.as_ref().map(|u| u.to_string());
        }
        InputReference::CollectionComponent(c) => {
            r.ref_type = "chapter".to_string();
            r.container_title = match &c.parent {
                citum_schema::reference::Parent::Embedded(parent) => {
                    parent.title.as_ref().map(ToString::to_string)
                }
                citum_schema::reference::Parent::Id(_) => None,
            };
            r.page = c.pages.as_ref().map(|p| p.to_string());
        }
        _ => {
            r.ref_type = "book".to_string();
        }
    }

    r
}

/// Convert a Contributor to CSL-JSON name format.
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

/// Render bibliography as BibLaTeX format.
fn render_biblatex(input: &InputBibliography) -> String {
    let mut out = String::new();
    for reference in &input.references {
        let id = reference.id().unwrap_or_else(|| "item".to_string());
        let entry_type = match reference {
            InputReference::SerialComponent(_) => "article",
            InputReference::CollectionComponent(_) => "incollection",
            _ => "book",
        };
        writeln!(&mut out, "@{}{{{},", entry_type, id).unwrap();
        if let Some(title) = reference.title() {
            writeln!(&mut out, "  title = {{{}}},", title).unwrap();
        }
        if let Some(contributor) = reference.author() {
            let names: Vec<String> = contributor_to_name_strings(contributor);
            if !names.is_empty() {
                writeln!(&mut out, "  author = {{{}}},", names.join(" and ")).unwrap();
            }
        }
        if let Some(issued) = reference.issued()
            && let Some(year) = issued.0.get(0..4)
        {
            writeln!(&mut out, "  year = {{{}}},", year).unwrap();
        }
        if let Some(doi) = reference.doi() {
            writeln!(&mut out, "  doi = {{{}}},", doi).unwrap();
        }
        writeln!(&mut out, "}}\n").unwrap();
    }
    out
}

/// Convert a Contributor to human-readable name strings.
fn contributor_to_name_strings(contributor: citum_schema::reference::Contributor) -> Vec<String> {
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
            .flat_map(contributor_to_name_strings)
            .collect(),
    }
}

/// Parse RIS format bibliography.
fn parse_ris(input: &str) -> Result<InputBibliography, Box<dyn Error>> {
    let mut references = Vec::<InputReference>::new();
    let mut current = Vec::<(String, String)>::new();
    let mut last_index: Option<usize> = None;

    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.len() < 6 {
            if let Some(i) = last_index
                && let Some((_, value)) = current.get_mut(i)
            {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(line.trim());
            }
            continue;
        }
        let tag = line[0..2].to_string();
        let separator = &line[2..6];
        if separator != "  - " {
            if let Some(i) = last_index
                && let Some((_, value)) = current.get_mut(i)
            {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(line.trim());
            }
            continue;
        }
        let value = line[6..].trim().to_string();
        if tag == "ER" {
            if !current.is_empty() {
                references.push(ris_record_to_reference(&current));
            }
            current.clear();
            last_index = None;
            continue;
        }
        current.push((tag, value));
        last_index = Some(current.len() - 1);
    }

    if !current.is_empty() {
        references.push(ris_record_to_reference(&current));
    }

    Ok(InputBibliography {
        references,
        ..Default::default()
    })
}

/// Convert a RIS record to an InputReference.
fn ris_record_to_reference(fields: &[(String, String)]) -> InputReference {
    use citum_schema::reference::{
        CollectionComponent, Monograph, Parent, Serial, SerialComponent,
    };

    // Build a HashMap once for efficient lookups
    let mut fields_map: HashMap<&str, Vec<&str>> = HashMap::new();
    for (k, v) in fields {
        fields_map.entry(k.as_str()).or_default().push(v.as_str());
    }

    let get = |tag: &str| -> Option<String> {
        fields_map
            .get(tag)
            .and_then(|v| v.first())
            .map(|s| s.to_string())
    };
    let get_all = |tag: &str| -> Vec<String> {
        fields_map
            .get(tag)
            .map(|v| v.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    };

    let id = get("ID")
        .or_else(|| get("L1"))
        .or_else(|| get("M1"))
        .map(Some)
        .unwrap_or(None);
    let title = get("TI").or_else(|| get("T1"));
    let ty = get("TY").unwrap_or_else(|| "BOOK".to_string());
    let author = {
        let authors: Vec<_> = get_all("AU")
            .into_iter()
            .map(|n| {
                let parts: Vec<_> = n.split(',').map(str::trim).collect();
                if parts.len() >= 2 {
                    Contributor::StructuredName(StructuredName {
                        family: MultilingualString::Simple(parts[0].to_string()),
                        given: MultilingualString::Simple(parts[1].to_string()),
                        suffix: None,
                        dropping_particle: None,
                        non_dropping_particle: None,
                    })
                } else {
                    Contributor::SimpleName(SimpleName {
                        name: MultilingualString::Simple(
                            parts.first().copied().unwrap_or("").to_string(),
                        ),
                        location: None,
                    })
                }
            })
            .collect();

        if authors.is_empty() {
            None
        } else if authors.len() == 1 {
            authors.into_iter().next()
        } else {
            use citum_schema::reference::ContributorList;
            Some(Contributor::ContributorList(ContributorList(authors)))
        }
    };

    let issued = get("PY")
        .or_else(|| get("Y1"))
        .map(|s| {
            let year = s.chars().take(4).collect::<String>();
            EdtfString(year)
        })
        .unwrap_or_else(|| EdtfString("".to_string()));

    let doi = get("DO");
    let note = get("N1");
    let page = match (get("SP"), get("EP")) {
        (Some(sp), Some(ep)) => Some(format!("{sp}-{ep}")),
        (Some(sp), None) => Some(sp),
        _ => None,
    };

    if ty == "JOUR" || ty == "JFULL" {
        // SerialComponent
        let parent_title = get("JO").or_else(|| get("JF"));
        let parent = Parent::Embedded(Serial {
            title: parent_title
                .map(Title::Single)
                .unwrap_or(Title::Single(String::new())),
            r#type: SerialType::AcademicJournal,
            short_title: None,
            editor: None,
            publisher: None,
            issn: None,
        });

        InputReference::SerialComponent(Box::new(SerialComponent {
            id,
            r#type: citum_schema::reference::SerialComponentType::Article,
            title: title.map(Title::Single),
            author,
            translator: None,
            parent,
            pages: page,
            volume: get("VL").map(citum_schema::reference::NumOrStr::Str),
            issue: get("IS").map(citum_schema::reference::NumOrStr::Str),
            url: get("UR").and_then(|u| u.parse().ok()),
            accessed: None,
            language: get("LA"),
            field_languages: Default::default(),
            issued,
            doi,
            genre: None,
            keywords: None,
            medium: None,
            note,
            ads_bibcode: None,
        }))
    } else if ty == "CHAP" {
        // CollectionComponent
        let parent_title = get("JO").or_else(|| get("JF"));
        let parent = Parent::Embedded(citum_schema::reference::Collection {
            id: None,
            r#type: citum_schema::reference::CollectionType::EditedBook,
            title: parent_title.map(Title::Single),
            short_title: None,
            editor: None,
            translator: None,
            issued: EdtfString("".to_string()),
            publisher: None,
            collection_number: None,
            url: None,
            accessed: None,
            language: None,
            field_languages: Default::default(),
            note: None,
            isbn: None,
            keywords: None,
        });

        InputReference::CollectionComponent(Box::new(CollectionComponent {
            id,
            title: title.map(Title::Single),
            author,
            translator: None,
            parent,
            pages: page.map(citum_schema::reference::NumOrStr::Str),
            url: get("UR").and_then(|u| u.parse().ok()),
            accessed: None,
            language: get("LA"),
            field_languages: Default::default(),
            issued,
            doi,
            genre: None,
            keywords: None,
            medium: None,
            note,
            r#type: citum_schema::reference::MonographComponentType::Chapter,
        }))
    } else {
        // Monograph (default - BOOK)
        InputReference::Monograph(Box::new(Monograph {
            id,
            r#type: MonographType::Book,
            title: title
                .map(Title::Single)
                .unwrap_or(Title::Single(String::new())),
            container_title: None,
            author,
            editor: None,
            translator: None,
            recipient: None,
            interviewer: None,
            issued,
            publisher: get("PB").map(|p| {
                Contributor::SimpleName(SimpleName {
                    name: MultilingualString::Simple(p),
                    location: None,
                })
            }),
            url: get("UR").and_then(|u| u.parse().ok()),
            accessed: None,
            language: get("LA"),
            field_languages: Default::default(),
            note,
            isbn: get("SN"),
            doi,
            ads_bibcode: None,
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
        }))
    }
}

/// Render bibliography as RIS format.
fn render_ris(input: &InputBibliography) -> String {
    let mut out = String::new();
    for reference in &input.references {
        let ty = match reference {
            InputReference::SerialComponent(_) => "JOUR",
            InputReference::CollectionComponent(_) => "CHAP",
            _ => "BOOK",
        };
        writeln!(&mut out, "TY  - {ty}").unwrap();
        if let Some(id) = reference.id() {
            writeln!(&mut out, "ID  - {id}").unwrap();
        }
        if let Some(title) = reference.title() {
            writeln!(&mut out, "TI  - {title}").unwrap();
        }
        if let Some(contributor) = reference.author() {
            for name in contributor_to_name_strings(contributor) {
                writeln!(&mut out, "AU  - {name}").unwrap();
            }
        }
        if let Some(issued) = reference.issued()
            && let Some(year) = issued.0.get(0..4)
        {
            writeln!(&mut out, "PY  - {year}").unwrap();
        }
        if let Some(doi) = reference.doi() {
            writeln!(&mut out, "DO  - {doi}").unwrap();
        }
        writeln!(&mut out, "ER  -\n").unwrap();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use citum_schema::reference::{Monograph, MonographType, Title};
    use std::path::Path;

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
    fn test_parse_ris_continuation_lines_append_to_previous_field() {
        let src = "\
TY  - JOUR
ID  - smith2020
TI  - First line of title
continued line of title
AU  - Smith, John
ER  -
";

        let parsed = parse_ris(src).expect("RIS should parse");
        assert_eq!(parsed.references.len(), 1);
        assert_eq!(
            parsed.references[0].title().map(|t| t.to_string()),
            Some("First line of title continued line of title".to_string())
        );
    }

    #[test]
    fn test_input_reference_to_csl_json_preserves_non_year_issued_as_literal() {
        let reference = InputReference::Monograph(Box::new(Monograph {
            id: Some("item-1".to_string()),
            r#type: MonographType::Book,
            title: Title::Single("Example".to_string()),
            container_title: None,
            author: None,
            editor: None,
            translator: None,
            recipient: None,
            interviewer: None,
            issued: EdtfString("undated".to_string()),
            publisher: None,
            url: None,
            accessed: None,
            language: None,
            field_languages: Default::default(),
            note: None,
            isbn: None,
            doi: None,
            ads_bibcode: None,
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
        }));

        let csl = input_reference_to_csl_json(&reference);
        let issued = csl.issued.expect("issued should be present");
        assert_eq!(issued.date_parts, None);
        assert_eq!(issued.literal, Some("undated".to_string()));
    }
}
