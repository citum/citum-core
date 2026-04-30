#![allow(
    missing_docs,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "test/bench/bin crate"
)]

use citum_schema_style::renderer::RenderItem;
use citum_schema_style::{CslnStyle, ItemType, Renderer, Variable};
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the Migrated Style
    let json = fs::read_to_string("citum.json").map_err(|e| {
        format!("Please run 'cargo run --bin citum-migrate' first to generate citum.json ({e})")
    })?;
    let style: CslnStyle = serde_json::from_str(&json)?;

    println!("Loaded Style: {}", style.info.title);

    // 2. Create a Mock Reference Item (Book)
    let mut variables = HashMap::new();
    variables.insert(Variable::Author, "Doe, John".to_string());
    variables.insert(Variable::Issued, "2020".to_string());
    variables.insert(Variable::Title, "The Rust Programming Language".to_string());
    variables.insert(Variable::Publisher, "No Starch Press".to_string());

    let item = RenderItem {
        item_type: ItemType::Book,
        variables,
    };

    // 3. Render Citation
    let renderer = Renderer;
    let cit_output = renderer.render_citation(&style.citation, &item);

    println!("\n=== RENDERED CITATION ===");
    println!("{cit_output}");
    println!("=========================");

    // 4. Render Bibliography
    let bib_output = renderer.render_citation(&style.bibliography, &item);
    println!("\n=== RENDERED BIBLIOGRAPHY ===");
    println!("{bib_output}");
    println!("=============================");

    Ok(())
}
