/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Djot and org-mode inline markup rendering for free-text fields.

use super::format::OutputFormat;
use jotdown::{Attributes, Container, Event, Parser};

#[derive(Default)]
struct DjotFrame {
    children: Vec<String>,
    classes: Vec<String>,
    link_url: Option<String>,
    has_explicit_link: bool,
    last_char: Option<char>,
}

impl DjotFrame {
    fn push_rendered(&mut self, rendered: String, logical_last_char: Option<char>) {
        self.children.push(rendered);
        if let Some(ch) = logical_last_char {
            self.last_char = Some(ch);
        }
    }

    fn prev_opens_quote(&self) -> bool {
        self.last_char
            .is_none_or(|c| c.is_whitespace() || "([{\u{2018}\u{201C}'\"".contains(c))
    }
}

fn span_classes(attrs: Option<&Attributes>) -> Vec<String> {
    attrs
        .into_iter()
        .flat_map(|attrs| attrs.iter())
        .filter_map(|(kind, val)| {
            use jotdown::AttributeKind;
            if matches!(kind, AttributeKind::Class) {
                Some(val.to_string())
            } else {
                None
            }
        })
        .flat_map(|classes| {
            classes
                .split_whitespace()
                .map(|class| class.to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn render_djot_inline_internal<F, G>(src: &str, fmt: &F, mut transform_text: G) -> (String, bool)
where
    F: OutputFormat<Output = String>,
    G: FnMut(&str) -> String,
{
    let parser = Parser::new(src);
    let mut stack = vec![DjotFrame::default()];

    for event in parser {
        match event {
            Event::Start(container, attrs) => {
                let link_url = if let Container::Link(url, _) = &container {
                    Some(url.to_string())
                } else {
                    None
                };
                stack.push(DjotFrame {
                    classes: span_classes(Some(&attrs)),
                    has_explicit_link: link_url.is_some(),
                    link_url,
                    ..Default::default()
                });
            }
            Event::End(container) => {
                if let Some(frame) = stack.pop() {
                    let inner_text = frame.children.join("");
                    let formatted = match container {
                        Container::Emphasis => fmt.emph(inner_text),
                        Container::Strong => fmt.strong(inner_text),
                        Container::Link(_, _) => {
                            if let Some(url) = frame.link_url.as_deref() {
                                fmt.link(url, inner_text)
                            } else {
                                inner_text
                            }
                        }
                        Container::Span => {
                            if frame
                                .classes
                                .iter()
                                .any(|class| class == "smallcaps" || class == "small-caps")
                            {
                                fmt.small_caps(inner_text)
                            } else {
                                inner_text
                            }
                        }
                        _ => inner_text,
                    };
                    if let Some(parent) = stack.last_mut() {
                        parent.push_rendered(formatted, frame.last_char);
                        parent.has_explicit_link |= frame.has_explicit_link;
                    }
                }
            }
            Event::Str(s) => {
                if let Some(frame) = stack.last_mut() {
                    let transformed = transform_text(s.as_ref());
                    frame.push_rendered(fmt.text(&transformed), transformed.chars().last());
                }
            }
            Event::Symbol(sym) => {
                if let Some(frame) = stack.last_mut() {
                    frame.push_rendered(fmt.text(sym.as_ref()), sym.chars().last());
                }
            }
            Event::LeftSingleQuote => {
                if let Some(frame) = stack.last_mut() {
                    frame.push_rendered(fmt.text("\u{2018}"), Some('\u{2018}'));
                }
            }
            Event::RightSingleQuote => {
                if let Some(frame) = stack.last_mut() {
                    let quote = if frame.prev_opens_quote() {
                        '\u{2018}'
                    } else {
                        '\u{2019}'
                    };
                    frame.push_rendered(fmt.text(&quote.to_string()), Some(quote));
                }
            }
            Event::LeftDoubleQuote => {
                if let Some(frame) = stack.last_mut() {
                    frame.push_rendered(fmt.text("\u{201C}"), Some('\u{201C}'));
                }
            }
            Event::RightDoubleQuote => {
                if let Some(frame) = stack.last_mut() {
                    let quote = if frame.prev_opens_quote() {
                        '\u{201C}'
                    } else {
                        '\u{201D}'
                    };
                    frame.push_rendered(fmt.text(&quote.to_string()), Some(quote));
                }
            }
            Event::Softbreak | Event::Hardbreak => {
                if let Some(frame) = stack.last_mut() {
                    frame.push_rendered(fmt.text(" "), Some(' '));
                }
            }
            _ => {}
        }
    }

    stack
        .into_iter()
        .next()
        .map(|frame| (frame.children.join(""), frame.has_explicit_link))
        .unwrap_or_default()
}

/// Render djot inline markup and map events to OutputFormat methods.
///
/// Parses the input as djot inline markup and transforms container and text
/// events into formatted output. Block-level containers are collapsed to their
/// text content. Inline containers (emphasis, strong, links, etc.) are rendered
/// using the format's methods.
///
/// # Arguments
/// * `src` - Input string with djot inline markup
/// * `fmt` - OutputFormat implementation for rendering
///
/// # Returns
/// Formatted string with markup applied according to the OutputFormat's methods
pub fn render_djot_inline<F: OutputFormat<Output = String>>(src: &str, fmt: &F) -> String {
    render_djot_inline_internal(src, fmt, str::to_string).0
}

/// Render djot inline markup while transforming text leaves and returning link metadata.
pub(crate) fn render_djot_inline_with_transform<F, G>(
    src: &str,
    fmt: &F,
    transform_text: G,
) -> (String, bool)
where
    F: OutputFormat<Output = String>,
    G: FnMut(&str) -> String,
{
    render_djot_inline_internal(src, fmt, transform_text)
}

/// Render org-mode inline markup by walking the orgize event stream.
///
/// Parses `src` as org-mode and maps inline elements to `OutputFormat` methods:
/// bold (`*text*`) → `strong`, italic (`/text/`) → `emph`, verbatim/code →
/// `text` (stripped), links (`[[url][desc]]`) → `link`, plain text → `text`.
/// Container elements (Bold, Italic) are collected via a stack so nested
/// markup is handled correctly.
pub fn render_org_inline<F: OutputFormat<Output = String>>(src: &str, fmt: &F) -> String {
    use orgize::Event;
    use orgize::Org;
    use orgize::elements::Element;

    let org = Org::parse(src);
    // Stack of (tag, accumulated_children) for open containers.
    // Tags: 0 = Bold, 1 = Italic, 2 = root paragraph accumulator.
    let mut stack: Vec<(u8, String)> = vec![(2, String::new())];

    for event in org.iter() {
        match event {
            Event::Start(Element::Bold) => stack.push((0, String::new())),
            Event::Start(Element::Italic) => stack.push((1, String::new())),
            Event::End(Element::Bold) => {
                if let Some((0, inner)) = stack.pop() {
                    let rendered = fmt.strong(inner);
                    if let Some(top) = stack.last_mut() {
                        top.1.push_str(&rendered);
                    }
                }
            }
            Event::End(Element::Italic) => {
                if let Some((1, inner)) = stack.pop() {
                    let rendered = fmt.emph(inner);
                    if let Some(top) = stack.last_mut() {
                        top.1.push_str(&rendered);
                    }
                }
            }
            Event::Start(Element::Link(link)) => {
                let desc = link.desc.as_deref().unwrap_or(&link.path);
                let rendered = fmt.link(&link.path, fmt.text(desc));
                if let Some(top) = stack.last_mut() {
                    top.1.push_str(&rendered);
                }
            }
            Event::Start(Element::Text { value }) => {
                if let Some(top) = stack.last_mut() {
                    top.1.push_str(&fmt.text(value));
                }
            }
            Event::Start(Element::Verbatim { value }) | Event::Start(Element::Code { value }) => {
                if let Some(top) = stack.last_mut() {
                    top.1.push_str(&fmt.text(value));
                }
            }
            _ => {}
        }
    }

    stack.into_iter().next().map(|(_, s)| s).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::html::Html;
    use crate::render::plain::PlainText;
    use crate::render::typst::Typst;

    #[test]
    fn test_djot_emphasis_plain() {
        let fmt = PlainText;
        let result = render_djot_inline("_foo_", &fmt);
        // PlainText.emph() wraps content in _..._
        assert_eq!(result, "_foo_");
    }

    #[test]
    fn test_djot_strong_single_asterisk() {
        let fmt = PlainText;
        // jotdown uses * for strong (bold), not **
        let result = render_djot_inline("*bar*", &fmt);
        // PlainText.strong() wraps content in **...**
        assert_eq!(result, "**bar**");
    }

    #[test]
    fn test_djot_unicode_math() {
        let fmt = PlainText;
        let result = render_djot_inline("H₂O", &fmt);
        assert_eq!(result, "H₂O");
    }

    #[test]
    fn test_djot_plain_no_markup() {
        let fmt = PlainText;
        let result = render_djot_inline("plain text with no markup", &fmt);
        assert_eq!(result, "plain text with no markup");
    }

    #[test]
    fn test_djot_combined_formatting() {
        let fmt = PlainText;
        // In djot, _text_ is emphasis and *text* is strong
        let result = render_djot_inline("_emphasized *bold* text_", &fmt);
        // Emphasis wraps in _..._. Inside that, strong wraps in **...**
        assert_eq!(result, "_emphasized **bold** text_");
    }

    #[test]
    fn test_djot_link() {
        let fmt = PlainText;
        // In djot, [text](url) is a link
        let result = render_djot_inline("[click here](https://example.com)", &fmt);
        // PlainText.link() just renders the link text (ignores URL)
        assert_eq!(result, "click here");
    }

    #[test]
    fn test_djot_nested_formatting_preserves_typst_markup() {
        let fmt = Typst;
        let result = render_djot_inline("_emphasized *bold* text_", &fmt);
        assert_eq!(result, "_emphasized *bold* text_");
    }

    #[test]
    fn test_djot_nested_link_preserves_inner_markup_html() {
        let fmt = Html;
        let result = render_djot_inline("[_linked emphasis_](https://example.com)", &fmt);
        assert_eq!(
            result,
            r#"<a href="https://example.com"><i>linked emphasis</i></a>"#
        );
    }

    #[test]
    fn test_djot_quotes_inside_emphasis_open_correctly() {
        let fmt = PlainText;
        let result = render_djot_inline("_\"Parmenides\" dialogue_", &fmt);
        assert_eq!(result, "_“Parmenides” dialogue_");
    }

    #[test]
    fn test_org_plain_text() {
        let fmt = PlainText;
        let result = render_org_inline("plain text with no markup", &fmt);
        assert_eq!(result, "plain text with no markup");
    }

    #[test]
    fn test_org_bold() {
        let fmt = PlainText;
        // PlainText.strong() wraps in **...**
        let result = render_org_inline("*bold*", &fmt);
        assert_eq!(result, "**bold**");
    }

    #[test]
    fn test_org_italic() {
        let fmt = PlainText;
        // PlainText.emph() wraps in _..._
        let result = render_org_inline("/italic/", &fmt);
        assert_eq!(result, "_italic_");
    }
}
