/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Entry point for the `citum` command-line interface.
//!
//! This binary wires the top-level CLI commands and delegates their work to
//! the library crates.

#![allow(missing_docs, reason = "bin")]

mod args;
mod commands;
mod output;
mod style_browser;
mod style_catalog;
mod style_resolver;
mod table;
mod typst_pdf;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    if let Err(e) = commands::run() {
        eprintln!("\nError: {e}");
        std::process::exit(1);
    }
}
