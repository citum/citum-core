use super::{
    ContributorForm, ContributorRole, CslnNode, DateForm, DateVariable, NumberVariable,
    SimpleVariable, TemplateCompiler, TemplateComponent, TemplateContributor, TemplateDate,
    TemplateNumber, TemplateTitle, TemplateVariable, TitleType, Variable,
};

impl TemplateCompiler {
    pub(super) fn compile_node(&self, node: &CslnNode) -> Option<TemplateComponent> {
        match node {
            CslnNode::Names(names) => self.compile_names(names),
            CslnNode::Date(date) => self.compile_date(date),
            CslnNode::Variable(var) => self.compile_variable(var),
            CslnNode::Term(term) => self.compile_term(term),
            _ => None,
        }
    }

    /// Compile a Names block into a Contributor component.
    pub(super) fn compile_names(
        &self,
        names: &citum_schema::NamesBlock,
    ) -> Option<TemplateComponent> {
        // Try to map the primary variable to a role
        let primary_role = self.map_variable_to_role(&names.variable);

        // Check if we should use a substitute instead of the primary
        // Rare contributor roles (composer, illustrator) often have author as first substitute
        let role = if let Some(role) = primary_role {
            // If primary is a rare role and we have substitutes, prefer the first common one
            let rare_roles = [
                ContributorRole::Composer,
                ContributorRole::Illustrator,
                ContributorRole::Interviewer,
                ContributorRole::Inventor,
                ContributorRole::Counsel,
                ContributorRole::CollectionEditor,
                ContributorRole::EditorialDirector,
                ContributorRole::OriginalAuthor,
                ContributorRole::ReviewedAuthor,
            ];

            if rare_roles.contains(&role) && !names.options.substitute.is_empty() {
                // Try to find a common role in the substitute list
                names
                    .options
                    .substitute
                    .iter()
                    .find_map(|var| self.map_variable_to_role(var))
                    .unwrap_or(role) // Fallback to primary if no valid substitute
            } else {
                role
            }
        } else {
            return None;
        };

        let form = match names.options.mode {
            Some(citum_schema::NameMode::Short) => ContributorForm::Short,
            Some(citum_schema::NameMode::Count) => ContributorForm::Short, // Map count to short
            _ => ContributorForm::Long,
        };

        let and = names.options.and.as_ref().map(|a| match a {
            citum_schema::AndTerm::Text => citum_schema::options::AndOptions::Text,
            citum_schema::AndTerm::Symbol => citum_schema::options::AndOptions::Symbol,
        });

        let shorten = names.options.et_al.as_ref().map(|et| {
            citum_schema::options::ShortenListOptions {
                min: et.min,
                use_first: et.use_first,
                use_last: None, // Legacy CSL 1.0 et-al doesn't have use_last
                and_others: citum_schema::options::AndOtherOptions::EtAl,
                delimiter_precedes_last: match names.options.delimiter_precedes_last {
                    Some(citum_schema::DelimiterPrecedes::Always) => {
                        citum_schema::options::DelimiterPrecedesLast::Always
                    }
                    Some(citum_schema::DelimiterPrecedes::Never) => {
                        citum_schema::options::DelimiterPrecedesLast::Never
                    }
                    Some(citum_schema::DelimiterPrecedes::AfterInvertedName) => {
                        citum_schema::options::DelimiterPrecedesLast::AfterInvertedName
                    }
                    _ => citum_schema::options::DelimiterPrecedesLast::Contextual,
                },
                subsequent_min: et.subsequent.as_ref().map(|s| s.min),
                subsequent_use_first: et.subsequent.as_ref().map(|s| s.use_first),
            }
        });

        let mut rendering = self.convert_formatting(&names.formatting);
        if let Some(label) = &names.options.label {
            rendering.strip_periods = label.formatting.strip_periods.or(rendering.strip_periods);
        }

        Some(TemplateComponent::Contributor(TemplateContributor {
            contributor: role,
            form,
            name_order: None, // Use global setting by default
            delimiter: names.options.delimiter.clone(),
            sort_separator: names.options.sort_separator.clone(),
            shorten,
            and,
            rendering,
            ..Default::default()
        }))
    }

    /// Map a Variable to `ContributorRole`.
    pub(super) fn map_variable_to_role(&self, var: &Variable) -> Option<ContributorRole> {
        match var {
            Variable::Author => Some(ContributorRole::Author),
            Variable::Editor => Some(ContributorRole::Editor),
            Variable::Translator => Some(ContributorRole::Translator),
            Variable::Director => Some(ContributorRole::Director),
            Variable::Composer => Some(ContributorRole::Composer),
            Variable::Illustrator => Some(ContributorRole::Illustrator),
            Variable::Interviewer => Some(ContributorRole::Interviewer),
            Variable::Recipient => Some(ContributorRole::Recipient),
            Variable::CollectionEditor => Some(ContributorRole::CollectionEditor),
            Variable::ContainerAuthor => Some(ContributorRole::ContainerAuthor),
            Variable::EditorialDirector => Some(ContributorRole::EditorialDirector),
            Variable::OriginalAuthor => Some(ContributorRole::OriginalAuthor),
            Variable::ReviewedAuthor => Some(ContributorRole::ReviewedAuthor),
            _ => None,
        }
    }

    /// Compile a Date block into a Date component.
    pub(super) fn compile_date(&self, date: &citum_schema::DateBlock) -> Option<TemplateComponent> {
        let date_var = self.map_variable_to_date(&date.variable)?;

        let form = match &date.options.parts {
            Some(citum_schema::DateParts::Year) => DateForm::Year,
            Some(citum_schema::DateParts::YearMonth) => DateForm::YearMonth,
            _ => match &date.options.form {
                Some(citum_schema::DateForm::Numeric) => DateForm::Full,
                Some(citum_schema::DateForm::Text) => DateForm::Full,
                None => DateForm::Year,
            },
        };

        Some(TemplateComponent::Date(TemplateDate {
            date: date_var,
            form,
            rendering: self.convert_formatting(&date.formatting),
            ..Default::default()
        }))
    }

    /// Map a Variable to `DateVariable`.
    pub(super) fn map_variable_to_date(&self, var: &Variable) -> Option<DateVariable> {
        match var {
            Variable::Issued => Some(DateVariable::Issued),
            Variable::Accessed => Some(DateVariable::Accessed),
            Variable::OriginalDate => Some(DateVariable::OriginalPublished),
            Variable::Submitted => Some(DateVariable::Submitted),
            Variable::EventDate => Some(DateVariable::EventDate),
            _ => None,
        }
    }

    /// Compile a Term block into a Term component.
    pub(super) fn compile_term(&self, term: &citum_schema::TermBlock) -> Option<TemplateComponent> {
        Some(TemplateComponent::Term(
            citum_schema::template::TemplateTerm {
                term: term.term,
                form: Some(term.form),
                rendering: self.convert_formatting(&term.formatting),
                overrides: None,
                ..Default::default()
            },
        ))
    }

    /// Build type overrides from `FormattingOptions`.
    fn build_type_overrides(
        &self,
        overrides: &std::collections::HashMap<
            citum_schema::ItemType,
            citum_schema::FormattingOptions,
        >,
    ) -> Option<
        std::collections::HashMap<
            citum_schema::template::TypeSelector,
            citum_schema::template::ComponentOverride,
        >,
    > {
        if overrides.is_empty() {
            None
        } else {
            Some(
                overrides
                    .iter()
                    .map(|(t, fmt)| {
                        use citum_schema::template::{ComponentOverride, TypeSelector};
                        (
                            TypeSelector::Single(self.item_type_to_string(t)),
                            ComponentOverride::Rendering(self.convert_formatting(fmt)),
                        )
                    })
                    .collect(),
            )
        }
    }

    /// Compile a Variable block into the appropriate component.
    pub(super) fn compile_variable(
        &self,
        var: &citum_schema::VariableBlock,
    ) -> Option<TemplateComponent> {
        // First, check if it's a contributor role
        if let Some(role) = self.map_variable_to_role(&var.variable) {
            return Some(TemplateComponent::Contributor(TemplateContributor {
                contributor: role,
                form: ContributorForm::Long,
                name_order: None, // Use global setting by default
                delimiter: None,
                rendering: self.convert_formatting(&var.formatting),
                ..Default::default()
            }));
        }

        // Check if it's a title
        if let Some(title_type) = self.map_variable_to_title(&var.variable) {
            // Convert overrides from FormattingOptions to Rendering
            if super::migrate_debug_enabled() {
                for (t, fmt) in &var.overrides {
                    eprintln!("  {t:?} -> {fmt:?}");
                }
            }
            let overrides = self.build_type_overrides(&var.overrides);
            return Some(TemplateComponent::Title(TemplateTitle {
                title: title_type,
                form: None,
                rendering: self.convert_formatting(&var.formatting),
                overrides,
                ..Default::default()
            }));
        }

        // Check if it's a number
        if let Some(num_var) = self.map_variable_to_number(&var.variable) {
            let mut rendering = self.convert_formatting(&var.formatting);
            if let Some(label) = &var.label {
                rendering.strip_periods =
                    label.formatting.strip_periods.or(rendering.strip_periods);
            }

            // Convert overrides from FormattingOptions to Rendering
            let overrides = self.build_type_overrides(&var.overrides);

            // Extract label form if present
            let label_form = var.label.as_ref().map(|l| self.map_label_form(&l.form));

            return Some(TemplateComponent::Number(TemplateNumber {
                number: num_var,
                form: None,
                label_form,
                rendering,
                overrides,
                ..Default::default()
            }));
        }

        // Check if it's a simple variable
        if let Some(simple_var) = self.map_variable_to_simple(&var.variable) {
            let mut rendering = self.convert_formatting(&var.formatting);
            let mut show_label = None;
            let mut strip_label_periods = None;

            if let Some(label) = &var.label {
                if matches!(simple_var, SimpleVariable::Locator) {
                    show_label = Some(true);
                    strip_label_periods = label.formatting.strip_periods;
                } else {
                    rendering.strip_periods =
                        label.formatting.strip_periods.or(rendering.strip_periods);
                }
            }

            // Convert overrides from FormattingOptions to Rendering
            let overrides = self.build_type_overrides(&var.overrides);
            return Some(TemplateComponent::Variable(TemplateVariable {
                variable: simple_var,
                show_label,
                strip_label_periods,
                rendering,
                overrides,
                ..Default::default()
            }));
        }

        None
    }

    /// Map a Variable to `TitleType`.
    pub(super) fn map_variable_to_title(&self, var: &Variable) -> Option<TitleType> {
        match var {
            Variable::Title => Some(TitleType::Primary),
            Variable::ContainerTitle => Some(TitleType::ParentSerial),
            Variable::CollectionTitle => Some(TitleType::ParentMonograph),
            _ => None,
        }
    }

    /// Map a Variable to `NumberVariable`.
    pub(super) fn map_variable_to_number(&self, var: &Variable) -> Option<NumberVariable> {
        match var {
            Variable::Volume => Some(NumberVariable::Volume),
            Variable::Issue => Some(NumberVariable::Issue),
            Variable::Page => Some(NumberVariable::Pages),
            Variable::Edition => Some(NumberVariable::Edition),
            Variable::ChapterNumber => Some(NumberVariable::ChapterNumber),
            Variable::CollectionNumber => Some(NumberVariable::CollectionNumber),
            Variable::NumberOfPages => Some(NumberVariable::NumberOfPages),
            Variable::CitationNumber => Some(NumberVariable::CitationNumber),
            Variable::Number => Some(NumberVariable::Number),
            _ => None,
        }
    }

    /// Map a Variable to `SimpleVariable`.
    pub(super) fn map_variable_to_simple(&self, var: &Variable) -> Option<SimpleVariable> {
        match var {
            Variable::DOI => Some(SimpleVariable::Doi),
            Variable::ISBN => Some(SimpleVariable::Isbn),
            Variable::ISSN => Some(SimpleVariable::Issn),
            Variable::URL => Some(SimpleVariable::Url),
            Variable::Publisher => Some(SimpleVariable::Publisher),
            Variable::PublisherPlace => Some(SimpleVariable::PublisherPlace),
            Variable::Genre => Some(SimpleVariable::Genre),
            Variable::Authority => Some(SimpleVariable::Authority),
            Variable::Archive => Some(SimpleVariable::Archive),
            Variable::ArchiveLocation => Some(SimpleVariable::ArchiveLocation),
            Variable::Version => Some(SimpleVariable::Version),
            Variable::Medium => Some(SimpleVariable::Medium),
            Variable::Source => Some(SimpleVariable::Source),
            Variable::Status => Some(SimpleVariable::Status),
            Variable::Locator => Some(SimpleVariable::Locator),
            Variable::PMID => Some(SimpleVariable::Pmid),
            Variable::PMCID => Some(SimpleVariable::Pmcid),
            Variable::Note => Some(SimpleVariable::Note),
            Variable::Annote => Some(SimpleVariable::Annote),
            Variable::Abstract => Some(SimpleVariable::Abstract),
            _ => None,
        }
    }
}
