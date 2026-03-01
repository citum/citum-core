use super::*;

impl TemplateCompiler {
    pub(super) fn sort_citation_components(&self, components: &mut [TemplateComponent]) {
        components.sort_by_key(|c| match c {
            TemplateComponent::Contributor(c) if c.contributor == ContributorRole::Author => 0,
            TemplateComponent::Contributor(_) => 1,
            TemplateComponent::Date(d) if d.date == DateVariable::Issued => 2,
            TemplateComponent::Date(_) => 3,
            TemplateComponent::Title(_) => 4,
            _ => 5,
        });
    }

    /// Sort components for bibliography: citation-number first (for numeric styles),
    /// then author, date, title, then rest.
    #[allow(dead_code)]
    pub(super) fn sort_bibliography_components(
        &self,
        components: &mut [TemplateComponent],
        is_numeric: bool,
    ) {
        components.sort_by_key(|c| match c {
            // Citation number goes first for numeric bibliography styles
            TemplateComponent::Number(n) if n.number == NumberVariable::CitationNumber => 0,
            TemplateComponent::Contributor(c) if c.contributor == ContributorRole::Author => 1,
            TemplateComponent::Date(d) if d.date == DateVariable::Issued => {
                if is_numeric {
                    20
                } else {
                    2
                }
            }
            TemplateComponent::Title(t) if t.title == TitleType::Primary => 3,
            TemplateComponent::Title(t) if t.title == TitleType::ParentSerial => 4,
            TemplateComponent::Title(t) if t.title == TitleType::ParentMonograph => 5,
            TemplateComponent::Number(_) => 6,
            TemplateComponent::Variable(_) => 7,
            TemplateComponent::Contributor(_) => 8,
            TemplateComponent::Date(_) => 9,
            TemplateComponent::Title(_) => 10,
            TemplateComponent::List(l) => {
                if self.has_variable_recursive(
                    &l.items,
                    &TemplateComponent::Title(TemplateTitle {
                        title: TitleType::Primary,
                        ..Default::default()
                    }),
                ) {
                    3
                } else if self.has_variable_recursive(
                    &l.items,
                    &TemplateComponent::Title(TemplateTitle {
                        title: TitleType::ParentSerial,
                        ..Default::default()
                    }),
                ) {
                    4
                } else if self.has_variable_recursive(
                    &l.items,
                    &TemplateComponent::Title(TemplateTitle {
                        title: TitleType::ParentMonograph,
                        ..Default::default()
                    }),
                ) {
                    5
                } else {
                    11
                }
            }
            _ => 99,
        });
    }
}
