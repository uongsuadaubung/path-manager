use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

use crate::tui::app::App;

pub fn draw_path_table(f: &mut Frame, app: &App, area: Rect) {
    let items = app.active_items();
    let total_count = items.len();

    let title = format!(
        " Paths ({}) {} ",
        total_count,
        if !app.search_query.is_empty() { "[Filtered]" } else { "" }
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_height = area.height.saturating_sub(3) as usize; // subtract borders & header
    let page_size = inner_height.max(1);

    // Calculate window scroll offset so selected_index is always visible
    let start_idx = if app.selected_index >= page_size {
        app.selected_index - page_size + 1
    } else {
        0
    };
    let end_idx = (start_idx + page_size).min(total_count);

    let header_cells = [
        Cell::from(Span::styled(" # ", Style::default().fg(Color::Cyan).bold())),
        Cell::from(Span::styled(" Status ", Style::default().fg(Color::Cyan).bold())),
        Cell::from(Span::styled(" Path ", Style::default().fg(Color::Cyan).bold())),
    ];
    let header_row = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1);

    let rows: Vec<Row> = (start_idx..end_idx)
        .map(|idx| {
            let item = items[idx];
            let is_selected = idx == app.selected_index;

            // Status Badge
            let (status_text, status_style) = if item.is_shadowed {
                ("🛡️ SHADOW", Style::default().fg(Color::Magenta))
            } else if !item.diagnostic.exists {
                ("✖ BROKEN", Style::default().fg(Color::Red).bold())
            } else if !item.diagnostic.has_executables {
                ("▲ NO-EXEC", Style::default().fg(Color::Yellow))
            } else {
                ("✓ OK", Style::default().fg(Color::LightGreen))
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            let num_cell = Cell::from(format!("{}{:>2}", prefix, item.original_index));
            let status_cell = Cell::from(Span::styled(status_text, status_style));

            // Truncate path if very long to prevent table layout breaking
            let max_path_len = area.width.saturating_sub(25) as usize;
            let display_path = if item.path.len() > max_path_len && max_path_len > 10 {
                format!("...{}", &item.path[item.path.len() - (max_path_len - 3)..])
            } else {
                item.path.clone()
            };

            let path_cell = Cell::from(display_path);

            let row_style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(30, 50, 80))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            Row::new(vec![num_cell, status_cell, path_cell]).style(row_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),       // # with indicator
            Constraint::Length(12),      // Status
            Constraint::Min(20),        // Path
        ],
    )
    .header(header_row)
    .block(block);

    f.render_widget(table, area);
}
