use citum_schema::options::LocatorConfig;
use csl_legacy::model::{CslNode, Layout, Style};

/// Extract locator rendering configuration from citation layout labels.
#[must_use]
pub fn extract_locator_config(style: &Style) -> Option<LocatorConfig> {
    let mut config = LocatorConfig::default();
    let mut has_config = false;

    if citation_layout_uses_locator_labels_with_strip_periods(&style.citation.layout) {
        config.strip_label_periods = Some(true);
        has_config = true;
    }

    has_config.then_some(config)
}

fn citation_layout_uses_locator_labels_with_strip_periods(layout: &Layout) -> bool {
    nodes_use_locator_labels_with_strip_periods(&layout.children)
}

fn nodes_use_locator_labels_with_strip_periods(nodes: &[CslNode]) -> bool {
    nodes
        .iter()
        .any(node_uses_locator_labels_with_strip_periods)
}

fn node_uses_locator_labels_with_strip_periods(node: &CslNode) -> bool {
    match node {
        CslNode::Label(label) => {
            label.variable.as_deref() == Some("locator") && label.strip_periods == Some(true)
        }
        CslNode::Group(group) => nodes_use_locator_labels_with_strip_periods(&group.children),
        CslNode::Choose(choose) => {
            nodes_use_locator_labels_with_strip_periods(&choose.if_branch.children)
                || choose
                    .else_if_branches
                    .iter()
                    .any(|branch| nodes_use_locator_labels_with_strip_periods(&branch.children))
                || choose
                    .else_branch
                    .as_ref()
                    .is_some_and(|branch| nodes_use_locator_labels_with_strip_periods(branch))
        }
        CslNode::Names(names) => nodes_use_locator_labels_with_strip_periods(&names.children),
        _ => false,
    }
}
