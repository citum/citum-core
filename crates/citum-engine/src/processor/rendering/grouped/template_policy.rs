/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Bibliography-template policy for `article-journal` and anonymous
//! dictionary/encyclopedia/chapter entries. Decides whether a template needs
//! filtering or rewriting for the current reference, and applies the rewrite.

use super::super::Renderer;
use super::component_predicates::{
    is_article_detail_component, is_doi_component, is_issued_date_component,
    is_parent_container_title_component, is_parent_monograph_title_component,
    is_primary_title_component, is_url_component, is_volume_component, reference_has_doi,
    reference_has_online_access, reference_has_pages,
};
use crate::reference::Reference;
use citum_schema::options::ArticleJournalNoPageFallback;
use citum_schema::template::{SimpleVariable, TemplateComponent};
use std::borrow::Cow;

#[derive(Clone, Copy)]
pub(super) enum ArticleJournalBibliographyMode {
    StandardDetail,
    DoiFallback,
}

#[derive(Clone, Copy)]
pub(super) enum AnonymousEntryBibliographyMode {
    ContainerLed,
    SuppressPrintLike,
}

impl Renderer<'_> {
    pub(super) fn apply_article_journal_bibliography_policy<'a>(
        &self,
        reference: &Reference,
        template: Cow<'a, [TemplateComponent]>,
    ) -> Cow<'a, [TemplateComponent]> {
        let Some(mode) = self.article_journal_bibliography_mode(reference) else {
            return template;
        };

        if !article_journal_template_needs_filter(template.as_ref(), mode) {
            return template;
        }

        Cow::Owned(filter_article_journal_template_components(
            template.as_ref(),
            mode,
        ))
    }

    pub(super) fn apply_anonymous_entry_bibliography_policy<'a>(
        &self,
        reference: &Reference,
        template: Cow<'a, [TemplateComponent]>,
    ) -> Option<Cow<'a, [TemplateComponent]>> {
        let Some(mode) = self.anonymous_entry_bibliography_mode(reference, template.as_ref())
        else {
            return Some(template);
        };

        match mode {
            AnonymousEntryBibliographyMode::ContainerLed => {
                Some(Cow::Owned(rewrite_anonymous_entry_template(
                    template.as_ref(),
                    mode,
                    reference.ref_type().as_str(),
                    reference_has_doi(reference),
                )))
            }
            AnonymousEntryBibliographyMode::SuppressPrintLike => None,
        }
    }

    fn article_journal_bibliography_mode(
        &self,
        reference: &Reference,
    ) -> Option<ArticleJournalBibliographyMode> {
        if reference.ref_type() != "article-journal" {
            return None;
        }

        let fallback = self
            .bibliography_config
            .as_ref()?
            .article_journal
            .as_ref()?
            .no_page_fallback?;

        match fallback {
            ArticleJournalNoPageFallback::Doi => {
                if reference_has_pages(reference) {
                    Some(ArticleJournalBibliographyMode::StandardDetail)
                } else if reference_has_doi(reference) {
                    Some(ArticleJournalBibliographyMode::DoiFallback)
                } else {
                    None
                }
            }
        }
    }

    fn anonymous_entry_bibliography_mode(
        &self,
        reference: &Reference,
        template: &[TemplateComponent],
    ) -> Option<AnonymousEntryBibliographyMode> {
        if !matches!(
            reference.ref_type().as_str(),
            "entry-dictionary" | "entry-encyclopedia" | "chapter"
        ) {
            return None;
        }

        if reference.ref_type() == "chapter" && !template_has_dictionary_entry_shape(template) {
            return None;
        }

        if self.reference_has_visible_author(reference) {
            return None;
        }

        if !template_has_primary_title(template) || !template_has_parent_container_title(template) {
            return None;
        }

        if reference_has_online_access(reference) {
            if template_has_pattern_message(template) {
                return None;
            }
            Some(AnonymousEntryBibliographyMode::ContainerLed)
        } else {
            Some(AnonymousEntryBibliographyMode::SuppressPrintLike)
        }
    }

    fn reference_has_visible_author(&self, reference: &Reference) -> bool {
        reference
            .author()
            .is_some_and(|author| !self.resolve_contributor_names(&author).is_empty())
    }
}

fn filter_article_journal_template_components(
    components: &[TemplateComponent],
    mode: ArticleJournalBibliographyMode,
) -> Vec<TemplateComponent> {
    components
        .iter()
        .filter_map(|component| filter_article_journal_template_component(component, mode))
        .collect()
}

fn filter_article_journal_template_component(
    component: &TemplateComponent,
    mode: ArticleJournalBibliographyMode,
) -> Option<TemplateComponent> {
    if should_suppress_article_journal_component(component, mode) {
        return None;
    }

    match component {
        TemplateComponent::Group(list) => {
            let mut filtered = list.clone();
            filtered.group = filter_article_journal_template_components(&list.group, mode);
            (!filtered.group.is_empty()).then_some(TemplateComponent::Group(filtered))
        }
        _ => Some(component.clone()),
    }
}

fn article_journal_template_needs_filter(
    components: &[TemplateComponent],
    mode: ArticleJournalBibliographyMode,
) -> bool {
    components
        .iter()
        .any(|component| article_journal_component_needs_filter(component, mode))
}

fn rewrite_anonymous_entry_template(
    template: &[TemplateComponent],
    mode: AnonymousEntryBibliographyMode,
    ref_type: &str,
    prefer_doi: bool,
) -> Vec<TemplateComponent> {
    match mode {
        AnonymousEntryBibliographyMode::ContainerLed => {
            let mut rewritten = Vec::new();

            if let Some(container_title) =
                find_preferred_parent_container_component(template, ref_type)
            {
                rewritten.push(container_title.clone());
            }
            if let Some(issued) = find_first_component(template, is_issued_date_component) {
                rewritten.push(issued.clone());
            }
            if let Some(primary_title) = find_first_component(template, is_primary_title_component)
            {
                rewritten.push(primary_title.clone());
            }
            if let Some(volume) = find_first_component(template, is_volume_component) {
                rewritten.push(volume.clone());
            }

            if prefer_doi {
                if let Some(doi) = find_first_component(template, is_doi_component) {
                    rewritten.push(doi.clone());
                }
            } else if let Some(url) = find_first_component(template, is_url_component) {
                rewritten.push(url.clone());
            }

            if rewritten.is_empty() {
                template.to_vec()
            } else {
                rewritten
            }
        }
        AnonymousEntryBibliographyMode::SuppressPrintLike => template.to_vec(),
    }
}

fn find_first_component(
    template: &[TemplateComponent],
    predicate: impl Fn(&TemplateComponent) -> bool,
) -> Option<&TemplateComponent> {
    template.iter().find(|component| predicate(component))
}

fn find_preferred_parent_container_component<'a>(
    template: &'a [TemplateComponent],
    ref_type: &str,
) -> Option<&'a TemplateComponent> {
    if matches!(
        ref_type,
        "chapter" | "entry-dictionary" | "entry-encyclopedia"
    ) && let Some(parent_monograph) =
        find_first_component(template, is_parent_monograph_title_component)
    {
        return Some(parent_monograph);
    }

    find_first_component(template, is_parent_container_title_component)
}

fn article_journal_component_needs_filter(
    component: &TemplateComponent,
    mode: ArticleJournalBibliographyMode,
) -> bool {
    if should_suppress_article_journal_component(component, mode) {
        return true;
    }

    match component {
        TemplateComponent::Group(group) => {
            article_journal_template_needs_filter(&group.group, mode)
        }
        _ => false,
    }
}

fn should_suppress_article_journal_component(
    component: &TemplateComponent,
    mode: ArticleJournalBibliographyMode,
) -> bool {
    match mode {
        ArticleJournalBibliographyMode::StandardDetail => is_doi_component(component),
        ArticleJournalBibliographyMode::DoiFallback => is_article_detail_component(component),
    }
}

fn template_has_primary_title(template: &[TemplateComponent]) -> bool {
    template.iter().any(is_primary_title_component)
}

fn template_has_parent_container_title(template: &[TemplateComponent]) -> bool {
    template.iter().any(is_parent_container_title_component)
}

fn template_has_dictionary_entry_shape(template: &[TemplateComponent]) -> bool {
    template.iter().any(|component| {
        matches!(
            component,
            TemplateComponent::Variable(variable) if variable.variable == SimpleVariable::Version
        )
    })
}

fn template_has_pattern_message(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_pattern_message)
}

fn component_has_pattern_message(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Message(message) => {
            message.message.starts_with("pattern.")
                || message
                    .args
                    .values()
                    .filter_map(|source| source.as_template_component())
                    .any(|child| component_has_pattern_message(&child))
        }
        TemplateComponent::Group(group) => group.group.iter().any(component_has_pattern_message),
        _ => false,
    }
}
