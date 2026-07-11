/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Shared scanning primitives for per-backend `OutputFormat::visible_runs` lexers.
//!
//! Each backend's markup (LaTeX commands, Typst functions, Markdown/Djot
//! delimiters) has its own syntax, but they share two recurring shapes: a
//! delimiter pair that nests (`{`/`}`, `(`/`)`, `[`/`]`) and needs to be
//! skipped or matched, and a run of visible byte ranges that needs to be
//! accumulated as markup is stripped away. This module factors both out so
//! the individual lexers in `html.rs`, `latex.rs`, `typst.rs`, `markdown.rs`,
//! and `djot.rs` stay focused on their own syntax.

use std::ops::Range;

/// Accumulates visible byte ranges over a fragment, merging adjacent pushes
/// into a single run so callers don't need to track run boundaries themselves.
#[derive(Default)]
pub(crate) struct RunBuilder {
    runs: Vec<Range<usize>>,
}

impl RunBuilder {
    /// Mark the byte range `start..end` as visible, extending the previous
    /// run when it directly abuts `start`. No-op when `start >= end`.
    pub(crate) fn push_visible(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        if let Some(last) = self.runs.last_mut()
            && last.end == start
        {
            last.end = end;
            return;
        }
        self.runs.push(start..end);
    }

    /// Consume the builder, returning the accumulated visible runs in order.
    pub(crate) fn finish(self) -> Vec<Range<usize>> {
        self.runs
    }
}

/// Find the index (into `chars`) of the delimiter matching `open_ch` at
/// `chars[open_i]`, tracking nested `open_ch`/`close_ch` pairs. Returns
/// `None` if `chars[open_i]` isn't `open_ch` or the delimiter is unmatched.
///
/// When `respect_escapes` is set, a backslash escapes the following
/// character: neither is inspected for delimiter matching.
pub(crate) fn find_matching(
    chars: &[(usize, char)],
    open_i: usize,
    open_ch: char,
    close_ch: char,
    respect_escapes: bool,
) -> Option<usize> {
    if chars.get(open_i).map(|&(_, c)| c) != Some(open_ch) {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open_i;
    while let Some(&(_, ch)) = chars.get(i) {
        if respect_escapes && ch == '\\' {
            i += 2;
            continue;
        }
        if ch == open_ch {
            depth += 1;
        } else if ch == close_ch {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Skip a balanced `open_ch`/`close_ch` group starting at `chars[i]`,
/// returning the index just past the matching close delimiter. Returns `i`
/// unchanged when `chars[i]` isn't `open_ch`, or when the group is
/// unterminated the length of `chars` (i.e. skip to the end).
pub(crate) fn skip_balanced(
    chars: &[(usize, char)],
    i: usize,
    open_ch: char,
    close_ch: char,
    respect_escapes: bool,
) -> usize {
    if chars.get(i).map(|&(_, c)| c) != Some(open_ch) {
        return i;
    }
    find_matching(chars, i, open_ch, close_ch, respect_escapes).map_or(chars.len(), |end| end + 1)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::*;

    fn chars_of(s: &str) -> Vec<(usize, char)> {
        s.char_indices().collect()
    }

    #[test]
    fn run_builder_merges_adjacent_pushes() {
        let mut builder = RunBuilder::default();
        builder.push_visible(0, 3);
        builder.push_visible(3, 5);
        builder.push_visible(7, 9);
        assert_eq!(builder.finish(), vec![0..5, 7..9]);
    }

    #[test]
    fn run_builder_ignores_empty_pushes() {
        let mut builder = RunBuilder::default();
        builder.push_visible(2, 2);
        assert_eq!(builder.finish(), Vec::<Range<usize>>::new());
    }

    #[test]
    fn find_matching_tracks_nesting_depth() {
        let chars = chars_of("[a[b]c]d");
        assert_eq!(find_matching(&chars, 0, '[', ']', false), Some(6));
    }

    #[test]
    fn find_matching_respects_escapes() {
        let chars = chars_of(r"[a\]b]c");
        assert_eq!(find_matching(&chars, 0, '[', ']', true), Some(5));
    }

    #[test]
    fn find_matching_returns_none_when_unterminated() {
        let chars = chars_of("[abc");
        assert_eq!(find_matching(&chars, 0, '[', ']', false), None);
    }

    #[test]
    fn skip_balanced_returns_index_past_close() {
        let chars = chars_of("{abc}def");
        assert_eq!(skip_balanced(&chars, 0, '{', '}', false), 5);
    }

    #[test]
    fn skip_balanced_is_noop_when_not_at_open() {
        let chars = chars_of("abc");
        assert_eq!(skip_balanced(&chars, 0, '{', '}', false), 0);
    }
}
