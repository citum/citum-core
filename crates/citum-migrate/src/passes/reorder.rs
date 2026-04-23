use citum_schema::template::{TemplateComponent, TemplateGroup};

/// Recursively reorder serial components (container-title, volume) in a template.
pub fn reorder_serial_components(components: &mut Vec<TemplateComponent>) {
    use citum_schema::template::{NumberVariable, TitleType};

    for component in components {
        if let TemplateComponent::Group(list) = component {
            // Check if this list contains both volume and parent-serial
            let has_volume = list.group.iter().any(|item| {
                matches!(
                    item,
                    TemplateComponent::Number(n) if n.number == NumberVariable::Volume
                )
            });
            let has_parent_serial = list.group.iter().any(|item| {
                matches!(
                    item,
                    TemplateComponent::Title(t) if t.title == TitleType::ParentSerial
                )
            });

            if has_volume && has_parent_serial {
                // Find positions
                let volume_pos = list.group.iter().position(|item| {
                    matches!(
                        item,
                        TemplateComponent::Number(n) if n.number == NumberVariable::Volume
                    )
                });
                let parent_serial_pos = list.group.iter().position(|item| {
                    matches!(
                        item,
                        TemplateComponent::Title(t) if t.title == TitleType::ParentSerial
                    )
                });

                // If volume is before parent-serial, swap them
                if let (Some(vol_pos), Some(ps_pos)) = (volume_pos, parent_serial_pos)
                    && vol_pos < ps_pos
                {
                    list.group.swap(vol_pos, ps_pos);
                }
            }

            // Recursively process nested lists
            for item in &mut list.group {
                if let TemplateComponent::Group(inner_list) = item {
                    reorder_serial_components_in_list(inner_list);
                }
            }
        }
    }
}

/// Helper to reorder components in a single list.
pub fn reorder_serial_components_in_list(list: &mut TemplateGroup) {
    use citum_schema::template::{NumberVariable, TitleType};

    // Check if this list contains both volume and parent-serial
    let has_volume = list.group.iter().any(|item| {
        matches!(
            item,
            TemplateComponent::Number(n) if n.number == NumberVariable::Volume
        )
    });
    let has_parent_serial = list.group.iter().any(|item| {
        matches!(
            item,
            TemplateComponent::Title(t) if t.title == TitleType::ParentSerial
        )
    });

    if has_volume && has_parent_serial {
        // Find positions
        let volume_pos = list.group.iter().position(|item| {
            matches!(
                item,
                TemplateComponent::Number(n) if n.number == NumberVariable::Volume
            )
        });
        let parent_serial_pos = list.group.iter().position(|item| {
            matches!(
                item,
                TemplateComponent::Title(t) if t.title == TitleType::ParentSerial
            )
        });

        // If volume is before parent-serial, swap them
        if let (Some(vol_pos), Some(ps_pos)) = (volume_pos, parent_serial_pos)
            && vol_pos < ps_pos
        {
            list.group.swap(vol_pos, ps_pos);
        }
    }
}

/// Move pages component to appear after the container-title/volume List.
pub fn reorder_pages_for_serials(components: &mut Vec<TemplateComponent>) {
    use citum_schema::template::{NumberVariable, TitleType};

    // Find the pages component position
    let pages_pos = components.iter().position(|c| {
        matches!(
            c,
            TemplateComponent::Number(n) if n.number == NumberVariable::Pages
        )
    });

    // Find the List containing parent-serial (container-title for journals)
    // Need to search recursively since parent-serial may be in a nested List
    let serial_list_pos = components.iter().position(contains_parent_serial_recursive);

    // If pages is BEFORE the serial list, move it to right after
    if let (Some(p_pos), Some(s_pos)) = (pages_pos, serial_list_pos)
        && p_pos < s_pos
    {
        let pages_component = components.remove(p_pos);
        // After removal, indices shift - insert at s_pos (which is now s_pos - 1 + 1 = s_pos)
        components.insert(s_pos, pages_component);
    }

    fn contains_parent_serial_recursive(component: &TemplateComponent) -> bool {
        match component {
            TemplateComponent::Title(t) if t.title == TitleType::ParentSerial => true,
            TemplateComponent::Group(list) => {
                list.group.iter().any(contains_parent_serial_recursive)
            }
            _ => false,
        }
    }
}

/// Reorder publisher-place for Chicago journal articles.
pub fn reorder_publisher_place_for_chicago(
    components: &mut Vec<TemplateComponent>,
    fixup_family: Option<crate::base_detector::FixupFamily>,
) {
    use crate::base_detector::FixupFamily;
    use citum_schema::template::{SimpleVariable, TitleType};

    // Only apply to Chicago styles
    if !matches!(fixup_family, Some(FixupFamily::Chicago)) {
        return;
    }

    // Find the publisher-place component (it's in a List with wrap: parentheses)
    let publisher_place_pos = components.iter().position(|c| {
        if let TemplateComponent::Group(list) = c {
            list.group.iter().any(|item| {
                matches!(
                    item,
                    TemplateComponent::Variable(v)
                    if v.variable == SimpleVariable::PublisherPlace
                )
            })
        } else {
            false
        }
    });

    // Find the parent-serial title position
    let parent_serial_pos = components.iter().position(|c| {
        matches!(
            c,
            TemplateComponent::Title(t) if t.title == TitleType::ParentSerial
        )
    });

    // If we found both, move publisher-place to right after parent-serial
    if let (Some(pp_pos), Some(ps_pos)) = (publisher_place_pos, parent_serial_pos)
        && pp_pos > ps_pos
    {
        // Remove the publisher-place List
        let mut publisher_place_component = components.remove(pp_pos);

        // Add space suffix to prevent default period separator
        if let TemplateComponent::Group(ref mut list) = publisher_place_component {
            list.rendering.suffix = Some(" ".to_string());
        }

        // Insert it right after parent-serial
        components.insert(ps_pos + 1, publisher_place_component);
    }
}

/// Reorder chapter components for APA style.
pub fn reorder_chapters_for_apa(
    components: &mut Vec<TemplateComponent>,
    fixup_family: Option<crate::base_detector::FixupFamily>,
) {
    use crate::base_detector::FixupFamily;
    use citum_schema::template::{ContributorRole, TitleType};

    // Only apply to APA styles
    if !matches!(fixup_family, Some(FixupFamily::Apa)) {
        return;
    }

    // Find the editor contributor
    let editor_pos = components.iter().position(|c| {
        matches!(
            c,
            TemplateComponent::Contributor(contrib)
            if contrib.contributor == ContributorRole::Editor
        )
    });

    // Find the parent-monograph title
    let parent_monograph_pos = components.iter().position(|c| {
        matches!(
            c,
            TemplateComponent::Title(t) if t.title == TitleType::ParentMonograph
        )
    });

    if let (Some(ed_pos), Some(pm_pos)) = (editor_pos, parent_monograph_pos)
        && ed_pos > pm_pos
    {
        // Swap them: move editor before parent-monograph
        let editor_comp = components.remove(ed_pos);
        components.insert(pm_pos, editor_comp);
    }
}

/// Reorder chapter components for Chicago style.
pub fn reorder_chapters_for_chicago(
    components: &mut Vec<TemplateComponent>,
    fixup_family: Option<crate::base_detector::FixupFamily>,
) {
    use crate::base_detector::FixupFamily;
    use citum_schema::template::{ContributorRole, TitleType};

    // Only apply to Chicago styles
    if !matches!(fixup_family, Some(FixupFamily::Chicago)) {
        return;
    }

    // Find the editor contributor (form: verb)
    let editor_pos = components.iter().position(|c| {
        matches!(
            c,
            TemplateComponent::Contributor(contrib)
            if contrib.contributor == ContributorRole::Editor
        )
    });

    // Find the parent-monograph title
    let parent_monograph_pos = components.iter().position(|c| {
        matches!(
            c,
            TemplateComponent::Title(t) if t.title == TitleType::ParentMonograph
        )
    });

    // If we found both and editor comes before parent-monograph, swap them
    if let (Some(editor_pos), Some(pm_pos)) = (editor_pos, parent_monograph_pos)
        && editor_pos < pm_pos
    {
        // Get mutable references to both components
        let editor_component = components.remove(editor_pos);
        let pm_component = components.remove(pm_pos - 1); // Adjust index after removal

        // Add "In " prefix and ", " suffix to parent-monograph for chapters
        let pm_with_prefix = pm_component.clone();
        // (Overrides removed from TemplateTitle)

        // Adjust editor for chapters: use given-first name order
        let mut editor_with_suffix = editor_component.clone();
        if let TemplateComponent::Contributor(ref mut contrib) = editor_with_suffix {
            // For chapters, editors should use given-first name order
            use citum_schema::template::NameOrder;
            contrib.name_order = Some(NameOrder::GivenFirst);
        }

        // Re-insert in new order: parent-monograph, then editor
        components.insert(editor_pos, pm_with_prefix);
        components.insert(editor_pos + 1, editor_with_suffix);
    }
}

/// Propagate type-specific overrides within Lists.
pub fn propagate_list_overrides(_components: &mut [TemplateComponent]) {
    // Overrides propagation logic removed as component-level overrides are deprecated.
}

/// Recursively ensure specific variables are un-suppressed for a given type.
#[allow(
    clippy::only_used_in_recursion,
    reason = "item_type propagated for future unsuppress logic"
)]
pub fn unsuppress_for_type(components: &mut [TemplateComponent], item_type: &str) {
    for component in components {
        if let TemplateComponent::Group(list) = component {
            unsuppress_for_type(&mut list.group, item_type);
        }
    }
}

/// Add space prefix to volume when it directly follows parent-serial title.
/// This handles numeric styles where journal and volume are siblings, not in a List.
pub fn add_volume_prefix_after_serial(components: &mut [TemplateComponent]) {
    use citum_schema::template::{NumberVariable, TitleType};

    for i in 1..components.len() {
        let prev_is_serial = matches!(
            components.get(i - 1),
            Some(TemplateComponent::Title(t)) if t.title == TitleType::ParentSerial
        );

        if prev_is_serial
            && let Some(TemplateComponent::Number(num)) = components.get_mut(i)
            && num.number == NumberVariable::Volume
        {
            // Add space prefix if not already present
            if num.rendering.prefix.is_none() {
                num.rendering.prefix = Some(" ".to_string());
            }
        }
    }
}

/// Move DOI and URL components to the end of the bibliography template.
pub fn move_access_components_to_end(components: &mut Vec<TemplateComponent>) {
    use citum_schema::template::SimpleVariable;

    // Find indices of access components (DOI, URL)
    let mut access_indices: Vec<usize> = Vec::new();
    for (i, c) in components.iter().enumerate() {
        if let TemplateComponent::Variable(v) = c
            && matches!(v.variable, SimpleVariable::Doi | SimpleVariable::Url)
        {
            access_indices.push(i);
        }
        // Also check for List items containing accessed date (URL + accessed date pattern)
        if let TemplateComponent::Group(list) = c {
            let has_access = list.group.iter().any(|item| {
                matches!(item, TemplateComponent::Variable(v) if v.variable == SimpleVariable::Url)
                    || matches!(item, TemplateComponent::Date(d) if d.date == citum_schema::template::DateVariable::Accessed)
            });
            if has_access {
                access_indices.push(i);
            }
        }
    }

    // Extract access components in reverse order (to preserve indices)
    let mut access_components: Vec<TemplateComponent> = Vec::new();
    for idx in access_indices.into_iter().rev() {
        access_components.push(components.remove(idx));
    }
    access_components.reverse();

    // Append access components at the end
    components.extend(access_components);
}
