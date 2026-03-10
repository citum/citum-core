use citum_schema::{self as csln, FormattingOptions, ItemType, Variable};
use csl_legacy::model::{self as legacy, CslNode as LNode};
use std::collections::HashMap;
use std::sync::OnceLock;

fn migrate_debug_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("CITUM_MIGRATE_DEBUG")
            .map(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CitationPositionTarget {
    First,
    Subsequent,
    IbidAny,
}

impl CitationPositionTarget {
    fn matches_token(self, token: &str) -> bool {
        match self {
            Self::First => token == "first",
            Self::Subsequent => token == "subsequent",
            Self::IbidAny => matches!(token, "ibid" | "ibid-with-locator"),
        }
    }
}

#[derive(Debug, Default)]
struct CitationPositionAnalysis {
    has_position_chooses: bool,
    explicit_subsequent: bool,
    explicit_ibid: bool,
    explicit_ibid_with_locator: bool,
    unsupported_mixed_conditions: bool,
}

fn analyze_position_choose_nodes(nodes: &[LNode], analysis: &mut CitationPositionAnalysis) {
    for node in nodes {
        match node {
            LNode::Choose(choose) => {
                if choose_has_position_condition(choose) {
                    analysis.has_position_chooses = true;
                    analyze_position_choose(choose, analysis);
                }

                analyze_position_choose_nodes(&choose.if_branch.children, analysis);
                for branch in &choose.else_if_branches {
                    analyze_position_choose_nodes(&branch.children, analysis);
                }
                if let Some(else_branch) = &choose.else_branch {
                    analyze_position_choose_nodes(else_branch, analysis);
                }
            }
            LNode::Group(group) => analyze_position_choose_nodes(&group.children, analysis),
            LNode::Names(names) => analyze_position_choose_nodes(&names.children, analysis),
            LNode::Substitute(substitute) => {
                analyze_position_choose_nodes(&substitute.children, analysis)
            }
            _ => {}
        }
    }
}

fn choose_has_position_condition(choose: &legacy::Choose) -> bool {
    choose.if_branch.position.is_some()
        || choose
            .else_if_branches
            .iter()
            .any(|branch| branch.position.is_some())
}

fn branch_has_non_position_conditions(branch: &legacy::ChooseBranch) -> bool {
    branch.type_.is_some()
        || branch.variable.is_some()
        || branch.is_numeric.is_some()
        || branch.is_uncertain_date.is_some()
        || branch.locator.is_some()
}

fn branch_is_position_only(branch: &legacy::ChooseBranch) -> bool {
    // `position` may be the only semantic condition in this branch. Any other
    // condition indicates a mixed tree that this migration pass does not map.
    branch.position.is_some()
        && !branch_has_non_position_conditions(branch)
        && matches!(
            branch.match_mode.as_deref(),
            None | Some("all") | Some("any")
        )
}

fn branch_is_unconditional(branch: &legacy::ChooseBranch) -> bool {
    branch.position.is_none()
        && !branch_has_non_position_conditions(branch)
        && branch.match_mode.is_none()
}

/// Per-choose accumulated state for `analyze_position_choose`.
#[derive(Default)]
struct BranchSeen {
    first: bool,
    subsequent: bool,
    ibid: bool,
    ibid_with_locator: bool,
    default_branch: bool,
}

/// Validates one branch of a position-choose and updates `seen`.
///
/// Returns `false` if the branch is unsupported (mixed conditions or duplicate token).
fn analyze_one_branch(
    branch: &legacy::ChooseBranch,
    seen: &mut BranchSeen,
    branch_label: &str,
) -> bool {
    if let Some(position) = &branch.position {
        if !branch_is_position_only(branch) {
            return false;
        }
        let mut matched_any = false;
        for token in position.split_whitespace() {
            let saw: &mut bool = match token {
                "first" => &mut seen.first,
                "subsequent" => &mut seen.subsequent,
                "ibid" => &mut seen.ibid,
                "ibid-with-locator" => &mut seen.ibid_with_locator,
                _ => return false,
            };
            if *saw {
                if migrate_debug_enabled() {
                    eprintln!(
                        "Upsampler: conflicting {token}-position branches at {branch_label}."
                    );
                }
                return false;
            }
            *saw = true;
            matched_any = true;
        }
        matched_any
    } else {
        if !branch_is_unconditional(branch) || seen.default_branch {
            return false;
        }
        seen.default_branch = true;
        true
    }
}

fn analyze_position_choose(choose: &legacy::Choose, analysis: &mut CitationPositionAnalysis) {
    let mut seen = BranchSeen::default();

    if !analyze_one_branch(&choose.if_branch, &mut seen, "if") {
        analysis.unsupported_mixed_conditions = true;
        return;
    }

    for (index, branch) in choose.else_if_branches.iter().enumerate() {
        let label = format!("else-if[{index}]");
        if !analyze_one_branch(branch, &mut seen, &label) {
            analysis.unsupported_mixed_conditions = true;
            return;
        }
    }

    if choose.else_branch.is_some() && seen.default_branch {
        analysis.unsupported_mixed_conditions = true;
        return;
    }

    analysis.explicit_subsequent |= seen.subsequent;
    analysis.explicit_ibid |= seen.ibid;
    analysis.explicit_ibid_with_locator |= seen.ibid_with_locator;
}

fn branch_has_position_token(branch: &legacy::ChooseBranch, token: &str) -> bool {
    branch
        .position
        .as_deref()
        .is_some_and(|position| position.split_whitespace().any(|value| value == token))
}

fn node_contains_ibid_term(node: &LNode) -> bool {
    match node {
        LNode::Text(text) => text.term.as_deref() == Some("ibid"),
        LNode::Group(group) => group.children.iter().any(node_contains_ibid_term),
        LNode::Names(names) => names.children.iter().any(node_contains_ibid_term),
        LNode::Substitute(substitute) => substitute.children.iter().any(node_contains_ibid_term),
        LNode::Choose(choose) => {
            choose
                .if_branch
                .children
                .iter()
                .any(node_contains_ibid_term)
                || choose
                    .else_if_branches
                    .iter()
                    .any(|branch| branch.children.iter().any(node_contains_ibid_term))
                || choose
                    .else_branch
                    .as_ref()
                    .is_some_and(|branch| branch.iter().any(node_contains_ibid_term))
        }
        _ => false,
    }
}

#[derive(Default)]
pub struct Upsampler {
    provenance: Option<crate::ProvenanceTracker>,
    pub et_al_min: Option<usize>,
    pub et_al_use_first: Option<usize>,
}

/// Citation templates extracted from CSL `position` conditions.
///
/// These templates are used to populate Citum citation position overrides:
/// `first`/default branch -> base citation template,
/// `subsequent` branch -> `citation.subsequent`,
/// `ibid`/`ibid-with-locator` branches -> `citation.ibid`.
#[derive(Debug, Clone, Default)]
pub struct CitationPositionTemplates {
    /// Template nodes for the first/default citation form.
    pub first: Option<Vec<csln::CslnNode>>,
    /// Template nodes for subsequent non-immediate repeats.
    pub subsequent: Option<Vec<csln::CslnNode>>,
    /// Template nodes for immediate repeats (`ibid`, `ibid-with-locator`).
    pub ibid: Option<Vec<csln::CslnNode>>,
    /// Whether the source tree mixed `position` with unsupported conditions.
    pub unsupported_mixed_conditions: bool,
}

impl CitationPositionTemplates {
    /// Returns true when at least one position-specific override is available.
    pub fn has_overrides(&self) -> bool {
        self.subsequent.is_some() || self.ibid.is_some()
    }
}

impl Upsampler {
    pub fn new() -> Self {
        Self {
            provenance: None,
            et_al_min: None,
            et_al_use_first: None,
        }
    }

    pub fn with_provenance(provenance: crate::ProvenanceTracker) -> Self {
        Self {
            provenance: Some(provenance),
            et_al_min: None,
            et_al_use_first: None,
        }
    }

    /// The entry point for converting a flattened legacy tree into CSLN nodes.
    pub fn upsample_nodes(&self, legacy_nodes: &[LNode]) -> Vec<csln::CslnNode> {
        let mut csln_nodes = Vec::new();
        let mut i = 0;

        while i < legacy_nodes.len() {
            let node = &legacy_nodes[i];

            if let LNode::Group(group) = node
                && let Some(collapsed) = self.try_collapse_label_variable(group)
            {
                csln_nodes.push(collapsed);
                i += 1;
                continue;
            }

            if let Some(mapped) = self.map_node(node) {
                csln_nodes.push(mapped);
            }

            i += 1;
        }

        csln_nodes
    }

    /// Extract position-scoped citation templates from legacy CSL `choose` trees.
    ///
    /// This method only maps trees where `position` is the sole conditional
    /// discriminator. If a tree mixes `position` with other condition attributes
    /// (`type`, `variable`, `locator`, etc.), the mapping is marked unsupported.
    pub fn extract_citation_position_templates(
        &self,
        legacy_nodes: &[LNode],
    ) -> CitationPositionTemplates {
        let mut analysis = CitationPositionAnalysis::default();
        analyze_position_choose_nodes(legacy_nodes, &mut analysis);

        if !analysis.has_position_chooses {
            return CitationPositionTemplates::default();
        }

        if analysis.unsupported_mixed_conditions {
            return CitationPositionTemplates {
                unsupported_mixed_conditions: true,
                ..Default::default()
            };
        }

        CitationPositionTemplates {
            first: self.upsample_position_variant(legacy_nodes, CitationPositionTarget::First),
            subsequent: analysis
                .explicit_subsequent
                .then(|| {
                    self.upsample_position_variant(legacy_nodes, CitationPositionTarget::Subsequent)
                })
                .flatten(),
            ibid: if analysis.explicit_ibid_with_locator || analysis.explicit_ibid {
                self.upsample_position_variant(legacy_nodes, CitationPositionTarget::IbidAny)
            } else {
                None
            },
            unsupported_mixed_conditions: false,
        }
    }

    fn upsample_position_variant(
        &self,
        legacy_nodes: &[LNode],
        target: CitationPositionTarget,
    ) -> Option<Vec<csln::CslnNode>> {
        let rewritten = self.rewrite_nodes_for_position(legacy_nodes, target);
        let upsampled = self.upsample_nodes(&rewritten);
        if upsampled.is_empty() {
            None
        } else {
            Some(upsampled)
        }
    }

    fn rewrite_nodes_for_position(
        &self,
        legacy_nodes: &[LNode],
        target: CitationPositionTarget,
    ) -> Vec<LNode> {
        let mut rewritten = Vec::new();

        for node in legacy_nodes {
            match node {
                LNode::Group(group) => {
                    let mut rewritten_group = group.clone();
                    rewritten_group.children =
                        self.rewrite_nodes_for_position(&group.children, target);
                    rewritten.push(LNode::Group(rewritten_group));
                }
                LNode::Names(names) => {
                    let mut rewritten_names = names.clone();
                    rewritten_names.children =
                        self.rewrite_nodes_for_position(&names.children, target);
                    rewritten.push(LNode::Names(rewritten_names));
                }
                LNode::Substitute(substitute) => {
                    let mut rewritten_substitute = substitute.clone();
                    rewritten_substitute.children =
                        self.rewrite_nodes_for_position(&substitute.children, target);
                    rewritten.push(LNode::Substitute(rewritten_substitute));
                }
                LNode::Choose(choose) if choose_has_position_condition(choose) => {
                    let selected = self.select_position_branch_children(choose, target);
                    rewritten.extend(self.rewrite_nodes_for_position(selected, target));
                }
                LNode::Choose(choose) => {
                    let mut rewritten_choose = choose.clone();
                    rewritten_choose.if_branch.children =
                        self.rewrite_nodes_for_position(&choose.if_branch.children, target);
                    rewritten_choose.else_if_branches = choose
                        .else_if_branches
                        .iter()
                        .cloned()
                        .map(|mut branch| {
                            branch.children =
                                self.rewrite_nodes_for_position(&branch.children, target);
                            branch
                        })
                        .collect();
                    rewritten_choose.else_branch = choose
                        .else_branch
                        .as_ref()
                        .map(|branch| self.rewrite_nodes_for_position(branch, target));
                    rewritten.push(LNode::Choose(rewritten_choose));
                }
                _ => rewritten.push(node.clone()),
            }
        }

        rewritten
    }

    fn select_position_branch_children<'a>(
        &self,
        choose: &'a legacy::Choose,
        target: CitationPositionTarget,
    ) -> &'a [LNode] {
        let mut fallback_branch: Option<&'a [LNode]> = None;
        let mut ibid_branch: Option<&'a [LNode]> = None;
        let mut ibid_with_locator_branch: Option<&'a [LNode]> = None;

        for branch in std::iter::once(&choose.if_branch).chain(choose.else_if_branches.iter()) {
            if target == CitationPositionTarget::IbidAny {
                if branch_has_position_token(branch, "ibid") {
                    ibid_branch = Some(&branch.children);
                }
                if branch_has_position_token(branch, "ibid-with-locator") {
                    ibid_with_locator_branch = Some(&branch.children);
                }
            } else if let Some(position) = &branch.position {
                if position
                    .split_whitespace()
                    .any(|token| target.matches_token(token))
                {
                    return &branch.children;
                }
            } else if fallback_branch.is_none() {
                fallback_branch = Some(&branch.children);
            }
        }

        if target == CitationPositionTarget::IbidAny {
            if let Some(children) = ibid_with_locator_branch
                && (children.iter().any(node_contains_ibid_term) || ibid_branch.is_none())
            {
                return children;
            }
            if let Some(children) = ibid_branch {
                return children;
            }
            if let Some(children) = ibid_with_locator_branch {
                return children;
            }
        }

        fallback_branch
            .or(choose.else_branch.as_deref())
            .unwrap_or(&[])
    }

    fn map_node(&self, node: &LNode) -> Option<csln::CslnNode> {
        match node {
            LNode::Text(t) => {
                if let Some(var_str) = &t.variable
                    && let Some(var) = self.map_variable(var_str)
                {
                    if let Some(ref prov) = self.provenance {
                        let var_name = format!("{:?}", var).to_lowercase();
                        prov.record_upsampling(&var_name, "Text", "Variable");
                    }
                    if migrate_debug_enabled() {
                        eprintln!(
                            "Upsampler: Text({:?}) macro_call_order={:?}",
                            var, t.macro_call_order
                        );
                    }
                    return Some(csln::CslnNode::Variable(csln::VariableBlock {
                        variable: var,
                        label: None,
                        formatting: self.map_formatting(
                            &t.formatting,
                            &t.prefix,
                            &t.suffix,
                            t.quotes,
                            t.strip_periods,
                        ),
                        overrides: HashMap::new(),
                        source_order: t.macro_call_order,
                    }));
                }
                if let Some(term) = &t.term {
                    if let Some(general_term) = csln::locale::Locale::parse_general_term(term) {
                        return Some(csln::CslnNode::Term(csln::TermBlock {
                            term: general_term,
                            form: self.map_term_form(t.form.as_deref()),
                            formatting: self.map_formatting(
                                &t.formatting,
                                &t.prefix,
                                &t.suffix,
                                t.quotes,
                                t.strip_periods,
                            ),
                            source_order: t.macro_call_order,
                        }));
                    }

                    // Fallback for unknown terms
                    let prefix = t.prefix.as_deref().unwrap_or("");
                    let suffix = t.suffix.as_deref().unwrap_or("");
                    let text_cased = self.apply_text_case(term, t.text_case.as_deref());
                    return Some(csln::CslnNode::Text {
                        value: format!("{}{}{}", prefix, text_cased, suffix),
                    });
                }
                if let Some(val) = &t.value {
                    return Some(csln::CslnNode::Text { value: val.clone() });
                }
                None
            }
            LNode::Group(g) => Some(csln::CslnNode::Group(csln::GroupBlock {
                children: self.upsample_nodes(&g.children),
                delimiter: g.delimiter.clone(),
                formatting: self.map_formatting(&g.formatting, &g.prefix, &g.suffix, None, None),
                source_order: g.macro_call_order,
            })),
            LNode::Date(d) => self.map_date(d),
            LNode::Names(n) => self.map_names(n),
            LNode::Choose(c) => self.map_choose(c),
            LNode::Number(n) => self.map_number(n),
            LNode::Label(l) => self.map_label(l),
            _ => None,
        }
    }

    fn map_names(&self, n: &legacy::Names) -> Option<csln::CslnNode> {
        let vars: Vec<&str> = n.variable.split_whitespace().collect();
        if vars.is_empty() {
            return None;
        }

        let variable = self.map_variable(vars[0])?;

        let mut options = csln::NamesOptions {
            delimiter: n.delimiter.clone(),
            ..Default::default()
        };

        // If multiple variables were provided, add the others to substitute
        for v in vars.iter().skip(1) {
            if let Some(var) = self.map_variable(v) {
                options.substitute.push(var);
            }
        }

        // Extract et-al defaults from Names node, falling back to upsampler defaults
        let mut et_al_min = n.et_al_min.or(self.et_al_min);
        let mut et_al_use_first = n.et_al_use_first.or(self.et_al_use_first);
        let et_al_subsequent =
            if n.et_al_subsequent_min.is_some() || n.et_al_subsequent_use_first.is_some() {
                let fallback_min = et_al_min.unwrap_or(0) as u8;
                let fallback_use_first = et_al_use_first.unwrap_or(0) as u8;
                Some(Box::new(csln::EtAlSubsequent {
                    min: n
                        .et_al_subsequent_min
                        .map(|v| v as u8)
                        .unwrap_or(fallback_min),
                    use_first: n
                        .et_al_subsequent_use_first
                        .map(|v| v as u8)
                        .unwrap_or(fallback_use_first),
                }))
            } else {
                None
            };

        let mut et_al_term = "et al.".to_string();
        let et_al_formatting = FormattingOptions::default();

        for child in &n.children {
            match child {
                LNode::Name(name) => {
                    options.mode = match name.form.as_deref() {
                        Some("short") => Some(csln::NameMode::Short),
                        Some("count") => Some(csln::NameMode::Count),
                        _ => Some(csln::NameMode::Long),
                    };
                    options.and = match name.and.as_deref() {
                        Some("text") => Some(csln::AndTerm::Text),
                        Some("symbol") => Some(csln::AndTerm::Symbol),
                        _ => None,
                    };
                    options.initialize_with = name.initialize_with.clone();
                    options.sort_separator = name.sort_separator.clone();
                    options.name_as_sort_order = match name.name_as_sort_order.as_deref() {
                        Some("first") => Some(csln::NameAsSortOrder::First),
                        Some("all") => Some(csln::NameAsSortOrder::All),
                        _ => None,
                    };
                    options.delimiter_precedes_last = match name.delimiter_precedes_last.as_deref()
                    {
                        Some("contextual") => Some(csln::DelimiterPrecedes::Contextual),
                        Some("after-inverted-name") => {
                            Some(csln::DelimiterPrecedes::AfterInvertedName)
                        }
                        Some("always") => Some(csln::DelimiterPrecedes::Always),
                        Some("never") => Some(csln::DelimiterPrecedes::Never),
                        _ => None,
                    };

                    // Name node can also have et-al attributes
                    if name.et_al_min.is_some() {
                        et_al_min = name.et_al_min;
                    }
                    if name.et_al_use_first.is_some() {
                        et_al_use_first = name.et_al_use_first;
                    }
                }
                LNode::Label(label) => {
                    options.label = Some(csln::LabelOptions {
                        variable: variable.clone(),
                        form: self.map_label_form(&label.form),
                        pluralize: true,
                        formatting: self.map_formatting(
                            &label.formatting,
                            &label.prefix,
                            &label.suffix,
                            None,
                            label.strip_periods,
                        ),
                    });
                }
                LNode::EtAl(et_al) => {
                    if let Some(term) = &et_al.term {
                        et_al_term = term.clone();
                    }
                    // Formatting from et-al node? Legacy model needs to capture it.
                    // For now, default.
                }
                LNode::Substitute(sub) => {
                    for sub_node in &sub.children {
                        if let LNode::Names(sub_names) = sub_node
                            && let Some(sub_var) = self.map_variable(&sub_names.variable)
                        {
                            options.substitute.push(sub_var);
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(min) = et_al_min {
            options.et_al = Some(csln::EtAlOptions {
                min: min as u8,
                use_first: et_al_use_first.unwrap_or(1) as u8,
                subsequent: et_al_subsequent,
                term: et_al_term,
                formatting: et_al_formatting,
            });
        }

        if migrate_debug_enabled() {
            eprintln!(
                "Upsampler: Names({:?}) macro_call_order={:?}",
                variable, n.macro_call_order
            );
        }
        Some(csln::CslnNode::Names(csln::NamesBlock {
            variable,
            options,
            formatting: FormattingOptions::default(),
            source_order: n.macro_call_order,
        }))
    }

    fn map_number(&self, n: &legacy::Number) -> Option<csln::CslnNode> {
        let variable = self.map_variable(&n.variable)?;
        Some(csln::CslnNode::Variable(csln::VariableBlock {
            variable,
            label: None,
            formatting: self.map_formatting(&n.formatting, &n.prefix, &n.suffix, None, None),
            overrides: HashMap::new(),
            source_order: n.macro_call_order,
        }))
    }

    fn map_label(&self, l: &legacy::Label) -> Option<csln::CslnNode> {
        if let Some(var_str) = &l.variable
            && let Some(var) = self.map_variable(var_str)
        {
            return Some(csln::CslnNode::Variable(csln::VariableBlock {
                variable: var.clone(),
                label: Some(csln::LabelOptions {
                    variable: var,
                    form: self.map_label_form(&l.form),
                    pluralize: true,
                    formatting: self.map_formatting(
                        &l.formatting,
                        &l.prefix,
                        &l.suffix,
                        None,
                        l.strip_periods,
                    ),
                }),
                formatting: FormattingOptions::default(),
                overrides: HashMap::new(),
                source_order: l.macro_call_order,
            }));
        }
        None
    }

    fn map_choose(&self, c: &legacy::Choose) -> Option<csln::CslnNode> {
        // Handle is-uncertain-date condition specially: prefer else branch since most dates
        // aren't uncertain. Full EDTF support would handle this dynamically at render time.
        if c.if_branch.is_uncertain_date.is_some() {
            // Use else branch (non-uncertain formatting) as default
            if let Some(else_children) = &c.else_branch {
                let nodes = self.upsample_nodes(else_children);
                return nodes.into_iter().next();
            } else if !c.else_if_branches.is_empty() {
                let nodes = self.upsample_nodes(&c.else_if_branches[0].children);
                return nodes.into_iter().next();
            }
            // Fall through to if-branch if no else exists
        }

        // Handle position conditions (ibid, subsequent, etc.) by preferring else branch.
        // Position conditions are for repeated citations - else branch has full first-citation.
        let has_position_condition = c.if_branch.position.is_some()
            || c.else_if_branches.iter().any(|b| b.position.is_some());
        if has_position_condition {
            if let Some(else_children) = &c.else_branch {
                let nodes = self.upsample_nodes(else_children);
                return nodes.into_iter().next();
            }
            // If no else, try to find a branch without position (the "first" case)
            for branch in &c.else_if_branches {
                if branch.position.is_none() {
                    let nodes = self.upsample_nodes(&branch.children);
                    return nodes.into_iter().next();
                }
            }
            // Fall through if all branches have position conditions
        }

        // Determine if the if-branch uses match="none" (negated type test).
        // A negated if-branch fires for everything NOT in its type list, so it
        // behaves like a default/else branch rather than a type-specific branch.
        let if_match_none = c.if_branch.match_mode.as_deref() == Some("none");

        let mut if_item_type = Vec::new();
        if !if_match_none && let Some(types) = &c.if_branch.type_ {
            for t in types.split_whitespace() {
                if let Some(it) = self.map_item_type(t) {
                    if_item_type.push(it);
                }
            }
        }

        let mut if_variables = Vec::new();
        if let Some(vars) = &c.if_branch.variable {
            for v in vars.split_whitespace() {
                if let Some(var) = self.map_variable(v) {
                    if_variables.push(var);
                }
            }
        }

        // Map all else-if branches. For branches with match="none" (negated type
        // condition), clear the type list — they act as broad defaults, not as
        // type-specific branches. This ensures compile_for_type selects them as
        // the else/fallback path for types not covered by positive branches.
        let mut else_if_branches: Vec<csln::ElseIfBranch> = Vec::new();
        let mut negated_else_nodes: Option<Vec<csln::CslnNode>> = None;

        for branch in &c.else_if_branches {
            let is_match_none = branch.match_mode.as_deref() == Some("none");

            if is_match_none && branch.type_.is_some() {
                // Treat this as a fallback else branch, since it fires for all
                // types NOT in its type list (i.e., the "default" case).
                // Only adopt the first such branch to avoid duplicates.
                if negated_else_nodes.is_none() {
                    negated_else_nodes = Some(self.upsample_nodes(&branch.children));
                }
                continue;
            }

            let mut branch_item_types = Vec::new();
            if let Some(types) = &branch.type_ {
                for t in types.split_whitespace() {
                    if let Some(it) = self.map_item_type(t) {
                        branch_item_types.push(it);
                    }
                }
            }
            let mut branch_variables = Vec::new();
            if let Some(vars) = &branch.variable {
                for v in vars.split_whitespace() {
                    if let Some(var) = self.map_variable(v) {
                        branch_variables.push(var);
                    }
                }
            }
            else_if_branches.push(csln::ElseIfBranch {
                if_item_type: branch_item_types,
                if_variables: branch_variables,
                children: self.upsample_nodes(&branch.children),
            });
        }

        // Determine the effective else_branch: prefer the existing else branch,
        // then fall back to the negated else-if content if present.
        let else_branch = c
            .else_branch
            .as_ref()
            .map(|e| self.upsample_nodes(e))
            .or(negated_else_nodes);

        // Handle the if-branch match="none" case: push the if-branch content as
        // the else fallback, since it fires for all non-listed types.
        let (then_branch, else_branch) = if if_match_none {
            let if_nodes = self.upsample_nodes(&c.if_branch.children);
            // The existing else_branch takes priority; if_nodes become the else
            // only when there isn't already one.
            let effective_else = else_branch.or(Some(if_nodes));
            (Vec::new(), effective_else)
        } else {
            (self.upsample_nodes(&c.if_branch.children), else_branch)
        };

        Some(csln::CslnNode::Condition(csln::ConditionBlock {
            if_item_type,
            if_variables,
            then_branch,
            else_if_branches,
            else_branch,
        }))
    }

    fn map_item_type(&self, s: &str) -> Option<ItemType> {
        match s {
            "article" => Some(ItemType::Article),
            "article-journal" => Some(ItemType::ArticleJournal),
            "article-magazine" => Some(ItemType::ArticleMagazine),
            "article-newspaper" => Some(ItemType::ArticleNewspaper),
            "bill" => Some(ItemType::Bill),
            "book" => Some(ItemType::Book),
            "broadcast" => Some(ItemType::Broadcast),
            "chapter" => Some(ItemType::Chapter),
            "dataset" => Some(ItemType::Dataset),
            "entry" => Some(ItemType::Entry),
            "entry-dictionary" => Some(ItemType::EntryDictionary),
            "entry-encyclopedia" => Some(ItemType::EntryEncyclopedia),
            "figure" => Some(ItemType::Figure),
            "graphic" => Some(ItemType::Graphic),
            "interview" => Some(ItemType::Interview),
            "legal_case" => Some(ItemType::LegalCase),
            "legislation" => Some(ItemType::Legislation),
            "manuscript" => Some(ItemType::Manuscript),
            "map" => Some(ItemType::Map),
            "motion_picture" => Some(ItemType::MotionPicture),
            "musical_score" => Some(ItemType::MusicalScore),
            "pamphlet" => Some(ItemType::Pamphlet),
            "paper-conference" => Some(ItemType::PaperConference),
            "patent" => Some(ItemType::Patent),
            "personal_communication" => Some(ItemType::PersonalCommunication),
            "post" => Some(ItemType::Post),
            "post-weblog" => Some(ItemType::PostWeblog),
            "report" => Some(ItemType::Report),
            "review" => Some(ItemType::Review),
            "review-book" => Some(ItemType::ReviewBook),
            "song" => Some(ItemType::Song),
            "software" => Some(ItemType::Software),
            "speech" => Some(ItemType::Speech),
            "standard" => Some(ItemType::Standard),
            "thesis" => Some(ItemType::Thesis),
            "treaty" => Some(ItemType::Treaty),
            "webpage" => Some(ItemType::Webpage),
            _ => None,
        }
    }

    fn map_date(&self, d: &legacy::Date) -> Option<csln::CslnNode> {
        let variable = self.map_variable(&d.variable)?;
        let mut year_form = None;
        let mut month_form = None;
        let mut day_form = None;

        for part in &d.parts {
            match part.name.as_str() {
                "year" => year_form = self.map_date_part_form(&part.form),
                "month" => month_form = self.map_date_part_form(&part.form),
                "day" => day_form = self.map_date_part_form(&part.form),
                _ => {}
            }
        }

        if migrate_debug_enabled() {
            eprintln!(
                "Upsampler: Date({:?}) macro_call_order={:?}",
                variable, d.macro_call_order
            );
        }
        Some(csln::CslnNode::Date(csln::DateBlock {
            variable,
            options: csln::DateOptions {
                form: match d.form.as_deref() {
                    Some("text") => Some(csln::DateForm::Text),
                    Some("numeric") => Some(csln::DateForm::Numeric),
                    _ => None,
                },
                parts: match d.date_parts.as_deref() {
                    Some("year") => Some(csln::DateParts::Year),
                    Some("year-month") => Some(csln::DateParts::YearMonth),
                    _ => None,
                },
                delimiter: d.delimiter.clone(),
                year_form,
                month_form,
                day_form,
            },
            formatting: self.map_formatting(&d.formatting, &d.prefix, &d.suffix, None, None),
            source_order: d.macro_call_order,
        }))
    }

    fn map_date_part_form(&self, form: &Option<String>) -> Option<csln::DatePartForm> {
        match form.as_deref() {
            Some("numeric") => Some(csln::DatePartForm::Numeric),
            Some("numeric-leading-zeros") => Some(csln::DatePartForm::NumericLeadingZeros),
            Some("ordinal") => Some(csln::DatePartForm::Ordinal),
            Some("long") => Some(csln::DatePartForm::Long),
            Some("short") => Some(csln::DatePartForm::Short),
            _ => None,
        }
    }

    fn try_collapse_label_variable(&self, group: &legacy::Group) -> Option<csln::CslnNode> {
        if group.children.len() == 2 {
            let first = &group.children[0];
            let second = &group.children[1];

            if let (LNode::Label(l), LNode::Text(t)) = (first, second)
                && let (Some(l_var), Some(t_var)) = (&l.variable, &t.variable)
                && l_var == t_var
                && let Some(var) = self.map_variable(t_var)
            {
                return Some(csln::CslnNode::Variable(csln::VariableBlock {
                    variable: var.clone(),
                    label: Some(csln::LabelOptions {
                        variable: var,
                        form: self.map_label_form(&l.form),
                        pluralize: true,
                        formatting: self.map_formatting(
                            &l.formatting,
                            &l.prefix,
                            &l.suffix,
                            None,
                            l.strip_periods,
                        ),
                    }),
                    formatting: self.map_formatting(
                        &t.formatting,
                        &t.prefix,
                        &t.suffix,
                        t.quotes,
                        t.strip_periods,
                    ),
                    overrides: HashMap::new(),
                    source_order: t.macro_call_order,
                }));
            }
        }
        None
    }

    fn map_variable(&self, s: &str) -> Option<Variable> {
        match s {
            "title" => Some(Variable::Title),
            "container-title" => Some(Variable::ContainerTitle),
            "collection-title" => Some(Variable::CollectionTitle),
            "original-title" => Some(Variable::OriginalTitle),
            "publisher" => Some(Variable::Publisher),
            "publisher-place" => Some(Variable::PublisherPlace),
            "archive" => Some(Variable::Archive),
            "archive-place" => Some(Variable::ArchivePlace),
            "archive_location" => Some(Variable::ArchiveLocation),
            "event" => Some(Variable::Event),
            "event-place" => Some(Variable::EventPlace),
            "page" => Some(Variable::Page),
            "locator" => Some(Variable::Locator),
            "version" => Some(Variable::Version),
            "volume" => Some(Variable::Volume),
            "number-of-volumes" => Some(Variable::NumberOfVolumes),
            "issue" => Some(Variable::Issue),
            "chapter-number" => Some(Variable::ChapterNumber),
            "medium" => Some(Variable::Medium),
            "status" => Some(Variable::Status),
            "edition" => Some(Variable::Edition),
            "section" => Some(Variable::Section),
            "source" => Some(Variable::Source),
            "genre" => Some(Variable::Genre),
            "note" => Some(Variable::Note),
            "annote" => Some(Variable::Annote),
            "abstract" => Some(Variable::Abstract),
            "keyword" => Some(Variable::Keyword),
            "number" => Some(Variable::Number),
            "URL" => Some(Variable::URL),
            "DOI" => Some(Variable::DOI),
            "ISBN" => Some(Variable::ISBN),
            "ISSN" => Some(Variable::ISSN),
            "PMID" => Some(Variable::PMID),
            "PMCID" => Some(Variable::PMCID),
            "call-number" => Some(Variable::CallNumber),
            "dimensions" => Some(Variable::Dimensions),
            "scale" => Some(Variable::Scale),
            "jurisdiction" => Some(Variable::Jurisdiction),
            "citation-label" => Some(Variable::CitationLabel),
            "citation-number" => Some(Variable::CitationNumber),
            "year-suffix" => Some(Variable::YearSuffix),
            "author" => Some(Variable::Author),
            "editor" => Some(Variable::Editor),
            "editorial-director" => Some(Variable::EditorialDirector),
            "translator" => Some(Variable::Translator),
            "illustrator" => Some(Variable::Illustrator),
            "original-author" => Some(Variable::OriginalAuthor),
            "container-author" => Some(Variable::ContainerAuthor),
            "collection-editor" => Some(Variable::CollectionEditor),
            "composer" => Some(Variable::Composer),
            "director" => Some(Variable::Director),
            "interviewer" => Some(Variable::Interviewer),
            "recipient" => Some(Variable::Recipient),
            "reviewed-author" => Some(Variable::ReviewedAuthor),
            "issued" => Some(Variable::Issued),
            "event-date" => Some(Variable::EventDate),
            "accessed" => Some(Variable::Accessed),
            "container" => Some(Variable::Submitted),
            "original-date" => Some(Variable::OriginalDate),
            "available-date" => Some(Variable::AvailableDate),
            _ => None,
        }
    }

    fn map_label_form(&self, form: &Option<String>) -> csln::LabelForm {
        match form.as_deref() {
            Some("short") => csln::LabelForm::Short,
            Some("symbol") => csln::LabelForm::Symbol,
            _ => csln::LabelForm::Long,
        }
    }

    /// Apply text-case transformation to a string.
    /// Handles CSL 1.0 text-case attribute values for term nodes.
    fn apply_text_case(&self, text: &str, case: Option<&str>) -> String {
        match case {
            Some("capitalize-first") => {
                let mut chars = text.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
            Some("capitalize-all") => text
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
            Some("lowercase") => text.to_lowercase(),
            Some("uppercase") => text.to_uppercase(),
            _ => text.to_string(),
        }
    }

    fn map_formatting(
        &self,
        f: &legacy::Formatting,
        prefix: &Option<String>,
        suffix: &Option<String>,
        quotes: Option<bool>,
        strip_periods: Option<bool>,
    ) -> FormattingOptions {
        FormattingOptions {
            font_style: f.font_style.as_ref().map(|s| match s.as_str() {
                "italic" => csln::FontStyle::Italic,
                "oblique" => csln::FontStyle::Oblique,
                _ => csln::FontStyle::Normal,
            }),
            font_weight: f.font_weight.as_ref().map(|s| match s.as_str() {
                "bold" => csln::FontWeight::Bold,
                "light" => csln::FontWeight::Light,
                _ => csln::FontWeight::Normal,
            }),
            font_variant: f.font_variant.as_ref().map(|s| match s.as_str() {
                "small-caps" => csln::FontVariant::SmallCaps,
                _ => csln::FontVariant::Normal,
            }),
            text_decoration: f.text_decoration.as_ref().map(|s| match s.as_str() {
                "underline" => csln::TextDecoration::Underline,
                _ => csln::TextDecoration::None,
            }),
            vertical_align: f.vertical_align.as_ref().map(|s| match s.as_str() {
                "superscript" => csln::VerticalAlign::Superscript,
                "subscript" => csln::VerticalAlign::Subscript,
                _ => csln::VerticalAlign::Baseline,
            }),
            quotes,
            prefix: prefix.clone(),
            suffix: suffix.clone(),
            strip_periods,
        }
    }
    fn map_term_form(&self, form: Option<&str>) -> csln::locale::TermForm {
        match form {
            Some("short") => csln::locale::TermForm::Short,
            Some("verb") => csln::locale::TermForm::Verb,
            Some("verb-short") => csln::locale::TermForm::VerbShort,
            Some("symbol") => csln::locale::TermForm::Symbol,
            _ => csln::locale::TermForm::Long,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csl_legacy::model::{Choose, ChooseBranch, CslNode, Formatting, Group, Text};

    fn literal_text(value: &str) -> CslNode {
        CslNode::Text(Text {
            value: Some(value.to_string()),
            variable: None,
            macro_name: None,
            term: None,
            form: None,
            prefix: None,
            suffix: None,
            quotes: None,
            text_case: None,
            strip_periods: None,
            plural: None,
            macro_call_order: None,
            formatting: Formatting::default(),
        })
    }

    fn choose_branch(position: Option<&str>, children: Vec<CslNode>) -> ChooseBranch {
        ChooseBranch {
            match_mode: None,
            type_: None,
            variable: None,
            is_numeric: None,
            is_uncertain_date: None,
            locator: None,
            position: position.map(ToString::to_string),
            children,
        }
    }

    fn group(children: Vec<CslNode>) -> CslNode {
        CslNode::Group(Group {
            delimiter: None,
            prefix: None,
            suffix: None,
            children,
            macro_call_order: None,
            formatting: Formatting::default(),
        })
    }

    fn collect_text_values<'a>(nodes: &'a [csln::CslnNode], output: &mut Vec<&'a str>) {
        for node in nodes {
            match node {
                csln::CslnNode::Text { value } => output.push(value.as_str()),
                csln::CslnNode::Group(group) => collect_text_values(&group.children, output),
                csln::CslnNode::Condition(condition) => {
                    collect_text_values(&condition.then_branch, output);
                    for branch in &condition.else_if_branches {
                        collect_text_values(&branch.children, output);
                    }
                    if let Some(else_branch) = &condition.else_branch {
                        collect_text_values(else_branch, output);
                    }
                }
                _ => {}
            }
        }
    }

    fn text_values(nodes: &[csln::CslnNode]) -> Vec<&str> {
        let mut values = Vec::new();
        collect_text_values(nodes, &mut values);
        values
    }

    #[test]
    fn extract_position_templates_preserves_nested_choose_siblings() {
        let choose = Choose {
            if_branch: choose_branch(Some("subsequent"), vec![literal_text("SHORT")]),
            else_if_branches: vec![],
            else_branch: Some(vec![literal_text("FULL")]),
        };
        let legacy_nodes = vec![group(vec![
            literal_text("PREFIX"),
            CslNode::Choose(choose),
            literal_text("SUFFIX"),
        ])];

        let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

        assert!(!extracted.unsupported_mixed_conditions);
        assert_eq!(
            text_values(
                extracted
                    .first
                    .as_deref()
                    .expect("first variant should exist")
            ),
            vec!["PREFIX", "FULL", "SUFFIX"]
        );
        assert_eq!(
            text_values(
                extracted
                    .subsequent
                    .as_deref()
                    .expect("subsequent variant should exist")
            ),
            vec!["PREFIX", "SHORT", "SUFFIX"]
        );
    }

    #[test]
    fn extract_position_templates_merges_multiple_position_chooses() {
        let choose = Choose {
            if_branch: choose_branch(Some("subsequent"), vec![literal_text("SHORT")]),
            else_if_branches: vec![choose_branch(
                Some("first"),
                vec![literal_text("FIRST-ONLY")],
            )],
            else_branch: Some(vec![literal_text("FULL")]),
        };
        let locator_choose = Choose {
            if_branch: choose_branch(Some("ibid-with-locator"), vec![literal_text("LOC")]),
            else_if_branches: vec![choose_branch(Some("ibid"), vec![literal_text("IBID")])],
            else_branch: Some(vec![literal_text("DATE")]),
        };
        let legacy_nodes = vec![
            literal_text("A"),
            CslNode::Choose(choose),
            literal_text("MID"),
            CslNode::Choose(locator_choose),
            literal_text("Z"),
        ];

        let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

        assert!(!extracted.unsupported_mixed_conditions);
        assert_eq!(
            text_values(
                extracted
                    .first
                    .as_deref()
                    .expect("first variant should exist")
            ),
            vec!["A", "FIRST-ONLY", "MID", "DATE", "Z"],
        );
        assert_eq!(
            text_values(
                extracted
                    .subsequent
                    .as_deref()
                    .expect("subsequent variant should exist")
            ),
            vec!["A", "SHORT", "MID", "DATE", "Z"]
        );
        assert_eq!(
            text_values(
                extracted
                    .ibid
                    .as_deref()
                    .expect("ibid variant should exist")
            ),
            vec!["A", "FULL", "MID", "IBID", "Z"]
        );
    }

    #[test]
    fn extract_position_templates_selects_ibid_per_choose() {
        let ibid_only_choose = Choose {
            if_branch: choose_branch(Some("ibid"), vec![literal_text("IBID-ONLY")]),
            else_if_branches: vec![],
            else_branch: Some(vec![literal_text("DEFAULT-A")]),
        };
        let ibid_with_locator_choose = Choose {
            if_branch: choose_branch(
                Some("ibid-with-locator"),
                vec![literal_text("IBID-WITH-LOC")],
            ),
            else_if_branches: vec![],
            else_branch: Some(vec![literal_text("DEFAULT-B")]),
        };
        let legacy_nodes = vec![
            CslNode::Choose(ibid_only_choose),
            literal_text("MID"),
            CslNode::Choose(ibid_with_locator_choose),
        ];

        let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

        assert!(!extracted.unsupported_mixed_conditions);
        assert_eq!(
            text_values(
                extracted
                    .ibid
                    .as_deref()
                    .expect("ibid variant should exist")
            ),
            vec!["IBID-ONLY", "MID", "IBID-WITH-LOC"]
        );
    }

    #[test]
    fn extract_position_templates_preserves_nested_non_position_choose() {
        let nested_choose = Choose {
            if_branch: ChooseBranch {
                match_mode: None,
                type_: None,
                variable: Some("title".to_string()),
                is_numeric: None,
                is_uncertain_date: None,
                locator: None,
                position: None,
                children: vec![literal_text("LEGAL")],
            },
            else_if_branches: vec![],
            else_branch: Some(vec![literal_text("GENERAL")]),
        };
        let choose = Choose {
            if_branch: choose_branch(Some("subsequent"), vec![CslNode::Choose(nested_choose)]),
            else_if_branches: vec![],
            else_branch: Some(vec![literal_text("FULL")]),
        };
        let legacy_nodes = vec![CslNode::Choose(choose)];

        let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

        assert!(!extracted.unsupported_mixed_conditions);
        assert!(matches!(
            extracted.subsequent.as_deref(),
            Some([csln::CslnNode::Condition(_)])
        ));
    }

    #[test]
    fn extract_position_templates_marks_mixed_branch_conditions_as_unsupported() {
        let mut mixed_branch = choose_branch(Some("subsequent"), vec![literal_text("SHORT")]);
        mixed_branch.variable = Some("title".to_string());
        let choose = Choose {
            if_branch: mixed_branch,
            else_if_branches: vec![choose_branch(Some("ibid"), vec![literal_text("IBID")])],
            else_branch: Some(vec![literal_text("FULL")]),
        };
        let legacy_nodes = vec![CslNode::Choose(choose)];

        let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

        assert!(extracted.unsupported_mixed_conditions);
        assert!(!extracted.has_overrides());
        assert!(extracted.first.is_none());
    }

    #[test]
    fn extract_position_templates_marks_duplicate_position_branches_as_unsupported() {
        let choose = Choose {
            if_branch: choose_branch(Some("subsequent"), vec![literal_text("SHORT")]),
            else_if_branches: vec![choose_branch(
                Some("subsequent"),
                vec![literal_text("AGAIN")],
            )],
            else_branch: Some(vec![literal_text("FULL")]),
        };
        let legacy_nodes = vec![CslNode::Choose(choose)];

        let extracted = Upsampler::new().extract_citation_position_templates(&legacy_nodes);

        assert!(extracted.unsupported_mixed_conditions);
    }
}
