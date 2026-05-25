// Minimal example of using citum-engine as a library.
//
// Loads a style from a YAML file and a bibliography from Citum's native
// JSON format, then formats two citations and a bibliography for a
// hypothetical document.

use std::error::Error;
use std::fs;

use citum_engine::{
    format_document, CitationOccurrence, CitationOccurrenceItem, FormatDocumentRequest,
    OutputFormatKind, RefsInput, StyleInput,
};

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Load the style. StyleInput::Path reads a YAML style from disk;
    //    StyleInput::Yaml accepts inline YAML. (Id and Uri variants require a
    //    resolver chain that lives in citum-server.)
    let style = StyleInput::Path("apa-7th.yaml".to_string());

    // 2. Load references in Citum's native JSON format. RefsInput::Json accepts
    //    the inline reference map. For BibLaTeX or CSL-JSON inputs, convert first.
    let refs_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        "references-native.json",
    )?)?;
    let refs = RefsInput::Json(refs_json);

    // 3. Describe the citations in document order. Your own pipeline owns the
    //    job of scanning Markdown / Djot / AST for citation keys; citum only
    //    needs the resolved occurrences.
    let citations = vec![
        CitationOccurrence {
            id: "c1".to_string(),
            items: vec![CitationOccurrenceItem {
                id: "smith2010".to_string(),
                locator: None,
                prefix: None,
                suffix: None,
                integral_name_state: None,
            }],
            mode: None,
            note_number: None,
            suppress_author: None,
            grouped: None,
            prefix: None,
            suffix: None,
        },
        CitationOccurrence {
            id: "c2".to_string(),
            items: vec![CitationOccurrenceItem {
                id: "smith2010".to_string(),
                locator: None,
                prefix: None,
                suffix: None,
                integral_name_state: None,
            }],
            mode: None,
            note_number: None,
            suppress_author: None,
            grouped: None,
            prefix: None,
            suffix: None,
        },
    ];

    // 4. Format. format_document returns formatted citations in document order
    //    plus a rendered bibliography in the requested output format.
    let result = format_document(FormatDocumentRequest {
        style,
        locale: Some("en-US".to_string()),
        output_format: OutputFormatKind::Html,
        refs,
        citations,
        document_options: None,
    })?;

    // 5. Substitute into your document. Here we just print.
    for cite in &result.formatted_citations {
        println!("{}: {}", cite.id, cite.text);
    }
    println!("\n--- Bibliography ---\n{}", result.bibliography.content);

    Ok(())
}
