/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Name-formatting helpers for contributor rendering.

use crate::values::{ProcHints, RenderOptions};
use citum_schema::options::contributors::NameForm;
use citum_schema::options::{
    AndOptions, AndOtherOptions, DemoteNonDroppingParticle, DisplayAsSort, ShortenListOptions,
};
use citum_schema::template::{ContributorForm, NameOrder};
use unicode_script::{Script, UnicodeScript};

/// Configuration for formatting a single name.
pub(crate) struct NameFormatContext<'a> {
    pub(crate) display_as_sort: Option<DisplayAsSort>,
    pub(crate) name_order: Option<&'a NameOrder>,
    pub(crate) initialize_with: Option<&'a String>,
    pub(crate) initialize_with_hyphen: Option<bool>,
    pub(crate) name_form: Option<NameForm>,
    pub(crate) demote_ndp: Option<&'a DemoteNonDroppingParticle>,
    pub(crate) sort_separator: Option<&'a String>,
    pub(crate) component_sort_separator: Option<&'a String>,
    pub(crate) script_configs:
        Option<&'a std::collections::HashMap<String, citum_schema::options::ScriptConfig>>,
    pub(crate) integral_name_state: Option<citum_schema::citation::IntegralNameState>,
    pub(crate) org_abbreviation_state: Option<citum_schema::citation::IntegralNameState>,
    pub(crate) use_integral_short_name: bool,
    pub(crate) short_name_display: Option<citum_schema::options::ShortNameDisplay>,
    pub(crate) subsequent_form: Option<citum_schema::options::SubsequentNameForm>,
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
            // Two-name lists never use the delimiter before the conjunction
            // in citation context, and never in a given-first bibliography
            // name list (e.g. an editor/chair group rendered
            // "F. A. Editor & S. Editor" rather than "..., & ..."), regardless
            // of the declared delimiter-precedes-last value: there is no
            // per-component override for this option today, and real styles
            // (APA) rely on this suppression in both cases for correct
            // output. See div-013.
            let use_delimiter = if context == crate::values::RenderContext::Citation
                || matches!(ctx.name_order, Some(NameOrder::GivenFirst))
            {
                false
            } else {
                // Bibliography, not given-first: honor delimiter-precedes-last,
                // mirroring the 3+-name arm below except that `Contextual`/
                // unset means "delimiter only for 3+ names", so it resolves
                // to `false` here (previously hardcoded to `true`).
                match delimiter_precedes_last {
                    Some(DelimiterPrecedesLast::Always) => true,
                    Some(DelimiterPrecedesLast::Never) => false,
                    Some(DelimiterPrecedesLast::Contextual) | None => false,
                    Some(DelimiterPrecedesLast::AfterInvertedName) => {
                        ctx.display_as_sort.as_ref().is_some_and(|das| {
                            matches!(das, DisplayAsSort::All)
                                || (matches!(das, DisplayAsSort::First) && first_names_len == 1)
                        })
                    }
                }
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
        // et-al-use-last: result + ellipsis + last names. citeproc-js places
        // the configured name delimiter before the ellipsis whenever more
        // than one name is shown before it (continuing the same list
        // punctuation used between those names); a single shown name is
        // followed by a plain space instead. This placement does not consult
        // `delimiter-precedes-et-al` — citeproc-js ignores that option here.
        let joined_last = formatted_last.join(et_al.delimiter);
        return if et_al.first_count > 1 {
            format!("{result}{}… {joined_last}", et_al.delimiter)
        } else {
            format!("{result} … {joined_last}")
        };
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
        format!("{result}{}{and_others_term}", et_al.delimiter)
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
#[allow(
    clippy::too_many_lines,
    reason = "linear context-building pipeline; no clean split point"
)]
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
        component_sort_separator: overrides.sort_separator,
        script_configs: options
            .config
            .multilingual
            .as_ref()
            .map(|multilingual| &multilingual.scripts),
        integral_name_state: hints.integral_name_state,
        org_abbreviation_state: hints.org_abbreviation_state,
        use_integral_short_name: matches!(
            options.mode,
            citum_schema::citation::CitationMode::Integral
        ),
        short_name_display: options
            .config
            .org_abbreviation_memory
            .as_ref()
            .map(|c| c.resolve().short_name_display),
        subsequent_form: options
            .config
            .integral_name_memory
            .as_ref()
            .map(|c| c.resolve().subsequent_form),
    };

    let delimiter = config.and_then(|c| c.delimiter.as_deref()).unwrap_or(", ");

    let formatted_first: Vec<String> = first_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let expand =
                hints.expand_given_names && !(hints.expand_given_names_primary_only && i > 0);
            format_single_name(name, form, i, &ctx, expand)
        })
        .collect();

    let formatted_last: Vec<String> = last_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let original_idx = names.len() - last_names.len() + i;
            let expand = hints.expand_given_names
                && !(hints.expand_given_names_primary_only && original_idx > 0);
            format_single_name(name, form, original_idx, &ctx, expand)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameAssemblyOrder {
    GivenFirst,
    NativeFamilyFirst,
    Inverted,
}

#[derive(Debug, Default)]
struct NameScriptFlags {
    has_han: bool,
    has_hiragana: bool,
    has_katakana: bool,
    has_hangul: bool,
}

impl NameScriptFlags {
    fn record(&mut self, value: &str) {
        for ch in value.chars() {
            match ch.script() {
                Script::Han => self.has_han = true,
                Script::Hiragana => self.has_hiragana = true,
                Script::Katakana => self.has_katakana = true,
                Script::Hangul => self.has_hangul = true,
                _ => {}
            }
        }
    }

    fn cjk_script_count(&self) -> usize {
        usize::from(self.has_han)
            + usize::from(self.has_hiragana)
            + usize::from(self.has_katakana)
            + usize::from(self.has_hangul)
    }

    fn candidate_keys(&self) -> Vec<&'static str> {
        let count = self.cjk_script_count();
        if count == 0 {
            return Vec::new();
        }
        // Mixed kana (Hiragana + Katakana, no Han/Hangul) matches "kana" before "cjk".
        if count > 1
            && !self.has_han
            && !self.has_hangul
            && (self.has_hiragana || self.has_katakana)
        {
            return vec!["kana", "Hrkt", "cjk"];
        }
        if count > 1 {
            return vec!["cjk"];
        }
        if self.has_katakana {
            return vec!["katakana", "Kana", "Hrkt", "kana", "cjk"];
        }
        if self.has_hiragana {
            return vec!["hiragana", "Hira", "Hrkt", "kana", "cjk"];
        }
        if self.has_han {
            return vec!["han", "Hani", "cjk"];
        }
        if self.has_hangul {
            return vec!["hangul", "Hang", "cjk"];
        }
        Vec::new()
    }
}

fn script_config_for_name<'a>(
    name: &crate::reference::FlatName,
    ctx: &'a NameFormatContext<'a>,
) -> Option<&'a citum_schema::options::ScriptConfig> {
    let configs = ctx.script_configs?;
    if configs.is_empty() {
        return None;
    }

    let mut flags = NameScriptFlags::default();
    for part in [
        name.family.as_deref(),
        name.given.as_deref(),
        name.dropping_particle.as_deref(),
        name.non_dropping_particle.as_deref(),
        name.suffix.as_deref(),
        // A transliterated name carries its source script here, so script
        // options (e.g. use-native-ordering) apply to the romanized form too.
        name.original_script.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        flags.record(part);
    }

    flags.candidate_keys().into_iter().find_map(|key| {
        configs.get(key).or_else(|| {
            // Style authors write ISO 15924 keys in their canonical casing
            // ("Han", "Hangul"); candidate keys are lowercase aliases.
            configs.iter().find_map(|(config_key, config)| {
                config_key.eq_ignore_ascii_case(key).then_some(config)
            })
        })
    })
}

/// Assemble a long-form name from its computed parts.
///
/// Inverted order uses "Family, Given"; native family-first uses family-first
/// display order without sort punctuation.
fn assemble_long_name(
    family_part: String,
    given_part: String,
    particle_part: String,
    suffix: &str,
    order: NameAssemblyOrder,
    name_part_delimiter: &str,
    sort_separator: &str,
) -> String {
    match order {
        NameAssemblyOrder::Inverted => assemble_inverted_long_name(
            family_part,
            given_part,
            particle_part,
            suffix,
            sort_separator,
        ),
        NameAssemblyOrder::NativeFamilyFirst => assemble_native_family_first_long_name(
            family_part,
            given_part,
            particle_part,
            suffix,
            name_part_delimiter,
        ),
        NameAssemblyOrder::GivenFirst => assemble_given_first_long_name(
            family_part,
            given_part,
            particle_part,
            suffix,
            name_part_delimiter,
        ),
    }
}

fn assemble_inverted_long_name(
    family_part: String,
    given_part: String,
    particle_part: String,
    suffix: &str,
    sort_separator: &str,
) -> String {
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
}

fn assemble_native_family_first_long_name(
    family_part: String,
    given_part: String,
    particle_part: String,
    suffix: &str,
    name_part_delimiter: &str,
) -> String {
    let mut parts = Vec::new();
    if !family_part.is_empty() {
        parts.push(family_part);
    }
    if !particle_part.is_empty() {
        parts.push(particle_part);
    }
    if !given_part.is_empty() {
        parts.push(given_part);
    }
    if !suffix.is_empty() {
        parts.push(suffix.to_string());
    }
    parts.join(name_part_delimiter)
}

fn assemble_given_first_long_name(
    family_part: String,
    given_part: String,
    particle_part: String,
    suffix: &str,
    name_part_delimiter: &str,
) -> String {
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
    parts.join(name_part_delimiter)
}

fn format_literal_name(literal: &str, short: Option<&str>, ctx: &NameFormatContext) -> String {
    if ctx.use_integral_short_name
        && let Some(short) = short
    {
        match ctx.org_abbreviation_state {
            Some(citum_schema::citation::IntegralNameState::First) => {
                return match ctx.short_name_display {
                    Some(citum_schema::options::ShortNameDisplay::ShortThenBracketed) => {
                        format!("{short} [{literal}]")
                    }
                    Some(citum_schema::options::ShortNameDisplay::ShortThenParenthetical) => {
                        format!("{short} ({literal})")
                    }
                    Some(citum_schema::options::ShortNameDisplay::FullThenBracketed) => {
                        format!("{literal} [{short}]")
                    }
                    _ => format!("{literal} ({short})"),
                };
            }
            Some(citum_schema::citation::IntegralNameState::Subsequent) => {
                return short.to_string();
            }
            _ => {}
        }
    }
    literal.to_string()
}

fn is_inverted_name_order(index: usize, ctx: &NameFormatContext) -> bool {
    match ctx.name_order {
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
    }
}

fn name_assembly_order(
    inverted: bool,
    script_config: Option<&citum_schema::options::ScriptConfig>,
    ctx: &NameFormatContext,
) -> NameAssemblyOrder {
    if inverted {
        return NameAssemblyOrder::Inverted;
    }
    let native_family_first =
        ctx.name_order.is_none() && script_config.is_some_and(|config| config.use_native_ordering);
    if native_family_first {
        NameAssemblyOrder::NativeFamilyFirst
    } else {
        NameAssemblyOrder::GivenFirst
    }
}

fn sort_separator_for_name<'a>(
    script_config: Option<&'a citum_schema::options::ScriptConfig>,
    ctx: &'a NameFormatContext<'a>,
) -> &'a str {
    ctx.component_sort_separator
        .map_or_else(
            || {
                script_config
                    .and_then(|config| config.sort_separator.as_deref())
                    .or_else(|| ctx.sort_separator.map(std::string::String::as_str))
            },
            |separator| Some(separator.as_str()),
        )
        .unwrap_or(", ")
}

/// Append the source-script form after a romanized long-form name when a
/// name pattern requests an `original-script` segment (e.g. "Hua Linfu 华林甫").
fn append_original_script(assembled: String, name: &crate::reference::FlatName) -> String {
    match &name.original_script {
        Some(original) if !original.is_empty() => format!("{assembled} {original}"),
        _ => assembled,
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
        return format_literal_name(literal, name.short_name.as_deref(), ctx);
    }

    let family = name.family.as_deref().unwrap_or("");
    let given = name.given.as_deref().unwrap_or("");
    let dp = name.dropping_particle.as_deref().unwrap_or("");
    let ndp = name.non_dropping_particle.as_deref().unwrap_or("");
    let suffix = name.suffix.as_deref().unwrap_or("");
    let script_config = script_config_for_name(name, ctx);

    // Determine if we should invert (Family, Given).
    // `display-as-sort: first` in the config limits inversion to the first name
    // even when the template requests `name-order: family-first` for all names.
    let inverted = is_inverted_name_order(index, ctx);
    let assembly_order = name_assembly_order(inverted, script_config, ctx);

    // Determine effective form; integral name-memory overrides template form
    // so first mentions render full name and subsequent mentions render short.
    // Only applies when a memory config is active (subsequent_form is Some).
    let effective_form = if ctx.use_integral_short_name && ctx.subsequent_form.is_some() {
        match ctx.integral_name_state {
            Some(citum_schema::citation::IntegralNameState::First) => &ContributorForm::Long,
            Some(citum_schema::citation::IntegralNameState::Subsequent) => {
                match ctx.subsequent_form {
                    Some(citum_schema::options::SubsequentNameForm::FamilyOnly) => {
                        &ContributorForm::FamilyOnly
                    }
                    _ => &ContributorForm::Short,
                }
            }
            _ => {
                if expand_given_names && matches!(form, ContributorForm::Short) {
                    &ContributorForm::Long
                } else {
                    form
                }
            }
        }
    } else if expand_given_names && matches!(form, ContributorForm::Short) {
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

            let name_part_delimiter = script_config
                .and_then(|config| config.delimiter.as_deref())
                .unwrap_or(" ");
            let sep = sort_separator_for_name(script_config, ctx);
            let assembled = assemble_long_name(
                family_part,
                given_part,
                particle_part,
                suffix,
                assembly_order,
                name_part_delimiter,
                sep,
            );
            append_original_script(assembled, name)
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
