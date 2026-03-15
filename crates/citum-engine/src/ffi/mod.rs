/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#![allow(unsafe_code)]

//! C-FFI for the Citum processor.
//!
//! This module provides a C-compatible interface for other languages
//! (like Lua, Python, or JavaScript) to use the processor.

mod biblatex;

use crate::processor::Processor;
use crate::reference::{Bibliography, Citation, Reference};
use crate::render::djot::Djot;
use crate::render::html::Html;
use crate::render::latex::Latex;
use crate::render::plain::PlainText;
use crate::render::typst::Typst;
use citum_schema::Style;
use citum_schema::locale::Locale;
use std::cell::RefCell;
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

/// Parse a `*const c_char` to `&str`, returning null on failure.
macro_rules! parse_c_str {
    ($ptr:expr) => {
        match unsafe { CStr::from_ptr($ptr) }.to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
}

/// Parse bibliography JSON string, handling both CSL-JSON and native formats.
fn parse_bibliography_json(bib_str: &str) -> Result<Bibliography, String> {
    // Try parsing as CSL-JSON bibliography first
    match serde_json::from_str::<Vec<csl_legacy::csl_json::Reference>>(bib_str) {
        Ok(legacy_refs) => Ok(legacy_refs
            .into_iter()
            .map(|r| (r.id.clone(), Reference::from(r)))
            .collect()),
        Err(_) => serde_json::from_str(bib_str)
            .map_err(|e| format!("Bibliography JSON parse error: {}", e)),
    }
}

/// Load and parse a Citum YAML style file from disk.
fn load_style_yaml(path: &str) -> Result<Style, String> {
    let src = std::fs::read_to_string(Path::new(path))
        .map_err(|e| format!("Failed to read style YAML: {}", e))?;
    serde_yaml::from_str(&src).map_err(|e| format!("Style YAML parse error: {}", e))
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

    let style_str = parse_c_str!(style_json);
    let bib_str = parse_c_str!(bib_json);

    let style: Style = match serde_json::from_str(style_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style JSON parse error: {}", e));
            return ptr::null_mut();
        }
    };

    let bib: Bibliography = match parse_bibliography_json(bib_str) {
        Ok(b) => b,
        Err(e) => {
            set_error(e);
            return ptr::null_mut();
        }
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

    let style_str = parse_c_str!(style_json);
    let bib_str = parse_c_str!(bib_json);
    let locale_str = parse_c_str!(locale_json);

    let style: Style = match serde_json::from_str(style_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style JSON parse error: {}", e));
            return ptr::null_mut();
        }
    };

    let bib: Bibliography = match parse_bibliography_json(bib_str) {
        Ok(b) => b,
        Err(e) => {
            set_error(e);
            return ptr::null_mut();
        }
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

    let style_path_str = parse_c_str!(style_yaml_path);
    let bib_path_str = parse_c_str!(bib_yaml_path);

    let style: Style = match load_style_yaml(style_path_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
            return ptr::null_mut();
        }
    };

    let loaded = match crate::io::load_bibliography_with_sets(Path::new(bib_path_str)) {
        Ok(b) => b,
        Err(e) => {
            set_error(format!("Failed to load bibliography: {}", e));
            return ptr::null_mut();
        }
    };

    let processor = match Processor::try_with_compound_sets(
        style,
        loaded.references,
        loaded.sets.unwrap_or_default(),
    ) {
        Ok(p) => Box::new(p),
        Err(e) => {
            set_error(format!("Invalid compound sets: {}", e));
            return ptr::null_mut();
        }
    };
    Box::into_raw(processor)
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

    let style_path_str = parse_c_str!(style_yaml_path);
    let bib_path_str = parse_c_str!(bib_path);

    let style: Style = match load_style_yaml(style_path_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(e);
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
    let bibliography_parsed = match ::biblatex::Bibliography::parse(&bib_src) {
        Ok(b) => b,
        Err(e) => {
            set_error(format!("BibLaTeX parse error: {}", e));
            return ptr::null_mut();
        }
    };

    let mut bib: Bibliography = indexmap::IndexMap::new();
    for entry in bibliography_parsed.iter() {
        let key = entry.key.clone();
        let reference = self::biblatex::input_reference_from_biblatex(entry);
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
