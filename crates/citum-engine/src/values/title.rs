/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Rendering logic for title fields with smartening and form selection.
//!
//! This module handles title component rendering, including main titles,
//! container titles, and smart quote handling.

use crate::reference::Reference;
use crate::render::rich_text::render_djot_inline_with_transform;
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_schema::reference::{Parent, types::Title};
use citum_schema::template::{TemplateTitle, TitleForm, TitleType};

/// Converts straight apostrophes and double quotes to curly quotes when the
/// surrounding context is unambiguous.
///
/// Ambiguous characters are preserved as straight quotes so titles containing
/// measurements or other non-quotation uses do not get rewritten arbitrarily.
fn smarten_title_quotes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut it = input.char_indices().peekable();
    let mut prev: Option<char> = None;
    let mut open_single_quotes = 0usize;
    let mut open_double_quotes = 0usize;

    while let Some((_, ch)) = it.next() {
        let next = it.peek().map(|(_, c)| *c);
        let prev_is_alpha = prev.is_some_and(|c| c.is_alphabetic());
        let prev_is_digit = prev.is_some_and(|c| c.is_ascii_digit());
        let prev_can_close_double_quote = prev.is_some_and(|c| {
            c.is_alphanumeric() || matches!(c, '\'' | '"' | '\u{2019}' | '\u{201D}')
        });
        let next_is_alpha = next.is_some_and(|c| c.is_alphabetic());
        let next_is_digit = next.is_some_and(|c| c.is_ascii_digit());
        let next_is_alnum = next.is_some_and(|c| c.is_alphanumeric());
        let prev_opens_quote =
            prev.is_none_or(|c| c.is_whitespace() || "([{\u{2018}\u{201C}'\"".contains(c));
        let next_closes_quote =
            next.is_none_or(|c| c.is_whitespace() || ".,;:!?)]}\u{2019}\u{201D}'\"".contains(c));

        match ch {
            '\'' => {
                if (prev_is_alpha && next_is_alpha) || (prev_opens_quote && next_is_digit) {
                    out.push('\u{2019}');
                } else if prev_opens_quote && next_is_alnum {
                    out.push('\u{2018}');
                    open_single_quotes += 1;
                } else if (open_single_quotes > 0 || prev_is_alpha || prev_is_digit)
                    && next_closes_quote
                {
                    out.push('\u{2019}');
                    open_single_quotes = open_single_quotes.saturating_sub(1);
                } else {
                    out.push('\'');
                }
            }
            '"' => {
                if prev_opens_quote && next_is_alnum {
                    out.push('\u{201C}');
                    open_double_quotes += 1;
                } else if open_double_quotes > 0 && prev_can_close_double_quote && next_closes_quote
                {
                    out.push('\u{201D}');
                    open_double_quotes -= 1;
                } else if prev_is_alpha && next_closes_quote {
                    out.push('\u{201D}');
                } else {
                    out.push('"');
                }
            }
            _ => out.push(ch),
        }

        prev = Some(ch);
    }
    out
}

fn title_text(title: &Title, form: Option<&TitleForm>) -> String {
    match title {
        Title::Shorthand(short, long) => {
            if matches!(form, Some(TitleForm::Short)) {
                short.clone()
            } else {
                long.clone()
            }
        }
        Title::Single(s) => s.clone(),
        _ => title.to_string(),
    }
}

fn parent_short_title(reference: &Reference, title_type: &TitleType) -> Option<String> {
    match title_type {
        TitleType::ParentMonograph => match reference {
            Reference::Monograph(_) => None,
            Reference::CollectionComponent(component) => match &component.parent {
                Parent::Embedded(parent) => parent.short_title.clone(),
                Parent::Id(_) => None,
            },
            _ => None,
        },
        TitleType::ParentSerial => match reference {
            Reference::SerialComponent(component) => match &component.parent {
                Parent::Embedded(parent) => parent.short_title.clone(),
                Parent::Id(_) => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn render_title_inline<F: crate::render::format::OutputFormat<Output = String>>(
    value: &str,
    fmt: &F,
) -> (String, bool) {
    render_djot_inline_with_transform(value, fmt, smarten_title_quotes)
}

fn looks_like_djot_markup(value: &str) -> bool {
    value.contains('_')
        || value.contains('*')
        || value.contains("](")
        || value.contains("{.")
        || value.contains('`')
}

impl ComponentValues for TemplateTitle {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        // Suppress title when disambiguate_only is set and only one work by
        // this author appears in the document (no disambiguation needed).
        // Used by author-class styles like MLA where the title in citations
        // exists solely to resolve same-author ambiguity.
        if self.disambiguate_only == Some(true) && hints.group_length <= 1 {
            return None;
        }

        if matches!(self.form, Some(TitleForm::Short))
            && let Some(short_title) = parent_short_title(reference, &self.title)
            && !short_title.is_empty()
        {
            let (value, pre_formatted) = if looks_like_djot_markup(&short_title) {
                let (value, _) = render_title_inline(&short_title, &F::default());
                (value, true)
            } else {
                (smarten_title_quotes(&short_title), false)
            };
            return Some(ProcValues {
                value,
                prefix: None,
                suffix: None,
                url: None,
                substituted_key: None,
                pre_formatted,
            });
        }

        // Get the raw title based on type and template requirement
        let raw_title = match self.title {
            TitleType::Primary => reference.title(),
            TitleType::ParentSerial => match reference {
                Reference::SerialComponent(r) => match &r.parent {
                    Parent::Embedded(p) => Some(&p.title),
                    _ => None,
                },
                _ => None,
            }
            .cloned(),
            TitleType::ParentMonograph => match reference {
                Reference::Monograph(r) => r.container_title.clone(),
                Reference::CollectionComponent(r) => match &r.parent {
                    Parent::Embedded(p) => p.title.clone(),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };

        // Resolve multilingual title if configured
        let value: Option<String> = raw_title.map(|title| match title {
            Title::Multilingual(m) => {
                let mode = options
                    .config
                    .multilingual
                    .as_ref()
                    .and_then(|ml| ml.title_mode.as_ref());
                let preferred_transliteration = options
                    .config
                    .multilingual
                    .as_ref()
                    .and_then(|ml| ml.preferred_transliteration.as_deref());
                let preferred_script = options
                    .config
                    .multilingual
                    .as_ref()
                    .and_then(|ml| ml.preferred_script.as_ref());
                let locale_str = options.locale.locale.as_str();

                let complex =
                    citum_schema::reference::types::MultilingualString::Complex(m.clone());
                crate::values::resolve_multilingual_string(
                    &complex,
                    mode,
                    preferred_transliteration,
                    preferred_script,
                    locale_str,
                )
            }
            _ => title_text(&title, self.form.as_ref()),
        });

        value.filter(|s: &String| !s.is_empty()).map(|value| {
            use citum_schema::options::LinkAnchor;
            let url = crate::values::resolve_effective_url(
                self.links.as_ref(),
                options.config.links.as_ref(),
                reference,
                LinkAnchor::Title,
            );
            let (value, has_explicit_link, pre_formatted) = if looks_like_djot_markup(&value) {
                let fmt = F::default();
                let (value, has_explicit_link) = render_title_inline(&value, &fmt);
                (value, has_explicit_link, true)
            } else {
                (smarten_title_quotes(&value), false, false)
            };
            ProcValues {
                value,
                prefix: None,
                suffix: None,
                url: if has_explicit_link { None } else { url },
                substituted_key: None,
                pre_formatted,
            }
        })
    }
}
