/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Preprocessor that flattens Pandoc grid tables into sequential block markup.
//!
//! Pandoc's *grid table* syntax uses `+---+---+` border rows and `| … | … |`
//! content rows. Neither `jotdown` (Djot) nor `pulldown-cmark` (Markdown)
//! understand this format; if left as-is, the raw `+`, `|`, and mid-line `>`
//! characters appear verbatim in rendered output, producing invalid Typst/LaTeX
//! (no-text-within-stars warnings) and garbled HTML.
//!
//! This module detects grid tables in the raw document text *before* any
//! parser runs. Each detected table is replaced with the cell contents laid out
//! sequentially, separated by blank lines. Block structure (block quotes, lists,
//! emphasis, …) is preserved; only the two-dimensional layout is dropped.
//!
//! This is intentionally a *graceful flatten* — not full table rendering.
//! True table layout (Typst `#table(…)`, LaTeX `tabular`) is left as a
//! separate follow-up.

use std::borrow::Cow;

// ── Public entry point ────────────────────────────────────────────────────────

/// Flatten Pandoc grid tables in raw document text into sequential block markup.
///
/// Returns [`Cow::Borrowed`] unchanged when no grid table is detected (the
/// common case — zero allocation). When a table is found each cell's block
/// content is emitted in reading order (row-major, left to right), separated
/// by blank lines; the surrounding text is left as-is.
///
/// Frontmatter (`---`/`...` delimiters) is not affected because `---` does not
/// match the `+[-=+]+` border pattern.
pub(super) fn flatten_grid_tables(content: &str) -> Cow<'_, str> {
    if !could_contain_grid_table(content) {
        return Cow::Borrowed(content);
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::with_capacity(content.len());
    let mut i = 0;

    while i < lines.len() {
        let line = lines.get(i).copied().unwrap_or("");
        if is_border_line(line) {
            // Try to parse a grid table starting at line i.
            if let Some((replacement, consumed)) = try_extract_table(&lines, i) {
                result.push_str(&replacement);
                i += consumed;
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
        i += 1;
    }

    // Preserve a missing trailing newline in the original.
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    Cow::Owned(result)
}

// ── Detection helpers ─────────────────────────────────────────────────────────

/// Cheap pre-check: does the content look like it might have a grid table?
///
/// A grid table requires at least one border line starting with `+` followed
/// by `-`, `=`, or `+`. This avoids allocating on typical prose documents.
fn could_contain_grid_table(content: &str) -> bool {
    content.lines().any(|line| {
        let b = line.trim_end().as_bytes();
        b.first() == Some(&b'+') && b.len() >= 3 && matches!(b.get(1), Some(b'-' | b'=' | b'+'))
    })
}

/// Return true if `line` matches a grid-table border line: `^[+][-=+]+[+]\s*$`.
fn is_border_line(line: &str) -> bool {
    let line = line.trim_end();
    if line.len() < 3 {
        return false;
    }
    let bytes = line.as_bytes();
    if bytes.first() != Some(&b'+') || bytes.last() != Some(&b'+') {
        return false;
    }
    let inner_end = bytes.len().saturating_sub(1);
    bytes
        .get(1..inner_end)
        .is_some_and(|inner| inner.iter().all(|&b| b == b'-' || b == b'=' || b == b'+'))
}

/// Return true if `line` is a grid-table content row: `^[|].*[|]\s*$`.
fn is_content_line(line: &str) -> bool {
    let line = line.trim_end();
    if line.len() < 2 {
        return false;
    }
    let bytes = line.as_bytes();
    bytes.first() == Some(&b'|') && bytes.last() == Some(&b'|')
}

// ── Column boundary parsing ───────────────────────────────────────────────────

/// Parse the byte-column positions of each `+` in a border line.
///
/// For example `+-----+----+` yields `[0, 6, 11]`.
/// Returns an empty `Vec` if the line is not a valid border.
fn parse_column_boundaries(border: &str) -> Vec<usize> {
    border
        .trim_end()
        .char_indices()
        .filter_map(|(i, c)| if c == '+' { Some(i) } else { None })
        .collect()
}

// ── Grid table extraction ─────────────────────────────────────────────────────

/// Attempt to parse a grid table starting at `lines[start]`.
///
/// Returns `Some((replacement_text, lines_consumed))` on success, or `None`
/// if the lines don't form a valid grid table (e.g. malformed borders, too
/// few rows). A valid table requires at least two border rows and at least one
/// content row.
fn try_extract_table(lines: &[&str], start: usize) -> Option<(String, usize)> {
    let first_border = lines.get(start).copied()?;
    if !is_border_line(first_border) {
        return None;
    }
    let boundaries = parse_column_boundaries(first_border);
    if boundaries.len() < 2 {
        return None;
    }

    // Scan forward collecting rows until we reach a line that is neither a
    // border nor a content row (or end of input).
    let mut rows: Vec<Vec<String>> = Vec::new(); // rows × cells
    let mut current_row_lines: Vec<&str> = Vec::new(); // content lines for current row
    let mut border_count = 0usize;
    let mut i = start;

    while let Some(&line) = lines.get(i) {
        if is_border_line(line) {
            border_count += 1;
            if !current_row_lines.is_empty() {
                rows.push(extract_cells_from_content_lines(
                    &current_row_lines,
                    &boundaries,
                ));
                current_row_lines.clear();
            }
            i += 1;
        } else if is_content_line(line) {
            current_row_lines.push(line);
            i += 1;
        } else {
            // Non-table line — stop scanning.
            break;
        }
    }

    // Flush any trailing content row (shouldn't happen in well-formed tables,
    // but handle it gracefully).
    if !current_row_lines.is_empty() {
        rows.push(extract_cells_from_content_lines(
            &current_row_lines,
            &boundaries,
        ));
    }

    // Need at least 2 borders (top + bottom) and 1 content row.
    if border_count < 2 || rows.is_empty() {
        return None;
    }

    let consumed = i - start;
    let replacement = flatten_rows(rows);
    Some((replacement, consumed))
}

/// Extract cell strings from a sequence of `| … | … |` content lines.
///
/// Each cell accumulates the text from all content lines that make it up.
/// `boundaries` is the list of `+` byte-positions from the first border row.
fn extract_cells_from_content_lines(content_lines: &[&str], boundaries: &[usize]) -> Vec<String> {
    let n_cells = boundaries.len().saturating_sub(1);
    let mut cells: Vec<Vec<&str>> = vec![Vec::new(); n_cells];

    for &line in content_lines {
        let line = line.trim_end();
        for col in 0..n_cells {
            // Use `.get()` to avoid panicking on ragged boundary lists.
            let Some(&col_start) = boundaries.get(col) else {
                continue;
            };
            let Some(&col_end) = boundaries.get(col + 1) else {
                continue;
            };
            let start = col_start + 1; // skip the `|`/`+`
            let end = col_end;
            // Guard against lines shorter than expected (ragged edge).
            let cell_slice = if start < line.len() {
                let end = end.min(line.len());
                // `start` and `end` are derived from `char_indices` on ASCII
                // border lines, so they land on char boundaries.
                line.get(start..end).unwrap_or("")
            } else {
                ""
            };
            // Strip one leading space of cell padding; trim trailing column-width padding.
            let cell_text = cell_slice
                .strip_prefix(' ')
                .unwrap_or(cell_slice)
                .trim_end();
            if let Some(slot) = cells.get_mut(col) {
                slot.push(cell_text);
            }
        }
    }

    // Join each cell's lines preserving the original line breaks.
    cells.into_iter().map(|lines| lines.join("\n")).collect()
}

// ── Flattening ────────────────────────────────────────────────────────────────

/// Convert parsed rows-of-cells into flat sequential block markup.
///
/// Non-empty cells are emitted one after another, each separated by a blank
/// line. Cells that are entirely whitespace are silently skipped (e.g. header
/// rows that only contain formatting characters from the source document).
fn flatten_rows(rows: Vec<Vec<String>>) -> String {
    let mut blocks: Vec<String> = Vec::new();

    for row in rows {
        for cell in row {
            let trimmed = cell.trim();
            if !trimmed.is_empty() {
                blocks.push(trimmed.to_string());
            }
        }
    }

    if blocks.is_empty() {
        return String::new();
    }

    // Separate blocks with a blank line so each parses as a distinct block.
    let mut out = blocks.join("\n\n");
    out.push('\n');
    out
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_table_returns_borrowed() {
        let prose = "Just some prose.\n\nNo table here.\n";
        assert!(
            matches!(flatten_grid_tables(prose), Cow::Borrowed(_)),
            "prose without a grid table should return Cow::Borrowed (zero allocation)"
        );
    }

    #[test]
    fn border_line_detection() {
        assert!(is_border_line("+-----+----+"));
        assert!(is_border_line("+====+========+"));
        assert!(is_border_line("+--+--+--+"));
        assert!(!is_border_line("| foo | bar |"));
        assert!(!is_border_line("---"));
        assert!(!is_border_line("+"));
    }

    #[test]
    fn content_line_detection() {
        assert!(is_content_line("| foo | bar |"));
        assert!(is_content_line("| > block quote |"));
        assert!(!is_content_line("+-----+----+"));
        assert!(!is_content_line("plain text"));
    }

    #[test]
    fn column_boundary_parsing() {
        let bounds = parse_column_boundaries("+-----+----+");
        assert_eq!(bounds, vec![0, 6, 11]);
        let bounds = parse_column_boundaries("+--+--+--+");
        assert_eq!(bounds, vec![0, 3, 6, 9]);
    }

    #[test]
    fn simple_two_cell_table_flattened() {
        let table = concat!(
            "+----------+----------+\n",
            "| hello    | world    |\n",
            "+----------+----------+\n",
        );
        let result = flatten_grid_tables(table);
        assert!(matches!(result, Cow::Owned(_)));
        let s = result.as_ref();
        assert!(s.contains("hello"), "left cell should appear");
        assert!(s.contains("world"), "right cell should appear");
        assert!(
            !s.contains('+'),
            "grid border characters should be stripped"
        );
        assert!(!s.contains('|'), "pipe characters should be stripped");
    }

    #[test]
    fn block_quote_in_cell_is_preserved() {
        let table = concat!(
            "+------------------+\n",
            "| > this is quoted |\n",
            "+------------------+\n",
        );
        let result = flatten_grid_tables(table);
        let s = result.as_ref();
        assert!(
            s.contains("> this is quoted"),
            "block-quote marker must survive flattening, got: {s}"
        );
        assert!(!s.contains('|'), "pipe should be stripped");
        assert!(!s.contains('+'), "border should be stripped");
    }

    #[test]
    fn multi_line_cell_block_quote_reconstructed() {
        // Two-column table; left cell has a multi-line block quote.
        let table = concat!(
            "+-----------+-----------+\n",
            "| > line 1  | > line A  |\n",
            "| > line 2  | > line B  |\n",
            "+-----------+-----------+\n",
        );
        let result = flatten_grid_tables(table);
        let s = result.as_ref();
        assert!(s.contains("> line 1\n> line 2"), "left cell lines joined");
        assert!(s.contains("> line A\n> line B"), "right cell lines joined");
    }

    #[test]
    fn text_outside_table_is_unchanged() {
        let content = concat!(
            "Before the table.\n",
            "\n",
            "+------+------+\n",
            "| A    | B    |\n",
            "+------+------+\n",
            "\n",
            "After the table.\n",
        );
        let result = flatten_grid_tables(content);
        let s = result.as_ref();
        assert!(s.starts_with("Before the table."));
        assert!(s.ends_with("After the table.\n"));
        assert!(!s.contains('+'));
        assert!(!s.contains('|'));
    }

    #[test]
    fn inline_markup_inside_cell_is_preserved() {
        let table = concat!(
            "+---------------------+\n",
            "| *italic* **strong** |\n",
            "+---------------------+\n",
        );
        let result = flatten_grid_tables(table);
        let s = result.as_ref();
        assert!(s.contains("*italic*"), "inline emphasis must survive");
        assert!(s.contains("**strong**"), "inline strong must survive");
    }
}
