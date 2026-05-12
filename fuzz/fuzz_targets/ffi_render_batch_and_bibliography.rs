#![no_main]

use citum_engine::ffi::{
    citum_processor_free, citum_processor_new, citum_render_bibliography_plain,
    citum_render_citations_json, citum_string_free,
};
use libfuzzer_sys::fuzz_target;
use std::ffi::CString;

fn processor() -> *mut citum_engine::Processor {
    let style = serde_json::to_string(&citum_schema::Style::default()).unwrap_or_default();
    let style = CString::new(style).unwrap_or_default();
    let bibliography = CString::new("{}").unwrap_or_default();
    unsafe { citum_processor_new(style.as_ptr(), bibliography.as_ptr()) }
}

fuzz_target!(|data: &[u8]| {
    let Ok(citations) = CString::new(data) else {
        return;
    };
    let format = CString::new("plain").unwrap_or_default();
    let processor = processor();
    let batch = unsafe { citum_render_citations_json(processor, citations.as_ptr(), format.as_ptr()) };
    let bibliography = unsafe { citum_render_bibliography_plain(processor) };
    unsafe {
        citum_string_free(batch);
        citum_string_free(bibliography);
        citum_processor_free(processor);
    }
});
