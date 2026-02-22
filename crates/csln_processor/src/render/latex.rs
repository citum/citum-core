/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! LaTeX output format.

use super::format::OutputFormat;
use csln_core::template::WrapPunctuation;

/// LaTeX renderer.
#[derive(Debug, Clone, Default)]
pub struct Latex;

impl OutputFormat for Latex {
    type Output = String;

    fn text(&self, s: &str) -> Self::Output {
        let mut res = String::with_capacity(s.len() + 10);
        for c in s.chars() {
            match c {
                '\\' => res.push_str(r"\textbackslash{}"),
                '{' => res.push_str(r"\{"),
                '}' => res.push_str(r"\}"),
                '$' => res.push_str(r"\$"),
                '&' => res.push_str(r"\&"),
                '#' => res.push_str(r"\#"),
                '_' => res.push_str(r"\_"),
                '%' => res.push_str(r"\%"),
                '~' => res.push_str(r"\textasciitilde{}"),
                '^' => res.push_str(r"\textasciicircum{}"),
                _ => res.push(c),
            }
        }
        res
    }

    fn join(&self, items: Vec<Self::Output>, delimiter: &str) -> Self::Output {
        items.join(&self.text(delimiter))
    }

    fn finish(&self, output: Self::Output) -> String {
        // Escape any bare & not already preceded by backslash.
        // Locale terms (e.g. the & from AndOptions::Symbol) bypass text() and
        // arrive here unescaped; this final pass makes the output valid LaTeX.
        let mut result = String::with_capacity(output.len() + 4);
        let mut prev = '\0';
        for c in output.chars() {
            if c == '&' && prev != '\\' {
                result.push_str(r"\&");
            } else {
                result.push(c);
            }
            prev = c;
        }
        result
    }

    fn emph(&self, content: Self::Output) -> Self::Output {
        format!(r"\textit{{{}}}", content)
    }

    fn strong(&self, content: Self::Output) -> Self::Output {
        format!(r"\textbf{{{}}}", content)
    }

    fn small_caps(&self, content: Self::Output) -> Self::Output {
        format!(r"\textsc{{{}}}", content)
    }

    fn quote(&self, content: Self::Output) -> Self::Output {
        format!("``{}''", content)
    }

    fn affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{}{}{}", self.text(prefix), content, self.text(suffix))
    }

    fn inner_affix(&self, prefix: &str, content: Self::Output, suffix: &str) -> Self::Output {
        format!("{}{}{}", self.text(prefix), content, self.text(suffix))
    }

    fn wrap_punctuation(&self, wrap: &WrapPunctuation, content: Self::Output) -> Self::Output {
        match wrap {
            WrapPunctuation::Parentheses => format!("({})", content),
            WrapPunctuation::Brackets => format!("[{}]", content),
            WrapPunctuation::Quotes => self.quote(content),
            WrapPunctuation::None => content,
        }
    }

    fn semantic(&self, _class: &str, content: Self::Output) -> Self::Output {
        // In LaTeX, we could use custom commands if we wanted semantic tagging
        // For now, just return content
        content
    }

    fn link(&self, url: &str, content: Self::Output) -> Self::Output {
        format!(r"\href{{{}}}{{{}}}", url, content)
    }

    fn bibliography(&self, entries: Vec<Self::Output>) -> Self::Output {
        entries.join("\\par\\vspace{0.5em}")
    }

    fn entry(
        &self,
        _id: &str,
        content: Self::Output,
        _url: Option<&str>,
        metadata: &super::format::ProcEntryMetadata,
    ) -> Self::Output {
        // When metadata is available, delegate the full entry layout to
        // \cslntooltip (defined in csln.sty).  With tooltips=true it wraps
        // content in a \parbox so \pdftooltip gets a properly-sized box;
        // with tooltips=false it is a passthrough that adds \hangindent itself.
        // When metadata is absent, fall back to the plain hanging-indent format.
        match build_pdf_tooltip(metadata) {
            Some(tooltip_text) => format!("\\cslntooltip{{{}}}{{{}}}", content, tooltip_text),
            None => format!("\\noindent\\hangindent=2em\\hangafter=1 {}", content),
        }
    }
}

/// Build a PDF tooltip string from entry metadata.
/// Format: "Author (Year). Title" with parts omitted if None.
fn build_pdf_tooltip(metadata: &super::format::ProcEntryMetadata) -> Option<String> {
    let has_content =
        metadata.author.is_some() || metadata.year.is_some() || metadata.title.is_some();
    if !has_content {
        return None;
    }

    let mut parts = Vec::new();

    if let Some(author) = &metadata.author {
        parts.push(escape_tooltip_text(author));
    }

    if let Some(year) = &metadata.year {
        let year_str = format!("({})", escape_tooltip_text(year));
        parts.push(year_str);
    }

    if let Some(title) = &metadata.title {
        parts.push(escape_tooltip_text(title));
    }

    let tooltip = parts.join(". ");
    if tooltip.is_empty() {
        None
    } else {
        Some(tooltip)
    }
}

/// Escape plain-text metadata for use in LaTeX PDF annotations.
/// Replaces special characters with spaces to avoid PDF/LaTeX parsing issues.
fn escape_tooltip_text(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '{' | '}' | '\\' | '%' | '#' | '$' | '&' | '_' | '^' | '~' => {
                result.push(' ');
            }
            _ => result.push(c),
        }
    }
    // Collapse multiple spaces into one
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
