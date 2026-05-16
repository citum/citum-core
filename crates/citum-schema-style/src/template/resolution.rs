/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Template variant resolution logic.

use std::collections::HashSet;

use crate::template::{
    Template, TemplateComponent, TemplateComponentSelector, TemplateVariant, TemplateVariantDiff,
    TemplateVariants, TypeSelector,
};
use crate::{CitationSpec, ResolutionError, Style};

pub(crate) struct StyleVariantContext {
    citation: Option<CitationVariantContext>,
    bibliography: Option<TemplateVariants>,
}

#[derive(Clone, Default)]
pub(crate) struct CitationVariantContext {
    type_variants: Option<TemplateVariants>,
    integral: Option<Box<CitationVariantContext>>,
    non_integral: Option<Box<CitationVariantContext>>,
    subsequent: Option<Box<CitationVariantContext>>,
    ibid: Option<Box<CitationVariantContext>>,
}

pub(crate) fn inherited_variant_context(style: &Style) -> Option<StyleVariantContext> {
    let context = StyleVariantContext {
        citation: style.citation.as_ref().map(citation_variant_context),
        bibliography: style
            .bibliography
            .as_ref()
            .and_then(|bib| bib.type_variants.clone()),
    };
    (context.citation.is_some() || context.bibliography.is_some()).then_some(context)
}

pub(crate) fn citation_variant_context(spec: &CitationSpec) -> CitationVariantContext {
    CitationVariantContext {
        type_variants: spec.type_variants.clone(),
        integral: spec
            .integral
            .as_deref()
            .map(citation_variant_context)
            .map(Box::new),
        non_integral: spec
            .non_integral
            .as_deref()
            .map(citation_variant_context)
            .map(Box::new),
        subsequent: spec
            .subsequent
            .as_deref()
            .map(citation_variant_context)
            .map(Box::new),
        ibid: spec
            .ibid
            .as_deref()
            .map(citation_variant_context)
            .map(Box::new),
    }
}

pub(crate) fn resolve_style_template_variants(
    style: &mut Style,
    inherited: Option<&StyleVariantContext>,
) -> Result<(), ResolutionError> {
    if let Some(citation) = style.citation.as_mut() {
        resolve_citation_template_variants(
            citation,
            inherited.and_then(|context| context.citation.as_ref()),
            "citation",
            None,
        )?;
    }
    if let Some(bibliography) = style.bibliography.as_mut() {
        let section_template = bibliography.resolve_template();
        resolve_template_variant_map(
            bibliography.type_variants.as_mut(),
            section_template.as_deref(),
            inherited.and_then(|context| context.bibliography.as_ref()),
            "bibliography.type-variants",
        )?;
    }
    Ok(())
}

pub(crate) fn resolve_citation_template_variants(
    spec: &mut CitationSpec,
    inherited: Option<&CitationVariantContext>,
    location: &str,
    fallback_template: Option<&[TemplateComponent]>,
) -> Result<(), ResolutionError> {
    let section_template = spec.resolve_template();
    let effective_section_template = section_template.as_deref().or(fallback_template);
    resolve_template_variant_map(
        spec.type_variants.as_mut(),
        effective_section_template,
        inherited.and_then(|context| context.type_variants.as_ref()),
        &format!("{location}.type-variants"),
    )?;

    for (name, child, inherited_child) in [
        (
            "integral",
            spec.integral.as_deref_mut(),
            inherited.and_then(|context| context.integral.as_deref()),
        ),
        (
            "non-integral",
            spec.non_integral.as_deref_mut(),
            inherited.and_then(|context| context.non_integral.as_deref()),
        ),
        (
            "subsequent",
            spec.subsequent.as_deref_mut(),
            inherited.and_then(|context| context.subsequent.as_deref()),
        ),
        (
            "ibid",
            spec.ibid.as_deref_mut(),
            inherited.and_then(|context| context.ibid.as_deref()),
        ),
    ] {
        if let Some(child) = child {
            resolve_citation_template_variants(
                child,
                inherited_child,
                &format!("{location}.{name}"),
                effective_section_template,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn resolve_template_variant_map(
    variants: Option<&mut TemplateVariants>,
    section_template: Option<&[TemplateComponent]>,
    inherited: Option<&TemplateVariants>,
    location: &str,
) -> Result<(), ResolutionError> {
    let Some(variants) = variants else {
        return Ok(());
    };
    let original = variants.clone();
    let mut resolved = TemplateVariants::new();
    let mut visiting = HashSet::new();

    for selector in original.keys() {
        let template = resolve_template_variant(
            selector,
            &original,
            &mut resolved,
            inherited,
            section_template,
            location,
            &mut visiting,
        )?;
        resolved.insert(selector.clone(), TemplateVariant::Full(template));
    }

    *variants = resolved;
    Ok(())
}

pub(crate) fn resolve_template_variant(
    selector: &TypeSelector,
    original: &TemplateVariants,
    resolved: &mut TemplateVariants,
    inherited: Option<&TemplateVariants>,
    section_template: Option<&[TemplateComponent]>,
    location: &str,
    visiting: &mut HashSet<TypeSelector>,
) -> Result<Template, ResolutionError> {
    let variant_location = format!("{location}[{selector}]");
    if let Some(template) = resolved
        .get(selector)
        .and_then(TemplateVariant::as_template)
        .map(<[TemplateComponent]>::to_vec)
    {
        return Ok(template);
    }

    if !visiting.insert(selector.clone()) {
        return Err(ResolutionError::TemplateVariantCycle {
            location: variant_location,
            selector: selector.to_string(),
        });
    }

    let variant =
        original
            .get(selector)
            .ok_or_else(|| ResolutionError::MissingTemplateVariantParent {
                location: variant_location.clone(),
                selector: selector.to_string(),
            })?;

    let template = match variant {
        TemplateVariant::Full(template) => template.clone(),
        TemplateVariant::Diff(diff) => {
            let mut parent = resolve_variant_parent_template(
                selector,
                diff,
                original,
                resolved,
                inherited,
                section_template,
                &variant_location,
                visiting,
            )?;
            apply_template_variant_diff(&mut parent, diff, &variant_location)?;
            parent
        }
    };

    visiting.remove(selector);
    Ok(template)
}

#[allow(
    clippy::too_many_arguments,
    reason = "Template variant resolution needs explicit inherited and local context."
)]
pub(crate) fn resolve_variant_parent_template(
    selector: &TypeSelector,
    diff: &TemplateVariantDiff,
    original: &TemplateVariants,
    resolved: &mut TemplateVariants,
    inherited: Option<&TemplateVariants>,
    section_template: Option<&[TemplateComponent]>,
    location: &str,
    visiting: &mut HashSet<TypeSelector>,
) -> Result<Template, ResolutionError> {
    if let Some(parent_selector) = &diff.extends {
        if parent_selector != selector && original.contains_key(parent_selector) {
            return resolve_template_variant(
                parent_selector,
                original,
                resolved,
                inherited,
                section_template,
                location,
                visiting,
            );
        }
        return inherited
            .and_then(|variants| variants.get(parent_selector))
            .and_then(TemplateVariant::as_template)
            .map(<[TemplateComponent]>::to_vec)
            .ok_or_else(|| ResolutionError::MissingTemplateVariantParent {
                location: location.to_string(),
                selector: parent_selector.to_string(),
            });
    }

    inherited
        .and_then(|variants| variants.get(selector))
        .and_then(TemplateVariant::as_template)
        .map(<[TemplateComponent]>::to_vec)
        .or_else(|| section_template.map(<[TemplateComponent]>::to_vec))
        .ok_or_else(|| ResolutionError::MissingTemplateVariantParent {
            location: location.to_string(),
            selector: selector.to_string(),
        })
}

pub(crate) fn apply_template_variant_diff(
    template: &mut Template,
    diff: &TemplateVariantDiff,
    location: &str,
) -> Result<(), ResolutionError> {
    for op in &diff.modify {
        let index = find_required_anchor(template, &op.match_selector, location)?;
        if let Some(component) = template.get_mut(index) {
            if let Some(label_form) = op.label_form.clone()
                && let TemplateComponent::Number(number) = component
            {
                number.label_form = Some(label_form);
            }
            component.rendering_mut().merge(&op.rendering);
        }
    }
    for op in &diff.remove {
        let index = find_required_anchor(template, &op.match_selector, location)?;
        template.remove(index);
    }
    for op in &diff.add {
        let anchor = match (&op.before, &op.after) {
            (Some(selector), None) => Some((selector, false)),
            (None, Some(selector)) => Some((selector, true)),
            _ => {
                return Err(ResolutionError::InvalidTemplateVariantAdd {
                    location: location.to_string(),
                });
            }
        };
        let Some((selector, insert_after)) = anchor else {
            return Err(ResolutionError::InvalidTemplateVariantAdd {
                location: location.to_string(),
            });
        };
        let anchor_index = find_required_anchor(template, selector, location)?;
        let insert_at = if insert_after {
            anchor_index.saturating_add(1)
        } else {
            anchor_index
        };
        template.insert(insert_at, op.component.clone());
    }
    Ok(())
}

pub(crate) fn find_required_anchor(
    template: &[TemplateComponent],
    selector: &TemplateComponentSelector,
    location: &str,
) -> Result<usize, ResolutionError> {
    find_optional_anchor(template, selector, location)?.ok_or_else(|| {
        ResolutionError::TemplateVariantAnchorNotFound {
            location: location.to_string(),
        }
    })
}

pub(crate) fn find_optional_anchor(
    template: &[TemplateComponent],
    selector: &TemplateComponentSelector,
    location: &str,
) -> Result<Option<usize>, ResolutionError> {
    if selector.is_empty() {
        return Err(ResolutionError::TemplateVariantAmbiguousAnchor {
            location: location.to_string(),
        });
    }
    let mut matches = template
        .iter()
        .enumerate()
        .filter_map(|(index, component)| selector.matches(component).then_some(index));
    let first = matches.next();
    if matches.next().is_some() {
        return Err(ResolutionError::TemplateVariantAmbiguousAnchor {
            location: location.to_string(),
        });
    }
    Ok(first)
}
