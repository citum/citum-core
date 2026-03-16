use super::TemplateComponent;

pub fn strip_author_component(component: &TemplateComponent) -> Option<TemplateComponent> {
    match component {
        TemplateComponent::Contributor(c)
            if c.contributor == citum_schema::template::ContributorRole::Author =>
        {
            None
        }
        TemplateComponent::List(list) => {
            let filtered_items: Vec<TemplateComponent> = list
                .items
                .iter()
                .filter_map(strip_author_component)
                .collect();

            if filtered_items.is_empty() {
                None
            } else {
                let mut filtered_list = list.clone();
                filtered_list.items = filtered_items;
                Some(TemplateComponent::List(filtered_list))
            }
        }
        _ => Some(component.clone()),
    }
}

/// Extract the leading affix used to separate grouped authors from item details.
pub fn leading_group_affix(component: &TemplateComponent) -> Option<String> {
    let r = component.rendering();
    let own_affix = r.prefix.clone().or(r.inner_prefix.clone()).or_else(|| {
        if let TemplateComponent::List(inner) = component {
            inner.items.first().and_then(leading_group_affix)
        } else {
            None
        }
    });

    own_affix.filter(|value| !value.is_empty())
}

/// Remove leading affixes from the first surviving grouped-citation component.
///
/// When the author component is stripped from an author-date template, the next
/// component often carries a prefix like `", "` that only makes sense when the
/// author is still present. Grouped citation assembly adds the author/date
/// delimiter separately, so the first surviving component must start "clean".
pub fn strip_leading_group_affixes(component: &mut TemplateComponent) {
    let r = component.rendering_mut();
    r.prefix = None;
    r.inner_prefix = None;
    if let TemplateComponent::List(inner) = component
        && let Some(first) = inner.items.first_mut()
    {
        strip_leading_group_affixes(first);
    }
}

/// Finds a grouping component (contributor or title) within a template.
///
/// Descends into lists to find the first semantically relevant component
/// for grouping citations by author or title.
pub fn find_grouping_component(component: &TemplateComponent) -> Option<&TemplateComponent> {
    match component {
        TemplateComponent::Contributor(_) | TemplateComponent::Title(_) => Some(component),
        TemplateComponent::List(list) => list.items.iter().find_map(find_grouping_component),
        _ => None,
    }
}

pub fn has_contributor_component(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Contributor(_) => true,
        TemplateComponent::List(list) => list.items.iter().any(has_contributor_component),
        _ => false,
    }
}
