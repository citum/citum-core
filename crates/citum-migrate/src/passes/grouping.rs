use citum_schema::template::{
    DateForm, DateVariable, DelimiterPunctuation, NumberVariable, Rendering, TemplateComponent,
    TemplateGroup, TemplateNumber, WrapConfig, WrapPunctuation,
};

fn group_vol_issue_both_top_level(
    components: &mut Vec<TemplateComponent>,
    vol_idx: usize,
    issue_idx: usize,
    vol_issue_delimiter: DelimiterPunctuation,
) {
    let min_idx = vol_idx.min(issue_idx);
    let max_idx = vol_idx.max(issue_idx);
    components.remove(max_idx);
    components.remove(min_idx);
    let vol_issue_list = TemplateComponent::Group(TemplateGroup {
        group: vec![
            TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::Volume,
                form: None,
                rendering: Rendering::default(),
                ..Default::default()
            }),
            TemplateComponent::Number(TemplateNumber {
                number: NumberVariable::Issue,
                form: None,
                rendering: Rendering {
                    wrap: Some(WrapConfig {
                        punctuation: WrapPunctuation::Parentheses,
                        inner_prefix: None,
                        inner_suffix: None,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ],
        delimiter: Some(vol_issue_delimiter),
        rendering: Rendering::default(),
        ..Default::default()
    });
    components.insert(min_idx, vol_issue_list);
}

fn group_vol_issue_and_date_top_level(
    components: &mut Vec<TemplateComponent>,
    vol_idx: usize,
    issue_idx: usize,
    date_idx: usize,
) {
    let date = normalize_inline_detail_component(components.remove(date_idx));
    let issue = normalize_inline_detail_component(components.remove(issue_idx));
    let volume = normalize_inline_detail_component(components.remove(vol_idx));

    let issue_date_group = TemplateComponent::Group(TemplateGroup {
        group: vec![issue, date],
        delimiter: Some(DelimiterPunctuation::Space),
        rendering: Rendering::default(),
        ..Default::default()
    });

    let detail_group = TemplateComponent::Group(TemplateGroup {
        group: vec![volume, issue_date_group],
        delimiter: Some(DelimiterPunctuation::Comma),
        rendering: Rendering::default(),
        ..Default::default()
    });

    components.insert(vol_idx, detail_group);
}

fn group_vol_issue_issue_at_top(
    components: &mut Vec<TemplateComponent>,
    issue_idx: usize,
    style_preset: Option<crate::preset_detector::StylePreset>,
    vol_issue_delimiter: DelimiterPunctuation,
) {
    let list_idx = components.iter().enumerate().find_map(|(idx, c)| {
        if let TemplateComponent::Group(list) = c
            && find_volume_in_list(list).is_some()
        {
            return Some(idx);
        }
        None
    });

    if let Some(list_idx) = list_idx {
        components.remove(issue_idx);

        let adjusted_list_idx = if issue_idx < list_idx {
            list_idx - 1
        } else {
            list_idx
        };

        let issue_with_parens = TemplateComponent::Number(TemplateNumber {
            number: NumberVariable::Issue,
            form: None,
            rendering: Rendering {
                wrap: Some(WrapConfig {
                    punctuation: WrapPunctuation::Parentheses,
                    inner_prefix: None,
                    inner_suffix: None,
                }),
                ..Default::default()
            },
            ..Default::default()
        });

        if let Some(TemplateComponent::Group(list)) = components.get_mut(adjusted_list_idx)
            && insert_issue_after_volume(
                &mut list.group,
                issue_with_parens,
                vol_issue_delimiter.clone(),
            )
            && matches!(style_preset, Some(crate::preset_detector::StylePreset::Apa))
            && !list_contains_title(list)
        {
            list.delimiter = Some(DelimiterPunctuation::Comma);
        }
    }
}

fn group_vol_issue_both_nested(
    components: &mut [TemplateComponent],
    vol_issue_delimiter: DelimiterPunctuation,
) {
    let issue_exists_nested = find_issue_in_components(components);
    let volume_exists_nested = components.iter().any(|c| {
        if let TemplateComponent::Group(list) = c {
            find_volume_in_list(list).is_some()
        } else {
            false
        }
    });

    if issue_exists_nested && volume_exists_nested {
        let issue_with_parens = TemplateComponent::Number(TemplateNumber {
            number: NumberVariable::Issue,
            form: None,
            rendering: Rendering {
                wrap: Some(WrapConfig {
                    punctuation: WrapPunctuation::Parentheses,
                    inner_prefix: None,
                    inner_suffix: None,
                }),
                ..Default::default()
            },
            ..Default::default()
        });

        for component in components.iter_mut() {
            if let TemplateComponent::Group(list) = component
                && find_volume_in_list(list).is_some()
                && insert_issue_after_volume(
                    &mut list.group,
                    issue_with_parens.clone(),
                    vol_issue_delimiter.clone(),
                )
            {
                break;
            }
        }
    }
}

/// Group adjacent volume and issue detail for migrated article-journal output.
pub fn group_volume_and_issue(
    components: &mut Vec<TemplateComponent>,
    options: &citum_schema::options::Config,
    style_preset: Option<crate::preset_detector::StylePreset>,
) {
    // Volume-issue spacing varies by style:
    // - APA (comma delimiter): no space, e.g., "2(2)"
    // - Chicago (colon delimiter): space, e.g., "2 (2)"
    let vol_issue_delimiter = if options
        .volume_pages_delimiter
        .as_ref()
        .is_some_and(|d| matches!(d, DelimiterPunctuation::Comma))
    {
        DelimiterPunctuation::None
    } else {
        DelimiterPunctuation::Space
    };

    // Check for issue at top level
    let issue_pos = components.iter().position(
        |c| matches!(c, TemplateComponent::Number(n) if n.number == NumberVariable::Issue),
    );

    // Check for volume at top level
    let vol_pos = components.iter().position(
        |c| matches!(c, TemplateComponent::Number(n) if n.number == NumberVariable::Volume),
    );

    let issue_date_pos = issue_pos.and_then(|issue_idx| {
        components
            .get(issue_idx + 1)
            .and_then(|component| is_adjacent_year_month_date(component).then_some(issue_idx + 1))
    });

    if let (Some(vol_idx), Some(issue_idx), Some(date_idx)) = (vol_pos, issue_pos, issue_date_pos)
        && vol_idx < issue_idx
        && issue_idx < date_idx
    {
        group_vol_issue_and_date_top_level(components, vol_idx, issue_idx, date_idx);
        return;
    }

    if let (Some(vol_idx), Some(issue_idx)) = (vol_pos, issue_pos) {
        group_vol_issue_both_top_level(components, vol_idx, issue_idx, vol_issue_delimiter);
        return;
    }

    if let Some(issue_idx) = issue_pos {
        group_vol_issue_issue_at_top(components, issue_idx, style_preset, vol_issue_delimiter);
    } else if vol_pos.is_none() {
        group_vol_issue_both_nested(components, vol_issue_delimiter);
    }
}

fn is_adjacent_year_month_date(component: &TemplateComponent) -> bool {
    matches!(
        component,
        TemplateComponent::Date(date)
            if date.date == DateVariable::Issued && date.form == DateForm::YearMonth
    )
}

fn normalize_inline_detail_component(mut component: TemplateComponent) -> TemplateComponent {
    match &mut component {
        TemplateComponent::Number(number) => {
            number.rendering.prefix = None;
            number.rendering.suffix = None;
        }
        TemplateComponent::Date(date) => {
            if date.rendering.wrap.is_none()
                && date.rendering.prefix.as_deref() == Some("(")
                && date.rendering.suffix.as_deref() == Some(")")
            {
                use citum_schema::template::WrapConfig;
                date.rendering.wrap = Some(WrapConfig {
                    punctuation: WrapPunctuation::Parentheses,
                    inner_prefix: None,
                    inner_suffix: None,
                });
            }
            date.rendering.prefix = None;
            date.rendering.suffix = None;
        }
        _ => {}
    }

    component
}

/// Return whether an issue number exists anywhere in nested components.
#[must_use]
pub fn find_issue_in_components(components: &[TemplateComponent]) -> bool {
    for component in components {
        match component {
            TemplateComponent::Number(n) if n.number == NumberVariable::Issue => {
                return true;
            }
            TemplateComponent::Group(list) => {
                if find_issue_in_components(&list.group) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Insert an issue component after volume, recursing through nested groups.
///
/// Returns true if successfully inserted.
pub fn insert_issue_after_volume(
    items: &mut Vec<TemplateComponent>,
    issue: TemplateComponent,
    delimiter: DelimiterPunctuation,
) -> bool {
    // First, check if volume is directly in this list
    if let Some(vol_pos) = items.iter().position(
        |c| matches!(c, TemplateComponent::Number(n) if n.number == NumberVariable::Volume),
    ) {
        // Remove volume from the list
        let volume = items.remove(vol_pos);

        // Create a new List containing [volume, issue] with no delimiter
        // This preserves the outer list's delimiter for other items
        let vol_issue_group = TemplateComponent::Group(TemplateGroup {
            group: vec![volume, issue],
            delimiter: Some(delimiter), // No space between volume and issue
            rendering: Rendering::default(),
            ..Default::default()
        });

        // Insert the new group where volume was
        items.insert(vol_pos, vol_issue_group);
        return true;
    }

    // Otherwise, recurse into nested lists
    for item in items.iter_mut() {
        if let TemplateComponent::Group(inner_list) = item
            && insert_issue_after_volume(&mut inner_list.group, issue.clone(), delimiter.clone())
        {
            return true;
        }
    }

    false
}

/// Return whether a group contains a volume variable, recursively.
#[must_use]
pub fn find_volume_in_list(list: &TemplateGroup) -> Option<()> {
    for item in &list.group {
        match item {
            TemplateComponent::Number(n) if n.number == NumberVariable::Volume => {
                return Some(());
            }
            TemplateComponent::Group(inner_list) => {
                if find_volume_in_list(inner_list).is_some() {
                    return Some(());
                }
            }
            _ => {}
        }
    }
    None
}

/// Return whether a group contains any title component, recursively.
#[must_use]
pub fn list_contains_title(list: &TemplateGroup) -> bool {
    list.group.iter().any(|c| {
        matches!(c, TemplateComponent::Title(_))
            || matches!(c, TemplateComponent::Group(l) if list_contains_title(l))
    })
}
