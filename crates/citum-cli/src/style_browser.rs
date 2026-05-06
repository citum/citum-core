use crate::commands::StyleCatalogRow;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style as TuiStyle},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState,
        Wrap,
    },
};
use std::error::Error;
use std::io;
use std::time::Duration;

/// Input data for the interactive style browser.
pub(crate) struct StyleBrowserConfig {
    /// Catalog rows from the shared style catalog service.
    pub(crate) rows: Vec<StyleCatalogRow>,
    /// Initial filter query from the command line.
    pub(crate) initial_query: String,
    /// User-facing label for the active source filter.
    pub(crate) source_label: String,
}

/// Side effects available from the style browser.
pub(crate) trait StyleBrowserActions {
    /// Install the selected style.
    fn install_style(&mut self, row: &StyleCatalogRow) -> Result<(), Box<dyn Error>>;

    /// Remove the selected installed style.
    fn remove_style(&mut self, row: &StyleCatalogRow) -> Result<(), Box<dyn Error>>;
}

/// Run the interactive style browser in an alternate terminal screen.
pub(crate) fn run_style_browser<A: StyleBrowserActions>(
    config: StyleBrowserConfig,
    actions: &mut A,
) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = StyleBrowserApp::new(config);

    let run_result = run_browser_loop(&mut terminal, &mut app, actions);
    let restore_result = restore_terminal(&mut terminal);
    match (run_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(restore_error)) => {
            Err(format!("{error}; additionally failed to restore terminal: {restore_error}").into())
        }
    }
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_browser_loop<B: Backend, A: StyleBrowserActions>(
    terminal: &mut Terminal<B>,
    app: &mut StyleBrowserApp,
    actions: &mut A,
) -> Result<(), Box<dyn Error>>
where
    B::Error: 'static,
{
    loop {
        terminal.draw(|frame| app.render(frame))?;
        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && app.handle_key(key, actions)?
        {
            return Ok(());
        }
    }
}

#[derive(Clone, Debug)]
struct BrowserRow {
    catalog: StyleCatalogRow,
    installed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BrowserFocus {
    List,
    Search,
    Detail,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PendingAction {
    Remove(String),
}

#[derive(Clone, Debug)]
struct StatusMessage {
    text: String,
    kind: StatusKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StatusKind {
    Info,
    Success,
    Error,
}

struct StyleBrowserApp {
    source_label: String,
    query: String,
    rows: Vec<BrowserRow>,
    filtered: Vec<usize>,
    selected: usize,
    focus: BrowserFocus,
    status: StatusMessage,
    pending: Option<PendingAction>,
}

impl StyleBrowserApp {
    fn new(config: StyleBrowserConfig) -> Self {
        let rows = merge_catalog_rows(config.rows);
        let installed_count = rows.iter().filter(|row| row.installed).count();
        let mut app = Self {
            source_label: config.source_label,
            query: config.initial_query,
            rows,
            filtered: Vec::new(),
            selected: 0,
            focus: BrowserFocus::List,
            status: StatusMessage {
                text: format!(
                    "Use / to filter, arrows to move, i to install, r to remove, q to quit. {installed_count} installed."
                ),
                kind: StatusKind::Info,
            },
            pending: None,
        };
        app.refresh_filter();
        app
    }

    fn refresh_filter(&mut self) {
        let query = self.query.to_lowercase();
        self.filtered = self
            .rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                if query.is_empty() || browser_row_matches_query(row, &query) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        if self.filtered.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    fn current_row(&self) -> Option<&BrowserRow> {
        self.filtered
            .get(self.selected)
            .and_then(|row_idx| self.rows.get(*row_idx))
    }

    fn handle_key<A: StyleBrowserActions>(
        &mut self,
        key: KeyEvent,
        actions: &mut A,
    ) -> Result<bool, Box<dyn Error>> {
        if is_quit_key(key) {
            return Ok(true);
        }
        if self.handle_pending_key(key, actions)? {
            return Ok(false);
        }

        if self.focus == BrowserFocus::Search {
            self.handle_search_key(key);
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('/') => {
                self.focus = BrowserFocus::Search;
                self.set_info("Search mode. Type to filter, Enter to return to the list.");
            }
            KeyCode::Esc => {
                self.focus = BrowserFocus::List;
                self.set_info("List focus.");
            }
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::PageDown => self.move_selection(10),
            KeyCode::PageUp => self.move_selection(-10),
            KeyCode::Home => self.selected = 0,
            KeyCode::End => {
                self.selected = self.filtered.len().saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('d') => {
                self.focus = if self.focus == BrowserFocus::Detail {
                    BrowserFocus::List
                } else {
                    BrowserFocus::Detail
                };
            }
            KeyCode::Char('i') => self.install_selected(actions)?,
            KeyCode::Char('r') => self.request_remove_selected(),
            _ => {}
        }
        Ok(false)
    }

    fn handle_pending_key<A: StyleBrowserActions>(
        &mut self,
        key: KeyEvent,
        actions: &mut A,
    ) -> Result<bool, Box<dyn Error>> {
        if self.pending.is_none() {
            return Ok(false);
        }
        match key.code {
            KeyCode::Char('y' | 'Y') => {
                if matches!(self.pending, Some(PendingAction::Remove(_))) {
                    self.remove_selected(actions)?;
                }
                self.pending = None;
                Ok(true)
            }
            KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                self.pending = None;
                self.set_info("Remove canceled.");
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                if self.query.is_empty() {
                    self.focus = BrowserFocus::List;
                    self.set_info("List focus.");
                } else {
                    self.query.clear();
                    self.refresh_filter();
                    self.set_info("Search cleared.");
                }
            }
            KeyCode::Enter => {
                self.focus = BrowserFocus::List;
                self.set_info("List focus.");
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.refresh_filter();
            }
            KeyCode::Char(ch) => {
                self.query.push(ch);
                self.refresh_filter();
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: isize) {
        if self.filtered.is_empty() {
            self.selected = 0;
            return;
        }
        let last = self.filtered.len().saturating_sub(1);
        if delta < 0 {
            self.selected = self.selected.saturating_sub(delta.unsigned_abs());
        } else {
            self.selected = self.selected.saturating_add(delta.unsigned_abs()).min(last);
        }
    }

    fn install_selected<A: StyleBrowserActions>(
        &mut self,
        actions: &mut A,
    ) -> Result<(), Box<dyn Error>> {
        let Some(row) = self.current_row().cloned() else {
            self.set_error("No style selected.");
            return Ok(());
        };
        if row.installed {
            self.set_info(format!("{} is already installed.", row.catalog.id));
            return Ok(());
        }
        match actions.install_style(&row.catalog) {
            Ok(()) => {
                self.mark_installed(&row.catalog.id, true);
                self.set_success(format!("Installed {}.", row.catalog.id));
            }
            Err(error) => self.set_error(format!("Install failed: {error}")),
        }
        Ok(())
    }

    fn request_remove_selected(&mut self) {
        let Some(row) = self.current_row() else {
            self.set_error("No style selected.");
            return;
        };
        if !row.installed {
            self.set_info(format!("{} is not installed.", row.catalog.id));
            return;
        }
        let id = row.catalog.id.clone();
        self.pending = Some(PendingAction::Remove(id.clone()));
        self.set_info(format!("Remove installed style {id}? y/n"));
    }

    fn remove_selected<A: StyleBrowserActions>(
        &mut self,
        actions: &mut A,
    ) -> Result<(), Box<dyn Error>> {
        let Some(row) = self.current_row().cloned() else {
            self.set_error("No style selected.");
            return Ok(());
        };
        match actions.remove_style(&row.catalog) {
            Ok(()) => {
                self.mark_installed(&row.catalog.id, false);
                self.set_success(format!("Removed {}.", row.catalog.id));
            }
            Err(error) => self.set_error(format!("Remove failed: {error}")),
        }
        Ok(())
    }

    fn mark_installed(&mut self, id: &str, installed: bool) {
        for row in &mut self.rows {
            if row.catalog.id == id {
                row.installed = installed;
            }
        }
        self.rows
            .retain(|row| row.installed || row.catalog.source != "installed");
        self.refresh_filter();
    }

    fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(3),
            ])
            .split(area);
        let &[header, body, footer] = root.as_ref() else {
            return;
        };
        self.render_header(frame, header);
        self.render_body(frame, body);
        self.render_footer(frame, footer);
    }

    fn render_header(&self, frame: &mut Frame<'_>, area: Rect) {
        let installed_count = self.rows.iter().filter(|row| row.installed).count();
        let query = if self.query.is_empty() {
            "-".to_string()
        } else {
            self.query.clone()
        };
        let line = Line::from(vec![
            Span::styled(
                "Citum Style Browser",
                TuiStyle::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  source "),
            Span::styled(
                self.source_label.as_str(),
                TuiStyle::default().fg(Color::Yellow),
            ),
            Span::raw("  query "),
            Span::styled(query, TuiStyle::default().fg(Color::LightBlue)),
            Span::raw("  results "),
            Span::styled(
                self.filtered.len().to_string(),
                TuiStyle::default().fg(Color::Green),
            ),
            Span::raw("  installed "),
            Span::styled(
                installed_count.to_string(),
                TuiStyle::default().fg(Color::Green),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(line)
                .block(Block::default().borders(Borders::ALL))
                .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn render_body(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if area.width < 96 {
            match self.focus {
                BrowserFocus::Detail => self.render_detail(frame, area),
                _ => self.render_list(frame, area),
            }
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(area);
        let &[list, detail] = chunks.as_ref() else {
            return;
        };
        self.render_list(frame, list);
        self.render_detail(frame, detail);
    }

    fn render_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let title = if self.focus == BrowserFocus::Search {
            "Styles - searching"
        } else {
            "Styles"
        };
        if area.width < 96 {
            self.render_compact_list(frame, area, title);
        } else {
            self.render_table(frame, area, title);
        }
    }

    fn render_table(&mut self, frame: &mut Frame<'_>, area: Rect, title: &str) {
        let rows: Vec<Row<'_>> = self
            .filtered
            .iter()
            .filter_map(|row_idx| self.rows.get(*row_idx))
            .map(style_table_row)
            .collect();
        let header = Row::new([
            Cell::from("Status"),
            Cell::from("Source"),
            Cell::from("ID"),
            Cell::from("Title"),
        ])
        .style(
            TuiStyle::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
        let widths = [
            Constraint::Length(10),
            Constraint::Length(18),
            Constraint::Length(36),
            Constraint::Min(20),
        ];
        let mut state = TableState::default();
        if !rows.is_empty() {
            state.select(Some(self.selected));
        }
        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(focus_border_style(self.focus == BrowserFocus::List)),
            )
            .column_spacing(2)
            .highlight_symbol("> ")
            .row_highlight_style(
                TuiStyle::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(table, area, &mut state);
    }

    fn render_compact_list(&mut self, frame: &mut Frame<'_>, area: Rect, title: &str) {
        let items: Vec<ListItem<'_>> = self
            .filtered
            .iter()
            .filter_map(|row_idx| self.rows.get(*row_idx))
            .map(compact_list_item)
            .collect();
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(self.selected));
        }
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(focus_border_style(self.focus == BrowserFocus::List)),
            )
            .highlight_style(
                TuiStyle::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_detail(&self, frame: &mut Frame<'_>, area: Rect) {
        let lines = if let Some(row) = self.current_row() {
            detail_lines(row)
        } else {
            vec![Line::from("No styles match the current query.")]
        };
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Details")
                    .borders(Borders::ALL)
                    .border_style(focus_border_style(self.focus == BrowserFocus::Detail)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    fn render_footer(&self, frame: &mut Frame<'_>, area: Rect) {
        let keys = self.footer_keys(area.width);
        let status = Line::from(Span::styled(
            self.status.text.as_str(),
            status_style(self.status.kind),
        ));
        let paragraph = Paragraph::new(vec![Line::from(keys), status])
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        if self.pending.is_some() {
            render_confirmation(frame, area, self.pending_remove_id());
        }
    }

    fn footer_keys(&self, width: u16) -> &'static str {
        if self.pending.is_some() {
            return "Confirm remove: y Remove  n Cancel  q Quit";
        }
        if self.focus == BrowserFocus::Search {
            return "Search mode: Enter apply  Esc clear/back  q Quit";
        }
        let Some(row) = self.current_row() else {
            return "q Quit  / Search";
        };
        match (width < 96, row.installed) {
            (true, true) => "q Quit  / Search  arrows/jk Move  d Details  r Remove",
            (true, false) => "q Quit  / Search  arrows/jk Move  d Details  i Install",
            (false, true) => {
                "q Quit  / Search  esc Back  arrows/jk Move  pg/home/end Jump  d Details  r Remove"
            }
            (false, false) => {
                "q Quit  / Search  esc Back  arrows/jk Move  pg/home/end Jump  d Details  i Install"
            }
        }
    }

    fn pending_remove_id(&self) -> Option<&str> {
        match self.pending.as_ref() {
            Some(PendingAction::Remove(id)) => Some(id.as_str()),
            None => None,
        }
    }

    fn set_info(&mut self, text: impl Into<String>) {
        self.status = StatusMessage {
            text: text.into(),
            kind: StatusKind::Info,
        };
    }

    fn set_success(&mut self, text: impl Into<String>) {
        self.status = StatusMessage {
            text: text.into(),
            kind: StatusKind::Success,
        };
    }

    fn set_error(&mut self, text: impl Into<String>) {
        self.status = StatusMessage {
            text: text.into(),
            kind: StatusKind::Error,
        };
    }
}

fn is_quit_key(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q' | 'Q'))
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}

fn merge_catalog_rows(rows: Vec<StyleCatalogRow>) -> Vec<BrowserRow> {
    let mut merged: Vec<BrowserRow> = Vec::new();
    for row in rows {
        let is_installed = row.source == "installed";
        if let Some(existing) = merged
            .iter_mut()
            .find(|candidate| candidate.catalog.id == row.id)
        {
            existing.installed |= is_installed;
            if existing.catalog.source == "installed" && !is_installed {
                existing.catalog = row;
                existing.installed = true;
            }
        } else {
            merged.push(BrowserRow {
                catalog: row,
                installed: is_installed,
            });
        }
    }
    merged
}

fn browser_row_matches_query(row: &BrowserRow, query: &str) -> bool {
    row.catalog.id.to_lowercase().contains(query)
        || row
            .catalog
            .aliases
            .iter()
            .any(|alias| alias.to_lowercase().contains(query))
        || row
            .catalog
            .title
            .as_ref()
            .is_some_and(|title| title.to_lowercase().contains(query))
        || row
            .catalog
            .description
            .as_ref()
            .is_some_and(|description| description.to_lowercase().contains(query))
        || row
            .catalog
            .fields
            .iter()
            .any(|field| field.to_lowercase().contains(query))
        || row.catalog.source.to_lowercase().contains(query)
}

fn style_table_row(row: &BrowserRow) -> Row<'_> {
    Row::new([
        Cell::from(status_label(row.installed)).style(status_cell_style(row.installed)),
        Cell::from(row.catalog.source.clone()).style(source_style(&row.catalog.source)),
        Cell::from(row.catalog.id.clone()).style(
            TuiStyle::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(row.catalog.title.clone().unwrap_or_else(|| "-".to_string()))
            .style(TuiStyle::default().fg(Color::Gray)),
    ])
}

fn compact_list_item(row: &BrowserRow) -> ListItem<'_> {
    ListItem::new(vec![
        Line::from(vec![
            Span::styled(
                status_label(row.installed),
                status_cell_style(row.installed),
            ),
            Span::raw("  "),
            Span::styled(
                row.catalog.source.as_str(),
                source_style(&row.catalog.source),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                row.catalog.id.as_str(),
                TuiStyle::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                row.catalog.title.as_deref().unwrap_or("-"),
                TuiStyle::default().fg(Color::Gray),
            ),
        ]),
    ])
}

fn status_label(installed: bool) -> &'static str {
    if installed { "INSTALLED" } else { "available" }
}

fn status_cell_style(installed: bool) -> TuiStyle {
    if installed {
        TuiStyle::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        TuiStyle::default().fg(Color::DarkGray)
    }
}

fn source_style(source: &str) -> TuiStyle {
    let color = if source == "embedded" {
        Color::LightBlue
    } else if source == "installed" {
        Color::Green
    } else if source.starts_with("registry:") {
        Color::Yellow
    } else {
        Color::Magenta
    };
    TuiStyle::default().fg(color)
}

fn detail_lines(row: &BrowserRow) -> Vec<Line<'static>> {
    let aliases = if row.catalog.aliases.is_empty() {
        "-".to_string()
    } else {
        row.catalog.aliases.join(", ")
    };
    let fields = if row.catalog.fields.is_empty() {
        "-".to_string()
    } else {
        row.catalog.fields.join(", ")
    };
    let mut lines = vec![
        label_value_line("ID", row.catalog.id.as_str()),
        label_value_line("Title", row.catalog.title.as_deref().unwrap_or("-")),
        label_value_line("Source", row.catalog.source.as_str()),
        label_value_line("Status", status_label(row.installed)),
        label_value_line("Aliases", aliases.as_str()),
        label_value_line("Fields", fields.as_str()),
    ];
    if let Some(description) = &row.catalog.description {
        lines.push(Line::from(""));
        lines.push(label_value_line("Summary", description));
    }
    if let Some(url) = &row.catalog.url {
        lines.push(Line::from(""));
        lines.push(label_value_line("URL", url));
    }
    lines
}

fn label_value_line(label: &str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label:<8}"),
            TuiStyle::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(value.into()),
    ])
}

fn focus_border_style(active: bool) -> TuiStyle {
    if active {
        TuiStyle::default().fg(Color::Cyan)
    } else {
        TuiStyle::default().fg(Color::DarkGray)
    }
}

fn status_style(kind: StatusKind) -> TuiStyle {
    match kind {
        StatusKind::Info => TuiStyle::default().fg(Color::Gray),
        StatusKind::Success => TuiStyle::default().fg(Color::Green),
        StatusKind::Error => TuiStyle::default().fg(Color::Red),
    }
}

fn render_confirmation(frame: &mut Frame<'_>, area: Rect, style_id: Option<&str>) {
    let popup_area = centered_rect(64, 5, area);
    frame.render_widget(Clear, popup_area);
    let prompt = if let Some(style_id) = style_id {
        format!("Remove {style_id}?\ny Remove   n Cancel")
    } else {
        "Remove installed style?\ny Remove   n Cancel".to_string()
    };
    frame.render_widget(
        Paragraph::new(prompt)
            .block(Block::default().title("Confirm").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        popup_area,
    );
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let popup_width = width.min(area.width);
    let popup_height = height.min(area.height);
    Rect {
        x: area.x + area.width.saturating_sub(popup_width) / 2,
        y: area.y + area.height.saturating_sub(popup_height) / 2,
        width: popup_width,
        height: popup_height,
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Panicking is acceptable and desired in focused CLI TUI tests."
)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[derive(Default)]
    struct FakeActions {
        installed: Vec<String>,
        removed: Vec<String>,
    }

    impl StyleBrowserActions for FakeActions {
        fn install_style(&mut self, row: &StyleCatalogRow) -> Result<(), Box<dyn Error>> {
            self.installed.push(row.id.clone());
            Ok(())
        }

        fn remove_style(&mut self, row: &StyleCatalogRow) -> Result<(), Box<dyn Error>> {
            self.removed.push(row.id.clone());
            Ok(())
        }
    }

    fn catalog_row(id: &str, source: &str, title: &str) -> StyleCatalogRow {
        StyleCatalogRow {
            source: source.to_string(),
            id: id.to_string(),
            aliases: Vec::new(),
            title: Some(title.to_string()),
            description: None,
            fields: Vec::new(),
            url: None,
        }
    }

    #[test]
    fn merge_catalog_rows_marks_embedded_style_as_installed_once() {
        let rows = merge_catalog_rows(vec![
            catalog_row("apa-7th", "embedded", "APA"),
            catalog_row("apa-7th", "installed", ""),
        ]);

        assert_eq!(rows.len(), 1);
        let row = rows.first().expect("merged row should exist");
        assert_eq!(row.catalog.source, "embedded");
        assert!(row.installed);
    }

    #[test]
    fn merge_catalog_rows_keeps_installed_only_style() {
        let rows = merge_catalog_rows(vec![catalog_row("custom-style", "installed", "")]);

        assert_eq!(rows.len(), 1);
        let row = rows.first().expect("merged row should exist");
        assert_eq!(row.catalog.source, "installed");
        assert!(row.installed);
    }

    #[test]
    fn browser_filter_matches_source_and_title() {
        let row = BrowserRow {
            catalog: catalog_row("apa-7th", "embedded", "American Psychological Association"),
            installed: false,
        };

        assert!(browser_row_matches_query(&row, "psychological association"));
        assert!(browser_row_matches_query(&row, "embedded"));
        assert!(!browser_row_matches_query(&row, "vancouver numeric"));
    }

    #[test]
    fn quit_key_is_global() {
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        assert!(is_quit_key(key));
        assert!(is_quit_key(ctrl_c));
    }

    #[test]
    fn remove_confirmation_marks_registry_row_available() {
        let mut app = StyleBrowserApp::new(StyleBrowserConfig {
            rows: vec![
                catalog_row("apa-7th", "embedded", "American Psychological Association"),
                catalog_row("apa-7th", "installed", ""),
            ],
            initial_query: String::new(),
            source_label: "all".to_string(),
        });
        let mut actions = FakeActions::default();

        app.handle_key(
            KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE),
            &mut actions,
        )
        .expect("remove request should be handled");
        app.handle_key(
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            &mut actions,
        )
        .expect("remove confirmation should be handled");

        assert_eq!(actions.removed, vec!["apa-7th"]);
        let row = app
            .current_row()
            .expect("registry row should remain visible");
        assert_eq!(row.catalog.source, "embedded");
        assert!(!row.installed);
    }

    #[test]
    fn remove_confirmation_drops_installed_only_row() {
        let mut app = StyleBrowserApp::new(StyleBrowserConfig {
            rows: vec![catalog_row("custom-style", "installed", "")],
            initial_query: String::new(),
            source_label: "all".to_string(),
        });
        let mut actions = FakeActions::default();

        app.handle_key(
            KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE),
            &mut actions,
        )
        .expect("remove request should be handled");
        app.handle_key(
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            &mut actions,
        )
        .expect("remove confirmation should be handled");

        assert_eq!(actions.removed, vec!["custom-style"]);
        assert!(app.current_row().is_none());
    }

    #[test]
    fn wide_render_includes_source_and_installed_state() {
        let mut app = StyleBrowserApp::new(StyleBrowserConfig {
            rows: vec![
                catalog_row("apa-7th", "embedded", "American Psychological Association"),
                catalog_row("apa-7th", "installed", ""),
            ],
            initial_query: String::new(),
            source_label: "all".to_string(),
        });
        let backend = TestBackend::new(120, 24);
        let mut terminal = Terminal::new(backend).expect("test backend should initialize");

        terminal
            .draw(|frame| app.render(frame))
            .expect("test render should succeed");
        let rendered =
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .fold(String::new(), |mut output, cell| {
                    output.push_str(cell.symbol());
                    output
                });

        assert!(rendered.contains("INSTALLED"));
        assert!(rendered.contains("embedded"));
        assert!(rendered.contains("apa-7th"));
        assert!(rendered.contains("American Psychological Association"));
    }
}
