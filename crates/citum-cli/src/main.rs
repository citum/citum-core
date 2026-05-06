#![allow(missing_docs, reason = "bin")]

mod args;
mod commands;
mod output;
mod style_resolver;
mod table;
mod typst_pdf;

fn main() {
    if let Err(e) = commands::run() {
        eprintln!("\nError: {e}");
        std::process::exit(1);
    }
}
