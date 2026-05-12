#![no_main]

use citum_engine::ffi::{
    citum_processor_free, citum_processor_new, citum_render_citation_plain, citum_string_free,
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
    let Ok(citation) = CString::new(data) else {
        return;
    };
    let processor = processor();
    let rendered = unsafe { citum_render_citation_plain(processor, citation.as_ptr()) };
    unsafe {
        citum_string_free(rendered);
        citum_processor_free(processor);
    }
});
