/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use crate::render::component::{ProcTemplateComponent, render_component_with_format};
    use crate::render::djot::Djot;
    use crate::render::html::Html;
    use crate::render::typst::Typst;
    use citum_schema::{tc_contributor, tc_title, tc_variable};

    #[test]
    fn test_html_title() {
        let component = ProcTemplateComponent {
            template_component: tc_title!(Primary, emph = true),
            value: "My Title".to_string(),
            ..Default::default()
        };

        let result = render_component_with_format::<Html>(&component);
        assert_eq!(
            result,
            r#"<span class="citum-title"><i>My Title</i></span>"#
        );
    }

    #[test]
    fn test_html_contributor() {
        let component = ProcTemplateComponent {
            template_component: tc_contributor!(Author, Long, small_caps = true),
            value: "Smith".to_string(),
            ..Default::default()
        };

        let result = render_component_with_format::<Html>(&component);
        assert_eq!(
            result,
            r#"<span class="citum-author"><span style="font-variant:small-caps">Smith</span></span>"#
        );
    }

    #[test]
    fn test_html_semantic_attributes_include_template_index() {
        let component = ProcTemplateComponent {
            template_component: tc_title!(Primary, emph = true),
            template_index: Some(2),
            value: "My Title".to_string(),
            ..Default::default()
        };

        let result = render_component_with_format::<Html>(&component);
        assert_eq!(
            result,
            r#"<span class="citum-title" data-index="2"><i>My Title</i></span>"#
        );
    }

    #[test]
    fn test_djot_title() {
        let component = ProcTemplateComponent {
            template_component: tc_title!(Primary, emph = true),
            value: "My Title".to_string(),
            ..Default::default()
        };

        let result = render_component_with_format::<Djot>(&component);
        assert_eq!(result, "[_My Title_]{.citum-title}");
    }

    #[test]
    fn test_djot_contributor() {
        let component = ProcTemplateComponent {
            template_component: tc_contributor!(Author, Long, small_caps = true),
            value: "Smith".to_string(),
            ..Default::default()
        };

        let result = render_component_with_format::<Djot>(&component);
        assert_eq!(result, "[[Smith]{.small-caps}]{.citum-author}");
    }

    #[test]
    fn test_html_link() {
        let component = ProcTemplateComponent {
            template_component: tc_variable!(Url),
            value: "https://example.com".to_string(),
            url: Some("https://example.com".to_string()),
            ..Default::default()
        };

        let result = render_component_with_format::<Html>(&component);
        assert_eq!(
            result,
            r#"<span class="citum-url"><a href="https://example.com">https://example.com</a></span>"#
        );
    }

    #[test]
    fn test_html_link_percent_encodes_href_breakout_chars() {
        let component = ProcTemplateComponent {
            template_component: tc_variable!(Url),
            value: "label".to_string(),
            url: Some(r#"" onmouseover="alert(1)"#.to_string()),
            ..Default::default()
        };

        let result = render_component_with_format::<Html>(&component);
        assert_eq!(
            result,
            r#"<span class="citum-url"><a href="%22%20onmouseover=%22alert(1)">label</a></span>"#
        );
    }

    #[test]
    fn test_html_title_link_doi() {
        use citum_schema::{
            options::{LinkAnchor, LinkTarget, LinksConfig},
            template::{TemplateTitle, TitleType},
        };
        let component = ProcTemplateComponent {
            template_component: citum_schema::template::TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                links: Some(LinksConfig {
                    target: Some(LinkTarget::Doi),
                    anchor: Some(LinkAnchor::Title),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            value: "My Title".to_string(),
            url: Some("https://doi.org/10.1001/test".to_string()),
            ..Default::default()
        };

        let result = render_component_with_format::<Html>(&component);
        assert_eq!(
            result,
            r#"<span class="citum-title"><a href="https://doi.org/10.1001/test">My Title</a></span>"#
        );
    }

    #[test]
    fn test_typst_title() {
        let component = ProcTemplateComponent {
            template_component: tc_title!(Primary, emph = true),
            value: "My Title".to_string(),
            ..Default::default()
        };

        let result = render_component_with_format::<Typst>(&component);
        assert_eq!(result, "_My Title_");
    }

    #[test]
    fn test_typst_contributor_small_caps() {
        let component = ProcTemplateComponent {
            template_component: tc_contributor!(Author, Long, small_caps = true),
            template_index: Some(4),
            value: "Smith".to_string(),
            ..Default::default()
        };

        let result = render_component_with_format::<Typst>(&component);
        assert_eq!(result, "#smallcaps[Smith]");
    }

    #[test]
    fn test_typst_link() {
        let component = ProcTemplateComponent {
            template_component: tc_variable!(Url),
            value: "https://example.com".to_string(),
            url: Some("https://example.com".to_string()),
            ..Default::default()
        };

        let result = render_component_with_format::<Typst>(&component);
        assert_eq!(
            result,
            r#"#link("https://example.com")[https://example.com]"#
        );
    }

    #[test]
    fn test_typst_title_link_doi() {
        use citum_schema::{
            options::{LinkAnchor, LinkTarget, LinksConfig},
            template::{TemplateTitle, TitleType},
        };

        let component = ProcTemplateComponent {
            template_component: citum_schema::template::TemplateComponent::Title(TemplateTitle {
                title: TitleType::Primary,
                links: Some(LinksConfig {
                    target: Some(LinkTarget::Doi),
                    anchor: Some(LinkAnchor::Title),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            value: "My Title".to_string(),
            url: Some("https://doi.org/10.1001/test".to_string()),
            ..Default::default()
        };

        let result = render_component_with_format::<Typst>(&component);
        assert_eq!(result, r#"#link("https://doi.org/10.1001/test")[My Title]"#);
    }
}
