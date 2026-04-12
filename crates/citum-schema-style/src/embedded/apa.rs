/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use std::sync::LazyLock;

use crate::{tc_contributor, tc_date, tc_variable, template::TemplateComponent};

const APA_BIBLIOGRAPHY_TEMPLATE_YAML: &str = r#"
- contributor: author
  form: long
  suffix: "."
  name-order: family-first
- date: issued
  form: year
  wrap:
    punctuation: parentheses
  prefix: " "
- title: primary
  emph: true
- delimiter: "; "
  wrap:
    punctuation: brackets
  prefix: " "
  group:
    - variable: genre
      text-case: capitalize-first
    - variable: medium
      text-case: capitalize-first
- number: edition
  wrap:
    punctuation: parentheses
  prefix: " "
- delimiter: ""
  wrap:
    punctuation: parentheses
  prefix: " "
  group:
    - term: volume
      form: short
      suffix: " "
    - number: volume
- contributor: translator
  form: long
  name-order: given-first
  label: {term: translator, form: short, placement: suffix}
  wrap:
    punctuation: parentheses
  prefix: " "
- contributor: editor
  form: long
  name-order: given-first
  label: {term: editor, form: short, placement: suffix}
  prefix: ". In "
- title: parent-monograph
  emph: true
  prefix: ", "
- delimiter: ", "
  group:
    - title: parent-serial
      emph: true
    - delimiter: ""
      group:
        - number: volume
          emph: true
        - number: issue
          wrap:
            punctuation: parentheses
- number: pages
  prefix: ", "
  suffix: "."
- variable: publisher
  prefix: ". "
  suffix: "."
- variable: doi
  prefix: "https://doi.org/"
- variable: url
  prefix: " "
- group:
    - variable: archive-name
    - variable: archive-location
      wrap:
        punctuation: parentheses
      prefix: " "
  delimiter: ""
  prefix: ". "
"#;

static APA_CITATION: LazyLock<Vec<TemplateComponent>> = LazyLock::new(|| {
    vec![
        tc_contributor!(Author, Short),
        tc_date!(Issued, Year),
        tc_variable!(Locator),
    ]
});

static APA_BIBLIOGRAPHY: LazyLock<Vec<TemplateComponent>> = LazyLock::new(|| {
    serde_yaml::from_str(APA_BIBLIOGRAPHY_TEMPLATE_YAML)
        .expect("embedded APA bibliography template should parse")
});

/// Embedded citation template for APA style.
///
/// Renders as: (Author, Year)
/// Example: (Smith & Jones, 2024)
pub fn citation() -> Vec<TemplateComponent> {
    APA_CITATION.clone()
}

/// Embedded bibliography template for APA style.
///
/// Renders the full bibliographic entry in APA format:
/// Author, A. A., & Author, B. B. (Year). Title of work. *Journal Title*, *Volume*(Issue), Pages. https://doi.org/xxx
///
/// # Panics
///
/// Panics if the embedded YAML template literal becomes invalid.
pub fn bibliography() -> Vec<TemplateComponent> {
    APA_BIBLIOGRAPHY.clone()
}
