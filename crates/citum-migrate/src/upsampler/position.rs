/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use super::{CitumNode, Upsampler, migrate_debug_enabled};
use csl_legacy::model::{self as legacy, CslNode as LNode};

/// Citation templates extracted from CSL `position` conditions.
///
/// These templates populate Citum citation position overrides: first/default,
/// subsequent, and immediate-repeat citation forms.
#[derive(Debug, Clone, Default)]
pub struct CitationPositionTemplates {
    /// Template nodes for the first/default citation form.
    pub first: Option<Vec<CitumNode>>,
    /// Template nodes for subsequent non-immediate repeats.
    pub subsequent: Option<Vec<CitumNode>>,
    /// Template nodes for immediate repeats (`ibid`, `ibid-with-locator`).
    pub ibid: Option<Vec<CitumNode>>,
    /// Whether the source tree mixed `position` with unsupported conditions.
    pub unsupported_mixed_conditions: bool,
}

fn unsupported_position_templates() -> CitationPositionTemplates {
    CitationPositionTemplates {
        unsupported_mixed_conditions: true,
        ..Default::default()
    }
}

fn unsupported_position_result<T>(result: Result<T, ()>) -> Result<T, CitationPositionTemplates> {
    result.map_err(|()| unsupported_position_templates())
}

impl CitationPositionTemplates {
    /// Returns true when at least one position-specific override is available.
    #[must_use]
    pub fn has_overrides(&self) -> bool {
        self.subsequent.is_some() || self.ibid.is_some()
    }
}

/// Citation position variant selected while rewriting legacy CSL position trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CitationPositionTarget {
    /// The default first-citation form.
    First,
    /// The non-immediate repeat citation form.
    Subsequent,
    /// Either immediate repeat form, with or without locator.
    IbidAny,
}

impl CitationPositionTarget {
    /// Returns true when the legacy CSL position token belongs to this target.
    pub(super) fn matches_token(self, token: &str) -> bool {
        match self {
            Self::First => token == "first",
            Self::Subsequent => token == "subsequent",
            Self::IbidAny => matches!(token, "ibid" | "ibid-with-locator"),
        }
    }
}

/// Summary of citation position conditions found in a legacy node tree.
#[derive(Debug, Default)]
pub(super) struct CitationPositionAnalysis {
    has_position_chooses: bool,
    explicit_subsequent: bool,
    explicit_ibid: bool,
    explicit_ibid_with_locator: bool,
    unsupported_mixed_conditions: bool,
}

impl CitationPositionAnalysis {
    /// Returns true when the legacy tree contains any position-conditioned choose.
    pub(super) fn has_position_chooses(&self) -> bool {
        self.has_position_chooses
    }

    /// Returns true when the legacy tree contains an explicit subsequent branch.
    pub(super) fn has_explicit_subsequent(&self) -> bool {
        self.explicit_subsequent
    }

    /// Returns true when the legacy tree contains any explicit ibid branch.
    pub(super) fn has_explicit_ibid(&self) -> bool {
        self.explicit_ibid || self.explicit_ibid_with_locator
    }

    /// Returns true when position branches cannot be safely specialized.
    pub(super) fn has_unsupported_mixed_conditions(&self) -> bool {
        self.unsupported_mixed_conditions
    }

    fn record_position_choose(&mut self) {
        self.has_position_chooses = true;
    }

    fn record_unsupported_mixed_conditions(&mut self) {
        self.unsupported_mixed_conditions = true;
    }
}

/// Analyze citation position conditions in a legacy CSL node tree.
pub(super) fn analyze_citation_positions(nodes: &[LNode]) -> CitationPositionAnalysis {
    let mut analysis = CitationPositionAnalysis::default();
    analyze_position_choose_nodes(nodes, &mut analysis);
    analysis
}

fn analyze_position_choose_nodes(nodes: &[LNode], analysis: &mut CitationPositionAnalysis) {
    for node in nodes {
        match node {
            LNode::Choose(choose) => {
                if choose_has_position_condition(choose) {
                    analysis.record_position_choose();
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
                analyze_position_choose_nodes(&substitute.children, analysis);
            }
            _ => {}
        }
    }
}

/// Returns true when a legacy choose has any CSL position condition.
pub(super) fn choose_has_position_condition(choose: &legacy::Choose) -> bool {
    choose.if_branch.position.is_some()
        || choose
            .else_if_branches
            .iter()
            .any(|branch| branch.position.is_some())
}

/// Returns true when the CSL position token can be specialized by the upsampler.
pub(super) fn is_supported_position_token(token: &str) -> bool {
    matches!(token, "first" | "subsequent" | "ibid" | "ibid-with-locator")
}

fn uses_default_match_mode(match_mode: Option<&str>) -> bool {
    matches!(match_mode, None | Some("all" | "any"))
}

fn branch_has_non_position_conditions(branch: &legacy::ChooseBranch) -> bool {
    branch.type_.is_some()
        || branch.variable.is_some()
        || branch.is_numeric.is_some()
        || branch.is_uncertain_date.is_some()
        || branch.locator.is_some()
}

fn branch_is_position_only(branch: &legacy::ChooseBranch) -> bool {
    // `position` may be the only semantic condition in this branch.
    branch.position.is_some()
        && !branch_has_non_position_conditions(branch)
        && uses_default_match_mode(branch.match_mode.as_deref())
}

/// Returns true when a branch carries no effective legacy CSL condition.
pub(super) fn branch_is_effectively_unconditional(branch: &legacy::ChooseBranch) -> bool {
    branch.position.is_none()
        && !branch_has_non_position_conditions(branch)
        && uses_default_match_mode(branch.match_mode.as_deref())
}

/// Per-choose accumulated state for pure fast-path position chooses.
#[derive(Default)]
struct BranchSeen {
    first: bool,
    subsequent: bool,
    ibid: bool,
    ibid_with_locator: bool,
    default_branch: bool,
}

fn seen_slot<'a>(seen: &'a mut BranchSeen, token: &str) -> Result<&'a mut bool, ()> {
    match token {
        "first" => Ok(&mut seen.first),
        "subsequent" => Ok(&mut seen.subsequent),
        "ibid" => Ok(&mut seen.ibid),
        "ibid-with-locator" => Ok(&mut seen.ibid_with_locator),
        _ => Err(()),
    }
}

fn record_explicit_position(
    analysis: &mut CitationPositionAnalysis,
    token: &str,
) -> Result<(), ()> {
    match token {
        "first" => Ok(()),
        "subsequent" => {
            analysis.explicit_subsequent = true;
            Ok(())
        }
        "ibid" => {
            analysis.explicit_ibid = true;
            Ok(())
        }
        "ibid-with-locator" => {
            analysis.explicit_ibid_with_locator = true;
            Ok(())
        }
        _ => Err(()),
    }
}

/// Validates one branch for the pure position-only fast path and updates `seen`.
///
/// Returns `false` when the choose requires mixed-tree specialization instead.
fn analyze_fast_path_branch(
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
            let Ok(saw) = seen_slot(seen, token) else {
                return false;
            };
            if *saw {
                if migrate_debug_enabled() {
                    tracing::debug!(
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
        if !branch_is_effectively_unconditional(branch) || seen.default_branch {
            return false;
        }
        seen.default_branch = true;
        true
    }
}

/// Returns true when a choose can be specialized by direct position branch selection.
pub(super) fn choose_uses_position_fast_path(choose: &legacy::Choose) -> bool {
    let mut seen = BranchSeen::default();
    if !analyze_fast_path_branch(&choose.if_branch, &mut seen, "if") {
        return false;
    }

    for (index, branch) in choose.else_if_branches.iter().enumerate() {
        let label = format!("else-if[{index}]");
        if !analyze_fast_path_branch(branch, &mut seen, &label) {
            return false;
        }
    }

    !(choose.else_branch.is_some() && seen.default_branch)
}

fn analyze_position_choose(choose: &legacy::Choose, analysis: &mut CitationPositionAnalysis) {
    let mut seen_pure = BranchSeen::default();

    for (index, branch) in std::iter::once(&choose.if_branch)
        .chain(choose.else_if_branches.iter())
        .enumerate()
    {
        let branch_label = if index == 0 {
            "if".to_string()
        } else {
            format!("else-if[{}]", index - 1)
        };

        let Some(position) = &branch.position else {
            continue;
        };

        let pure_position_branch = branch_is_position_only(branch);
        let mut matched_any = false;
        for token in position.split_whitespace() {
            if record_explicit_position(analysis, token).is_err() {
                analysis.record_unsupported_mixed_conditions();
                return;
            }
            let Ok(saw) = seen_slot(&mut seen_pure, token) else {
                analysis.record_unsupported_mixed_conditions();
                return;
            };
            if pure_position_branch && *saw {
                if migrate_debug_enabled() {
                    tracing::debug!(
                        "Upsampler: conflicting {token}-position branches at {branch_label}."
                    );
                }
                analysis.record_unsupported_mixed_conditions();
                return;
            }
            if pure_position_branch {
                *saw = true;
            }
            matched_any = true;
        }

        if !matched_any {
            analysis.record_unsupported_mixed_conditions();
            return;
        }
    }
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

/// Select children for the requested position target from a position-only choose.
pub(super) fn select_position_branch_children(
    choose: &legacy::Choose,
    target: CitationPositionTarget,
) -> &[LNode] {
    let mut fallback_branch: Option<&[LNode]> = None;
    let mut ibid_branch: Option<&[LNode]> = None;
    let mut ibid_with_locator_branch: Option<&[LNode]> = None;

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

impl Upsampler {
    /// Extract position-scoped citation templates from legacy CSL `choose` trees.
    ///
    /// Pure position-only trees still use direct branch selection. Mixed trees
    /// are specialized by stripping `position` from matching branches while
    /// preserving their remaining predicates and non-position sibling content.
    #[must_use]
    pub fn extract_citation_position_templates(
        &self,
        legacy_nodes: &[LNode],
    ) -> CitationPositionTemplates {
        let analysis = analyze_citation_positions(legacy_nodes);

        if !analysis.has_position_chooses() {
            return CitationPositionTemplates::default();
        }

        if analysis.has_unsupported_mixed_conditions() {
            return unsupported_position_templates();
        }

        let Ok(first) =
            self.extract_position_variant_if(legacy_nodes, true, CitationPositionTarget::First)
        else {
            return unsupported_position_templates();
        };

        let Ok(subsequent) = self.extract_position_variant_if(
            legacy_nodes,
            analysis.has_explicit_subsequent(),
            CitationPositionTarget::Subsequent,
        ) else {
            return unsupported_position_templates();
        };

        let Ok(ibid) = self.extract_position_variant_if(
            legacy_nodes,
            analysis.has_explicit_ibid(),
            CitationPositionTarget::IbidAny,
        ) else {
            return unsupported_position_templates();
        };

        CitationPositionTemplates {
            first,
            subsequent,
            ibid,
            unsupported_mixed_conditions: false,
        }
    }

    fn upsample_position_variant(
        &self,
        legacy_nodes: &[LNode],
        target: CitationPositionTarget,
    ) -> Result<Option<Vec<CitumNode>>, ()> {
        let rewritten = self.rewrite_nodes_for_position(legacy_nodes, target)?;
        let upsampled = self.upsample_nodes(&rewritten);
        if upsampled.is_empty() {
            Ok(None)
        } else {
            Ok(Some(upsampled))
        }
    }

    fn extract_position_variant_if(
        &self,
        legacy_nodes: &[LNode],
        should_extract: bool,
        target: CitationPositionTarget,
    ) -> Result<Option<Vec<CitumNode>>, CitationPositionTemplates> {
        if !should_extract {
            return Ok(None);
        }

        unsupported_position_result(self.upsample_position_variant(legacy_nodes, target))
    }

    fn rewrite_nodes_for_position(
        &self,
        legacy_nodes: &[LNode],
        target: CitationPositionTarget,
    ) -> Result<Vec<LNode>, ()> {
        let mut rewritten = Vec::new();

        for node in legacy_nodes {
            if let Some(rewritten_container) = self.rewrite_child_container(node, target)? {
                rewritten.push(rewritten_container);
                continue;
            }

            match node {
                LNode::Choose(choose) if choose_has_position_condition(choose) => {
                    if choose_uses_position_fast_path(choose) {
                        let selected = select_position_branch_children(choose, target);
                        rewritten.extend(self.rewrite_nodes_for_position(selected, target)?);
                    } else {
                        rewritten.extend(self.rewrite_mixed_position_choose(choose, target)?);
                    }
                }
                LNode::Choose(choose) => {
                    rewritten.push(self.rewrite_non_position_choose(choose, target)?);
                }
                _ => rewritten.push(node.clone()),
            }
        }

        Ok(rewritten)
    }

    fn rewrite_child_container(
        &self,
        node: &LNode,
        target: CitationPositionTarget,
    ) -> Result<Option<LNode>, ()> {
        let rewritten = match node {
            LNode::Group(group) => {
                let mut rewritten_group = group.clone();
                rewritten_group.children =
                    self.rewrite_nodes_for_position(&group.children, target)?;
                Some(LNode::Group(rewritten_group))
            }
            LNode::Names(names) => {
                let mut rewritten_names = names.clone();
                rewritten_names.children =
                    self.rewrite_nodes_for_position(&names.children, target)?;
                Some(LNode::Names(rewritten_names))
            }
            LNode::Substitute(substitute) => {
                let mut rewritten_substitute = substitute.clone();
                rewritten_substitute.children =
                    self.rewrite_nodes_for_position(&substitute.children, target)?;
                Some(LNode::Substitute(rewritten_substitute))
            }
            _ => None,
        };

        Ok(rewritten)
    }

    fn rewrite_choose_branch_children(
        &self,
        branch: &legacy::ChooseBranch,
        target: CitationPositionTarget,
    ) -> Result<legacy::ChooseBranch, ()> {
        let mut rewritten_branch = branch.clone();
        rewritten_branch.children = self.rewrite_nodes_for_position(&branch.children, target)?;
        Ok(rewritten_branch)
    }

    fn rewrite_non_position_choose(
        &self,
        choose: &legacy::Choose,
        target: CitationPositionTarget,
    ) -> Result<LNode, ()> {
        let mut rewritten_choose = choose.clone();
        rewritten_choose.if_branch =
            self.rewrite_choose_branch_children(&choose.if_branch, target)?;
        rewritten_choose.else_if_branches = choose
            .else_if_branches
            .iter()
            .map(|branch| self.rewrite_choose_branch_children(branch, target))
            .collect::<Result<Vec<_>, _>>()?;
        rewritten_choose.else_branch = choose
            .else_branch
            .as_ref()
            .map(|branch| self.rewrite_nodes_for_position(branch, target))
            .transpose()?;
        Ok(LNode::Choose(rewritten_choose))
    }

    fn rewrite_mixed_position_branch(
        &self,
        branch: &legacy::ChooseBranch,
        target: CitationPositionTarget,
    ) -> Result<Option<legacy::ChooseBranch>, ()> {
        let Some(position) = &branch.position else {
            return self
                .rewrite_choose_branch_children(branch, target)
                .map(Some);
        };

        let matched_target =
            position
                .split_whitespace()
                .try_fold(false, |matched_target, token| {
                    if !is_supported_position_token(token) {
                        return Err(());
                    }
                    Ok(matched_target || target.matches_token(token))
                })?;

        if !matched_target {
            return Ok(None);
        }

        let mut rewritten_branch = self.rewrite_choose_branch_children(branch, target)?;
        rewritten_branch.position = None;
        Ok(Some(rewritten_branch))
    }

    fn assemble_rewritten_position_choose(
        &self,
        mut rewritten_branches: Vec<legacy::ChooseBranch>,
        rewritten_else: Option<Vec<LNode>>,
    ) -> Result<Vec<LNode>, ()> {
        let unconditional_positions = rewritten_branches
            .iter()
            .enumerate()
            .filter_map(|(index, branch)| {
                branch_is_effectively_unconditional(branch).then_some(index)
            })
            .collect::<Vec<_>>();

        if unconditional_positions.len() + usize::from(rewritten_else.is_some()) > 1 {
            return Err(());
        }

        if rewritten_branches.is_empty() {
            return Ok(rewritten_else.unwrap_or_default());
        }

        if let Some(index) = unconditional_positions.first().copied() {
            if index == 0 {
                return Ok(rewritten_branches.remove(0).children);
            }

            let else_branch = rewritten_branches.remove(index).children;
            let mut branches = rewritten_branches.into_iter();
            let Some(if_branch) = branches.next() else {
                return Ok(else_branch);
            };

            return Ok(vec![LNode::Choose(legacy::Choose {
                if_branch,
                else_if_branches: branches.collect(),
                else_branch: Some(else_branch),
            })]);
        }

        let mut branches = rewritten_branches.into_iter();
        let Some(if_branch) = branches.next() else {
            return Ok(rewritten_else.unwrap_or_default());
        };

        Ok(vec![LNode::Choose(legacy::Choose {
            if_branch,
            else_if_branches: branches.collect(),
            else_branch: rewritten_else,
        })])
    }

    fn rewrite_mixed_position_choose(
        &self,
        choose: &legacy::Choose,
        target: CitationPositionTarget,
    ) -> Result<Vec<LNode>, ()> {
        let mut rewritten_branches = Vec::new();

        for branch in std::iter::once(&choose.if_branch).chain(choose.else_if_branches.iter()) {
            if let Some(rewritten_branch) = self.rewrite_mixed_position_branch(branch, target)? {
                rewritten_branches.push(rewritten_branch);
            }
        }

        let rewritten_else = choose
            .else_branch
            .as_ref()
            .map(|branch| self.rewrite_nodes_for_position(branch, target))
            .transpose()?;
        self.assemble_rewritten_position_choose(rewritten_branches, rewritten_else)
    }
}
