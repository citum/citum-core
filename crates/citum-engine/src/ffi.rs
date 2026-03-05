/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#![allow(unsafe_code)]

//! C-FFI for the Citum processor.
//!
//! This module provides a C-compatible interface for other languages
//! (like Lua, Python, or JavaScript) to use the processor.

use crate::processor::Processor;
use crate::reference::{Bibliography, Citation, Reference};
use crate::render::djot::Djot;
use crate::render::html::Html;
use crate::render::latex::Latex;
use crate::render::plain::PlainText;
use crate::render::typst::Typst;
use citum_schema::Style;
use citum_schema::locale::Locale;
use citum_schema::reference::InputReference;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Set the last error message.
fn set_error(msg: String) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
}

/// Helper to safely create a C string from a Rust string, returning null if it contains null bytes.
fn safe_c_string(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c) => c.into_raw(),
        Err(e) => {
            set_error(format!("String contains null bytes: {}", e));
            ptr::null_mut()
        }
    }
}

/// Get the last error message.
///
/// # Safety
/// The returned string must be freed with `citum_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_get_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|s| safe_c_string(s.clone()))
            .unwrap_or(ptr::null_mut())
    })
}

/// Create a new processor instance from JSON strings with default English locale.
///
/// # Safety
/// The caller must ensure that `style_json` and `bib_json` are valid
/// null-terminated C strings. The returned pointer must be freed
/// with `citum_processor_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_processor_new(
    style_json: *const c_char,
    bib_json: *const c_char,
) -> *mut Processor {
    if style_json.is_null() || bib_json.is_null() {
        return ptr::null_mut();
    }

    let style_str = match unsafe { CStr::from_ptr(style_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let bib_str = match unsafe { CStr::from_ptr(bib_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let style: Style = match serde_json::from_str(style_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style JSON parse error: {}", e));
            return ptr::null_mut();
        }
    };

    // Try parsing as CSL-JSON bibliography first
    let bib: Bibliography =
        match serde_json::from_str::<Vec<csl_legacy::csl_json::Reference>>(bib_str) {
            Ok(legacy_refs) => legacy_refs
                .into_iter()
                .map(|r| (r.id.clone(), Reference::from(r)))
                .collect(),
            Err(_) => match serde_json::from_str(bib_str) {
                Ok(b) => b,
                Err(e) => {
                    set_error(format!("Bibliography JSON parse error: {}", e));
                    return ptr::null_mut();
                }
            },
        };

    let processor = Box::new(Processor::new(style, bib));
    Box::into_raw(processor)
}

/// Create a new processor instance with a specific locale.
///
/// # Safety
/// The caller must ensure all string pointers are valid null-terminated C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_processor_new_with_locale(
    style_json: *const c_char,
    bib_json: *const c_char,
    locale_json: *const c_char,
) -> *mut Processor {
    if style_json.is_null() || bib_json.is_null() || locale_json.is_null() {
        return ptr::null_mut();
    }

    let style_str = match unsafe { CStr::from_ptr(style_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let bib_str = match unsafe { CStr::from_ptr(bib_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let locale_str = match unsafe { CStr::from_ptr(locale_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let style: Style = match serde_json::from_str(style_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style JSON parse error: {}", e));
            return ptr::null_mut();
        }
    };

    let bib: Bibliography =
        match serde_json::from_str::<Vec<csl_legacy::csl_json::Reference>>(bib_str) {
            Ok(legacy_refs) => legacy_refs
                .into_iter()
                .map(|r| (r.id.clone(), Reference::from(r)))
                .collect(),
            Err(_) => match serde_json::from_str(bib_str) {
                Ok(b) => b,
                Err(e) => {
                    set_error(format!("Bibliography JSON parse error: {}", e));
                    return ptr::null_mut();
                }
            },
        };

    let locale: Locale = match serde_json::from_str(locale_str) {
        Ok(l) => l,
        Err(e) => {
            set_error(format!("Locale JSON parse error: {}", e));
            return ptr::null_mut();
        }
    };

    let processor = Box::new(Processor::with_locale(style, bib, locale));
    Box::into_raw(processor)
}

/// Create a new processor from Citum YAML files on disk (primary format).
///
/// Reads the style from `style_yaml_path` and the bibliography from
/// `bib_yaml_path`. Both are Citum YAML files.
///
/// # Safety
/// Both path pointers must be valid null-terminated UTF-8 C strings.
/// The returned pointer must be freed with `citum_processor_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_processor_new_from_yaml(
    style_yaml_path: *const c_char,
    bib_yaml_path: *const c_char,
) -> *mut Processor {
    if style_yaml_path.is_null() || bib_yaml_path.is_null() {
        return ptr::null_mut();
    }

    let style_path_str = match unsafe { CStr::from_ptr(style_yaml_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let bib_path_str = match unsafe { CStr::from_ptr(bib_yaml_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let style_src = match std::fs::read_to_string(Path::new(style_path_str)) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Failed to read style YAML: {}", e));
            return ptr::null_mut();
        }
    };
    let style: Style = match serde_yaml::from_str(&style_src) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style YAML parse error: {}", e));
            return ptr::null_mut();
        }
    };

    let bib = match crate::io::load_bibliography(Path::new(bib_path_str)) {
        Ok(b) => b,
        Err(e) => {
            set_error(format!("Failed to load bibliography: {}", e));
            return ptr::null_mut();
        }
    };

    let processor = Box::new(Processor::new(style, bib));
    Box::into_raw(processor)
}

/// Convert a biblatex entry into an InputReference.
fn input_reference_from_biblatex(entry: &biblatex::Entry) -> InputReference {
    let id = Some(entry.key.clone());
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

    use citum_schema::reference::{
        InputReference,
        contributor::{Contributor, SimpleName},
        date::EdtfString,
        types::*,
    };
    use url::Url;

    let title = field_str("title")
        .map(Title::Single)
        .unwrap_or(Title::Single(String::new()));
    let issued = field_str("date")
        .map(EdtfString)
        .unwrap_or(EdtfString(String::new()));
    let publisher = field_str("publisher").map(|p| {
        Contributor::SimpleName(SimpleName {
            name: p.into(),
            location: field_str("location"),
        })
    });

    let author = entry
        .author()
        .ok()
        .map(|p| contributors_from_biblatex_persons(&p));
    let editor = entry.editors().ok().map(|e| {
        let all_persons: Vec<biblatex::Person> =
            e.into_iter().flat_map(|(persons, _)| persons).collect();
        contributors_from_biblatex_persons(&all_persons)
    });

    let language = field_str("langid").or_else(|| field_str("language"));

    match entry.entry_type.to_string().to_lowercase().as_str() {
        "book" | "mvbook" | "collection" | "mvcollection" | "manual" => {
            InputReference::Monograph(Box::new(Monograph {
                id,
                r#type: if matches!(
                    entry.entry_type.to_string().to_lowercase().as_str(),
                    "manual"
                ) {
                    MonographType::Manual
                } else {
                    MonographType::Book
                },
                title,
                author,
                editor,
                translator: None,
                issued,
                publisher,
                url: field_str("url").and_then(|u| Url::parse(&u).ok()),
                accessed: None,
                language,
                field_languages: HashMap::new(),
                note: field_str("note"),
                isbn: field_str("isbn"),
                doi: field_str("doi"),
                ads_bibcode: field_str("bibcode"),
                edition: field_str("edition"),
                report_number: if matches!(
                    entry.entry_type.to_string().to_lowercase().as_str(),
                    "report"
                ) {
                    field_str("number")
                } else {
                    None
                },
                collection_number: if !matches!(
                    entry.entry_type.to_string().to_lowercase().as_str(),
                    "report"
                ) {
                    field_str("number")
                } else {
                    None
                },
                genre: field_str("type"),
                medium: None,
                keywords: None,
                original_date: None,
                original_title: None,
            }))
        }
        "inbook" | "incollection" | "inproceedings" => {
            let parent_title = field_str("booktitle")
                .map(Title::Single)
                .unwrap_or(Title::Single(String::new()));
            InputReference::CollectionComponent(Box::new(CollectionComponent {
                id,
                r#type: MonographComponentType::Chapter,
                title: Some(title),
                author,
                translator: None,
                issued,
                parent: Parent::Embedded(Collection {
                    id: None,
                    r#type: CollectionType::EditedBook,
                    title: Some(parent_title),
                    short_title: None,
                    editor,
                    translator: None,
                    issued: EdtfString(String::new()),
                    publisher,
                    collection_number: field_str("number"),
                    url: None,
                    accessed: None,
                    language: None,
                    field_languages: HashMap::new(),
                    note: None,
                    isbn: None,
                    keywords: None,
                }),
                pages: field_str("pages").map(NumOrStr::Str),
                url: field_str("url").and_then(|u| Url::parse(&u).ok()),
                accessed: field_str("urldate").map(EdtfString),
                language,
                field_languages: HashMap::new(),
                note: field_str("note"),
                doi: field_str("doi"),
                genre: field_str("type"),
                medium: None,
                keywords: None,
            }))
        }
        "article" => {
            let parent_title = field_str("journaltitle")
                .or_else(|| field_str("journal"))
                .map(Title::Single)
                .unwrap_or(Title::Single(String::new()));
            InputReference::SerialComponent(Box::new(SerialComponent {
                id,
                r#type: SerialComponentType::Article,
                title: Some(title),
                author,
                translator: None,
                issued,
                parent: Parent::Embedded(Serial {
                    r#type: SerialType::AcademicJournal,
                    title: parent_title,
                    short_title: None,
                    editor: None,
                    publisher: None,
                    issn: field_str("issn"),
                }),
                url: field_str("url").and_then(|u| Url::parse(&u).ok()),
                accessed: field_str("urldate").map(EdtfString),
                language,
                field_languages: HashMap::new(),
                note: field_str("note"),
                doi: field_str("doi"),
                ads_bibcode: field_str("bibcode"),
                pages: field_str("pages"),
                volume: field_str("volume").map(NumOrStr::Str),
                issue: field_str("number").map(NumOrStr::Str),
                genre: field_str("type"),
                medium: None,
                keywords: None,
            }))
        }
        _ => InputReference::Monograph(Box::new(Monograph {
            id,
            r#type: MonographType::Document,
            title,
            author,
            editor,
            translator: None,
            issued,
            publisher,
            url: field_str("url").and_then(|u| Url::parse(&u).ok()),
            accessed: field_str("urldate").map(EdtfString),
            language,
            field_languages: HashMap::new(),
            note: field_str("note"),
            isbn: field_str("isbn"),
            doi: field_str("doi"),
            ads_bibcode: field_str("bibcode"),
            edition: field_str("edition"),
            report_number: if matches!(
                entry.entry_type.to_string().to_lowercase().as_str(),
                "report"
            ) {
                field_str("number")
            } else {
                None
            },
            collection_number: if !matches!(
                entry.entry_type.to_string().to_lowercase().as_str(),
                "report"
            ) {
                field_str("number")
            } else {
                None
            },
            genre: field_str("type"),
            medium: None,
            keywords: None,
            original_date: None,
            original_title: None,
        })),
    }
}

fn contributors_from_biblatex_persons(
    persons: &[biblatex::Person],
) -> citum_schema::reference::contributor::Contributor {
    use citum_schema::reference::contributor::{Contributor, ContributorList, StructuredName};
    let contributors: Vec<Contributor> = persons
        .iter()
        .map(|p| {
            Contributor::StructuredName(StructuredName {
                given: p.given_name.clone().into(),
                family: p.name.clone().into(),
                suffix: if p.suffix.is_empty() {
                    None
                } else {
                    Some(p.suffix.clone())
                },
                dropping_particle: None,
                non_dropping_particle: if p.prefix.is_empty() {
                    None
                } else {
                    Some(p.prefix.clone())
                },
            })
        })
        .collect();
    Contributor::ContributorList(ContributorList(contributors))
}

/// Create a new processor from a Citum YAML style and a biblatex `.bib` file.
///
/// Reads the style from `style_yaml_path` (Citum YAML) and the bibliography
/// from `bib_path` (biblatex `.bib`). Entries are converted via
/// `input_reference_from_biblatex`.
///
/// # Safety
/// Both path pointers must be valid null-terminated UTF-8 C strings.
/// The returned pointer must be freed with `citum_processor_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_processor_new_from_bib(
    style_yaml_path: *const c_char,
    bib_path: *const c_char,
) -> *mut Processor {
    if style_yaml_path.is_null() || bib_path.is_null() {
        return ptr::null_mut();
    }

    let style_path_str = match unsafe { CStr::from_ptr(style_yaml_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let bib_path_str = match unsafe { CStr::from_ptr(bib_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let style_src = match std::fs::read_to_string(Path::new(style_path_str)) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Failed to read style YAML: {}", e));
            return ptr::null_mut();
        }
    };
    let style: Style = match serde_yaml::from_str(&style_src) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style YAML parse error: {}", e));
            return ptr::null_mut();
        }
    };

    let bib_src = match std::fs::read_to_string(Path::new(bib_path_str)) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Failed to read bibliography: {}", e));
            return ptr::null_mut();
        }
    };
    let bibliography_parsed = match biblatex::Bibliography::parse(&bib_src) {
        Ok(b) => b,
        Err(e) => {
            set_error(format!("BibLaTeX parse error: {}", e));
            return ptr::null_mut();
        }
    };

    let mut bib: Bibliography = indexmap::IndexMap::new();
    for entry in bibliography_parsed.iter() {
        let key = entry.key.clone();
        let reference = input_reference_from_biblatex(entry);
        bib.insert(key, reference);
    }

    let processor = Box::new(Processor::new(style, bib));
    Box::into_raw(processor)
}

/// Free a processor instance.
///
/// # Safety
/// The pointer must have been created by a `citum_processor_new` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_processor_free(processor: *mut Processor) {
    if !processor.is_null() {
        let _ = unsafe { Box::from_raw(processor) };
    }
}

/// Helper to render a citation to a string using a specific format.
unsafe fn render_citation<F>(processor: *mut Processor, cite_json: *const c_char) -> *mut c_char
where
    F: crate::render::format::OutputFormat<Output = String>,
{
    if processor.is_null() || cite_json.is_null() {
        return ptr::null_mut();
    }

    let processor = unsafe { &*processor };
    let cite_str = match unsafe { CStr::from_ptr(cite_json) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Invalid UTF-8 in citation JSON: {}", e));
            return ptr::null_mut();
        }
    };

    let citation: Citation = match serde_json::from_str(cite_str) {
        Ok(c) => c,
        Err(e) => {
            set_error(format!("Citation JSON parse error: {}", e));
            return ptr::null_mut();
        }
    };

    match processor.process_citation_with_format::<F>(&citation) {
        Ok(rendered) => safe_c_string(rendered),
        Err(e) => {
            set_error(format!("Rendering error: {}", e));
            ptr::null_mut()
        }
    }
}

/// Render a citation to a LaTeX string.
///
/// # Safety
/// The caller must ensure that `processor` is a valid pointer and
/// `cite_json` is a valid null-terminated C string. The returned
/// string must be freed with `citum_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_citation_latex(
    processor: *mut Processor,
    cite_json: *const c_char,
) -> *mut c_char {
    unsafe { render_citation::<Latex>(processor, cite_json) }
}

/// Render a citation to an HTML string.
///
/// # Safety
/// The caller must ensure that `processor` is a valid pointer and
/// `cite_json` is a valid null-terminated C string. The returned
/// string must be freed with `citum_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_citation_html(
    processor: *mut Processor,
    cite_json: *const c_char,
) -> *mut c_char {
    unsafe { render_citation::<Html>(processor, cite_json) }
}

/// Render a citation to a Plain Text string.
///
/// # Safety
/// The caller must ensure that `processor` is a valid pointer and
/// `cite_json` is a valid null-terminated C string. The returned
/// string must be freed with `citum_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_citation_plain(
    processor: *mut Processor,
    cite_json: *const c_char,
) -> *mut c_char {
    unsafe { render_citation::<PlainText>(processor, cite_json) }
}

/// Render a citation to a Djot string.
///
/// # Safety
/// See `citum_render_citation_html`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_citation_djot(
    processor: *mut Processor,
    cite_json: *const c_char,
) -> *mut c_char {
    unsafe { render_citation::<Djot>(processor, cite_json) }
}

/// Render a citation to a Typst string.
///
/// # Safety
/// See `citum_render_citation_html`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_citation_typst(
    processor: *mut Processor,
    cite_json: *const c_char,
) -> *mut c_char {
    unsafe { render_citation::<Typst>(processor, cite_json) }
}

/// Helper to render the bibliography to a string using a specific format.
unsafe fn render_bibliography<F>(processor: *mut Processor) -> *mut c_char
where
    F: crate::render::format::OutputFormat<Output = String>,
{
    if processor.is_null() {
        return ptr::null_mut();
    }

    let processor = unsafe { &*processor };
    let rendered = processor.render_bibliography_with_format::<F>();
    safe_c_string(rendered)
}

/// Render the bibliography to a LaTeX string.
///
/// # Safety
/// The caller must ensure that `processor` is a valid pointer.
/// The returned string must be freed with `citum_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_bibliography_latex(processor: *mut Processor) -> *mut c_char {
    unsafe { render_bibliography::<Latex>(processor) }
}

/// Render the bibliography to an HTML string.
///
/// # Safety
/// The caller must ensure that `processor` is a valid pointer.
/// The returned string must be freed with `citum_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_bibliography_html(processor: *mut Processor) -> *mut c_char {
    unsafe { render_bibliography::<Html>(processor) }
}

/// Render the bibliography to a Plain Text string.
///
/// # Safety
/// The caller must ensure that `processor` is a valid pointer.
/// The returned string must be freed with `citum_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_bibliography_plain(processor: *mut Processor) -> *mut c_char {
    unsafe { render_bibliography::<PlainText>(processor) }
}

/// Render the bibliography to a Djot string.
///
/// # Safety
/// See `citum_render_bibliography_html`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_bibliography_djot(processor: *mut Processor) -> *mut c_char {
    unsafe { render_bibliography::<Djot>(processor) }
}

/// Render the bibliography to a Typst string.
///
/// # Safety
/// See `citum_render_bibliography_html`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_bibliography_typst(processor: *mut Processor) -> *mut c_char {
    unsafe { render_bibliography::<Typst>(processor) }
}

/// Helper to render the grouped bibliography to a string using a specific format.
unsafe fn render_grouped_bibliography<F>(processor: *mut Processor) -> *mut c_char
where
    F: crate::render::format::OutputFormat<Output = String>,
{
    if processor.is_null() {
        return ptr::null_mut();
    }

    let processor = unsafe { &*processor };
    let rendered = processor.render_grouped_bibliography_with_format::<F>();
    safe_c_string(rendered)
}

/// Render the grouped bibliography to an HTML string.
///
/// # Safety
/// See `citum_render_bibliography_html`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_bibliography_grouped_html(
    processor: *mut Processor,
) -> *mut c_char {
    unsafe { render_grouped_bibliography::<Html>(processor) }
}

/// Render the grouped bibliography to a Plain Text string.
///
/// # Safety
/// See `citum_render_bibliography_html`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_bibliography_grouped_plain(
    processor: *mut Processor,
) -> *mut c_char {
    unsafe { render_grouped_bibliography::<PlainText>(processor) }
}

/// Render multiple citations in batch to a JSON array of strings.
///
/// # Safety
/// `citations_json` must be a null-terminated JSON array of `Citation` objects.
/// The returned JSON string must be freed with `citum_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_render_citations_json(
    processor: *mut Processor,
    citations_json: *const c_char,
    format: *const c_char,
) -> *mut c_char {
    if processor.is_null() || citations_json.is_null() || format.is_null() {
        return ptr::null_mut();
    }

    let processor = unsafe { &*processor };
    let citations_str = match unsafe { CStr::from_ptr(citations_json) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Invalid UTF-8 in citations JSON: {}", e));
            return ptr::null_mut();
        }
    };

    let citations: Vec<Citation> = match serde_json::from_str(citations_str) {
        Ok(c) => c,
        Err(e) => {
            set_error(format!("Citations JSON parse error: {}", e));
            return ptr::null_mut();
        }
    };

    let format_str = unsafe { CStr::from_ptr(format) }
        .to_str()
        .unwrap_or("plain");

    let result = match format_str {
        "html" => processor.process_citations_with_format::<Html>(&citations),
        "latex" => processor.process_citations_with_format::<Latex>(&citations),
        "djot" => processor.process_citations_with_format::<Djot>(&citations),
        "typst" => processor.process_citations_with_format::<Typst>(&citations),
        _ => processor.process_citations_with_format::<PlainText>(&citations),
    };

    match result {
        Ok(rendered) => match serde_json::to_string(&rendered) {
            Ok(json) => safe_c_string(json),
            Err(e) => {
                set_error(format!("Failed to serialize result: {}", e));
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_error(format!("Batch rendering error: {}", e));
            ptr::null_mut()
        }
    }
}

/// Free a string allocated by the processor.
///
/// # Safety
/// The pointer must have been returned by one of the rendering functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_string_free(s: *mut c_char) {
    if !s.is_null() {
        let _ = unsafe { CString::from_raw(s) };
    }
}

/// Get the version of the Citum engine.
///
/// # Safety
/// The returned string must be freed with `citum_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn citum_version() -> *mut c_char {
    safe_c_string(env!("CARGO_PKG_VERSION").to_string())
}
