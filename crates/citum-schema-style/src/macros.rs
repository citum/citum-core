/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Declarative macros for the Citum ecosystem.

/// Generates a string-backed enum and its `as_str` method.
/// Preserves any doc comments and derive macros on the enum and its variants.
#[macro_export]
macro_rules! str_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident = $val:expr
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                #[doc = "String-backed enum variant."]
                $variant,
            )+
        }

        impl $name {
            #[doc = "Returns the string value associated with this variant."]
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( Self::$variant => $val, )+
                }
            }
        }
    }
}

/// Dispatches an operation across all variants of `TemplateComponent`.
/// Requires `$target` to be a `TemplateComponent` and provides `$inner`
/// to the closure/expression provided in `$action`.
#[macro_export]
macro_rules! dispatch_component {
    ($target:expr, |$inner:ident| $action:expr) => {
        match $target {
            $crate::template::TemplateComponent::Contributor($inner) => $action,
            $crate::template::TemplateComponent::Date($inner) => $action,
            $crate::template::TemplateComponent::Title($inner) => $action,
            $crate::template::TemplateComponent::Number($inner) => $action,
            $crate::template::TemplateComponent::Variable($inner) => $action,
            $crate::template::TemplateComponent::Group($inner) => $action,
            $crate::template::TemplateComponent::Term($inner) => $action,
        }
    };
}

/// Merges fields from a target struct `source` into a mutable `target` if `source.field.is_some()`.
/// This simplifies boilerplate in configuration merge implementations.
#[macro_export]
macro_rules! merge_options {
    ($target:expr, $source:expr, $($field:ident),+ $(,)?) => {
        $(
            if $source.$field.is_some() {
                $target.$field = $source.$field.clone();
            }
        )+
    };
}

// AST Builder macros for tests and embedded styles.
// These use a quasi-DSL to quickly stamp out TemplateComponents.

/// Build a contributor `TemplateComponent` with optional rendering overrides.
#[macro_export]
macro_rules! tc_contributor {
    ($role:ident, $form:ident $(, $key:ident = $val:expr)*) => {
        $crate::template::TemplateComponent::Contributor(
            $crate::template::TemplateContributor {
                contributor: $crate::template::ContributorRole::$role,
                form: $crate::template::ContributorForm::$form,
                rendering: $crate::template::Rendering {
                    $( $key: Some($val.into()), )*
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    };
}

/// Build a date `TemplateComponent` with optional rendering overrides.
#[macro_export]
macro_rules! tc_date {
    ($date_var:ident, $form:ident $(, $key:ident = $val:expr)*) => {
        $crate::template::TemplateComponent::Date(
            $crate::template::TemplateDate {
                date: $crate::template::DateVariable::$date_var,
                form: $crate::template::DateForm::$form,
                rendering: $crate::template::Rendering {
                    $( $key: Some($val.into()), )*
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    };
}

/// Build a title `TemplateComponent` with optional rendering overrides.
#[macro_export]
macro_rules! tc_title {
    ($title_type:ident $(, $key:ident = $val:expr)*) => {
        $crate::template::TemplateComponent::Title(
            $crate::template::TemplateTitle {
                title: $crate::template::TitleType::$title_type,
                rendering: $crate::template::Rendering {
                    $( $key: Some($val.into()), )*
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    };
}

/// Build a number `TemplateComponent` with optional rendering overrides.
#[macro_export]
macro_rules! tc_number {
    ($num_var:ident $(, $key:ident = $val:expr)*) => {
        $crate::template::TemplateComponent::Number(
            $crate::template::TemplateNumber {
                number: $crate::template::NumberVariable::$num_var,
                rendering: $crate::template::Rendering {
                    $( $key: Some($val.into()), )*
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    };
}

/// Build a variable `TemplateComponent` with optional rendering overrides.
#[macro_export]
macro_rules! tc_variable {
    ($var:ident $(, $key:ident = $val:expr)*) => {
        $crate::template::TemplateComponent::Variable(
            $crate::template::TemplateVariable {
                variable: $crate::template::SimpleVariable::$var,
                rendering: $crate::template::Rendering {
                    $( $key: Some($val.into()), )*
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    };
}

/// Build a term `TemplateComponent` with optional rendering overrides.
#[macro_export]
macro_rules! tc_term {
    ($term_var:ident $(, $key:ident = $val:expr)*) => {
        $crate::template::TemplateComponent::Term(
            $crate::template::TemplateTerm {
                term: $crate::locale::GeneralTerm::$term_var,
                rendering: $crate::template::Rendering {
                    $( $key: Some($val.into()), )*
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    };
}

/// Build a group `TemplateComponent` with optional rendering options.
#[macro_export]
macro_rules! tc_group {
    ([$($item:expr),* $(,)?] $(, $key:ident = $val:expr)*) => {
        $crate::template::TemplateComponent::Group(
            $crate::template::TemplateGroup {
                group: vec![$($item),*],
                rendering: $crate::template::Rendering {
                    $( $key: Some($val.into()), )*
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    };
}

// Reference builder macros for tests and fixtures.
// These construct native Citum InputReference values without verbose struct literals.

/// Builds an `InputReference::Monograph` (book) with a single structured-name author.
#[macro_export]
macro_rules! ref_book {
    ($id:expr, $family:expr, $given:expr, $year:expr, $title:expr) => {
        $crate::reference::InputReference::Monograph(::std::boxed::Box::new(
            $crate::reference::Monograph {
                id: Some($id.into()),
                r#type: $crate::reference::MonographType::Book,
                title: Some($crate::reference::Title::Single($title.to_string())),
                author: Some($crate::reference::Contributor::StructuredName(
                    $crate::reference::StructuredName {
                        family: $crate::reference::MultilingualString::Simple($family.to_string()),
                        given: $crate::reference::MultilingualString::Simple($given.to_string()),
                        ..Default::default()
                    },
                )),
                issued: $crate::reference::EdtfString($year.to_string()),
                ..Default::default()
            },
        ))
    };
}

/// Builds an `InputReference::Monograph` (book) with multiple structured-name authors.
#[macro_export]
macro_rules! ref_book_authors {
    ($id:expr, [$(($family:expr, $given:expr)),* $(,)?], $year:expr, $title:expr) => {{
        let _authors: Vec<$crate::reference::Contributor> = vec![
            $(
                $crate::reference::Contributor::StructuredName(
                    $crate::reference::StructuredName {
                        family: $crate::reference::MultilingualString::Simple(
                            $family.to_string(),
                        ),
                        given: $crate::reference::MultilingualString::Simple($given.to_string()),
                        ..Default::default()
                    },
                ),
            )*
        ];
        $crate::reference::InputReference::Monograph(::std::boxed::Box::new(
            $crate::reference::Monograph {
                id: Some($id.into()),
                r#type: $crate::reference::MonographType::Book,
                title: Some($crate::reference::Title::Single($title.to_string())),
                author: Some($crate::reference::Contributor::ContributorList(
                    $crate::reference::ContributorList(_authors),
                )),
                issued: $crate::reference::EdtfString($year.to_string()),
                ..Default::default()
            },
        ))
    }};
}

/// Builds an `InputReference::SerialComponent` (journal article) with a single author.
#[macro_export]
macro_rules! ref_article {
    ($id:expr, $family:expr, $given:expr, $year:expr, $title:expr) => {
        $crate::reference::InputReference::SerialComponent(::std::boxed::Box::new(
            $crate::reference::SerialComponent {
                id: Some($id.into()),
                r#type: $crate::reference::SerialComponentType::Article,
                title: Some($crate::reference::Title::Single($title.to_string())),
                author: Some($crate::reference::Contributor::StructuredName(
                    $crate::reference::StructuredName {
                        family: $crate::reference::MultilingualString::Simple($family.to_string()),
                        given: $crate::reference::MultilingualString::Simple($given.to_string()),
                        ..Default::default()
                    },
                )),
                issued: $crate::reference::EdtfString($year.to_string()),
                container: Some($crate::reference::WorkRelation::Embedded(
                    ::std::boxed::Box::new($crate::reference::InputReference::Serial(
                        ::std::boxed::Box::new($crate::reference::Serial {
                            r#type: $crate::reference::SerialType::AcademicJournal,
                            title: Some($crate::reference::Title::Single(String::new())),
                            ..Default::default()
                        }),
                    )),
                )),
                ..Default::default()
            },
        ))
    };
}

/// Builds an `InputReference::SerialComponent` (journal article) with multiple authors.
#[macro_export]
macro_rules! ref_article_authors {
    ($id:expr, [$(($family:expr, $given:expr)),* $(,)?], $year:expr, $title:expr) => {{
        let _authors: Vec<$crate::reference::Contributor> = vec![
            $(
                $crate::reference::Contributor::StructuredName(
                    $crate::reference::StructuredName {
                        family: $crate::reference::MultilingualString::Simple(
                            $family.to_string(),
                        ),
                        given: $crate::reference::MultilingualString::Simple($given.to_string()),
                        ..Default::default()
                    },
                ),
            )*
        ];
        $crate::reference::InputReference::SerialComponent(::std::boxed::Box::new(
            $crate::reference::SerialComponent {
                id: Some($id.into()),
                r#type: $crate::reference::SerialComponentType::Article,
                title: Some($crate::reference::Title::Single($title.to_string())),
                author: Some($crate::reference::Contributor::ContributorList(
                    $crate::reference::ContributorList(_authors),
                )),
                issued: $crate::reference::EdtfString($year.to_string()),
                ..Default::default()
            },
        ))
    }};
}

/// Builds a `CitationLocator` value.
#[macro_export]
macro_rules! citation_locator {
    ($label:ident, $value:expr) => {
        $crate::citation::CitationLocator::single(
            $crate::citation::LocatorType::$label,
            $value,
        )
    };
    ($l1:ident => $v1:expr, $l2:ident => $v2:expr $(, $lrest:ident => $vrest:expr)* $(,)?) => {
        $crate::citation::CitationLocator::compound(vec![
            $crate::citation::LocatorSegment::new(
                $crate::citation::LocatorType::$l1,
                $v1,
            ),
            $crate::citation::LocatorSegment::new(
                $crate::citation::LocatorType::$l2,
                $v2,
            ),
            $(
                $crate::citation::LocatorSegment::new(
                    $crate::citation::LocatorType::$lrest,
                    $vrest,
                )
            ),*
        ]).expect("compound locator macro requires at least two segments")
    };
}

/// Builds a `CitationItem` with optional named fields.
#[macro_export]
macro_rules! citation_item {
    ($id:expr $(, $key:ident = $val:expr)*) => {{
        #[allow(unused_mut)]
        let mut _item = $crate::citation::CitationItem {
            id: $id.to_string(),
            ..Default::default()
        };
        $( citation_item!(@set _item, $key, $val); )*
        _item
    }};
    (@set $item:ident, locator, $val:expr) => { $item.locator = Some($val); };
    (@set $item:ident, prefix, $val:expr) => { $item.prefix = Some($val.to_string()); };
    (@set $item:ident, suffix, $val:expr) => { $item.suffix = Some($val.to_string()); };
}

/// Builds a `Citation` from a list of `CitationItem` expressions with optional named fields.
#[macro_export]
macro_rules! citation {
    ([$($item:expr),* $(,)?] $(, $key:ident = $val:expr)* $(,)?) => {
        $crate::citation::Citation {
            items: vec![$($item),*],
            $($key: $val,)*
            ..Default::default()
        }
    };
}

/// Builds a `Citation` with one `CitationItem`.
#[macro_export]
macro_rules! cite {
    ($id:expr) => {
        $crate::citation::Citation {
            items: vec![$crate::citation::CitationItem {
                id: $id.to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }
    };
    ($id:expr, $key:ident = $val:expr) => {
        $crate::citation::Citation {
            items: vec![$crate::citation::CitationItem {
                id: $id.to_string(),
                ..Default::default()
            }],
            $key: $val,
            ..Default::default()
        }
    };
}

/// Builds an `IndexMap<String, InputReference>` from key-value pairs.
#[macro_export]
macro_rules! bib_map {
    ($($key:expr => $val:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut _map = indexmap::IndexMap::new();
        $( _map.insert($key.to_string(), $val); )*
        _map
    }};
}
