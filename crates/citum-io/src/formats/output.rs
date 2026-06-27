/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! BibLaTeX and RIS bibliography rendering helpers.

use std::fmt::Write as _;

use citum_schema::InputBibliography;
use citum_schema::reference::{ClassExtension, Contributor};

/// Convert a contributor to a list of plain name strings ("Family, Given").
fn contributor_name_strings(contributor: Contributor) -> Vec<String> {
    match contributor {
        Contributor::SimpleName(n) => vec![n.name.to_string()],
        Contributor::StructuredName(n) => vec![format!("{}, {}", n.family, n.given)],
        Contributor::Multilingual(n) => {
            vec![format!("{}, {}", n.original.family, n.original.given)]
        }
        Contributor::ContributorList(list) => list
            .0
            .into_iter()
            .flat_map(contributor_name_strings)
            .collect(),
    }
}

/// Render a bibliography as a BibLaTeX `.bib` string.
pub(crate) fn render_biblatex(input: &InputBibliography) -> String {
    let mut out = String::new();
    for reference in &input.references {
        let id = reference.id().unwrap_or_else(|| "item".into());
        let entry_type = match reference.extension() {
            ClassExtension::SerialComponent(_) => "article",
            ClassExtension::CollectionComponent(_) => "incollection",
            _ => "book",
        };
        let _ = writeln!(&mut out, "@{entry_type}{{{id},");
        if let Some(title) = reference.title() {
            let _ = writeln!(&mut out, "  title = {{{title}}},");
        }
        if let Some(contributor) = reference.author() {
            let names = contributor_name_strings(contributor);
            if !names.is_empty() {
                let _ = writeln!(&mut out, "  author = {{{}}},", names.join(" and "));
            }
        }
        if let Some(issued) = reference.effective_issued_date()
            && let Some(year) = issued.0.get(0..4)
        {
            let _ = writeln!(&mut out, "  year = {{{year}}},");
        }
        if let Some(doi) = reference.doi() {
            let _ = writeln!(&mut out, "  doi = {{{doi}}},");
        }
        let _ = writeln!(&mut out, "}}\n");
    }
    out
}

/// Render a bibliography as a RIS string.
pub(crate) fn render_ris(input: &InputBibliography) -> String {
    let mut out = String::new();
    for reference in &input.references {
        let ty = match reference.extension() {
            ClassExtension::SerialComponent(_) => "JOUR",
            ClassExtension::CollectionComponent(_) => "CHAP",
            _ => "BOOK",
        };
        let _ = writeln!(&mut out, "TY  - {ty}");
        if let Some(id) = reference.id() {
            let _ = writeln!(&mut out, "ID  - {id}");
        }
        if let Some(title) = reference.title() {
            let _ = writeln!(&mut out, "TI  - {title}");
        }
        if let Some(contributor) = reference.author() {
            for name in contributor_name_strings(contributor) {
                let _ = writeln!(&mut out, "AU  - {name}");
            }
        }
        if let Some(issued) = reference.effective_issued_date()
            && let Some(year) = issued.0.get(0..4)
        {
            let _ = writeln!(&mut out, "PY  - {year}");
        }
        if let Some(doi) = reference.doi() {
            let _ = writeln!(&mut out, "DO  - {doi}");
        }
        let _ = writeln!(&mut out, "ER  -\n");
    }
    out
}
