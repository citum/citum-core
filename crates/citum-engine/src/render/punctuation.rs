/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Shared punctuation classification and collision resolution.

use citum_schema::options::{Config, StrongTerminalCommaPolicy};

/// Return the resolved strong-terminal/comma policy from a processed config.
pub(crate) fn strong_terminal_comma_policy(config: Option<&Config>) -> StrongTerminalCommaPolicy {
    config
        .and_then(|config| config.punctuation.as_ref())
        .and_then(|punctuation| punctuation.strong_terminal_comma_policy)
        .unwrap_or_default()
}

/// Return whether `ch` participates in punctuation-collision resolution.
pub(crate) fn is_terminal_punctuation(ch: char) -> bool {
    matches!(ch, ':' | '.' | ';' | '!' | '?' | ',' | '…')
}

/// Return whether `ch` is a strong terminal punctuation mark.
pub(crate) fn is_strong_terminal(ch: char) -> bool {
    matches!(ch, '!' | '?' | '…')
}

/// Resolve one punctuation pair while preserving the established compatibility matrix.
pub(crate) fn resolve_punctuation_collision(
    first: char,
    second: char,
    strong_terminal_comma_policy: StrongTerminalCommaPolicy,
) -> String {
    if second == ','
        && is_strong_terminal(first)
        && strong_terminal_comma_policy == StrongTerminalCommaPolicy::KeepTerminal
    {
        return first.to_string();
    }

    match (first, second) {
        (':', ':') => ":".to_string(),
        ('.', ':') => ".:".to_string(),
        (';', ':') => ";".to_string(),
        ('!', ':') => "!".to_string(),
        ('?', ':') => "?".to_string(),
        (',', ':') => ",:".to_string(),
        (':', '.') => ":".to_string(),
        ('.', '.') => ".".to_string(),
        (';', '.') => ";".to_string(),
        ('!', '.') => "!".to_string(),
        ('?', '.') => "?".to_string(),
        (',', '.') => ",.".to_string(),
        (':', ';') => ":;".to_string(),
        ('.', ';') => ".;".to_string(),
        (';', ';') => ";".to_string(),
        ('!', ';') => "!;".to_string(),
        ('?', ';') => "?;".to_string(),
        (',', ';') => ",;".to_string(),
        (':', '!') => "!".to_string(),
        ('.', '!') => ".!".to_string(),
        (';', '!') => "!".to_string(),
        ('!', '!') => "!".to_string(),
        ('?', '!') => "?!".to_string(),
        (',', '!') => ",!".to_string(),
        (':', '?') => "?".to_string(),
        ('.', '?') => ".?".to_string(),
        (';', '?') => "?".to_string(),
        ('!', '?') => "!?".to_string(),
        ('?', '?') => "?".to_string(),
        (',', '?') => ",?".to_string(),
        (':', ',') => ":,".to_string(),
        ('.', ',') => ".,".to_string(),
        (';', ',') => ";,".to_string(),
        ('!', ',') => "!,".to_string(),
        ('?', ',') => "?,".to_string(),
        (',', ',') => ",".to_string(),
        _ => format!("{first}{second}"),
    }
}
