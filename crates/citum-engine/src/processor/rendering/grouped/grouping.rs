//! Author-group formation for grouped citation rendering.

use crate::reference::Reference;

/// Group citation items by author, respecting individual citation hints.
///
/// Returns a list of (author_key, items) tuples. If any item has disambiguation
/// hints, all items are grouped individually by citation id.
pub(crate) fn group_citation_items_by_author<'a>(
    renderer: &super::super::Renderer<'_>,
    items: &'a [crate::reference::CitationItem],
) -> Vec<(String, Vec<&'a crate::reference::CitationItem>)> {
    let preserve_individual_citations = items.iter().any(|item| {
        renderer
            .hints
            .get(&item.id)
            .is_some_and(|hints| hints.min_names_to_show.is_some() || hints.expand_given_names)
    });

    let mut groups: Vec<(String, Vec<&'a crate::reference::CitationItem>)> = Vec::new();

    for item in items {
        let reference = renderer.bibliography.get(&item.id);
        let author_key = if preserve_individual_citations {
            item.id.clone()
        } else {
            reference.map(author_grouping_key).unwrap_or_default()
        };

        match groups.last_mut() {
            Some(group) if !author_key.is_empty() && group.0 == author_key => {
                group.1.push(item);
            }
            _ => groups.push((author_key, vec![item])),
        }
    }

    groups
}

/// Generate a grouping key for a reference.
///
/// Prefers author, then editor, then title. The key is lowercased for
/// case-insensitive grouping.
fn author_grouping_key(reference: &Reference) -> String {
    reference
        .author()
        .map_or_else(
            || {
                reference.editor().map_or_else(
                    || {
                        reference
                            .title()
                            .map_or_else(String::new, |title| title.to_string())
                    },
                    |editor| editor.to_string(),
                )
            },
            |author| author.to_string(),
        )
        .to_lowercase()
}
