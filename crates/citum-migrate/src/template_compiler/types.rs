use super::*;

impl TemplateCompiler {
    pub(super) fn collect_types_with_branches(&self, nodes: &[CslnNode]) -> Vec<ItemType> {
        let mut types = Vec::new();
        Self::collect_types_recursive(nodes, &mut types);
        types.sort_by_key(|t| self.item_type_to_string(t));
        types.dedup_by_key(|t| self.item_type_to_string(t));
        types
    }

    #[allow(dead_code)]
    pub(super) fn collect_types_recursive(nodes: &[CslnNode], types: &mut Vec<ItemType>) {
        for node in nodes {
            match node {
                CslnNode::Group(g) => {
                    Self::collect_types_recursive(&g.children, types);
                }
                CslnNode::Condition(c) => {
                    // Collect types from if branch
                    types.extend(c.if_item_type.clone());

                    // Collect types from else-if branches
                    for else_if in &c.else_if_branches {
                        types.extend(else_if.if_item_type.clone());
                    }

                    // Recurse into branches
                    Self::collect_types_recursive(&c.then_branch, types);
                    for else_if in &c.else_if_branches {
                        Self::collect_types_recursive(&else_if.children, types);
                    }
                    if let Some(ref else_nodes) = c.else_branch {
                        Self::collect_types_recursive(else_nodes, types);
                    }
                }
                _ => {}
            }
        }
    }

    /// Compile a complete template for a specific item type.
    ///
    /// When encountering type-based conditions, selects the matching branch
    /// for the given type, or falls back to else branch if no match.
    /// Currently unused - infrastructure for future type_templates generation.
    #[allow(dead_code)]
    pub(super) fn compile_for_type(
        &self,
        nodes: &[CslnNode],
        target_type: &ItemType,
    ) -> Vec<TemplateComponent> {
        let mut components = Vec::new();

        for node in nodes {
            if let Some(component) = self.compile_node(node) {
                components.push(component);
            } else {
                match node {
                    CslnNode::Group(g) => {
                        components.extend(self.compile_for_type(&g.children, target_type));
                    }
                    CslnNode::Condition(c) => {
                        // Check if this is a type-based condition
                        let has_type_condition = !c.if_item_type.is_empty()
                            || c.else_if_branches
                                .iter()
                                .any(|b| !b.if_item_type.is_empty());

                        if has_type_condition {
                            // Select the matching branch for target_type
                            if c.if_item_type.contains(target_type) {
                                components
                                    .extend(self.compile_for_type(&c.then_branch, target_type));
                            } else {
                                // Check else-if branches
                                let mut found = false;
                                for else_if in &c.else_if_branches {
                                    if else_if.if_item_type.contains(target_type) {
                                        components.extend(
                                            self.compile_for_type(&else_if.children, target_type),
                                        );
                                        found = true;
                                        break;
                                    }
                                }
                                if !found {
                                    // Fall back to else branch
                                    if let Some(ref else_nodes) = c.else_branch {
                                        components
                                            .extend(self.compile_for_type(else_nodes, target_type));
                                    }
                                }
                            }
                        } else {
                            // Not a type condition, use default compile behavior
                            components.extend(self.compile_for_type(&c.then_branch, target_type));
                            if let Some(ref else_nodes) = c.else_branch {
                                let else_components =
                                    self.compile_for_type(else_nodes, target_type);
                                for ec in else_components {
                                    if !components.iter().any(|c| self.same_variable(c, &ec)) {
                                        components.push(ec);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        components
    }

    /// Convert ItemType to its string representation.
    #[allow(dead_code)]
    pub(super) fn item_type_to_string(&self, item_type: &ItemType) -> String {
        match item_type {
            ItemType::Article => "article".to_string(),
            ItemType::ArticleJournal => "article-journal".to_string(),
            ItemType::ArticleMagazine => "article-magazine".to_string(),
            ItemType::ArticleNewspaper => "article-newspaper".to_string(),
            ItemType::Bill => "bill".to_string(),
            ItemType::Book => "book".to_string(),
            ItemType::Broadcast => "broadcast".to_string(),
            ItemType::Chapter => "chapter".to_string(),
            ItemType::Dataset => "dataset".to_string(),
            ItemType::Entry => "entry".to_string(),
            ItemType::EntryDictionary => "entry-dictionary".to_string(),
            ItemType::EntryEncyclopedia => "entry-encyclopedia".to_string(),
            ItemType::Figure => "figure".to_string(),
            ItemType::Graphic => "graphic".to_string(),
            ItemType::Interview => "interview".to_string(),
            ItemType::LegalCase => "legal_case".to_string(),
            ItemType::Legislation => "legislation".to_string(),
            ItemType::Manuscript => "manuscript".to_string(),
            ItemType::Map => "map".to_string(),
            ItemType::MotionPicture => "motion_picture".to_string(),
            ItemType::MusicalScore => "musical_score".to_string(),
            ItemType::Pamphlet => "pamphlet".to_string(),
            ItemType::PaperConference => "paper-conference".to_string(),
            ItemType::Patent => "patent".to_string(),
            ItemType::PersonalCommunication => "personal_communication".to_string(),
            ItemType::Post => "post".to_string(),
            ItemType::PostWeblog => "post-weblog".to_string(),
            ItemType::Report => "report".to_string(),
            ItemType::Review => "review".to_string(),
            ItemType::ReviewBook => "review-book".to_string(),
            ItemType::Song => "song".to_string(),
            ItemType::Speech => "speech".to_string(),
            ItemType::Thesis => "thesis".to_string(),
            ItemType::Treaty => "treaty".to_string(),
            ItemType::Webpage => "webpage".to_string(),
            ItemType::Software => "software".to_string(),
            ItemType::Standard => "standard".to_string(),
        }
    }
}
