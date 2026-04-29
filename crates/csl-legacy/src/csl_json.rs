/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! CSL-JSON reference model.
//!
//! This module provides a CSL-JSON compatible reference model for parsing
//! existing bibliographic data in the legacy CSL-JSON format.
//!
//! Note: This is a legacy format with known limitations. The preferred format
//! for new data is the Citum InputReference model in citum_schema.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};

/// A bibliographic reference item.
/// This is compatible with CSL-JSON format.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Reference {
    /// Unique identifier for the reference.
    pub id: String,
    /// The type of reference (book, article-journal, etc.)
    #[serde(rename = "type")]
    pub ref_type: String,
    /// Authors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Vec<Name>>,
    /// Editors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<Vec<Name>>,
    /// Translators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translator: Option<Vec<Name>>,
    /// Recipient
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<Vec<Name>>,
    /// Director
    #[serde(skip_serializing_if = "Option::is_none")]
    pub director: Option<Vec<Name>>,
    /// Interviewer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interviewer: Option<Vec<Name>>,
    /// Primary title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Container title (journal, book, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_title: Option<String>,
    /// Collection title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_title: Option<String>,
    /// Collection number (series number)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_number: Option<StringOrNumber>,
    /// Issued date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued: Option<DateVariable>,
    /// Accessed date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<DateVariable>,
    /// Volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<StringOrNumber>,
    /// Issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<StringOrNumber>,
    /// Page or page range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    /// Edition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<StringOrNumber>,
    /// DOI
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DOI")]
    pub doi: Option<String>,
    /// URL
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "URL")]
    pub url: Option<String>,
    /// ISBN
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ISBN")]
    pub isbn: Option<String>,
    /// ISSN
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ISSN")]
    pub issn: Option<String>,
    /// Publisher
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    /// Publisher place
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_place: Option<String>,
    /// Authority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// Section
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// Medium
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
    /// Archive or repository name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive: Option<String>,
    /// Archive shelfmark, repository location, or call number.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "archive_location")]
    pub archive_location: Option<String>,
    /// Number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Chapter or session identifier used by some legal and legislative sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_number: Option<String>,
    /// Printing number (printing run identifier).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printing_number: Option<String>,
    /// Genre
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    /// Language (BCP 47)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Original title in the source language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,
    /// Abstract
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    /// Note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Number of pages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_pages: Option<StringOrNumber>,
    /// Number of volumes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_volumes: Option<StringOrNumber>,
    /// Additional fields not explicitly modeled
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A name (person or organization).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Name {
    /// Family name for a structured personal name.
    pub family: Option<String>,
    /// Given name for a structured personal name.
    pub given: Option<String>,
    /// Literal name (for organizations or single-field names)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal: Option<String>,
    /// Name suffix (Jr., III, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Dropping particle (de, van, etc. that sorts with given name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dropping_particle: Option<String>,
    /// Non-dropping particle (de, van, etc. that sorts with family name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_dropping_particle: Option<String>,
}

impl Name {
    /// Create a new structured name.
    #[must_use]
    pub fn new(family: &str, given: &str) -> Self {
        Self {
            family: Some(family.to_string()),
            given: Some(given.to_string()),
            ..Default::default()
        }
    }

    /// Create a literal name (organization or single-field).
    #[must_use]
    pub fn literal(name: &str) -> Self {
        Self {
            literal: Some(name.to_string()),
            ..Default::default()
        }
    }

    /// Get the family name or literal.
    #[must_use]
    pub fn family_or_literal(&self) -> &str {
        self.family
            .as_deref()
            .or(self.literal.as_deref())
            .unwrap_or("")
    }
}

/// A date variable (CSL-JSON format).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DateVariable {
    /// Date parts: [[year, month, day], [`end_year`, `end_month`, `end_day`]]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_date_parts_opt"
    )]
    pub date_parts: Option<Vec<Vec<i32>>>,
    /// Literal date string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal: Option<String>,
    /// Raw date string (for parsing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    /// Season (1-4)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_season_opt"
    )]
    pub season: Option<i32>,
    /// Circa (approximate date)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circa: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IntOrString {
    Int(i32),
    String(String),
}

fn deserialize_date_parts_opt<'de, D>(deserializer: D) -> Result<Option<Vec<Vec<i32>>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<Vec<Vec<IntOrString>>>::deserialize(deserializer)?;
    raw.map(|rows| {
        rows.into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|value| match value {
                        IntOrString::Int(n) => Ok(n),
                        IntOrString::String(text) => text.parse::<i32>().map_err(|_| {
                            serde::de::Error::custom(format!(
                                "invalid date-parts component {:?}: expected integer or integer-like string",
                                text
                            ))
                        }),
                    })
                    .collect::<Result<Vec<_>, D::Error>>()
            })
            .collect::<Result<Vec<_>, D::Error>>()
    })
    .transpose()
}

fn deserialize_season_opt<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<IntOrString>::deserialize(deserializer)?;
    raw.map(|value| match value {
        IntOrString::Int(n) => Ok(n),
        IntOrString::String(text) => {
            let normalized = text.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "spring" => Ok(1),
                "summer" => Ok(2),
                "fall" | "autumn" => Ok(3),
                "winter" => Ok(4),
                _ => text.parse::<i32>().map_err(|_| {
                    serde::de::Error::custom(format!(
                        "invalid season {:?}: expected integer, integer-like string, or named season",
                        text
                    ))
                }),
            }
        }
    })
    .transpose()
}

impl DateVariable {
    /// Create a date with year only.
    #[must_use]
    pub fn year(year: i32) -> Self {
        Self {
            date_parts: Some(vec![vec![year]]),
            ..Default::default()
        }
    }

    /// Create a date with year and month.
    #[must_use]
    pub fn year_month(year: i32, month: i32) -> Self {
        Self {
            date_parts: Some(vec![vec![year, month]]),
            ..Default::default()
        }
    }

    /// Create a full date.
    #[must_use]
    pub fn full(year: i32, month: i32, day: i32) -> Self {
        Self {
            date_parts: Some(vec![vec![year, month, day]]),
            ..Default::default()
        }
    }

    /// Get the year from the first date part.
    #[must_use]
    pub fn year_value(&self) -> Option<i32> {
        self.date_parts
            .as_ref()
            .and_then(|parts| parts.first())
            .and_then(|date| date.first())
            .copied()
    }

    /// Get the month from the first date part.
    #[must_use]
    pub fn month_value(&self) -> Option<i32> {
        self.date_parts
            .as_ref()
            .and_then(|parts| parts.first())
            .and_then(|date| date.get(1))
            .copied()
    }

    /// Get the day from the first date part.
    #[must_use]
    pub fn day_value(&self) -> Option<i32> {
        self.date_parts
            .as_ref()
            .and_then(|parts| parts.first())
            .and_then(|date| date.get(2))
            .copied()
    }
}

/// A value that can be either a string or number.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrNumber {
    /// A string value preserved as written in the source data.
    String(String),
    /// A numeric value preserved as an integer.
    Number(i64),
}

impl std::fmt::Display for StringOrNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Number(n) => write!(f, "{n}"),
        }
    }
}

impl Reference {
    /// Parse structured CSL variables from the note field.
    ///
    /// Extracts key-value pairs from the Zotero "Extra" field (stored in `note`),
    /// following citeproc-js's `parseNoteFieldHacks` behavior. Lines contain
    /// `key: value` patterns where the key is a CSL variable name (e.g., `DOI`,
    /// `publisher`, `issued`). Type overrides are always applied; other fields
    /// only set if currently None. Only recognized keys are consumed; unrecognized
    /// `key: value` lines are left in `note` unchanged.
    ///
    /// # Algorithm
    /// 1. Skip if note is None or empty
    /// 2. Split by newline
    /// 3. For each line, parse `key: value` where key is alphabetic
    /// 4. Stop at first non-matching line (unless it's the first line)
    /// 5. Extract type, dates, names, and strings to appropriate fields
    /// 6. Rebuild note with only unrecognized/unparsed lines
    pub fn parse_note_field_hacks(&mut self) {
        let note = match &self.note {
            Some(n) if !n.is_empty() => n.clone(),
            _ => return,
        };

        let lines: Vec<&str> = note.lines().collect();
        if lines.is_empty() {
            return;
        }

        let mut parsed_indices = HashSet::new();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip blank lines
            if trimmed.is_empty() {
                continue;
            }

            // Parse key: value pattern; skip non-matching lines so recognized
            // pairs later in the note (after free-text) are still extracted.
            let Some((key, value)) = parse_key_value(trimmed) else {
                continue;
            };

            // Process the key-value pair; only mark as parsed when the key is recognized
            if key.eq_ignore_ascii_case("type") {
                self.ref_type = value.to_string();
                parsed_indices.insert(idx);
            } else if is_date_variable(key) {
                handle_date_variable(self, key, value);
                parsed_indices.insert(idx);
            } else if is_name_variable(key) {
                handle_name_variable(self, key, value);
                parsed_indices.insert(idx);
            } else if is_string_variable(key) {
                handle_string_variable(self, key, value);
                parsed_indices.insert(idx);
            }
        }

        // Rebuild note from unparsed lines
        let remaining_lines: Vec<&str> = lines
            .iter()
            .enumerate()
            .filter(|(idx, _)| !parsed_indices.contains(idx))
            .map(|(_, line)| *line)
            .collect();

        if remaining_lines.is_empty() {
            self.note = None;
        } else {
            self.note = Some(remaining_lines.join("\n"));
        }
    }
}

/// Parse a line into `(key, value)` if it matches `key: value` pattern.
/// Key must start with an ASCII letter and contain only word chars, hyphens,
/// underscores. Uppercase keys (e.g. `DOI`, `ISBN`, `URL`) are preserved
/// as-is so they can be matched by their canonical uppercase names downstream.
fn parse_key_value(line: &str) -> Option<(&str, &str)> {
    if line.is_empty() {
        return None;
    }

    let colon_pos = line.find(':')?;
    let key = &line[..colon_pos];

    // Validate key: must start with an ASCII letter (upper or lower)
    let first_char = key.chars().next()?;
    if !first_char.is_ascii_alphabetic() {
        return None;
    }

    // Rest of key: word chars, hyphens, underscores
    if !key
        .chars()
        .skip(1)
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }

    let value = line[colon_pos + 1..].trim();
    Some((key, value))
}

/// Check if key is a date variable.
fn is_date_variable(key: &str) -> bool {
    matches!(
        key,
        "issued" | "event-date" | "original-date" | "available-date" | "accessed" | "submitted"
    )
}

/// Handle date variable extraction.
fn handle_date_variable(ref_obj: &mut Reference, key: &str, value: &str) {
    let date = parse_date_variable(value);

    match key {
        "issued" if ref_obj.issued.is_none() => {
            ref_obj.issued = Some(date);
        }
        "accessed" if ref_obj.accessed.is_none() => {
            ref_obj.accessed = Some(date);
        }
        "event-date" | "original-date" | "available-date" | "submitted" => {
            // Store in extra as JSON
            ref_obj.extra.insert(
                key.to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
        _ => {}
    }
}

/// Parse a date string into DateVariable.
/// Supports YYYY, YYYY-MM, YYYY-MM-DD, and ranges like YYYY/YYYY or YYYY–YYYY.
fn parse_date_variable(value: &str) -> DateVariable {
    let trimmed = value.trim();

    // Check for range: YYYY/YYYY or YYYY–YYYY
    if let Some(sep_pos) = trimmed.find('/').or_else(|| trimmed.find('–')) {
        let start_str = trimmed[..sep_pos].trim();
        let end_str = trimmed[sep_pos + 1..].trim();

        if let (Some(start_parts), Some(end_parts)) =
            (parse_date_parts(start_str), parse_date_parts(end_str))
        {
            return DateVariable {
                date_parts: Some(vec![start_parts, end_parts]),
                ..Default::default()
            };
        }
    }

    // Try to parse as YYYY-MM-DD, YYYY-MM, or YYYY
    if let Some(parts) = parse_date_parts(trimmed) {
        DateVariable {
            date_parts: Some(vec![parts]),
            ..Default::default()
        }
    } else {
        // Store as raw for downstream parsing
        DateVariable {
            raw: Some(trimmed.to_string()),
            ..Default::default()
        }
    }
}

/// Parse date parts from a string like "2020", "2020-03", or "2020-03-15".
fn parse_date_parts(s: &str) -> Option<Vec<i32>> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }

    let mut result = Vec::new();
    for part in parts {
        result.push(part.trim().parse::<i32>().ok()?);
    }

    Some(result)
}

/// Check if key is a name variable.
fn is_name_variable(key: &str) -> bool {
    matches!(
        key,
        "editor"
            | "translator"
            | "interviewer"
            | "director"
            | "recipient"
            | "author"
            | "collection-editor"
            | "container-author"
            | "editorial-director"
            | "illustrator"
            | "original-author"
            | "reviewed-author"
            | "composer"
            | "narrator"
            | "performer"
            | "producer"
            | "script-writer"
            | "compiler"
    )
}

/// Handle name variable extraction.
fn handle_name_variable(ref_obj: &mut Reference, key: &str, value: &str) {
    let name = parse_name_variable(value);

    match key {
        "author" if ref_obj.author.is_none() => {
            ref_obj.author = Some(vec![name]);
        }
        "editor" if ref_obj.editor.is_none() => {
            ref_obj.editor = Some(vec![name]);
        }
        "translator" if ref_obj.translator.is_none() => {
            ref_obj.translator = Some(vec![name]);
        }
        "interviewer" if ref_obj.interviewer.is_none() => {
            ref_obj.interviewer = Some(vec![name]);
        }
        "director" if ref_obj.director.is_none() => {
            ref_obj.director = Some(vec![name]);
        }
        "recipient" if ref_obj.recipient.is_none() => {
            ref_obj.recipient = Some(vec![name]);
        }
        // Fields not on Reference: store in extra as JSON
        "collection-editor" | "container-author" | "editorial-director" | "illustrator"
        | "original-author" | "reviewed-author" | "composer" | "narrator" | "performer"
        | "producer" | "script-writer" | "compiler" => {
            ref_obj.extra.insert(
                key.to_string(),
                serde_json::to_value(vec![name]).unwrap_or(serde_json::Value::Null),
            );
        }
        _ => {}
    }
}

/// Parse a name from a string.
/// Format: "family || given" for structured, otherwise literal.
fn parse_name_variable(value: &str) -> Name {
    let trimmed = value.trim();

    if let Some(sep_pos) = trimmed.find("||") {
        let family = trimmed[..sep_pos].trim().to_string();
        let given = trimmed[sep_pos + 2..].trim().to_string();

        Name {
            family: Some(family),
            given: Some(given),
            ..Default::default()
        }
    } else {
        Name {
            literal: Some(trimmed.to_string()),
            ..Default::default()
        }
    }
}

/// Check if key is a recognized string variable.
fn is_string_variable(key: &str) -> bool {
    matches!(
        key,
        "container-title"
            | "collection-title"
            | "volume-title"
            | "event-title"
            | "event-place"
            | "event-location"
            | "publisher"
            | "publisher-place"
            | "archive"
            | "archive-place"
            | "archive-location"
            | "archive-collection"
            | "archive_collection"
            | "genre"
            | "medium"
            | "dimensions"
            | "section"
            | "volume"
            | "issue"
            | "page"
            | "edition"
            | "number"
            | "number-of-volumes"
            | "number-of-pages"
            | "chapter-number"
            | "part-number"
            | "part-title"
            | "supplement-number"
            | "references"
            | "source"
            | "status"
            | "DOI"
            | "ISBN"
            | "ISSN"
            | "URL"
            | "original-title"
            | "reviewed-title"
            | "reviewed-genre"
            | "call-number"
            | "abstract"
            | "language"
            | "jurisdiction"
            | "authority"
            | "citation-number"
            | "citation-label"
            | "annote"
            | "keyword"
            | "title-short"
            | "collection-number"
    )
}

/// Handle string variable extraction.
#[allow(
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    reason = "match statement for all CSL string variable mappings"
)]
fn handle_string_variable(ref_obj: &mut Reference, key: &str, value: &str) {
    let trimmed = value.trim();

    match key {
        "container-title" if ref_obj.container_title.is_none() => {
            ref_obj.container_title = Some(trimmed.to_string());
        }
        "collection-title" if ref_obj.collection_title.is_none() => {
            ref_obj.collection_title = Some(trimmed.to_string());
        }
        "collection-number" if ref_obj.collection_number.is_none() => {
            ref_obj.collection_number = Some(StringOrNumber::String(trimmed.to_string()));
        }
        "part-title" => {
            ref_obj.extra.insert(
                key.to_string(),
                serde_json::Value::String(trimmed.to_string()),
            );
        }
        "publisher-place" if ref_obj.publisher_place.is_none() => {
            ref_obj.publisher_place = Some(trimmed.to_string());
        }
        "publisher" if ref_obj.publisher.is_none() => {
            ref_obj.publisher = Some(trimmed.to_string());
        }
        "archive-place" | "archive-location" if ref_obj.archive_location.is_none() => {
            ref_obj.archive_location = Some(trimmed.to_string());
        }
        "archive" if ref_obj.archive.is_none() => {
            ref_obj.archive = Some(trimmed.to_string());
        }
        "volume" if ref_obj.volume.is_none() => {
            ref_obj.volume = Some(StringOrNumber::String(trimmed.to_string()));
        }
        "issue" if ref_obj.issue.is_none() => {
            ref_obj.issue = Some(StringOrNumber::String(trimmed.to_string()));
        }
        "page" if ref_obj.page.is_none() => {
            ref_obj.page = Some(trimmed.to_string());
        }
        "edition" if ref_obj.edition.is_none() => {
            ref_obj.edition = Some(StringOrNumber::String(trimmed.to_string()));
        }
        "number-of-pages" if ref_obj.number_of_pages.is_none() => {
            ref_obj.number_of_pages = Some(StringOrNumber::String(trimmed.to_string()));
        }
        "number-of-volumes" if ref_obj.number_of_volumes.is_none() => {
            ref_obj.number_of_volumes = Some(StringOrNumber::String(trimmed.to_string()));
        }
        "chapter-number" if ref_obj.chapter_number.is_none() => {
            ref_obj.chapter_number = Some(trimmed.to_string());
        }
        "genre" if ref_obj.genre.is_none() => {
            ref_obj.genre = Some(trimmed.to_string());
        }
        "medium" if ref_obj.medium.is_none() => {
            ref_obj.medium = Some(trimmed.to_string());
        }
        "language" if ref_obj.language.is_none() => {
            ref_obj.language = Some(trimmed.to_string());
        }
        "original-title" if ref_obj.original_title.is_none() => {
            ref_obj.original_title = Some(trimmed.to_string());
        }
        "abstract" if ref_obj.abstract_text.is_none() => {
            ref_obj.abstract_text = Some(trimmed.to_string());
        }
        "DOI" if ref_obj.doi.is_none() => {
            ref_obj.doi = Some(trimmed.to_string());
        }
        "ISBN" if ref_obj.isbn.is_none() => {
            ref_obj.isbn = Some(trimmed.to_string());
        }
        "ISSN" if ref_obj.issn.is_none() => {
            ref_obj.issn = Some(trimmed.to_string());
        }
        "URL" if ref_obj.url.is_none() => {
            ref_obj.url = Some(trimmed.to_string());
        }
        "authority" if ref_obj.authority.is_none() => {
            ref_obj.authority = Some(trimmed.to_string());
        }
        "section" if ref_obj.section.is_none() => {
            ref_obj.section = Some(trimmed.to_string());
        }
        "number" if ref_obj.number.is_none() => {
            ref_obj.number = Some(trimmed.to_string());
        }
        // Fields not on Reference: store in extra as JSON
        "volume-title" | "event-title" | "event-place" | "event-location"
        | "archive-collection" | "archive_collection" | "dimensions" | "part-number"
        | "supplement-number" | "references" | "source" | "status" | "reviewed-title"
        | "reviewed-genre" | "call-number" | "jurisdiction" | "citation-number"
        | "citation-label" | "annote" | "keyword" | "title-short" => {
            let key_to_store = match key {
                "archive_collection" => "archive-collection",
                "event-location" => "event-place",
                _ => key,
            };
            ref_obj.extra.insert(
                key_to_store.to_string(),
                serde_json::Value::String(trimmed.to_string()),
            );
        }
        _ => {}
    }
}

/// A bibliography is a collection of references keyed by ID.
/// Uses `IndexMap` to preserve insertion order for numeric citation styles.
pub type Bibliography = indexmap::IndexMap<String, Reference>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csl_json() {
        let json = r#"{
            "id": "kuhn1962",
            "type": "book",
            "author": [{"family": "Kuhn", "given": "Thomas S."}],
            "title": "The Structure of Scientific Revolutions",
            "issued": {"date-parts": [[1962]]},
            "publisher": "University of Chicago Press",
            "publisher-place": "Chicago"
        }"#;

        let reference: Reference = serde_json::from_str(json).unwrap();
        assert_eq!(reference.id, "kuhn1962");
        assert_eq!(reference.ref_type, "book");
        assert_eq!(
            reference.author.as_ref().unwrap()[0].family,
            Some("Kuhn".to_string())
        );
        assert_eq!(reference.issued.as_ref().unwrap().year_value(), Some(1962));
    }

    #[test]
    fn test_date_variable() {
        let date = DateVariable::year(2023);
        assert_eq!(date.year_value(), Some(2023));
        assert_eq!(date.month_value(), None);

        let date = DateVariable::year_month(2023, 6);
        assert_eq!(date.year_value(), Some(2023));
        assert_eq!(date.month_value(), Some(6));
    }

    #[test]
    fn test_parse_note_field_type_override() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("type: article-journal".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(ref_obj.ref_type, "article-journal");
        assert_eq!(ref_obj.note, None);
    }

    #[test]
    fn test_parse_note_field_string_variable() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("genre: H.R.\nstatus: enacted".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(ref_obj.genre, Some("H.R.".to_string()));
        assert!(ref_obj.extra.contains_key("status"));
        assert_eq!(ref_obj.note, None);
    }

    #[test]
    fn test_parse_note_field_date() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("issued: 2020-03-15".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert!(ref_obj.issued.is_some());
        let issued = ref_obj.issued.unwrap();
        assert_eq!(issued.year_value(), Some(2020));
        assert_eq!(issued.month_value(), Some(3));
        assert_eq!(issued.day_value(), Some(15));
        assert_eq!(ref_obj.note, None);
    }

    #[test]
    fn test_parse_note_field_name() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("editor: Smith || John".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert!(ref_obj.editor.is_some());
        let editors = ref_obj.editor.unwrap();
        assert_eq!(editors.len(), 1);
        assert_eq!(editors[0].family, Some("Smith".to_string()));
        assert_eq!(editors[0].given, Some("John".to_string()));
        assert_eq!(ref_obj.note, None);
    }

    #[test]
    fn test_parse_note_field_preserves_existing() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            publisher: Some("OldPub".to_string()),
            note: Some("publisher: NewPub".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(ref_obj.publisher, Some("OldPub".to_string()));
        assert_eq!(ref_obj.note, None);
    }

    #[test]
    fn test_parse_note_field_strips_parsed() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("genre: H.R.\nThis is an extra note line".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(ref_obj.genre, Some("H.R.".to_string()));
        assert_eq!(ref_obj.note, Some("This is an extra note line".to_string()));
    }

    #[test]
    fn test_parse_note_field_empty() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: None,
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(ref_obj.note, None);
    }

    #[test]
    fn test_parse_note_field_first_line_free_text() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("This is free text on first line\ngenre: H.R.\nstatus: enacted".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(ref_obj.genre, Some("H.R.".to_string()));
        assert!(ref_obj.extra.contains_key("status"));
        assert_eq!(
            ref_obj.note,
            Some("This is free text on first line".to_string())
        );
    }

    #[test]
    fn test_parse_note_field_date_range() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("issued: 2020/2021".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert!(ref_obj.issued.is_some());
        let issued = ref_obj.issued.unwrap();
        assert!(issued.date_parts.is_some());
        let parts = issued.date_parts.unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0][0], 2020);
        assert_eq!(parts[1][0], 2021);
    }

    #[test]
    fn test_parse_note_field_name_literal() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("author: United Nations".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert!(ref_obj.author.is_some());
        let authors = ref_obj.author.unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].literal, Some("United Nations".to_string()));
    }

    #[test]
    fn test_parse_note_field_uppercase_keys() {
        // DOI, ISBN, ISSN, URL are uppercase in CSL/Zotero Extra
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "article-journal".to_string(),
            note: Some(
                "DOI: 10.1000/xyz123\nISBN: 978-3-16-148410-0\nURL: https://example.org"
                    .to_string(),
            ),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(ref_obj.doi, Some("10.1000/xyz123".to_string()));
        assert_eq!(ref_obj.isbn, Some("978-3-16-148410-0".to_string()));
        assert_eq!(ref_obj.url, Some("https://example.org".to_string()));
        assert_eq!(ref_obj.note, None);
    }

    #[test]
    fn test_parse_note_field_unknown_key_preserved() {
        // Unrecognized keys must not be silently stripped from note; the loop
        // continues past them (it only stops on lines that cannot be parsed as
        // `key: value` at all), so subsequent recognized keys are still applied.
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("genre: memoir\nfoo: bar\npublisher: MIT Press".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(ref_obj.genre, Some("memoir".to_string()));
        assert_eq!(ref_obj.publisher, Some("MIT Press".to_string()));
        // "foo: bar" is unrecognized — it stays in note unchanged
        assert_eq!(ref_obj.note, Some("foo: bar".to_string()));
    }

    #[test]
    fn test_parse_note_field_archive_location() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "book".to_string(),
            note: Some("archive-location: Box 5, Folder 3".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(
            ref_obj.archive_location,
            Some("Box 5, Folder 3".to_string())
        );
        assert_eq!(ref_obj.note, None);
    }

    #[test]
    fn test_parse_note_field_part_title_stored_in_extra() {
        let mut ref_obj = Reference {
            id: "test".to_string(),
            ref_type: "webpage".to_string(),
            note: Some("part-title: Part title".to_string()),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();
        assert_eq!(
            ref_obj.extra.get("part-title"),
            Some(&serde_json::Value::String("Part title".to_string()))
        );
        assert_eq!(ref_obj.note, None);
    }

    #[test]
    fn test_parse_note_field_recognized_keys_after_free_text() {
        // A free-text line in the middle must not stop extraction of later key:value pairs.
        let mut ref_obj = Reference {
            id: "broadcast-test".to_string(),
            ref_type: "broadcast".to_string(),
            note: Some(
                "Some free-form description with no colon\ngenre: Documentary\nevent-place: United States".to_string(),
            ),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();

        assert_eq!(ref_obj.genre, Some("Documentary".to_string()));
        assert_eq!(
            ref_obj
                .extra
                .get("event-place")
                .map(|v| v.as_str().unwrap_or("")),
            Some("United States"),
        );
        // The free-text line should remain in the note.
        assert!(
            ref_obj
                .note
                .as_deref()
                .unwrap_or("")
                .contains("Some free-form description")
        );
    }

    #[test]
    fn test_parse_note_field_recognized_keys_after_midnote_free_text() {
        // Recognized pairs both before and after a free-text line are extracted.
        let mut ref_obj = Reference {
            id: "bill-test".to_string(),
            ref_type: "bill".to_string(),
            note: Some(
                "genre: H.R.\nsome unrecognized prose line here\nstatus: enacted".to_string(),
            ),
            ..Default::default()
        };

        ref_obj.parse_note_field_hacks();

        assert_eq!(ref_obj.genre, Some("H.R.".to_string()));
        assert!(ref_obj.extra.contains_key("status"));
        assert_eq!(
            ref_obj.extra.get("status").and_then(|v| v.as_str()),
            Some("enacted"),
        );
        assert!(
            ref_obj
                .note
                .as_deref()
                .unwrap_or("")
                .contains("some unrecognized prose line")
        );
    }
}
