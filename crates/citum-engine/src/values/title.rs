/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Rendering logic for title fields with smartening, form selection,
//! and text-case transforms.

use crate::reference::Reference;
use crate::render::rich_text::render_djot_inline_with_transform;
use crate::values::text_case::{self, apply_text_case, capitalize_first_word};
use crate::values::{ComponentValues, ProcHints, ProcValues, RenderOptions};
use citum_schema::options::titles::TextCase;
use citum_schema::reference::types::{StructuredTitle, Subtitle, Title};
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
        let prev_is_alpha = prev.is_some_and(char::is_alphabetic);
        let prev_is_digit = prev.is_some_and(|c| c.is_ascii_digit());
        let prev_can_close_double_quote = prev.is_some_and(|c| {
            c.is_alphanumeric() || matches!(c, '\'' | '"' | '\u{2019}' | '\u{201D}')
        });
        let next_is_alpha = next.is_some_and(char::is_alphabetic);
        let next_is_digit = next.is_some_and(|c| c.is_ascii_digit());
        let next_is_alnum = next.is_some_and(char::is_alphanumeric);
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
        TitleType::ParentMonograph => {
            if reference.ref_type() == "chapter" || reference.ref_type() == "paper-conference" {
                reference.container_title().and_then(|t| match t {
                    Title::Shorthand(short, _) => Some(short),
                    Title::Single(s) => Some(s),
                    _ => None,
                })
            } else {
                None
            }
        }
        TitleType::ParentSerial => {
            if reference.ref_type().contains("article") || reference.ref_type() == "broadcast" {
                reference.container_title().and_then(|t| match t {
                    Title::Shorthand(short, _) => Some(short),
                    Title::Single(s) => Some(s),
                    _ => None,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn looks_like_djot_markup(value: &str) -> bool {
    value.contains('_')
        || value.contains('*')
        || value.contains("](")
        || value.contains("{.")
        || value.contains('`')
}

/// Build a text-transform closure that applies case transform then smart quotes.
///
/// The closure is used as the Djot text-leaf transform, so `.nocase` spans
/// bypass it automatically via the rich-text renderer.
fn make_case_transform(case: TextCase) -> impl FnMut(&str) -> String {
    let mut seen_alpha = false;
    move |text: &str| {
        let cased = match case {
            TextCase::Sentence | TextCase::SentenceApa | TextCase::SentenceNlm => {
                let lowered = text.to_lowercase();
                if seen_alpha {
                    lowered
                } else {
                    // Capitalize the first alphabetic character we encounter
                    let result = capitalize_first_word(&lowered);
                    if result.chars().any(|c: char| c.is_alphabetic()) {
                        seen_alpha = true;
                    }
                    result
                }
            }
            _ => apply_text_case(text, case),
        };
        smarten_title_quotes(&cased)
    }
}

/// Render a single title part through Djot with case transform + smart quotes.
/// Returns (`rendered_value`, `has_explicit_link`).
fn render_part_with_case<F: crate::render::format::OutputFormat<Output = String>>(
    value: &str,
    fmt: &F,
    case: Option<TextCase>,
) -> (String, bool) {
    if looks_like_djot_markup(value) {
        match case {
            Some(tc) => render_djot_inline_with_transform(value, fmt, make_case_transform(tc)),
            None => render_djot_inline_with_transform(value, fmt, smarten_title_quotes),
        }
    } else {
        let result = match case {
            Some(tc) => smarten_title_quotes(&apply_text_case(value, tc)),
            None => smarten_title_quotes(value),
        };
        (result, false)
    }
}

/// Render a structured title with per-part case transforms.
///
/// For `SentenceApa`, each subtitle gets sentence-case (first word capitalized).
/// For `SentenceNlm`, subtitles are lowercased (no first-word capitalization).
fn render_structured_title<F: crate::render::format::OutputFormat<Output = String>>(
    st: &StructuredTitle,
    fmt: &F,
    case: Option<TextCase>,
) -> (String, bool) {
    let subtitle_case = case.map(|c| match c {
        TextCase::SentenceNlm => TextCase::Lowercase,
        other => other,
    });

    let (main_rendered, mut has_link) = render_part_with_case(&st.main, fmt, case);
    let mut parts = vec![main_rendered];

    let subs: Vec<&str> = match &st.sub {
        Subtitle::String(s) => vec![s.as_str()],
        Subtitle::Vector(v) => v.iter().map(std::string::String::as_str).collect(),
    };

    for sub in subs {
        let (sub_rendered, sub_link) = render_part_with_case(sub, fmt, subtitle_case);
        has_link |= sub_link;
        parts.push(sub_rendered);
    }

    (parts.join(": "), has_link)
}

/// Resolve the effective text-case for this title component.
fn resolve_effective_text_case(
    template: &TemplateTitle,
    reference: &Reference,
    options: &RenderOptions<'_>,
) -> Option<TextCase> {
    // 1. Template-level override takes precedence
    if let Some(tc) = template.rendering.text_case {
        return Some(apply_language_fallback(tc, reference));
    }

    // 2. Global title-category config
    let ref_type = reference.ref_type();
    let lang = reference.language();
    let lang_str = lang.as_deref();

    if let Some(rendering) = crate::render::component::get_title_category_rendering(
        &template.title,
        Some(&ref_type),
        lang_str,
        options.config,
    ) && let Some(tc) = rendering.text_case
    {
        return Some(apply_language_fallback(tc, reference));
    }

    None
}

/// Apply language-aware fallback: non-English → as-is for English-specific transforms.
fn apply_language_fallback(case: TextCase, reference: &Reference) -> TextCase {
    let lang = reference.language();
    text_case::resolve_text_case(case, lang.as_deref())
}

impl ComponentValues for TemplateTitle {
    fn values<F: crate::render::format::OutputFormat<Output = String>>(
        &self,
        reference: &Reference,
        hints: &ProcHints,
        options: &RenderOptions<'_>,
    ) -> Option<ProcValues<F::Output>> {
        if self.disambiguate_only == Some(true) && hints.group_length <= 1 {
            return None;
        }

        if matches!(self.form, Some(TitleForm::Short))
            && let Some(short_title) = parent_short_title(reference, &self.title)
            && !short_title.is_empty()
        {
            let (value, pre_formatted) = if looks_like_djot_markup(&short_title) {
                let (value, _) = render_djot_inline_with_transform(
                    &short_title,
                    &F::default(),
                    smarten_title_quotes,
                );
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

        let title = match self.title {
            TitleType::Primary => reference.title(),
            TitleType::ParentMonograph => match reference {
                Reference::Monograph(_)
                | Reference::CollectionComponent(_)
                | Reference::AudioVisual(_) => reference.container_title(),
                _ => None,
            },
            TitleType::ParentSerial => match reference {
                Reference::SerialComponent(_) | Reference::LegalCase(_) | Reference::Treaty(_) => {
                    reference.container_title()
                }
                _ => None,
            },
            _ => None,
        };

        let effective_case = resolve_effective_text_case(self, reference, options);
        let fmt = F::default();

        // Render title with structured-title-aware case transforms
        let rendered: Option<(String, bool, bool)> = title.as_ref().map(|title| match title {
            Title::Structured(st) => {
                let raw_text = title_text(title, self.form.as_ref());
                let (value, has_link) = render_structured_title(st, &fmt, effective_case);
                let pre_formatted = looks_like_djot_markup(&raw_text);
                (value, has_link, pre_formatted)
            }
            Title::Multilingual(m) => {
                let (mode, preferred_transliteration, preferred_script) =
                    resolve_multilingual_title_config(options);
                let locale_str = options.locale.locale.as_str();

                let complex =
                    citum_schema::reference::types::MultilingualString::Complex(m.clone());
                let value = crate::values::resolve_multilingual_string(
                    &complex,
                    mode,
                    preferred_transliteration,
                    preferred_script,
                    locale_str,
                );
                let (rendered, has_link) = render_part_with_case(&value, &fmt, effective_case);
                let pre_formatted = looks_like_djot_markup(&value);
                (rendered, has_link, pre_formatted)
            }
            _ => {
                let value = title_text(title, self.form.as_ref());
                let (rendered, has_link) = render_part_with_case(&value, &fmt, effective_case);
                let pre_formatted = looks_like_djot_markup(&value);
                (rendered, has_link, pre_formatted)
            }
        });

        rendered
            .filter(|(v, _, _): &(String, bool, bool)| !v.is_empty())
            .map(|(value, has_explicit_link, pre_formatted)| {
                use citum_schema::options::LinkAnchor;
                let url = crate::values::resolve_effective_url(
                    self.links.as_ref(),
                    options.config.links.as_ref(),
                    reference,
                    LinkAnchor::Title,
                );
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

/// Resolve multilingual title config (mode, transliteration, script) from render options.
fn resolve_multilingual_title_config<'a>(
    options: &'a RenderOptions<'a>,
) -> (
    Option<&'a citum_schema::options::MultilingualMode>,
    Option<&'a [String]>,
    Option<&'a String>,
) {
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
    (mode, preferred_transliteration, preferred_script)
}
