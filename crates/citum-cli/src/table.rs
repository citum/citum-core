/// Unified table builder for CLI list output.
///
/// Provides a consistent table rendering interface using `UTF8_BORDERS_ONLY`
/// preset with dynamic terminal width and cyan headers.
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};

/// Build a formatted table with cyan headers and UTF8_BORDERS_ONLY styling.
///
/// Returns a string containing the rendered table. Headers are displayed in cyan.
pub fn build_table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(
            headers
                .iter()
                .map(|h| Cell::new(h).fg(Color::Cyan))
                .collect::<Vec<_>>(),
        );

    for row in rows {
        table.add_row(row.iter().map(Cell::new).collect::<Vec<_>>());
    }

    format!("{table}")
}
