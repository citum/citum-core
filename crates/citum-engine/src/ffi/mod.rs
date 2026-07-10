/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! C-FFI for the Citum processor.
//!
//! This module provides a C-compatible interface for other languages
//! (like Lua, Python, or JavaScript) to use the processor.
//!
//! Exports use `#[cfg_attr(not(test), unsafe(no_mangle))]`: the lib-test
//! binary links `citum-engine` twice (once compiled for tests, once as an
//! rlib behind the `citum-io` dev-dependency), and two copies of an
//! unmangled symbol are a duplicate-symbol link error under lld when the
//! `ffi` feature is enabled. Mangling the test-build copies keeps
//! `cargo test --all-features` linkable while the in-module unit tests
//! still call the functions by their Rust names.

#![allow(unsafe_code, reason = "FFI interface")]

use crate::processor::{Processor, RunState};
use crate::reference::{Bibliography, Citation, Reference};
use crate::render::djot::Djot;
use crate::render::html::Html;
use crate::render::latex::Latex;
use crate::render::markdown::Markdown;
use crate::render::plain::PlainText;
use crate::render::typst::Typst;
use citum_schema::Style;
use citum_schema::locale::Locale;
use citum_schema::reference::InputReference;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Opaque per-handle FFI session: an owned [`Processor`] plus its current
/// render-run state.
///
/// `pub` only so the `pub extern "C"` functions below may name it in their
/// signatures (Rust's private-interface lint); its fields stay private, so
/// it should be treated as an opaque handle by every caller exactly as
/// `Processor` was before this type existed, even though `citum_engine::ffi`
/// is itself a public module. This replaces the implicit per-`Processor`
/// mutable state with an explicit [`RunState`] per
/// docs/specs/EXPLICIT_RENDER_RUN_STATE.md.
/// Citation-rendering calls (`citum_render_citation_*`) register into `run`
/// via `&mut`; bibliography calls (`citum_render_bibliography_*`) render
/// from a cloned, finalized snapshot of `run` so accumulation across calls
/// on one handle is preserved without pausing further citation registration.
pub struct FfiSession {
    processor: Processor,
    run: RunState,
}

impl FfiSession {
    fn new(processor: Processor) -> Self {
        let run = processor.begin_run();
        Self { processor, run }
    }
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
            set_error(format!("String contains null bytes: {e}"));
            ptr::null_mut()
        }
    }
}

unsafe fn parse_c_str<'a>(ptr: *const c_char, label: &str) -> Result<&'a str, ()> {
    if ptr.is_null() {
        set_error(format!("{label} pointer is null"));
        return Err(());
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().map_err(|err| {
        set_error(format!("Invalid UTF-8 in {label}: {err}"));
    })
}

fn parse_output_format(format: &str) -> Result<&'static str, ()> {
    match format {
        "html" => Ok("html"),
        "latex" => Ok("latex"),
        "djot" => Ok("djot"),
        "typst" => Ok("typst"),
        "plain" => Ok("plain"),
        "markdown" => Ok("markdown"),
        other => {
            set_error(format!("Unsupported output format: {other}"));
            Err(())
        }
    }
}

/// Parse bibliography JSON string, handling both CSL-JSON and native formats.
fn parse_bibliography_json(bib_str: &str) -> Result<Bibliography, String> {
    // Try parsing as CSL-JSON bibliography first
    match serde_json::from_str::<Vec<csl_legacy::csl_json::Reference>>(bib_str) {
        Ok(legacy_refs) => Ok(legacy_refs
            .into_iter()
            .map(|r| (r.id.clone(), Reference::from(r)))
            .collect()),
        Err(_) => {
            serde_json::from_str(bib_str).map_err(|e| format!("Bibliography JSON parse error: {e}"))
        }
    }
}

/// Parse bibliography YAML string into a `Bibliography`.
///
/// Supports Citum YAML (`InputBibliography` with `references:` field),
/// a flat `IndexMap<id, Reference>`, and a `Vec<InputReference>`.
fn parse_bibliography_yaml(bib_str: &str) -> Result<Bibliography, String> {
    // Try Citum native YAML: { references: [...], ... }
    // Capture the error here so we can report it if all attempts fail.
    let native_err = match serde_yaml::from_str::<citum_schema::InputBibliography>(bib_str) {
        Ok(input_bib) => {
            let bib: Bibliography = input_bib
                .references
                .into_iter()
                .filter_map(|r| r.id().map(|id| (id.to_string(), r)))
                .collect();
            return Ok(bib);
        }
        Err(e) => e,
    };

    // Try flat IndexMap<String, InputReference>.
    // Map key is authoritative: always assign it as the reference id so the
    // map key and stored id are never out of sync.
    if let Ok(map) = serde_yaml::from_str::<indexmap::IndexMap<String, InputReference>>(bib_str) {
        let bib: Bibliography = map
            .into_iter()
            .map(|(key, mut r)| {
                r.set_id(key.clone());
                (key, r)
            })
            .collect();
        return Ok(bib);
    }

    // Try Vec<InputReference>
    if let Ok(refs) = serde_yaml::from_str::<Vec<InputReference>>(bib_str) {
        let bib: Bibliography = refs
            .into_iter()
            .filter_map(|r| r.id().map(|id| (id.to_string(), r)))
            .collect();
        return Ok(bib);
    }

    Err(format!(
        "Bibliography YAML parse error (tried InputBibliography, flat map, and Vec): {native_err}"
    ))
}

/// Get the last error message.
///
/// # Safety
/// The returned string must be freed with `citum_string_free`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_get_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map_or(ptr::null_mut(), |s| safe_c_string(s.clone()))
    })
}

/// Create a new processor instance from JSON strings with default English locale.
///
/// # Safety
/// The caller must ensure that `style_json` and `bib_json` are valid
/// null-terminated C strings. The returned pointer must be freed
/// with `citum_processor_free`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_processor_new(
    style_json: *const c_char,
    bib_json: *const c_char,
) -> *mut FfiSession {
    let Ok(style_str) = (unsafe { parse_c_str(style_json, "style_json") }) else {
        return ptr::null_mut();
    };
    let Ok(bib_str) = (unsafe { parse_c_str(bib_json, "bib_json") }) else {
        return ptr::null_mut();
    };

    let style: Style = match serde_json::from_str(style_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style JSON parse error: {e}"));
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

    let session = Box::new(FfiSession::new(Processor::new(style, bib)));
    Box::into_raw(session)
}

/// Create a new processor instance with a specific locale.
///
/// # Safety
/// The caller must ensure all string pointers are valid null-terminated C strings.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_processor_new_with_locale(
    style_json: *const c_char,
    bib_json: *const c_char,
    locale_json: *const c_char,
) -> *mut FfiSession {
    let Ok(style_str) = (unsafe { parse_c_str(style_json, "style_json") }) else {
        return ptr::null_mut();
    };
    let Ok(bib_str) = (unsafe { parse_c_str(bib_json, "bib_json") }) else {
        return ptr::null_mut();
    };
    let Ok(locale_str) = (unsafe { parse_c_str(locale_json, "locale_json") }) else {
        return ptr::null_mut();
    };

    let style: Style = match serde_json::from_str(style_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style JSON parse error: {e}"));
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
            set_error(format!("Locale JSON parse error: {e}"));
            return ptr::null_mut();
        }
    };

    let session = Box::new(FfiSession::new(Processor::with_locale(style, bib, locale)));
    Box::into_raw(session)
}

/// Create a new processor instance from YAML strings with default English locale.
///
/// # Safety
/// The caller must ensure that `style_yaml` and `bib_yaml` are valid
/// null-terminated C strings. The returned pointer must be freed
/// with `citum_processor_free`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_processor_new_from_yaml(
    style_yaml: *const c_char,
    bib_yaml: *const c_char,
) -> *mut FfiSession {
    let Ok(style_str) = (unsafe { parse_c_str(style_yaml, "style_yaml") }) else {
        return ptr::null_mut();
    };
    let Ok(bib_str) = (unsafe { parse_c_str(bib_yaml, "bib_yaml") }) else {
        return ptr::null_mut();
    };

    let style: Style = match Style::from_yaml_str(style_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style YAML parse error: {e}"));
            return ptr::null_mut();
        }
    };

    let bib: Bibliography = match parse_bibliography_yaml(bib_str) {
        Ok(b) => b,
        Err(e) => {
            set_error(e);
            return ptr::null_mut();
        }
    };

    let session = Box::new(FfiSession::new(Processor::new(style, bib)));
    Box::into_raw(session)
}

/// Create a new processor instance with a specific locale from YAML strings.
///
/// # Safety
/// The caller must ensure all string pointers are valid null-terminated C strings.
/// The returned pointer must be freed with `citum_processor_free`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_processor_new_with_locale_from_yaml(
    style_yaml: *const c_char,
    bib_yaml: *const c_char,
    locale_yaml: *const c_char,
) -> *mut FfiSession {
    let Ok(style_str) = (unsafe { parse_c_str(style_yaml, "style_yaml") }) else {
        return ptr::null_mut();
    };
    let Ok(bib_str) = (unsafe { parse_c_str(bib_yaml, "bib_yaml") }) else {
        return ptr::null_mut();
    };
    let Ok(locale_str) = (unsafe { parse_c_str(locale_yaml, "locale_yaml") }) else {
        return ptr::null_mut();
    };

    let style: Style = match Style::from_yaml_str(style_str) {
        Ok(s) => s,
        Err(e) => {
            set_error(format!("Style YAML parse error: {e}"));
            return ptr::null_mut();
        }
    };

    let bib: Bibliography = match parse_bibliography_yaml(bib_str) {
        Ok(b) => b,
        Err(e) => {
            set_error(e);
            return ptr::null_mut();
        }
    };

    let locale: Locale = match Locale::from_yaml_str(locale_str) {
        Ok(l) => l,
        Err(e) => {
            set_error(format!("Locale YAML parse error: {e}"));
            return ptr::null_mut();
        }
    };

    let session = Box::new(FfiSession::new(Processor::with_locale(style, bib, locale)));
    Box::into_raw(session)
}

/// Free a processor instance.
///
/// # Safety
/// The pointer must have been created by a `citum_processor_new` function.
/// Passing the same pointer more than once, or passing a pointer allocated by
/// any other API, is undefined behavior.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_processor_free(processor: *mut FfiSession) {
    if !processor.is_null() {
        let _ = unsafe { Box::from_raw(processor) };
    }
}

/// Helper to render a citation to a string using a specific format.
unsafe fn render_citation<F>(processor: *mut FfiSession, cite_json: *const c_char) -> *mut c_char
where
    F: crate::render::format::OutputFormat<Output = String>,
{
    if processor.is_null() {
        set_error("processor pointer is null".to_string());
        return ptr::null_mut();
    }

    let session = unsafe { &mut *processor };
    let Ok(cite_str) = (unsafe { parse_c_str(cite_json, "cite_json") }) else {
        return ptr::null_mut();
    };

    let citation: Citation = match serde_json::from_str(cite_str) {
        Ok(c) => c,
        Err(e) => {
            set_error(format!("Citation JSON parse error: {e}"));
            return ptr::null_mut();
        }
    };

    match session
        .processor
        .process_citation_with_format::<F>(&citation, &mut session.run)
    {
        Ok(rendered) => safe_c_string(rendered),
        Err(e) => {
            set_error(format!("Rendering error: {e}"));
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
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_citation_latex(
    processor: *mut FfiSession,
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
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_citation_html(
    processor: *mut FfiSession,
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
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_citation_plain(
    processor: *mut FfiSession,
    cite_json: *const c_char,
) -> *mut c_char {
    unsafe { render_citation::<PlainText>(processor, cite_json) }
}

/// Render a citation to a Djot string.
///
/// # Safety
/// See `citum_render_citation_html`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_citation_djot(
    processor: *mut FfiSession,
    cite_json: *const c_char,
) -> *mut c_char {
    unsafe { render_citation::<Djot>(processor, cite_json) }
}

/// Render a citation to a Typst string.
///
/// # Safety
/// See `citum_render_citation_html`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_citation_typst(
    processor: *mut FfiSession,
    cite_json: *const c_char,
) -> *mut c_char {
    unsafe { render_citation::<Typst>(processor, cite_json) }
}

/// Helper to render the bibliography to a string using a specific format.
///
/// Renders from a cloned, finalized snapshot of the session's current run so
/// citation registration on the original `session.run` is unaffected — the
/// handle can keep accumulating citations after this call.
unsafe fn render_bibliography<F>(processor: *mut FfiSession) -> *mut c_char
where
    F: crate::render::format::OutputFormat<Output = String>,
{
    if processor.is_null() {
        set_error("processor pointer is null".to_string());
        return ptr::null_mut();
    }

    let session = unsafe { &*processor };
    let run = session.run.clone().finalize();
    let rendered = session.processor.render_bibliography_with_format::<F>(&run);
    safe_c_string(rendered)
}

/// Render the bibliography to a LaTeX string.
///
/// # Safety
/// The caller must ensure that `processor` is a valid pointer.
/// The returned string must be freed with `citum_string_free`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_bibliography_latex(
    processor: *mut FfiSession,
) -> *mut c_char {
    unsafe { render_bibliography::<Latex>(processor) }
}

/// Render the bibliography to an HTML string.
///
/// # Safety
/// The caller must ensure that `processor` is a valid pointer.
/// The returned string must be freed with `citum_string_free`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_bibliography_html(processor: *mut FfiSession) -> *mut c_char {
    unsafe { render_bibliography::<Html>(processor) }
}

/// Render the bibliography to a Plain Text string.
///
/// # Safety
/// The caller must ensure that `processor` is a valid pointer.
/// The returned string must be freed with `citum_string_free`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_bibliography_plain(
    processor: *mut FfiSession,
) -> *mut c_char {
    unsafe { render_bibliography::<PlainText>(processor) }
}

/// Render the bibliography to a Djot string.
///
/// # Safety
/// See `citum_render_bibliography_html`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_bibliography_djot(processor: *mut FfiSession) -> *mut c_char {
    unsafe { render_bibliography::<Djot>(processor) }
}

/// Render the bibliography to a Typst string.
///
/// # Safety
/// See `citum_render_bibliography_html`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_bibliography_typst(
    processor: *mut FfiSession,
) -> *mut c_char {
    unsafe { render_bibliography::<Typst>(processor) }
}

/// Helper to render the grouped bibliography to a string using a specific format.
///
/// See `render_bibliography` for why this clones-and-finalizes a snapshot
/// rather than consuming the session's own run.
unsafe fn render_grouped_bibliography<F>(processor: *mut FfiSession) -> *mut c_char
where
    F: crate::render::format::OutputFormat<Output = String>,
{
    if processor.is_null() {
        set_error("processor pointer is null".to_string());
        return ptr::null_mut();
    }

    let session = unsafe { &*processor };
    let run = session.run.clone().finalize();
    let rendered = session
        .processor
        .render_grouped_bibliography_with_format::<F>(&run);
    safe_c_string(rendered)
}

/// Render the grouped bibliography to an HTML string.
///
/// # Safety
/// See `citum_render_bibliography_html`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_bibliography_grouped_html(
    processor: *mut FfiSession,
) -> *mut c_char {
    unsafe { render_grouped_bibliography::<Html>(processor) }
}

/// Render the grouped bibliography to a Plain Text string.
///
/// # Safety
/// See `citum_render_bibliography_html`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_bibliography_grouped_plain(
    processor: *mut FfiSession,
) -> *mut c_char {
    unsafe { render_grouped_bibliography::<PlainText>(processor) }
}

/// Render multiple citations in batch to a JSON array of strings.
///
/// # Safety
/// `citations_json` must be a null-terminated JSON array of `Citation` objects.
/// The returned JSON string must be freed with `citum_string_free`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_render_citations_json(
    processor: *mut FfiSession,
    citations_json: *const c_char,
    format: *const c_char,
) -> *mut c_char {
    if processor.is_null() {
        set_error("processor pointer is null".to_string());
        return ptr::null_mut();
    }

    let session = unsafe { &mut *processor };
    let Ok(citations_str) = (unsafe { parse_c_str(citations_json, "citations_json") }) else {
        return ptr::null_mut();
    };

    let citations: Vec<Citation> = match serde_json::from_str(citations_str) {
        Ok(c) => c,
        Err(e) => {
            set_error(format!("Citations JSON parse error: {e}"));
            return ptr::null_mut();
        }
    };

    let Ok(format_str) = (unsafe { parse_c_str(format, "format") }).and_then(parse_output_format)
    else {
        return ptr::null_mut();
    };

    let processor = &session.processor;
    let run = &mut session.run;
    let result = match format_str {
        "html" => processor.process_citations_with_format::<Html>(&citations, run),
        "latex" => processor.process_citations_with_format::<Latex>(&citations, run),
        "djot" => processor.process_citations_with_format::<Djot>(&citations, run),
        "typst" => processor.process_citations_with_format::<Typst>(&citations, run),
        "markdown" => processor.process_citations_with_format::<Markdown>(&citations, run),
        _ => processor.process_citations_with_format::<PlainText>(&citations, run),
    };

    match result {
        Ok(rendered) => match serde_json::to_string(&rendered) {
            Ok(json) => safe_c_string(json),
            Err(e) => {
                set_error(format!("Failed to serialize result: {e}"));
                ptr::null_mut()
            }
        },
        Err(e) => {
            set_error(format!("Batch rendering error: {e}"));
            ptr::null_mut()
        }
    }
}

/// Free a string allocated by the processor.
///
/// # Safety
/// The pointer must have been returned by one of the rendering functions.
/// Passing the same pointer more than once, or passing a pointer allocated by
/// any other API, is undefined behavior.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_string_free(s: *mut c_char) {
    if !s.is_null() {
        let _ = unsafe { CString::from_raw(s) };
    }
}

/// Get the version of the Citum engine.
///
/// # Safety
/// The returned string must be freed with `citum_string_free`.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn citum_version() -> *mut c_char {
    safe_c_string(env!("CARGO_PKG_VERSION").to_string())
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

    fn c_string(value: &str) -> CString {
        CString::new(value).expect("test string has no interior NUL")
    }

    fn processor() -> *mut FfiSession {
        let style = serde_json::to_string(&Style::default()).expect("style serializes");
        let bibliography = "{}";
        unsafe { citum_processor_new(c_string(&style).as_ptr(), c_string(bibliography).as_ptr()) }
    }

    fn last_error() -> String {
        let ptr = unsafe { citum_get_last_error() };
        assert!(!ptr.is_null(), "last error should be set");
        let error = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("error is UTF-8")
            .to_string();
        unsafe { citum_string_free(ptr) };
        error
    }

    #[test]
    fn processor_new_rejects_null_style_pointer() {
        let bibliography = c_string("{}");
        let processor = unsafe { citum_processor_new(ptr::null(), bibliography.as_ptr()) };
        assert!(processor.is_null());
        assert!(last_error().contains("style_json pointer is null"));
    }

    #[test]
    fn processor_new_rejects_invalid_utf8() {
        let invalid = [0xff, 0x00];
        let bibliography = c_string("{}");
        let processor = unsafe {
            citum_processor_new(invalid.as_ptr().cast::<c_char>(), bibliography.as_ptr())
        };
        assert!(processor.is_null());
        assert!(last_error().contains("Invalid UTF-8 in style_json"));
    }

    #[test]
    fn processor_new_rejects_invalid_json() {
        let style = c_string("{");
        let bibliography = c_string("{}");
        let processor = unsafe { citum_processor_new(style.as_ptr(), bibliography.as_ptr()) };
        assert!(processor.is_null());
        assert!(last_error().contains("Style JSON parse error"));
    }

    #[test]
    fn render_citation_rejects_null_processor() {
        let citation = c_string("{}");
        let rendered = unsafe { citum_render_citation_plain(ptr::null_mut(), citation.as_ptr()) };
        assert!(rendered.is_null());
        assert!(last_error().contains("processor pointer is null"));
    }

    #[test]
    fn render_citation_rejects_null_citation_pointer() {
        let processor = processor();
        assert!(!processor.is_null());
        let rendered = unsafe { citum_render_citation_plain(processor, ptr::null()) };
        assert!(rendered.is_null());
        assert!(last_error().contains("cite_json pointer is null"));
        unsafe { citum_processor_free(processor) };
    }

    #[test]
    fn batch_render_rejects_invalid_format() {
        let processor = processor();
        assert!(!processor.is_null());
        let citations = c_string("[]");
        let format = c_string("bogus");
        let rendered =
            unsafe { citum_render_citations_json(processor, citations.as_ptr(), format.as_ptr()) };
        assert!(rendered.is_null());
        assert!(last_error().contains("Unsupported output format"));
        unsafe { citum_processor_free(processor) };
    }

    #[test]
    fn processor_new_from_yaml_returns_valid_pointer() {
        let style = serde_yaml::to_string(&Style::default()).expect("style serializes");
        let bib = "references: []";
        let processor = unsafe {
            citum_processor_new_from_yaml(c_string(&style).as_ptr(), c_string(bib).as_ptr())
        };
        assert!(!processor.is_null());
        unsafe { citum_processor_free(processor) };
    }

    #[test]
    fn processor_new_from_yaml_rejects_invalid_style() {
        let bib = "references: []";
        let processor = unsafe {
            citum_processor_new_from_yaml(
                c_string("not: valid: yaml: [").as_ptr(),
                c_string(bib).as_ptr(),
            )
        };
        assert!(processor.is_null());
        assert!(last_error().contains("Style YAML parse error"));
    }

    #[test]
    fn processor_new_with_locale_from_yaml_returns_valid_pointer() {
        let style = serde_yaml::to_string(&Style::default()).expect("style serializes");
        let bib = "references: []";
        // Use RawLocale wire format, not the internal Locale type.
        let locale = "locale: en-US";
        let processor = unsafe {
            citum_processor_new_with_locale_from_yaml(
                c_string(&style).as_ptr(),
                c_string(bib).as_ptr(),
                c_string(locale).as_ptr(),
            )
        };
        assert!(!processor.is_null());
        unsafe { citum_processor_free(processor) };
    }
}
