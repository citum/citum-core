#![allow(missing_docs, reason = "test")]

use citum_engine::processor::Processor;
use citum_engine::render::latex::Latex;
use citum_schema::{
    Style,
    citation::{Citation, CitationItem, CitationMode},
};
use std::fs;
use std::path::Path;

fn main() {
    let style_str = fs::read_to_string("styles/apa-7th.yaml").unwrap();
    let style: Style = serde_yaml::from_str(&style_str).unwrap();

    let bib =
        citum_engine::io::load_bibliography(Path::new("bindings/latex/example-refs.yaml")).unwrap();
    let processor = Processor::new(style, bib);

    let cite = Citation {
        id: Some("cite1".to_string()),
        mode: CitationMode::NonIntegral,
        position: None,
        suppress_author: false,
        prefix: None,
        suffix: None,
        note_number: None,
        items: vec![CitationItem {
            id: "weinberg1971".to_string(),
            locator: None,
            prefix: None,
            suffix: None,
            integral_name_state: None,
        }],
    };

    match processor.process_citation_with_format::<Latex>(&cite) {
        Ok(res) => println!("CITATION: {res}"),
        Err(e) => println!("ERROR: {e:?}"),
    }

    // Try suppress author just in case
    let cite_sa = Citation {
        mode: CitationMode::NonIntegral,
        suppress_author: true,
        ..cite.clone()
    };
    match processor.process_citation_with_format::<Latex>(&cite_sa) {
        Ok(res) => println!("CITATION SA: {res}"),
        Err(e) => println!("ERROR: {e:?}"),
    }
}
