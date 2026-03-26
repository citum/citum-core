#![allow(missing_docs, reason = "test")]

use citum_migrate::passes::grouping::group_volume_and_issue;
use citum_schema::{
    options::Config,
    template::{
        DateForm, DateVariable, NumberVariable, Rendering, TemplateComponent, TemplateDate,
        TemplateGroup, TemplateNumber, WrapPunctuation,
    },
};

fn assert_number_component(
    component: Option<&TemplateComponent>,
    expected: NumberVariable,
    expect_clear_affixes: bool,
) {
    assert!(matches!(
        component,
        Some(TemplateComponent::Number(number))
            if number.number == expected
                && (!expect_clear_affixes
                    || (number.rendering.prefix.is_none() && number.rendering.suffix.is_none()))
    ));
}

fn assert_normalized_year_month_date(component: Option<&TemplateComponent>) {
    assert!(matches!(
        component,
        Some(TemplateComponent::Date(date))
            if date.date == DateVariable::Issued
                && date.form == DateForm::YearMonth
                && date.rendering.wrap == Some(WrapPunctuation::Parentheses)
                && date.rendering.prefix.is_none()
                && date.rendering.suffix.is_none()
    ));
}

fn expect_group<'a>(component: Option<&'a TemplateComponent>, context: &str) -> &'a TemplateGroup {
    match component {
        Some(TemplateComponent::Group(group)) => group,
        other => panic!("expected {context}, got {other:?}"),
    }
}

#[test]
fn migration_groups_adjacent_issue_and_year_month_inside_article_journal_detail_block() {
    let mut template = vec![
        TemplateComponent::Number(TemplateNumber {
            number: NumberVariable::Volume,
            rendering: Rendering {
                suffix: Some(", ".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }),
        TemplateComponent::Number(TemplateNumber {
            number: NumberVariable::Issue,
            rendering: Rendering {
                suffix: Some(" ".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }),
        TemplateComponent::Date(TemplateDate {
            date: DateVariable::Issued,
            form: DateForm::YearMonth,
            rendering: Rendering {
                prefix: Some("(".to_string()),
                suffix: Some(")".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }),
        TemplateComponent::Number(TemplateNumber {
            number: NumberVariable::Pages,
            ..Default::default()
        }),
    ];

    group_volume_and_issue(&mut template, &Config::default(), None);

    let detail_group = expect_group(template.first(), "grouped journal detail block");

    assert_eq!(
        detail_group.delimiter,
        Some(citum_schema::template::DelimiterPunctuation::Comma)
    );
    assert_number_component(detail_group.group.first(), NumberVariable::Volume, true);

    let issue_date_group = expect_group(detail_group.group.get(1), "nested issue/date group");

    assert_eq!(
        issue_date_group.delimiter,
        Some(citum_schema::template::DelimiterPunctuation::Space)
    );
    assert_number_component(issue_date_group.group.first(), NumberVariable::Issue, true);
    assert_normalized_year_month_date(issue_date_group.group.get(1));
    assert_number_component(template.get(1), NumberVariable::Pages, false);
}
