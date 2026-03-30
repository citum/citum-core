/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Citum Processor
//!
//! This crate provides the core citation and bibliography processing functionality
//! for the Citation Style Language Next (Citum) project. It takes style definitions,
//! bibliographic data, and citation information and produces formatted output.
//!
//! The processor is designed to be pluggable with different renderers and supports
//! advanced features like disambiguation, sorting, and localization.
//!
//! # Example
//!
//! ```rust
//! use citum_engine::{
//!     Bibliography, Citation, CitationItem, CitationSpec, Config, Contributor,
//!     ContributorForm, ContributorList, ContributorRole, DateForm, EdtfString,
//!     Monograph, MonographType, MultilingualString, Processing, Processor, Reference,
//!     Rendering, StructuredName, Style, StyleInfo, TemplateComponent, TemplateContributor,
//!     TemplateDate, TemplateDateVariable, Title, WrapPunctuation,
//! };
//!
//! // Create a simple style using native Citum types
//! let style = Style {
//!     info: StyleInfo {
//!         title: Some("Simple".to_string()),
//!         id: Some("simple".to_string()),
//!         ..Default::default()
//!     },
//!     options: Some(Config {
//!         processing: Some(Processing::AuthorDate),
//!         ..Default::default()
//!     }),
//!     citation: Some(CitationSpec {
//!         template: Some(vec![
//!             TemplateComponent::Contributor(TemplateContributor {
//!                 contributor: ContributorRole::Author,
//!                 form: ContributorForm::Short,
//!                 rendering: Rendering::default(),
//!                 ..Default::default()
//!             }),
//!             TemplateComponent::Date(TemplateDate {
//!                 date: TemplateDateVariable::Issued,
//!                 form: DateForm::Year,
//!                 rendering: Rendering::default(),
//!                 ..Default::default()
//!             }),
//!         ]),
//!         wrap: Some(WrapPunctuation::Parentheses.into()),
//!         ..Default::default()
//!     }),
//!     ..Default::default()
//! };
//!
//! // Create a bibliography using native Citum reference data
//! let mut bib = Bibliography::new();
//! let reference = Reference::Monograph(Box::new(Monograph {
//!     id: Some("kuhn1962".to_string()),
//!     r#type: MonographType::Book,
//!     title: Some(Title::Single("The Structure of Scientific Revolutions".to_string())),
//!     container_title: None,
//!     author: Some(Contributor::ContributorList(ContributorList(vec![
//!         Contributor::StructuredName(StructuredName {
//!             family: MultilingualString::Simple("Kuhn".to_string()),
//!             given: MultilingualString::Simple("Thomas".to_string()),
//!             suffix: None,
//!             dropping_particle: None,
//!             non_dropping_particle: None,
//!         }),
//!     ]))),
//!     editor: None,
//!     translator: None,
//!     recipient: None,
//!     interviewer: None,
//!     guest: None,
//!     issued: EdtfString("1962".to_string()),
//!     publisher: None,
//!     url: None,
//!     accessed: None,
//!     language: None,
//!     field_languages: Default::default(),
//!     note: None,
//!     isbn: None,
//!     doi: None,
//!     edition: None,
//!     report_number: None,
//!     collection_number: None,
//!     genre: None,
//!     medium: None,
//!     archive: None,
//!     archive_location: None,
//!     archive_info: None,
//!     eprint: None,
//!     keywords: None,
//!     original_date: None,
//!     original_title: None,
//!     ads_bibcode: None,
//! }));
//! bib.insert("kuhn1962".to_string(), reference);
//!
//! // Create processor and render
//! let processor = Processor::new(style, bib);
//! let citation = Citation {
//!     id: Some("c1".to_string()),
//!     items: vec![CitationItem { id: "kuhn1962".to_string(), ..Default::default() }],
//!     ..Default::default()
//! };
//! let result = processor.process_citation(&citation).unwrap();
//! assert_eq!(result, "(Kuhn, 1962)");
//! ```

/// Error types returned by citation and bibliography processing.
pub mod error;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod grouping;
/// File loading and deserialization helpers for processor inputs.
pub mod io;
/// Citation, bibliography, sorting, and document processing logic.
pub mod processor;
pub mod reference;
/// Output-format renderers and string conversion helpers.
pub mod render;
/// Template value resolution and formatting helpers.
pub mod values;

pub use citum_schema::options::{Config, Processing};
pub use citum_schema::reference::{
    Contributor, ContributorList, EdtfString, Monograph, MonographType, MultilingualString,
    StructuredName, Title,
};
pub use citum_schema::template::{
    ContributorForm, ContributorRole, DateForm, DateVariable as TemplateDateVariable, Rendering,
    TemplateComponent, TemplateContributor, TemplateDate, WrapPunctuation,
};
pub use citum_schema::{CitationSpec, Style, StyleInfo};
pub use error::ProcessorError;
pub use processor::document::DocumentFormat;
pub use processor::{ProcessedReferences, Processor};
pub use reference::{Bibliography, Citation, CitationItem, Reference};
pub use render::{ProcTemplate, ProcTemplateComponent, citation_to_string, refs_to_string};
pub use values::{ComponentValues, ProcHints, ProcValues, RenderContext, RenderOptions};

// Re-export Locale from citum_schema for convenience
pub use citum_schema::locale::Locale;
