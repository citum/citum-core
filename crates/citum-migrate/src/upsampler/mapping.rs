/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

use super::{CitumNode, Upsampler, migrate_debug_enabled};
use crate::upsampler::position::choose_has_position_condition;
use citum_schema::{self as citum, FormattingOptions, ItemType, Variable};
use csl_legacy::model::{self as legacy, CslNode as LNode};
use std::collections::HashMap;

impl Upsampler {
    /// Convert one legacy CSL node into a Citum schema node.
    pub(super) fn map_node(&self, node: &LNode) -> Option<CitumNode> {
        match node {
            LNode::Text(t) => {
                if let Some(var_str) = &t.variable
                    && let Some(var) = self.map_variable(var_str)
                {
                    if let Some(ref prov) = self.provenance {
                        let var_name = format!("{var:?}").to_lowercase();
                        prov.record_upsampling(&var_name, "Text", "Variable");
                    }
                    if migrate_debug_enabled() {
                        tracing::debug!(
                            "Upsampler: Text({:?}) macro_call_order={:?}",
                            var,
                            t.macro_call_order
                        );
                    }
                    return Some(CitumNode::Variable(citum::VariableBlock {
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
                    if let Some(general_term) = citum::locale::Locale::parse_general_term(term) {
                        return Some(CitumNode::Term(citum::TermBlock {
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
                    return Some(CitumNode::Text {
                        value: format!("{prefix}{text_cased}{suffix}"),
                    });
                }
                if let Some(val) = &t.value {
                    return Some(CitumNode::Text { value: val.clone() });
                }
                None
            }
            LNode::Group(g) => Some(CitumNode::Group(citum::GroupBlock {
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

    fn apply_name_child_options(
        &self,
        n: &legacy::Names,
        variable: &citum::Variable,
        options: &mut citum::NamesOptions,
        et_al_min: &mut Option<usize>,
        et_al_use_first: &mut Option<usize>,
        et_al_term: &mut String,
    ) {
        for child in &n.children {
            match child {
                LNode::Name(name) => {
                    options.mode = match name.form.as_deref() {
                        Some("short") => Some(citum::NameMode::Short),
                        Some("count") => Some(citum::NameMode::Count),
                        _ => Some(citum::NameMode::Long),
                    };
                    options.and = match name.and.as_deref() {
                        Some("text") => Some(citum::AndTerm::Text),
                        Some("symbol") => Some(citum::AndTerm::Symbol),
                        _ => None,
                    };
                    options.initialize_with = name.initialize_with.clone();
                    options.sort_separator = name.sort_separator.clone();
                    options.name_as_sort_order = match name.name_as_sort_order.as_deref() {
                        Some("first") => Some(citum::NameAsSortOrder::First),
                        Some("all") => Some(citum::NameAsSortOrder::All),
                        _ => None,
                    };
                    options.delimiter_precedes_last = match name.delimiter_precedes_last.as_deref()
                    {
                        Some("contextual") => Some(citum::DelimiterPrecedes::Contextual),
                        Some("after-inverted-name") => {
                            Some(citum::DelimiterPrecedes::AfterInvertedName)
                        }
                        Some("always") => Some(citum::DelimiterPrecedes::Always),
                        Some("never") => Some(citum::DelimiterPrecedes::Never),
                        _ => None,
                    };

                    // Name node can also have et-al attributes
                    if name.et_al_min.is_some() {
                        *et_al_min = name.et_al_min;
                    }
                    if name.et_al_use_first.is_some() {
                        *et_al_use_first = name.et_al_use_first;
                    }
                }
                LNode::Label(label) => {
                    options.label = Some(citum::LabelOptions {
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
                        *et_al_term = term.clone();
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
    }

    fn map_names(&self, n: &legacy::Names) -> Option<CitumNode> {
        let vars: Vec<&str> = n.variable.split_whitespace().collect();
        if vars.is_empty() {
            return None;
        }

        #[allow(clippy::indexing_slicing, reason = "vars is not empty")]
        let variable = self.map_variable(vars[0])?;

        let mut options = citum::NamesOptions {
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
                Some(Box::new(citum::EtAlSubsequent {
                    min: n.et_al_subsequent_min.map_or(fallback_min, |v| v as u8),
                    use_first: n
                        .et_al_subsequent_use_first
                        .map_or(fallback_use_first, |v| v as u8),
                }))
            } else {
                None
            };

        let mut et_al_term = "et al.".to_string();
        let et_al_formatting = FormattingOptions::default();

        self.apply_name_child_options(
            n,
            &variable,
            &mut options,
            &mut et_al_min,
            &mut et_al_use_first,
            &mut et_al_term,
        );

        if let Some(min) = et_al_min {
            options.et_al = Some(citum::EtAlOptions {
                min: min as u8,
                use_first: et_al_use_first.unwrap_or(1) as u8,
                subsequent: et_al_subsequent,
                term: et_al_term,
                formatting: et_al_formatting,
            });
        }

        if migrate_debug_enabled() {
            tracing::debug!(
                "Upsampler: Names({:?}) macro_call_order={:?}",
                variable,
                n.macro_call_order
            );
        }
        Some(CitumNode::Names(citum::NamesBlock {
            variable,
            options,
            formatting: FormattingOptions::default(),
            source_order: n.macro_call_order,
        }))
    }

    fn map_number(&self, n: &legacy::Number) -> Option<CitumNode> {
        let variable = self.map_variable(&n.variable)?;
        Some(CitumNode::Variable(citum::VariableBlock {
            variable,
            label: None,
            formatting: self.map_formatting(&n.formatting, &n.prefix, &n.suffix, None, None),
            overrides: HashMap::new(),
            source_order: n.macro_call_order,
        }))
    }

    fn map_label(&self, l: &legacy::Label) -> Option<CitumNode> {
        if let Some(var_str) = &l.variable
            && let Some(var) = self.map_variable(var_str)
        {
            return Some(CitumNode::Variable(citum::VariableBlock {
                variable: var.clone(),
                label: Some(citum::LabelOptions {
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

    fn upsample_first_node(&self, nodes: &[LNode]) -> Option<CitumNode> {
        self.upsample_nodes(nodes).into_iter().next()
    }

    fn map_uncertain_date_choose(&self, choose: &legacy::Choose) -> Option<CitumNode> {
        choose.if_branch.is_uncertain_date.as_ref()?;

        choose
            .else_branch
            .as_deref()
            .and_then(|else_children| self.upsample_first_node(else_children))
            .or_else(|| {
                choose
                    .else_if_branches
                    .first()
                    .and_then(|branch| self.upsample_first_node(&branch.children))
            })
            .or_else(|| {
                // A bare uncertain-date branch means "emit this only for uncertain dates".
                // Migration defaults to the common certain-date case, so the absence of an
                // else branch should compile to no output rather than unconditional output.
                Some(CitumNode::Group(citum::GroupBlock {
                    children: vec![],
                    delimiter: None,
                    formatting: FormattingOptions::default(),
                    source_order: None,
                }))
            })
    }

    fn map_position_choose_fallback(&self, choose: &legacy::Choose) -> Option<CitumNode> {
        if !choose_has_position_condition(choose) {
            return None;
        }

        choose
            .else_branch
            .as_deref()
            .and_then(|else_children| self.upsample_first_node(else_children))
            .or_else(|| {
                choose
                    .else_if_branches
                    .iter()
                    .find(|branch| branch.position.is_none())
                    .and_then(|branch| self.upsample_first_node(&branch.children))
            })
    }

    fn map_branch_item_types(&self, type_names: Option<&str>) -> Vec<ItemType> {
        type_names
            .into_iter()
            .flat_map(|types| types.split_whitespace())
            .filter_map(|item_type| self.map_item_type(item_type))
            .collect()
    }

    fn map_branch_variables(&self, variable_names: Option<&str>) -> Vec<Variable> {
        variable_names
            .into_iter()
            .flat_map(|variables| variables.split_whitespace())
            .filter_map(|variable| self.map_variable(variable))
            .collect()
    }

    fn map_negated_else_if_fallback(
        &self,
        branch: &legacy::ChooseBranch,
    ) -> Option<Vec<CitumNode>> {
        (branch.match_mode.as_deref() == Some("none") && branch.type_.is_some())
            .then(|| self.upsample_nodes(&branch.children))
    }

    fn map_else_if_branch(&self, branch: &legacy::ChooseBranch) -> citum::ElseIfBranch {
        citum::ElseIfBranch {
            if_item_type: self.map_branch_item_types(branch.type_.as_deref()),
            if_variables: self.map_branch_variables(branch.variable.as_deref()),
            children: self.upsample_nodes(&branch.children),
        }
    }

    fn resolve_condition_else_branch(
        &self,
        choose: &legacy::Choose,
        negated_else_nodes: Option<Vec<CitumNode>>,
    ) -> Option<Vec<CitumNode>> {
        choose
            .else_branch
            .as_ref()
            .map(|branch| self.upsample_nodes(branch))
            .or(negated_else_nodes)
    }

    fn resolve_condition_branches(
        &self,
        choose: &legacy::Choose,
        if_match_none: bool,
        else_branch: Option<Vec<CitumNode>>,
    ) -> (Vec<CitumNode>, Option<Vec<CitumNode>>) {
        if if_match_none {
            let if_nodes = self.upsample_nodes(&choose.if_branch.children);
            (Vec::new(), else_branch.or(Some(if_nodes)))
        } else {
            (self.upsample_nodes(&choose.if_branch.children), else_branch)
        }
    }

    /// Convert a legacy conditional tree into a Citum condition or fallback node.
    pub(super) fn map_choose(&self, c: &legacy::Choose) -> Option<CitumNode> {
        // Handle is-uncertain-date condition specially: prefer else branch since most dates
        // aren't uncertain. Full EDTF support would handle this dynamically at render time.
        if let Some(node) = self.map_uncertain_date_choose(c) {
            return Some(node);
        }

        // Handle position conditions (ibid, subsequent, etc.) by preferring else branch.
        // Position conditions are for repeated citations - else branch has full first-citation.
        if let Some(node) = self.map_position_choose_fallback(c) {
            return Some(node);
        }

        // Determine if the if-branch uses match="none" (negated type test).
        // A negated if-branch fires for everything NOT in its type list, so it
        // behaves like a default/else branch rather than a type-specific branch.
        let if_match_none = c.if_branch.match_mode.as_deref() == Some("none");

        let if_item_type = if if_match_none {
            Vec::new()
        } else {
            self.map_branch_item_types(c.if_branch.type_.as_deref())
        };
        let if_variables = self.map_branch_variables(c.if_branch.variable.as_deref());

        // Map all else-if branches. For branches with match="none" (negated type
        // condition), clear the type list — they act as broad defaults, not as
        // type-specific branches. This ensures compile_for_type selects them as
        // the else/fallback path for types not covered by positive branches.
        let mut else_if_branches: Vec<citum::ElseIfBranch> = Vec::new();
        let mut negated_else_nodes: Option<Vec<CitumNode>> = None;

        for branch in &c.else_if_branches {
            if let Some(fallback_nodes) = self.map_negated_else_if_fallback(branch) {
                // Treat this as a fallback else branch, since it fires for all
                // types NOT in its type list (i.e., the "default" case).
                // Only adopt the first such branch to avoid duplicates.
                if negated_else_nodes.is_none() {
                    negated_else_nodes = Some(fallback_nodes);
                }
                continue;
            }

            else_if_branches.push(self.map_else_if_branch(branch));
        }

        // Determine the effective else_branch: prefer the existing else branch,
        // then fall back to the negated else-if content if present.
        let else_branch = self.resolve_condition_else_branch(c, negated_else_nodes);

        // Handle the if-branch match="none" case: push the if-branch content as
        // the else fallback, since it fires for all non-listed types.
        let (then_branch, else_branch) =
            self.resolve_condition_branches(c, if_match_none, else_branch);

        Some(CitumNode::Condition(citum::ConditionBlock {
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

    fn map_date(&self, d: &legacy::Date) -> Option<CitumNode> {
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
            tracing::debug!(
                "Upsampler: Date({:?}) macro_call_order={:?}",
                variable,
                d.macro_call_order
            );
        }
        Some(CitumNode::Date(citum::DateBlock {
            variable,
            options: citum::DateOptions {
                form: match d.form.as_deref() {
                    Some("text") => Some(citum::DateForm::Text),
                    Some("numeric") => Some(citum::DateForm::Numeric),
                    _ => None,
                },
                parts: match d.date_parts.as_deref() {
                    Some("year") => Some(citum::DateParts::Year),
                    Some("year-month") => Some(citum::DateParts::YearMonth),
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

    fn map_date_part_form(&self, form: &Option<String>) -> Option<citum::DatePartForm> {
        match form.as_deref() {
            Some("numeric") => Some(citum::DatePartForm::Numeric),
            Some("numeric-leading-zeros") => Some(citum::DatePartForm::NumericLeadingZeros),
            Some("ordinal") => Some(citum::DatePartForm::Ordinal),
            Some("long") => Some(citum::DatePartForm::Long),
            Some("short") => Some(citum::DatePartForm::Short),
            _ => None,
        }
    }

    /// Collapse adjacent label and variable output into one Citum variable node.
    pub(super) fn try_collapse_label_variable(&self, group: &legacy::Group) -> Option<CitumNode> {
        if group.children.len() == 2 {
            #[allow(clippy::indexing_slicing, reason = "group.children.len() == 2")]
            let first = &group.children[0];
            #[allow(clippy::indexing_slicing, reason = "group.children.len() == 2")]
            let second = &group.children[1];

            if let (LNode::Label(l), LNode::Text(t)) = (first, second)
                && let (Some(l_var), Some(t_var)) = (&l.variable, &t.variable)
                && l_var == t_var
                && let Some(var) = self.map_variable(t_var)
            {
                return Some(CitumNode::Variable(citum::VariableBlock {
                    variable: var.clone(),
                    label: Some(citum::LabelOptions {
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

    fn map_label_form(&self, form: &Option<String>) -> citum::LabelForm {
        match form.as_deref() {
            Some("short") => citum::LabelForm::Short,
            Some("symbol") => citum::LabelForm::Symbol,
            _ => citum::LabelForm::Long,
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
                "italic" => citum::FontStyle::Italic,
                "oblique" => citum::FontStyle::Oblique,
                _ => citum::FontStyle::Normal,
            }),
            font_weight: f.font_weight.as_ref().map(|s| match s.as_str() {
                "bold" => citum::FontWeight::Bold,
                "light" => citum::FontWeight::Light,
                _ => citum::FontWeight::Normal,
            }),
            font_variant: f.font_variant.as_ref().map(|s| match s.as_str() {
                "small-caps" => citum::FontVariant::SmallCaps,
                _ => citum::FontVariant::Normal,
            }),
            text_decoration: f.text_decoration.as_ref().map(|s| match s.as_str() {
                "underline" => citum::TextDecoration::Underline,
                _ => citum::TextDecoration::None,
            }),
            vertical_align: f.vertical_align.as_ref().map(|s| match s.as_str() {
                "superscript" => citum::VerticalAlign::Superscript,
                "subscript" => citum::VerticalAlign::Subscript,
                _ => citum::VerticalAlign::Baseline,
            }),
            quotes,
            prefix: prefix.clone(),
            suffix: suffix.clone(),
            strip_periods,
        }
    }
    fn map_term_form(&self, form: Option<&str>) -> citum::locale::TermForm {
        match form {
            Some("short") => citum::locale::TermForm::Short,
            Some("verb") => citum::locale::TermForm::Verb,
            Some("verb-short") => citum::locale::TermForm::VerbShort,
            Some("symbol") => citum::locale::TermForm::Symbol,
            _ => citum::locale::TermForm::Long,
        }
    }
}
