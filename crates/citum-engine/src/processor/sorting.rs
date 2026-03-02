use crate::reference::Reference;
use citum_schema::locale::Locale;
use citum_schema::options::{Config, SortKey};

fn compare_optional_years(
    a_year: Option<i32>,
    b_year: Option<i32>,
    ascending: bool,
) -> std::cmp::Ordering {
    let cmp = match (a_year, b_year) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    };

    if ascending { cmp } else { cmp.reverse() }
}

/// Sorter for bibliography and citation entries.
pub struct Sorter<'a> {
    config: &'a Config,
    locale: &'a Locale,
}

impl<'a> Sorter<'a> {
    /// Creates a new `Sorter` instance.
    pub fn new(config: &'a Config, locale: &'a Locale) -> Self {
        Self { config, locale }
    }

    /// Sort references according to style instructions.
    ///
    /// This handles multi-key sorting based on the style's `SortSpec`. It includes
    /// specific logic for handling anonymous works (falling back from author to editor
    /// to title) and stripping articles for title-based sorting.
    pub fn sort_references<'b>(&self, references: Vec<&'b Reference>) -> Vec<&'b Reference> {
        let mut refs = references;
        let processing = self.config.processing.as_ref().cloned().unwrap_or_default();
        let proc_config = processing.config();

        if let Some(sort_config) = &proc_config.sort {
            // Build a composite sort that handles all keys together
            // Built-in processing defaults are bibliography-facing only; citation
            // cluster ordering remains explicit at the citation spec level.
            let resolved = sort_config.resolve();
            refs.sort_by(|a, b| {
                for sort in &resolved.template {
                    let cmp = match sort.key {
                        SortKey::Author => {
                            let a_sort_key = a
                                .author()
                                .and_then(|c| c.to_names_vec().first().cloned())
                                .map(|n| n.family_or_literal().to_lowercase())
                                .filter(|s| !s.is_empty())
                                .or_else(|| {
                                    a.editor()
                                        .and_then(|c| c.to_names_vec().first().cloned())
                                        .map(|n| n.family_or_literal().to_lowercase())
                                        .filter(|s| !s.is_empty())
                                })
                                .or_else(|| {
                                    a.title().map(|t| {
                                        self.locale
                                            .strip_sort_articles(&t.to_string())
                                            .to_lowercase()
                                    })
                                })
                                .unwrap_or_default();
                            let b_sort_key = b
                                .author()
                                .and_then(|c| c.to_names_vec().first().cloned())
                                .map(|n| n.family_or_literal().to_lowercase())
                                .filter(|s| !s.is_empty())
                                .or_else(|| {
                                    b.editor()
                                        .and_then(|c| c.to_names_vec().first().cloned())
                                        .map(|n| n.family_or_literal().to_lowercase())
                                        .filter(|s| !s.is_empty())
                                })
                                .or_else(|| {
                                    b.title().map(|t| {
                                        self.locale
                                            .strip_sort_articles(&t.to_string())
                                            .to_lowercase()
                                    })
                                })
                                .unwrap_or_default();

                            if sort.ascending {
                                a_sort_key.cmp(&b_sort_key)
                            } else {
                                b_sort_key.cmp(&a_sort_key)
                            }
                        }
                        SortKey::Year => {
                            let a_year = a
                                .issued()
                                .and_then(|d| d.year().parse::<i32>().ok())
                                .filter(|year| *year != 0);
                            let b_year = b
                                .issued()
                                .and_then(|d| d.year().parse::<i32>().ok())
                                .filter(|year| *year != 0);

                            compare_optional_years(a_year, b_year, sort.ascending)
                        }
                        SortKey::Title => {
                            let a_title = self
                                .locale
                                .strip_sort_articles(
                                    &a.title().map(|t| t.to_string()).unwrap_or_default(),
                                )
                                .to_lowercase();
                            let b_title = self
                                .locale
                                .strip_sort_articles(
                                    &b.title().map(|t| t.to_string()).unwrap_or_default(),
                                )
                                .to_lowercase();

                            if sort.ascending {
                                a_title.cmp(&b_title)
                            } else {
                                b_title.cmp(&a_title)
                            }
                        }
                        SortKey::CitationNumber => std::cmp::Ordering::Equal,
                        // Handle future SortKey variants (non_exhaustive)
                        _ => std::cmp::Ordering::Equal,
                    };

                    // If this key produces a non-equal comparison, use it
                    // Otherwise, continue to the next key
                    if cmp != std::cmp::Ordering::Equal {
                        return cmp;
                    }
                }
                std::cmp::Ordering::Equal
            });
        }

        refs
    }
}
