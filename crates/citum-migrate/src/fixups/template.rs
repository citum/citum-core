use citum_schema::template::{DateVariable, SimpleVariable, TemplateComponent, TitleType};

pub(super) fn note_citation_template_is_underfit(template: &[TemplateComponent]) -> bool {
    template.len() == 1 && component_is_contributor_only(&template[0])
}

fn component_is_contributor_only(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Contributor(_) => true,
        TemplateComponent::List(list) => list.items.iter().all(component_is_contributor_only),
        _ => false,
    }
}

pub(super) fn citation_template_is_author_year_only(template: &[TemplateComponent]) -> bool {
    let mut has_contributor = false;
    let mut has_date = false;

    for component in template {
        match component {
            TemplateComponent::Contributor(_) => has_contributor = true,
            TemplateComponent::Date(_) => has_date = true,
            TemplateComponent::List(list) => {
                for item in &list.items {
                    match item {
                        TemplateComponent::Contributor(_) => has_contributor = true,
                        TemplateComponent::Date(_) => has_date = true,
                        _ => return false,
                    }
                }
            }
            _ => return false,
        }
    }

    has_contributor && has_date
}

pub(super) fn normalize_contributor_form_to_short(template: &mut [TemplateComponent]) -> bool {
    let mut changed = false;
    for component in template {
        match component {
            TemplateComponent::Contributor(c) => {
                if c.form == citum_schema::template::ContributorForm::Long {
                    c.form = citum_schema::template::ContributorForm::Short;
                    changed = true;
                }
            }
            TemplateComponent::List(list) => {
                if normalize_contributor_form_to_short(&mut list.items) {
                    changed = true;
                }
            }
            _ => {}
        }
    }
    changed
}

pub(super) fn normalize_author_date_inferred_contributors(
    template: &mut [TemplateComponent],
    drop_component_shorten: bool,
) -> bool {
    let mut changed = false;
    for component in template {
        match component {
            TemplateComponent::Contributor(c) => {
                if c.form == citum_schema::template::ContributorForm::Long {
                    c.form = citum_schema::template::ContributorForm::Short;
                    changed = true;
                }
                if c.name_order == Some(citum_schema::template::NameOrder::GivenFirst) {
                    c.name_order = Some(citum_schema::template::NameOrder::FamilyFirst);
                    changed = true;
                }
                if drop_component_shorten && c.shorten.is_some() {
                    c.shorten = None;
                    changed = true;
                }
            }
            TemplateComponent::List(list) => {
                if normalize_author_date_inferred_contributors(
                    &mut list.items,
                    drop_component_shorten,
                ) {
                    changed = true;
                }
            }
            _ => {}
        }
    }
    changed
}

pub(super) fn should_merge_inferred_type_template(
    type_name: &str,
    inferred_template: &[TemplateComponent],
    candidate_template: &[TemplateComponent],
) -> bool {
    match type_name {
        "patent" => candidate_template.len() <= 12,
        "entry-encyclopedia" => {
            !template_targets_type(inferred_template, type_name)
                && !template_has_parent_title(candidate_template)
        }
        "webpage" => {
            (!template_targets_type(inferred_template, type_name)
                || !template_has_accessed_date(inferred_template))
                && template_has_accessed_date(candidate_template)
        }
        "legal-case" | "legal_case" => !template_targets_type(inferred_template, type_name),
        "personal_communication" | "personal-communication" => {
            !template_targets_type(inferred_template, type_name)
        }
        "article-journal" | "article-magazine" | "article-newspaper" | "book" | "report"
        | "broadcast" | "interview" | "motion_picture" | "motion-picture" => {
            inferred_candidate_structurally_diverges(inferred_template, candidate_template)
        }
        _ => false,
    }
}

pub(super) fn scrub_inferred_literal_artifacts(component: &mut TemplateComponent) {
    match component {
        TemplateComponent::Title(title) => {
            if title.title == TitleType::Primary
                && let Some(prefix) = title.rendering.prefix.as_ref()
                && let Some(cleaned) = scrub_year_only_prefix(prefix)
            {
                title.rendering.prefix = Some(cleaned);
            }
            scrub_overrides_map(title.overrides.as_mut());
        }
        TemplateComponent::Number(number) => {
            if number.number == citum_schema::template::NumberVariable::Pages
                && let Some(prefix) = number.rendering.prefix.as_ref()
                && let Some(cleaned) = scrub_pages_year_literal_prefix(prefix)
            {
                number.rendering.prefix = Some(cleaned);
            }
            scrub_overrides_map(number.overrides.as_mut());
        }
        TemplateComponent::List(list) => {
            for item in &mut list.items {
                scrub_inferred_literal_artifacts(item);
            }
            scrub_overrides_map(list.overrides.as_mut());
        }
        TemplateComponent::Contributor(contributor) => {
            scrub_overrides_map(contributor.overrides.as_mut());
        }
        TemplateComponent::Date(date) => {
            scrub_overrides_map(date.overrides.as_mut());
        }
        TemplateComponent::Variable(variable) => {
            scrub_overrides_map(variable.overrides.as_mut());
        }
        TemplateComponent::Term(term) => {
            scrub_overrides_map(term.overrides.as_mut());
        }
        _ => {}
    }
}

fn scrub_component_override_literals(
    override_value: &mut citum_schema::template::ComponentOverride,
) {
    match override_value {
        citum_schema::template::ComponentOverride::Component(component) => {
            scrub_inferred_literal_artifacts(component)
        }
        citum_schema::template::ComponentOverride::Rendering(rendering) => {
            if let Some(prefix) = rendering.prefix.as_ref() {
                if let Some(cleaned) = scrub_year_only_prefix(prefix) {
                    rendering.prefix = Some(cleaned);
                } else if let Some(cleaned) = scrub_pages_year_literal_prefix(prefix) {
                    rendering.prefix = Some(cleaned);
                }
            }
        }
    }
}

fn scrub_year_only_prefix(prefix: &str) -> Option<String> {
    let trimmed = prefix.trim();
    if !is_four_digit_year(trimmed) {
        return None;
    }

    if prefix.starts_with(' ') && prefix.ends_with(' ') {
        Some(" ".to_string())
    } else {
        None
    }
}

fn scrub_pages_year_literal_prefix(prefix: &str) -> Option<String> {
    if prefix
        .strip_prefix("; ")
        .and_then(|s| s.strip_suffix("; "))
        .is_some_and(|s| is_four_digit_year(s.trim()))
    {
        return Some("; ".to_string());
    }

    if prefix
        .strip_prefix(". ")
        .and_then(|s| s.strip_suffix(": "))
        .is_some_and(|s| is_four_digit_year(s.trim()))
    {
        return Some(": ".to_string());
    }

    None
}

fn is_four_digit_year(value: &str) -> bool {
    value.len() == 4
        && value.chars().all(|ch| ch.is_ascii_digit())
        && value
            .parse::<u16>()
            .is_ok_and(|year| (1800..=2100).contains(&year))
}

fn template_targets_type(template: &[TemplateComponent], target_type: &str) -> bool {
    template
        .iter()
        .any(|component| component_targets_type(component, target_type))
}

fn component_targets_type(component: &TemplateComponent, target_type: &str) -> bool {
    let overrides = match component {
        TemplateComponent::Contributor(c) => c.overrides.as_ref(),
        TemplateComponent::Date(d) => d.overrides.as_ref(),
        TemplateComponent::Title(t) => t.overrides.as_ref(),
        TemplateComponent::Number(n) => n.overrides.as_ref(),
        TemplateComponent::Variable(v) => v.overrides.as_ref(),
        TemplateComponent::List(l) => l.overrides.as_ref(),
        TemplateComponent::Term(t) => t.overrides.as_ref(),
        _ => None,
    };

    if let Some(overrides) = overrides
        && overrides
            .keys()
            .any(|selector| selector.matches(target_type))
    {
        return true;
    }

    if let TemplateComponent::List(list) = component {
        return list
            .items
            .iter()
            .any(|item| component_targets_type(item, target_type));
    }

    false
}

fn template_has_parent_title(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_parent_title)
}

fn component_has_parent_title(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Title(t) => {
            t.title == TitleType::ParentMonograph || t.title == TitleType::ParentSerial
        }
        TemplateComponent::List(list) => list.items.iter().any(component_has_parent_title),
        _ => false,
    }
}

fn template_has_accessed_date(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_accessed_date)
}

fn component_has_accessed_date(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Date(d) => d.date == DateVariable::Accessed,
        TemplateComponent::List(list) => list.items.iter().any(component_has_accessed_date),
        _ => false,
    }
}

fn inferred_candidate_structurally_diverges(
    inferred_template: &[TemplateComponent],
    candidate_template: &[TemplateComponent],
) -> bool {
    let inferred_has_primary_title = template_has_primary_title(inferred_template);
    let candidate_has_primary_title = template_has_primary_title(candidate_template);
    let inferred_has_parent_serial = template_has_parent_serial(inferred_template);
    let candidate_has_parent_serial = template_has_parent_serial(candidate_template);
    let inferred_has_publisher = template_has_publisher(inferred_template);
    let candidate_has_publisher = template_has_publisher(candidate_template);
    let inferred_has_volume = template_has_volume(inferred_template);
    let candidate_has_volume = template_has_volume(candidate_template);

    (inferred_has_primary_title && !candidate_has_primary_title)
        || (!inferred_has_parent_serial && candidate_has_parent_serial)
        || (inferred_has_publisher && !candidate_has_publisher)
        || (!inferred_has_volume && candidate_has_volume)
}

fn template_has_primary_title(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_primary_title)
}

fn component_has_primary_title(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Title(t) => t.title == TitleType::Primary,
        TemplateComponent::List(list) => list.items.iter().any(component_has_primary_title),
        _ => false,
    }
}

fn template_has_parent_serial(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_parent_serial)
}

fn component_has_parent_serial(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Title(t) => t.title == TitleType::ParentSerial,
        TemplateComponent::List(list) => list.items.iter().any(component_has_parent_serial),
        _ => false,
    }
}

fn template_has_publisher(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_publisher)
}

fn component_has_publisher(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Variable(v) => v.variable == SimpleVariable::Publisher,
        TemplateComponent::List(list) => list.items.iter().any(component_has_publisher),
        _ => false,
    }
}

fn template_has_volume(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_volume)
}

fn component_has_volume(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Number(n) => n.number == citum_schema::template::NumberVariable::Volume,
        TemplateComponent::List(list) => list.items.iter().any(component_has_volume),
        _ => false,
    }
}

fn scrub_overrides_map(
    overrides: Option<
        &mut std::collections::HashMap<
            citum_schema::template::TypeSelector,
            citum_schema::template::ComponentOverride,
        >,
    >,
) {
    let Some(map) = overrides else { return };
    for val in map.values_mut() {
        scrub_component_override_literals(val);
    }
}
