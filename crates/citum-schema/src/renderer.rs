//! Minimal renderer for the legacy Citum (CSL Node) AST.
//!
//! This module provides a basic AST renderer used internally for schema tests and migration workflows.
//! It is not the production rendering engine (see `citum_engine` for that). The renderer handles
//! conditional branches, variable substitution, formatting, and basic grouping of citation elements.

use crate::{
    ConditionBlock, CslnNode, DateBlock, GroupBlock, ItemType, NamesBlock, TermBlock, Variable,
    VariableBlock,
};
use std::collections::HashMap;

/// A mock reference item with metadata for rendering.
///
/// This is an internal type used by the legacy Renderer, distinct from the input `CitationItem`
/// in `crate::citation`. It holds an item type and a flat HashMap of resolved variable values
/// keyed by legacy CSL variable names.
pub struct RenderItem {
    /// Legacy CSL item type used by conditional rendering branches.
    pub item_type: ItemType,
    /// Pre-resolved variable values keyed by legacy CSL variable name.
    pub variables: HashMap<Variable, String>,
}

/// Minimal renderer for the legacy Citum AST used in schema tests and migration.
///
/// This renderer walks a Citum AST and produces a string output by concatenating rendered nodes.
/// It handles text nodes, variables with optional labels, dates, names, groups, conditions,
/// and term lookups. Formatting (italic, bold, underline, superscript) and prefix/suffix options
/// are applied to text output.
pub struct Renderer;

impl Renderer {
    /// Render a citation by concatenating the rendered output of each AST node.
    ///
    /// Takes a sequence of Citum AST nodes and a reference item, walks the tree, and produces
    /// a single string by joining all rendered node outputs.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A slice of Citum AST nodes to render.
    /// * `item` - The reference item providing variables and type information.
    ///
    /// # Returns
    ///
    /// A `String` containing the concatenated rendered output of all nodes.
    pub fn render_citation(&self, nodes: &[CslnNode], item: &RenderItem) -> String {
        let mut output = String::new();
        for node in nodes {
            output.push_str(&self.render_node(node, item));
        }
        output
    }

    /// Render a single Citum AST node.
    ///
    /// Dispatches on the node variant and calls the appropriate rendering method.
    fn render_node(&self, node: &CslnNode, item: &RenderItem) -> String {
        match node {
            CslnNode::Text { value } => value.clone(),
            CslnNode::Variable(var_block) => self.render_variable(var_block, item),
            CslnNode::Date(date_block) => self.render_date(date_block, item),
            CslnNode::Names(names_block) => self.render_names(names_block, item),
            CslnNode::Group(group_block) => self.render_group(group_block, item),
            CslnNode::Condition(cond_block) => self.render_condition(cond_block, item),
            CslnNode::Term(term_block) => self.render_term(term_block),
        }
    }

    /// Render a term block by looking up the term and applying formatting.
    fn render_term(&self, block: &TermBlock) -> String {
        self.apply_formatting(
            &format!("{:?}", block.term).to_lowercase(),
            &block.formatting,
        )
    }

    /// Render a variable block by looking up the variable value and applying optional label and formatting.
    fn render_variable(&self, block: &VariableBlock, item: &RenderItem) -> String {
        if let Some(val) = item.variables.get(&block.variable) {
            let mut text = val.clone();

            if let Some(label_opts) = &block.label {
                let prefix = label_opts.formatting.prefix.as_deref().unwrap_or("");
                let suffix = label_opts.formatting.suffix.as_deref().unwrap_or("");
                let label_text = match block.variable {
                    Variable::Page => "p.",
                    Variable::Volume => "vol.",
                    _ => "",
                };
                text = format!("{}{}{}{}", prefix, label_text, suffix, text);
            }

            self.apply_formatting(&text, &block.formatting)
        } else {
            String::new()
        }
    }

    /// Render a date block by looking up the date variable and applying formatting.
    fn render_date(&self, block: &DateBlock, item: &RenderItem) -> String {
        if let Some(val) = item.variables.get(&block.variable) {
            self.apply_formatting(val, &block.formatting)
        } else {
            String::new()
        }
    }

    /// Render a names block with variable substitution, initialization, and sorting options.
    fn render_names(&self, block: &NamesBlock, item: &RenderItem) -> String {
        let active_val = if let Some(val) = item.variables.get(&block.variable) {
            Some(val.clone())
        } else {
            block
                .options
                .substitute
                .iter()
                .find_map(|sub_var| item.variables.get(sub_var).cloned())
        };

        if let Some(mut formatted) = active_val {
            if let Some(init) = &block.options.initialize_with
                && !formatted.as_str().contains(init.as_str())
            {
                formatted = format!("{} [Init: {}]", formatted, init);
            }

            if let Some(order) = &block.options.name_as_sort_order {
                formatted = format!("{} [Sort: {:?}]", formatted, order);
            }

            self.apply_formatting(&formatted, &block.formatting)
        } else {
            String::new()
        }
    }

    /// Render a group block by rendering all children and joining them with a delimiter.
    ///
    /// Returns an empty string if all children render to empty strings (suppresses empty groups).
    fn render_group(&self, block: &GroupBlock, item: &RenderItem) -> String {
        let mut parts = Vec::new();
        for child in &block.children {
            let rendered = self.render_node(child, item);
            if !rendered.is_empty() {
                parts.push(rendered);
            }
        }

        if parts.is_empty() {
            return String::new();
        }

        let delimiter = block.delimiter.as_deref().unwrap_or("");
        let content = parts.join(delimiter);

        self.apply_formatting(&content, &block.formatting)
    }

    /// Render a condition block by evaluating if/else-if/else branches based on item type and variable presence.
    ///
    /// Evaluates the main if-branch first, then else-if branches in order, and finally the else-branch.
    fn render_condition(&self, block: &ConditionBlock, item: &RenderItem) -> String {
        // Check if the main if-branch matches
        let type_match =
            block.if_item_type.is_empty() || block.if_item_type.contains(&item.item_type);
        let var_match = block.if_variables.is_empty()
            || block
                .if_variables
                .iter()
                .any(|v| item.variables.contains_key(v));

        let match_found = if block.if_item_type.is_empty() && block.if_variables.is_empty() {
            false
        } else {
            type_match && var_match
        };

        if match_found {
            let mut output = String::new();
            for child in &block.then_branch {
                output.push_str(&self.render_node(child, item));
            }
            return output;
        }

        // Check else-if branches in order
        for else_if in &block.else_if_branches {
            let type_match =
                else_if.if_item_type.is_empty() || else_if.if_item_type.contains(&item.item_type);
            let var_match = else_if.if_variables.is_empty()
                || else_if
                    .if_variables
                    .iter()
                    .any(|v| item.variables.contains_key(v));

            let branch_match = if else_if.if_item_type.is_empty() && else_if.if_variables.is_empty()
            {
                false
            } else {
                type_match && var_match
            };

            if branch_match {
                let mut output = String::new();
                for child in &else_if.children {
                    output.push_str(&self.render_node(child, item));
                }
                return output;
            }
        }

        // Fall back to else branch
        if let Some(else_branch) = &block.else_branch {
            let mut output = String::new();
            for child in else_branch {
                output.push_str(&self.render_node(child, item));
            }
            output
        } else {
            String::new()
        }
    }

    /// Apply formatting options (prefix, suffix, font style, weight, decoration, alignment) to text.
    fn apply_formatting(&self, text: &str, fmt: &crate::FormattingOptions) -> String {
        let prefix = fmt.prefix.as_deref().unwrap_or("");
        let suffix = fmt.suffix.as_deref().unwrap_or("");

        let mut res = text.to_string();
        if fmt.font_style == Some(crate::FontStyle::Italic) {
            res = format!("_{}_", res);
        }
        if fmt.font_weight == Some(crate::FontWeight::Bold) {
            res = format!("*{}*", res);
        }
        if fmt.text_decoration == Some(crate::TextDecoration::Underline) {
            res = format!("<u>{}</u>", res);
        }
        if fmt.vertical_align == Some(crate::VerticalAlign::Superscript) {
            res = format!("^{}^", res);
        }

        format!("{}{}{}", prefix, res, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that RenderItem can be constructed with an item type and variables.
    #[test]
    fn test_render_item_construction() {
        let mut variables = HashMap::new();
        variables.insert(Variable::Author, "Smith, John".to_string());

        let item = RenderItem {
            item_type: ItemType::Book,
            variables,
        };

        assert_eq!(item.item_type, ItemType::Book);
        assert_eq!(
            item.variables.get(&Variable::Author),
            Some(&"Smith, John".to_string())
        );
    }

    /// Test that Renderer produces a simple text node output.
    #[test]
    fn test_render_simple_text_node() {
        let renderer = Renderer;
        let nodes = vec![CslnNode::Text {
            value: "Hello, World!".to_string(),
        }];
        let item = RenderItem {
            item_type: ItemType::Book,
            variables: HashMap::new(),
        };

        let output = renderer.render_citation(&nodes, &item);
        assert_eq!(output, "Hello, World!");
    }

    /// Test that Renderer concatenates multiple text nodes.
    #[test]
    fn test_render_multiple_text_nodes() {
        let renderer = Renderer;
        let nodes = vec![
            CslnNode::Text {
                value: "First".to_string(),
            },
            CslnNode::Text {
                value: " Second".to_string(),
            },
        ];
        let item = RenderItem {
            item_type: ItemType::Book,
            variables: HashMap::new(),
        };

        let output = renderer.render_citation(&nodes, &item);
        assert_eq!(output, "First Second");
    }

    /// Test that Renderer renders a minimal group with a single text node.
    #[test]
    fn test_render_minimal_group() {
        let renderer = Renderer;
        let nodes = vec![CslnNode::Group(GroupBlock {
            children: vec![CslnNode::Text {
                value: "grouped".to_string(),
            }],
            delimiter: None,
            formatting: crate::FormattingOptions::default(),
            source_order: None,
        })];
        let item = RenderItem {
            item_type: ItemType::Book,
            variables: HashMap::new(),
        };

        let output = renderer.render_citation(&nodes, &item);
        assert_eq!(output, "grouped");
    }

    /// Test that Renderer suppresses empty groups.
    #[test]
    fn test_render_empty_group_suppressed() {
        let renderer = Renderer;
        let nodes = vec![
            CslnNode::Text {
                value: "Before".to_string(),
            },
            CslnNode::Group(GroupBlock {
                children: vec![],
                delimiter: None,
                formatting: crate::FormattingOptions::default(),
                source_order: None,
            }),
            CslnNode::Text {
                value: "After".to_string(),
            },
        ];
        let item = RenderItem {
            item_type: ItemType::Book,
            variables: HashMap::new(),
        };

        let output = renderer.render_citation(&nodes, &item);
        assert_eq!(output, "BeforeAfter");
    }
}
