/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! CSL-JSON bibliography load and write helpers.

use citum_schema::reference::InputReference;
use csl_legacy::csl_json::{DateVariable, Name, Reference as LegacyReference, StringOrNumber};

/// Convert an [`InputReference`] to a CSL-JSON [`LegacyReference`].
pub(crate) fn input_reference_to_csl_json(reference: &InputReference) -> LegacyReference {
    let id = reference.id().unwrap_or_else(|| "item".into());
    let mut r = LegacyReference {
        id: id.to_string(),
        ..Default::default()
    };

    r.title = reference.title().map(|t| t.to_string());
    r.language = reference.language().map(|lang| lang.to_string());
    r.note = reference.note().map(|rt| rt.raw().to_string());
    r.doi = reference.doi();
    r.issued = reference.effective_issued_date().and_then(|d| {
        let year = d.0.get(0..4)?.parse::<i32>().ok()?;
        Some(DateVariable::year(year))
    });
    r.author = reference.author().map(contributor_to_csl_names);
    r.editor = reference.editor().map(contributor_to_csl_names);
    r.translator = reference.translator().map(contributor_to_csl_names);
    r.publisher = reference.publisher().map(|p| p.name.to_string());

    match reference.extension() {
        citum_schema::reference::ClassExtension::Monograph(m) => {
            r.ref_type = "book".to_string();
            r.isbn.clone_from(&m.isbn);
            r.url = m.url.as_ref().map(std::string::ToString::to_string);
            r.edition = reference.edition().map(StringOrNumber::String);
        }
        citum_schema::reference::ClassExtension::SerialComponent(s) => {
            r.ref_type = "article-journal".to_string();
            r.container_title = reference.container_title().map(|t| t.to_string());
            r.page.clone_from(&s.pages);
            r.volume = reference
                .volume()
                .map(|v| StringOrNumber::String(v.to_string()));
            r.issue = reference
                .issue()
                .map(|v| StringOrNumber::String(v.to_string()));
            r.url = s.url.as_ref().map(std::string::ToString::to_string);
        }
        citum_schema::reference::ClassExtension::CollectionComponent(c) => {
            r.ref_type = "chapter".to_string();
            r.container_title = reference.container_title().map(|t| t.to_string());
            r.page = c.pages.as_ref().map(std::string::ToString::to_string);
        }
        _ => {
            r.ref_type = "book".to_string();
        }
    }

    r
}

fn contributor_to_csl_names(contributor: citum_schema::reference::Contributor) -> Vec<Name> {
    let mut names = Vec::new();
    match contributor {
        citum_schema::reference::Contributor::SimpleName(n) => {
            names.push(Name::literal(&n.name.to_string()));
        }
        citum_schema::reference::Contributor::StructuredName(n) => {
            names.push(Name {
                family: Some(n.family.to_string()),
                given: Some(n.given.to_string()),
                suffix: n.suffix,
                dropping_particle: n.dropping_particle,
                non_dropping_particle: n.non_dropping_particle,
                literal: None,
            });
        }
        citum_schema::reference::Contributor::Multilingual(n) => {
            names.push(Name {
                family: Some(n.original.family.to_string()),
                given: Some(n.original.given.to_string()),
                suffix: n.original.suffix,
                dropping_particle: n.original.dropping_particle,
                non_dropping_particle: n.original.non_dropping_particle,
                literal: None,
            });
        }
        citum_schema::reference::Contributor::ContributorList(list) => {
            for member in list.0 {
                names.extend(contributor_to_csl_names(member));
            }
        }
    }
    names
}
