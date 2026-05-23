/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use csl_legacy::model::{self as legacy, CslNode as LNode};

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
                if super::migrate_debug_enabled() {
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
                if super::migrate_debug_enabled() {
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
