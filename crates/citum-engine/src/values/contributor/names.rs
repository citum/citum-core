//! Name-formatting helpers for contributor rendering.

use crate::values::{ProcHints, RenderOptions};
use citum_schema::options::contributors::NameForm;
use citum_schema::options::{
    AndOptions, AndOtherOptions, DemoteNonDroppingParticle, DisplayAsSort, ShortenListOptions,
};
use citum_schema::template::{ContributorForm, NameOrder};

/// Configuration for formatting a single name.
pub(crate) struct NameFormatContext<'a> {
    pub(crate) display_as_sort: Option<DisplayAsSort>,
    pub(crate) name_order: Option<&'a NameOrder>,
    pub(crate) initialize_with: Option<&'a String>,
    pub(crate) initialize_with_hyphen: Option<bool>,
    pub(crate) name_form: Option<NameForm>,
    pub(crate) demote_ndp: Option<&'a DemoteNonDroppingParticle>,
    pub(crate) sort_separator: Option<&'a String>,
}

/// Per-call template overrides passed to [`format_names`].
///
/// Bundles the optional override parameters that come from a
/// `TemplateContributor` so that call sites do not need to spell out each
/// one individually.
pub struct NamesOverrides<'a> {
    /// Override for name display order (given-first vs family-first).
    pub name_order: Option<&'a NameOrder>,
    /// Override for the sort separator (e.g. `","` or `" "`).
    pub sort_separator: Option<&'a String>,
    /// Override for et-al shortening options.
    pub shorten: Option<&'a ShortenListOptions>,
    /// Override for the "and" conjunction between names.
    pub and: Option<&'a AndOptions>,
    /// Override for the `initialize-with` string used to form initials.
    pub initialize_with: Option<&'a String>,
    /// Override for the name form (full, initials, family-only).
    pub name_form: Option<NameForm>,
}

/// Partition names into (`first_names`, `use_et_al`, `last_names`) based on et-al options.
fn partition_et_al<'a>(
    names: &'a [crate::reference::FlatName],
    shorten: Option<&'a ShortenListOptions>,
    hints: &'a ProcHints,
) -> (
    Vec<&'a crate::reference::FlatName>,
    bool,
    Vec<&'a crate::reference::FlatName>,
) {
    if let Some(opts) = shorten {
        // Determine effective min/use_first based on citation position.
        let is_subsequent = matches!(
            hints.position,
            Some(
                citum_schema::citation::Position::Subsequent
                    | citum_schema::citation::Position::Ibid
                    | citum_schema::citation::Position::IbidWithLocator
            )
        );
        let effective_min_threshold = if is_subsequent {
            opts.subsequent_min.unwrap_or(opts.min) as usize
        } else {
            opts.min as usize
        };
        let effective_use_first = if is_subsequent {
            opts.subsequent_use_first.unwrap_or(opts.use_first) as usize
        } else {
            opts.use_first as usize
        };

        // When min_names_to_show is set (name expansion disambiguation),
        // determine effective threshold for et-al application.
        let effective_min = if let Some(expanded) = hints.min_names_to_show {
            expanded.max(effective_use_first)
        } else {
            effective_use_first
        };

        // Apply et-al only if the list exceeds the minimum threshold
        if names.len() >= effective_min_threshold {
            if effective_min >= names.len() {
                (names.iter().collect::<Vec<_>>(), false, Vec::new())
            } else {
                let first: Vec<&crate::reference::FlatName> =
                    names.iter().take(effective_min).collect();
                let last: Vec<&crate::reference::FlatName> = if let Some(ul) = opts.use_last {
                    let take_last = ul as usize;
                    let skip = std::cmp::max(effective_min, names.len().saturating_sub(take_last));
                    names.iter().skip(skip).collect()
                } else {
                    Vec::new()
                };
                (first, true, last)
            }
        } else {
            (names.iter().collect::<Vec<_>>(), false, Vec::new())
        }
    } else {
        (names.iter().collect::<Vec<_>>(), false, Vec::new())
    }
}

/// Join a list of formatted names with a conjunction and Oxford-comma rules.
fn join_names_with_conjunction(
    formatted_first: &[String],
    and_str: Option<&str>,
    delimiter: &str,
    delimiter_precedes_last: Option<&citum_schema::options::DelimiterPrecedesLast>,
    first_names_len: usize,
    ctx: &NameFormatContext,
    context: crate::values::RenderContext,
) -> String {
    use citum_schema::options::{DelimiterPrecedesLast, DisplayAsSort};

    match and_str {
        None => {
            // No conjunction - just join all with delimiter
            formatted_first.join(delimiter)
        }
        Some(conjunction) if formatted_first.len() == 2 => {
            // For two names: citations don't use delimiter before conjunction,
            // but bibliographies do (contextual Oxford comma).
            let use_delimiter = if context == crate::values::RenderContext::Bibliography {
                if matches!(ctx.name_order, Some(NameOrder::GivenFirst)) {
                    false
                } else {
                    // In bibliography, check delimiter-precedes-last setting
                    match delimiter_precedes_last {
                        Some(DelimiterPrecedesLast::Always) => true,
                        Some(DelimiterPrecedesLast::Never) => false,
                        Some(DelimiterPrecedesLast::Contextual) | None => true, // Default: use comma in bibliography
                        Some(DelimiterPrecedesLast::AfterInvertedName) => {
                            ctx.display_as_sort.as_ref().is_some_and(|das| {
                                matches!(das, DisplayAsSort::All | DisplayAsSort::First)
                            })
                        }
                    }
                }
            } else {
                // In citations, never use delimiter before conjunction for 2 names
                false
            };

            #[allow(clippy::indexing_slicing, reason = "length checked")]
            if use_delimiter {
                format!(
                    "{}{}{} {}",
                    formatted_first[0], delimiter, conjunction, formatted_first[1]
                )
            } else {
                format!(
                    "{} {} {}",
                    formatted_first[0], conjunction, formatted_first[1]
                )
            }
        }
        Some(conjunction) => {
            if let Some((last, rest)) = formatted_first.split_last() {
                // Check if delimiter should precede "and" (Oxford comma)
                let use_delimiter = match delimiter_precedes_last {
                    Some(DelimiterPrecedesLast::Always) => true,
                    Some(DelimiterPrecedesLast::Never) => false,
                    Some(DelimiterPrecedesLast::Contextual) | None => true, // Default: comma for 3+ names
                    Some(DelimiterPrecedesLast::AfterInvertedName) => {
                        ctx.display_as_sort.as_ref().is_some_and(|das| {
                            matches!(das, DisplayAsSort::All)
                                || (matches!(das, DisplayAsSort::First) && first_names_len == 1)
                        })
                    }
                };
                if use_delimiter {
                    format!(
                        "{}{}{} {}",
                        rest.join(delimiter),
                        delimiter,
                        conjunction,
                        last
                    )
                } else {
                    format!("{} {} {}", rest.join(delimiter), conjunction, last)
                }
            } else {
                String::new()
            }
        }
    }
}

/// Parameters controlling et-al abbreviation formatting.
struct EtAlContext<'a> {
    and_others: AndOtherOptions,
    delimiter: &'a str,
    delimiter_precedes: Option<&'a citum_schema::options::DelimiterPrecedesLast>,
    first_count: usize,
}

/// Apply et-al suffix or return result unchanged.
fn apply_et_al(
    result: String,
    formatted_last: &[String],
    et_al: EtAlContext<'_>,
    ctx: &NameFormatContext,
    locale: &citum_schema::locale::Locale,
) -> String {
    use citum_schema::options::DelimiterPrecedesLast;

    if !formatted_last.is_empty() {
        // et-al-use-last: result + ellipsis + last names
        // CSL typically uses an ellipsis (...) for this.
        return format!("{} … {}", result, formatted_last.join(et_al.delimiter));
    }

    // Determine delimiter before "et al." based on delimiter_precedes_et_al option
    let use_delimiter = match et_al.delimiter_precedes {
        Some(DelimiterPrecedesLast::Always) => true,
        Some(DelimiterPrecedesLast::Never) => false,
        Some(DelimiterPrecedesLast::AfterInvertedName) => {
            // Use delimiter if last displayed name was inverted (family-first)
            ctx.display_as_sort.as_ref().is_some_and(|das| {
                matches!(das, DisplayAsSort::All)
                    || (matches!(das, DisplayAsSort::First) && et_al.first_count == 1)
            })
        }
        Some(DelimiterPrecedesLast::Contextual) | None => {
            // Default: use delimiter only if more than one name displayed
            et_al.first_count > 1
        }
    };

    let and_others_term = match et_al.and_others {
        AndOtherOptions::EtAl => locale.et_al(),
        AndOtherOptions::Text => locale.et_al().trim_end_matches('.'),
    };

    if use_delimiter {
        format!("{result}, {and_others_term}")
    } else {
        format!("{result} {and_others_term}")
    }
}

/// Format a list of names according to style options.
///
/// # Panics
///
/// This function assumes the non-empty input check at the top remains in place;
/// violating that invariant can trigger indexing or `unwrap()` panics in later
/// formatting branches.
#[must_use]
pub fn format_names(
    names: &[crate::reference::FlatName],
    form: &ContributorForm,
    options: &RenderOptions<'_>,
    overrides: &NamesOverrides<'_>,
    hints: &ProcHints,
) -> String {
    if names.is_empty() {
        return String::new();
    }

    let config = options.config.contributors.as_ref();
    let locale = options.locale;

    // Determine shortening options:
    // 1. Use explicit override from template (e.g. bibliography et-al)
    // 2. Else use global config
    let shorten = overrides
        .shorten
        .or_else(|| config.and_then(|c| c.shorten.as_ref()));

    let and_others = shorten.map_or(AndOtherOptions::EtAl, |opts| opts.and_others);

    let (first_names, use_et_al, last_names) = partition_et_al(names, shorten, hints);

    // Build format context once
    let ctx = NameFormatContext {
        display_as_sort: config.and_then(|c| c.display_as_sort),
        name_order: overrides.name_order,
        initialize_with: overrides
            .initialize_with
            .or_else(|| config.and_then(|c| c.initialize_with.as_ref())),
        initialize_with_hyphen: config.and_then(|c| c.initialize_with_hyphen),
        name_form: overrides
            .name_form
            .or_else(|| config.and_then(|c| c.name_form)),
        demote_ndp: config.and_then(|c| c.demote_non_dropping_particle.as_ref()),
        sort_separator: overrides
            .sort_separator
            .or_else(|| config.and_then(|c| c.sort_separator.as_ref())),
    };

    let delimiter = config.and_then(|c| c.delimiter.as_deref()).unwrap_or(", ");

    let formatted_first: Vec<String> = first_names
        .iter()
        .enumerate()
        .map(|(i, name)| format_single_name(name, form, i, &ctx, hints.expand_given_names))
        .collect();

    let formatted_last: Vec<String> = last_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let original_idx = names.len() - last_names.len() + i;
            format_single_name(name, form, original_idx, &ctx, hints.expand_given_names)
        })
        .collect();

    // Determine "and" setting: use override if provided, else global config
    let and_option = overrides
        .and
        .or_else(|| config.and_then(|c| c.and.as_ref()));

    // Determine conjunction between last two names
    // Default (None or no config) means no conjunction, matching CSL behavior
    let and_str = match and_option {
        Some(AndOptions::Text) => Some(locale.and_term(false)),
        Some(AndOptions::Symbol) => Some(locale.and_term(true)),
        Some(AndOptions::None) | None => None, // No conjunction
        _ => None,                             // Catch-all for future non_exhaustive variants
    };
    // When "et al." is applied, most styles expect comma-separated shown names
    // before the abbreviation (e.g., "Smith, Jones, et al."), not a final
    // conjunction ("Smith, Jones, and Brown, et al.").
    let and_str = if use_et_al && formatted_last.is_empty() {
        None
    } else {
        and_str
    };

    // Check if delimiter should precede last name (Oxford comma)
    let delimiter_precedes_last = config.and_then(|c| c.delimiter_precedes_last.as_ref());

    let result = if formatted_first.len() == 1 {
        #[allow(clippy::unwrap_used, reason = "length checked")]
        formatted_first.first().unwrap().clone()
    } else {
        join_names_with_conjunction(
            &formatted_first,
            and_str,
            delimiter,
            delimiter_precedes_last,
            first_names.len(),
            &ctx,
            options.context,
        )
    };

    if !use_et_al {
        return result;
    }

    apply_et_al(
        result,
        &formatted_last,
        EtAlContext {
            and_others,
            delimiter,
            delimiter_precedes: config.and_then(|c| c.delimiter_precedes_et_al.as_ref()),
            first_count: first_names.len(),
        },
        &ctx,
        locale,
    )
}

/// Initialize a given name by extracting initials.
///
/// Splits the given name on word separators (space, hyphen, non-breaking space),
/// and converts each part to its first character followed by the initialize suffix.
fn initialize_given_name(
    given: &str,
    initialize_with: Option<&String>,
    initialize_with_hyphen: Option<bool>,
) -> String {
    let init = initialize_with.map_or(". ", std::string::String::as_str);
    let separators = if initialize_with_hyphen == Some(false) {
        vec![' ', '\u{00A0}'] // Non-breaking space too
    } else {
        vec![' ', '-', '\u{00A0}']
    };

    let mut result = String::new();
    let mut current_part = String::new();

    for c in given.chars() {
        if separators.contains(&c) {
            if !current_part.is_empty() {
                if let Some(first) = current_part.chars().next() {
                    result.push(first);
                    result.push_str(init);
                }
                current_part.clear();
            }
            // Preserve only non-whitespace separators (e.g., hyphen for J.-P.).
            // Strip any trailing separator space before the hyphen so we get
            // "J.-P." rather than "J. -P." when init contains a trailing space.
            if !c.is_whitespace() {
                let trimmed_len = result.trim_end().len();
                result.truncate(trimmed_len);
                result.push(c);
            }
        } else {
            current_part.push(c);
        }
    }

    if !current_part.is_empty()
        && let Some(first) = current_part.chars().next()
    {
        result.push(first);
        result.push_str(init);
    }
    result.trim().to_string()
}

/// Assemble a long-form name from its computed parts.
///
/// When `inverted` is true uses "Family, Given" order; otherwise "Given Family".
fn assemble_long_name(
    family_part: String,
    given_part: String,
    particle_part: String,
    suffix: &str,
    inverted: bool,
    sort_separator: &str,
) -> String {
    if inverted {
        // "Family, Given" format
        // Family Part + sort_separator + Given Part + Particle Part + Suffix
        let mut suffix_part = String::new();
        if !given_part.is_empty() {
            suffix_part.push_str(&given_part);
        }
        if !particle_part.is_empty() {
            if !suffix_part.is_empty() {
                suffix_part.push(' ');
            }
            suffix_part.push_str(&particle_part);
        }
        if !suffix.is_empty() {
            if !suffix_part.is_empty() {
                suffix_part.push(' ');
            }
            suffix_part.push_str(suffix);
        }

        if suffix_part.is_empty() {
            family_part
        } else {
            format!("{family_part}{sort_separator}{suffix_part}")
        }
    } else {
        // "Given Family" format
        // Given Part + Particle Part + Family Part + Suffix
        let mut parts = Vec::new();
        if !given_part.is_empty() {
            parts.push(given_part);
        }
        if !particle_part.is_empty() {
            parts.push(particle_part);
        }
        if !family_part.is_empty() {
            if let Some(last) = parts.last_mut()
                && last.ends_with('-')
            {
                last.push_str(&family_part);
            } else {
                parts.push(family_part);
            }
        }
        if !suffix.is_empty() {
            parts.push(suffix.to_string());
        }

        parts.join(" ")
    }
}

/// Format a single name.
pub(crate) fn format_single_name(
    name: &crate::reference::FlatName,
    form: &ContributorForm,
    index: usize,
    ctx: &NameFormatContext,
    expand_given_names: bool,
) -> String {
    fn join_particle_family(particle: &str, family: &str) -> String {
        if particle.ends_with('-') {
            format!("{particle}{family}")
        } else {
            format!("{particle} {family}")
        }
    }

    // Handle literal names (e.g., corporate authors)
    if let Some(literal) = &name.literal {
        return literal.clone();
    }

    let family = name.family.as_deref().unwrap_or("");
    let given = name.given.as_deref().unwrap_or("");
    let dp = name.dropping_particle.as_deref().unwrap_or("");
    let ndp = name.non_dropping_particle.as_deref().unwrap_or("");
    let suffix = name.suffix.as_deref().unwrap_or("");

    // Determine if we should invert (Family, Given).
    // `display-as-sort: first` in the config limits inversion to the first name
    // even when the template requests `name-order: family-first` for all names.
    let inverted = match ctx.name_order {
        Some(NameOrder::GivenFirst) => false,
        Some(NameOrder::FamilyFirst) => match ctx.display_as_sort {
            Some(DisplayAsSort::First) => index == 0,
            _ => true,
        },
        Some(NameOrder::FamilyFirstOnly) => index == 0,
        None => match ctx.display_as_sort {
            Some(DisplayAsSort::All) => true,
            Some(DisplayAsSort::First) => index == 0,
            _ => false,
        },
    };

    // Determine effective form
    let effective_form = if expand_given_names && matches!(form, ContributorForm::Short) {
        &ContributorForm::Long
    } else {
        form
    };

    match effective_form {
        ContributorForm::FamilyOnly => {
            // FamilyOnly form strictly outputs literally just the family name without non-dropping particles.
            family.to_string()
        }
        ContributorForm::Short => {
            // Short form usually just family name, but includes non-dropping particle
            // e.g. "van Beethoven" (unless demoted? CSL spec says demote only affects sorting/display of full names mostly?)
            // Spec: "demote-non-dropping-particle ... This attribute does not affect ... the short form"
            // So for short form, we keep ndp with family.

            if ndp.is_empty() {
                family.to_string()
            } else {
                format!("{ndp} {family}")
            }
        }
        ContributorForm::Long | ContributorForm::Verb | ContributorForm::VerbShort => {
            // Determine parts based on demotion
            let demote = matches!(
                ctx.demote_ndp,
                Some(DemoteNonDroppingParticle::DisplayAndSort)
            );

            let family_part = if !ndp.is_empty() && !demote {
                join_particle_family(ndp, family)
            } else {
                family.to_string()
            };

            // Determine how to render the given name based on NameForm.
            // initialize-with only controls the separator between initials, not whether
            // to use initials at all. name-form controls the form.
            let effective_name_form = match ctx.name_form {
                Some(f) => f,
                None => NameForm::Full,
            };

            let given_part = match effective_name_form {
                NameForm::FamilyOnly => String::new(),
                NameForm::Initials => {
                    initialize_given_name(given, ctx.initialize_with, ctx.initialize_with_hyphen)
                }
                NameForm::Full => given.to_string(),
            };

            // Construct particle part (dropping + demoted non-dropping)
            let mut particle_part = String::new();
            if !dp.is_empty() {
                particle_part.push_str(dp);
            }
            if demote && !ndp.is_empty() {
                if !particle_part.is_empty() {
                    particle_part.push(' ');
                }
                particle_part.push_str(ndp);
            }

            let sep = ctx.sort_separator.map_or(", ", std::string::String::as_str);
            assemble_long_name(
                family_part,
                given_part,
                particle_part,
                suffix,
                inverted,
                sep,
            )
        }
    }
}

/// Format contributors in short form for citation grouping.
#[must_use]
pub fn format_contributors_short(
    names: &[crate::reference::FlatName],
    options: &RenderOptions<'_>,
) -> String {
    format_names(
        names,
        &ContributorForm::Short,
        options,
        &NamesOverrides {
            name_order: None,
            sort_separator: None,
            shorten: None,
            and: None,
            initialize_with: None,
            name_form: None,
        },
        &ProcHints::default(),
    )
}
