use std::borrow::Cow;

/// Extracts the last path segment from a `/`-separated identifier.
pub(crate) fn short_name_from_identifier(identifier: &str) -> Cow<'_, str> {
    identifier
        .rsplit('/')
        .next()
        .map_or_else(|| Cow::Borrowed(identifier), Cow::Borrowed)
}

/// Truncates `s` to `max_len` chars, appending `...` if clipped.
pub(crate) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
