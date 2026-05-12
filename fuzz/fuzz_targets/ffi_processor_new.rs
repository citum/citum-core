#![no_main]

use citum_engine::ffi::{citum_processor_free, citum_processor_new};
use libfuzzer_sys::fuzz_target;
use std::ffi::CString;

fuzz_target!(|data: &[u8]| {
    let split = data.iter().position(|byte| *byte == b'\n').unwrap_or(data.len());
    let (style, bibliography) = data.split_at(split);
    let bibliography = bibliography.get(1..).unwrap_or(b"");

    let Ok(style) = CString::new(style) else {
        return;
    };
    let Ok(bibliography) = CString::new(bibliography) else {
        return;
    };

    let processor = unsafe { citum_processor_new(style.as_ptr(), bibliography.as_ptr()) };
    unsafe { citum_processor_free(processor) };
});
