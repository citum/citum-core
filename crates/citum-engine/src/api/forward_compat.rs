/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Forward-compat unknown-field walking for `citum check`.
//!
//! Tolerant style option/section structs capture unknown keys into a
//! `unknown_fields` map at parse time (see
//! `docs/specs/FORWARD_COMPATIBILITY.md`). This module walks a parsed
//! [`Style`] and reports every populated capture, so CLI surfaces such as
//! `citum check --strict` can emit them as warnings or errors.

use citum_schema::CitationSpec;
use citum_schema::Style;
use citum_schema::options::{
    BibliographyOptions, CitationOptions, Config, LocatorConfig, SubstituteConfig,
};

/// A single populated `unknown_fields` capture located in a parsed [`Style`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownFieldPath {
    /// Dotted path to the struct that captured the keys.
    pub path: String,
    /// The unknown keys captured at that path.
    pub keys: Vec<String>,
}

/// Walk a parsed [`Style`] and collect every populated `unknown_fields`
/// capture (top-level, options, nested option structs, nested citation
/// specs).
#[must_use]
pub fn collect_unknown_field_paths(style: &Style) -> Vec<UnknownFieldPath> {
    let mut out = Vec::new();
    push_keys(&mut out, "$", style.unknown_fields.keys());

    if let Some(options) = &style.options {
        walk_config(&mut out, "$.options", options);
    }
    if let Some(citation) = &style.citation {
        walk_citation_spec(&mut out, "$.citation", citation);
    }
    if let Some(bib) = &style.bibliography {
        push_keys(&mut out, "$.bibliography", bib.unknown_fields.keys());
        if let Some(bo) = &bib.options {
            push_keys(&mut out, "$.bibliography.options", bo.unknown_fields.keys());
            walk_bibliography_options_nested(&mut out, "$.bibliography.options", bo);
        }
    }

    out
}

fn walk_citation_spec(out: &mut Vec<UnknownFieldPath>, base: &str, spec: &CitationSpec) {
    push_keys(out, base, spec.unknown_fields.keys());
    if let Some(co) = &spec.options {
        push_keys(out, &format!("{base}.options"), co.unknown_fields.keys());
        walk_citation_options_nested(out, &format!("{base}.options"), co);
    }
    if let Some(child) = &spec.integral {
        walk_citation_spec(out, &format!("{base}.integral"), child);
    }
    if let Some(child) = &spec.non_integral {
        walk_citation_spec(out, &format!("{base}.non-integral"), child);
    }
    if let Some(child) = &spec.subsequent {
        walk_citation_spec(out, &format!("{base}.subsequent"), child);
    }
    if let Some(child) = &spec.ibid {
        walk_citation_spec(out, &format!("{base}.ibid"), child);
    }
}

fn walk_config(out: &mut Vec<UnknownFieldPath>, base: &str, c: &Config) {
    push_keys(out, base, c.unknown_fields.keys());
    if let Some(contributors) = &c.contributors {
        push_keys(
            out,
            &format!("{base}.contributors"),
            contributors.unknown_fields.keys(),
        );
    }
    if let Some(SubstituteConfig::Explicit(sub)) = &c.substitute {
        push_keys(
            out,
            &format!("{base}.substitute"),
            sub.unknown_fields.keys(),
        );
    }
    if let Some(dates) = &c.dates {
        push_keys(out, &format!("{base}.dates"), dates.unknown_fields.keys());
    }
    if let Some(titles) = &c.titles {
        push_keys(out, &format!("{base}.titles"), titles.unknown_fields.keys());
    }
    if let Some(locators) = &c.locators {
        walk_locator_config(out, &format!("{base}.locators"), locators);
    }
    if let Some(notes) = &c.notes {
        push_keys(out, &format!("{base}.notes"), notes.unknown_fields.keys());
    }
    if let Some(integral) = &c.integral_names {
        push_keys(
            out,
            &format!("{base}.integral-names"),
            integral.unknown_fields.keys(),
        );
    }
}

fn walk_locator_config(out: &mut Vec<UnknownFieldPath>, base: &str, lc: &LocatorConfig) {
    push_keys(out, base, lc.unknown_fields.keys());
    for (kind, cfg) in &lc.kinds {
        push_keys(
            out,
            &format!("{base}.kinds.{kind:?}"),
            cfg.unknown_fields.keys(),
        );
    }
    for (idx, pattern) in lc.patterns.iter().enumerate() {
        push_keys(
            out,
            &format!("{base}.patterns[{idx}]"),
            pattern.unknown_fields.keys(),
        );
    }
}

fn walk_citation_options_nested(out: &mut Vec<UnknownFieldPath>, base: &str, co: &CitationOptions) {
    if let Some(contributors) = &co.contributors {
        push_keys(
            out,
            &format!("{base}.contributors"),
            contributors.unknown_fields.keys(),
        );
    }
    if let Some(dates) = &co.dates {
        push_keys(out, &format!("{base}.dates"), dates.unknown_fields.keys());
    }
    if let Some(titles) = &co.titles {
        push_keys(out, &format!("{base}.titles"), titles.unknown_fields.keys());
    }
    if let Some(locators) = &co.locators {
        walk_locator_config(out, &format!("{base}.locators"), locators);
    }
    if let Some(notes) = &co.notes {
        push_keys(out, &format!("{base}.notes"), notes.unknown_fields.keys());
    }
    if let Some(integral) = &co.integral_names {
        push_keys(
            out,
            &format!("{base}.integral-names"),
            integral.unknown_fields.keys(),
        );
    }
}

fn walk_bibliography_options_nested(
    out: &mut Vec<UnknownFieldPath>,
    base: &str,
    bo: &BibliographyOptions,
) {
    if let Some(contributors) = &bo.contributors {
        push_keys(
            out,
            &format!("{base}.contributors"),
            contributors.unknown_fields.keys(),
        );
    }
    if let Some(dates) = &bo.dates {
        push_keys(out, &format!("{base}.dates"), dates.unknown_fields.keys());
    }
    if let Some(titles) = &bo.titles {
        push_keys(out, &format!("{base}.titles"), titles.unknown_fields.keys());
    }
    if let Some(article_journal) = &bo.article_journal {
        push_keys(
            out,
            &format!("{base}.article-journal"),
            article_journal.unknown_fields.keys(),
        );
    }
    if let Some(compound) = &bo.compound_numeric {
        push_keys(
            out,
            &format!("{base}.compound-numeric"),
            compound.unknown_fields.keys(),
        );
    }
    if let Some(partitioning) = &bo.sort_partitioning {
        push_keys(
            out,
            &format!("{base}.sort-partitioning"),
            partitioning.unknown_fields.keys(),
        );
    }
}

fn push_keys<'a, I>(out: &mut Vec<UnknownFieldPath>, path: &str, keys: I)
where
    I: IntoIterator<Item = &'a String>,
{
    let collected: Vec<String> = keys.into_iter().cloned().collect();
    if !collected.is_empty() {
        out.push(UnknownFieldPath {
            path: path.to_string(),
            keys: collected,
        });
    }
}
