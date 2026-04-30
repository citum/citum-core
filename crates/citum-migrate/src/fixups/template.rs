use citum_schema::template::{DateVariable, SimpleVariable, TemplateComponent, TitleType};

pub(super) fn note_citation_template_is_underfit(template: &[TemplateComponent]) -> bool {
    template.len() == 1 && template.first().is_some_and(component_is_contributor_only)
}

fn component_is_contributor_only(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Contributor(_) => true,
        TemplateComponent::Group(list) => list.group.iter().all(component_is_contributor_only),
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
            TemplateComponent::Group(list) => {
                for item in &list.group {
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
        #[allow(
            clippy::collapsible_match,
            reason = "cannot use match guard due to mutable borrow of captured variable"
        )]
        match component {
            TemplateComponent::Contributor(c) => {
                if c.form == citum_schema::template::ContributorForm::Long {
                    c.form = citum_schema::template::ContributorForm::Short;
                    changed = true;
                }
            }
            TemplateComponent::Group(list) => {
                if normalize_contributor_form_to_short(&mut list.group) {
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
        #[allow(
            clippy::collapsible_match,
            reason = "cannot use match guard due to mutable borrow of captured variable"
        )]
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
            TemplateComponent::Group(list) => {
                if normalize_author_date_inferred_contributors(
                    &mut list.group,
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
        "entry-dictionary" | "entry-encyclopedia" => {
            !template_targets_type(inferred_template, type_name)
                && template_has_parent_title(candidate_template)
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
        }
        TemplateComponent::Number(number) => {
            if number.number == citum_schema::template::NumberVariable::Pages
                && let Some(prefix) = number.rendering.prefix.as_ref()
                && let Some(cleaned) = scrub_pages_year_literal_prefix(prefix)
            {
                number.rendering.prefix = Some(cleaned);
            }
        }
        TemplateComponent::Group(list) => {
            for item in &mut list.group {
                scrub_inferred_literal_artifacts(item);
            }
        }
        TemplateComponent::Contributor(_contributor) => {}
        TemplateComponent::Date(_date) => {}
        TemplateComponent::Variable(_variable) => {}
        TemplateComponent::Term(_term) => {}
        _ => {}
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

fn template_targets_type(_template: &[TemplateComponent], _target_type: &str) -> bool {
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
        TemplateComponent::Group(list) => list.group.iter().any(component_has_parent_title),
        _ => false,
    }
}

fn template_has_accessed_date(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_accessed_date)
}

fn component_has_accessed_date(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Date(d) => d.date == DateVariable::Accessed,
        TemplateComponent::Group(list) => list.group.iter().any(component_has_accessed_date),
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
        TemplateComponent::Group(list) => list.group.iter().any(component_has_primary_title),
        _ => false,
    }
}

fn template_has_parent_serial(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_parent_serial)
}

fn component_has_parent_serial(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Title(t) => t.title == TitleType::ParentSerial,
        TemplateComponent::Group(list) => list.group.iter().any(component_has_parent_serial),
        _ => false,
    }
}

fn template_has_publisher(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_publisher)
}

fn component_has_publisher(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Variable(v) => v.variable == SimpleVariable::Publisher,
        TemplateComponent::Group(list) => list.group.iter().any(component_has_publisher),
        _ => false,
    }
}

fn template_has_volume(template: &[TemplateComponent]) -> bool {
    template.iter().any(component_has_volume)
}

fn component_has_volume(component: &TemplateComponent) -> bool {
    match component {
        TemplateComponent::Number(n) => n.number == citum_schema::template::NumberVariable::Volume,
        TemplateComponent::Group(list) => list.group.iter().any(component_has_volume),
        _ => false,
    }
}

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
    use super::should_merge_inferred_type_template;
    use citum_schema::template::{TemplateComponent, TemplateTitle, TitleType};

    fn parent_serial_title() -> TemplateComponent {
        TemplateComponent::Title(TemplateTitle {
            title: TitleType::ParentSerial,
            ..Default::default()
        })
    }

    #[test]
    fn merges_entry_encyclopedia_candidates_with_parent_titles() {
        assert!(should_merge_inferred_type_template(
            "entry-encyclopedia",
            &[],
            &[parent_serial_title()],
        ));
    }

    #[test]
    fn merges_entry_dictionary_candidates_with_parent_titles() {
        assert!(should_merge_inferred_type_template(
            "entry-dictionary",
            &[],
            &[parent_serial_title()],
        ));
    }

    #[test]
    fn skips_entry_dictionary_candidates_without_parent_titles() {
        assert!(!should_merge_inferred_type_template(
            "entry-dictionary",
            &[],
            &[],
        ));
    }
}
