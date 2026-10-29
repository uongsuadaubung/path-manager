use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::language::text::*;
use crate::tui::app::App;

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw_add_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(65, 25, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", MODAL_ADD_TITLE))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightGreen).bold());

    let lines = vec![
        Line::raw(""),
        Line::styled(format!("  Target: {} PATH", app.scope.name()), Style::default().fg(Color::Cyan)),
        Line::styled(format!("  {}", MODAL_ADD_PROMPT), Style::default().fg(Color::White)),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(Color::LightGreen).bold()),
            Span::styled(&app.input_buffer, Style::default().fg(Color::White).bold()),
            Span::styled("█", Style::default().fg(Color::LightGreen)),
        ]),
        Line::raw(""),
        Line::styled("  [Enter] Confirm & Save   [Esc] Cancel", Style::default().fg(Color::DarkGray)),
    ];

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

pub fn draw_edit_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(65, 25, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", MODAL_EDIT_TITLE))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow).bold());

    let lines = vec![
        Line::raw(""),
        Line::styled(format!("  Target: {} PATH", app.scope.name()), Style::default().fg(Color::Cyan)),
        Line::styled(format!("  {}", MODAL_EDIT_PROMPT), Style::default().fg(Color::White)),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(&app.input_buffer, Style::default().fg(Color::White).bold()),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Line::raw(""),
        Line::styled("  [Enter] Save Changes   [Esc] Cancel", Style::default().fg(Color::DarkGray)),
    ];

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

pub fn draw_delete_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(55, 25, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", MODAL_DELETE_TITLE))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red).bold());

    let path_str = app.selected_item().map(|i| i.path.as_str()).unwrap_or("");

    let lines = vec![
        Line::raw(""),
        Line::styled(format!("  {}", MODAL_DELETE_PROMPT), Style::default().fg(Color::White).bold()),
        Line::raw(""),
        Line::styled(format!("  {}", path_str), Style::default().fg(Color::Red)),
        Line::raw(""),
        Line::styled(format!("  {}", MODAL_CONFIRM_HINT), Style::default().fg(Color::Yellow)),
    ];

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

pub fn draw_help_modal(f: &mut Frame, _app: &App) {
    let area = centered_rect(70, 75, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", MODAL_HELP_TITLE))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan).bold());

    let lines = vec![
        Line::styled(" ⌨️  KEYBOARD SHORTCUTS", Style::default().fg(Color::Cyan).bold()),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  ↑ / ↓, k / j         ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Navigate through the PATH list"),
        ]),
        Line::from(vec![
            Span::styled("  1 / 2, Tab           ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Switch scope between User PATH and System PATH"),
        ]),
        Line::from(vec![
            Span::styled("  a                    ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Add a new path entry"),
        ]),
        Line::from(vec![
            Span::styled("  e                    ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Edit the selected path entry"),
        ]),
        Line::from(vec![
            Span::styled("  d / Delete           ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Remove the selected path entry"),
        ]),
        Line::from(vec![
            Span::styled("  K / J                ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Move path up / down (reorder PATH precedence)"),
        ]),
        Line::from(vec![
            Span::styled("  /                    ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Search / Filter paths in real-time (Esc to clear)"),
        ]),
        Line::from(vec![
            Span::styled("  c                    ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Clean duplicates & redundant shadowed entries"),
        ]),
        Line::from(vec![
            Span::styled("  b                    ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Backup current PATH list to text file"),
        ]),
        Line::from(vec![
            Span::styled("  i                    ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Import & merge PATH list from backup text file"),
        ]),
        Line::from(vec![
            Span::styled("  r                    ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Refresh paths from Windows Registry"),
        ]),
        Line::from(vec![
            Span::styled("  q / Ctrl+C           ", Style::default().fg(Color::Red).bold()),
            Span::raw("Quit PATH Manager"),
        ]),
        Line::raw(""),
        Line::styled(" 🛡️  DIAGNOSTICS LEGEND", Style::default().fg(Color::Cyan).bold()),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  ✓ OK                 ", Style::default().fg(Color::LightGreen).bold()),
            Span::raw("Directory exists and contains executable binaries"),
        ]),
        Line::from(vec![
            Span::styled("  ✖ BROKEN             ", Style::default().fg(Color::Red).bold()),
            Span::raw("Directory does not exist on disk"),
        ]),
        Line::from(vec![
            Span::styled("  ▲ NO-EXEC            ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Directory exists but has no executable files (.exe, .bat, etc.)"),
        ]),
        Line::from(vec![
            Span::styled("  🛡️ SHADOWED           ", Style::default().fg(Color::Magenta).bold()),
            Span::raw("User path already present in System PATH (redundant)"),
        ]),
        Line::raw(""),
        Line::styled(" Press [Esc] or [?] or [Enter] to close this help window.", Style::default().fg(Color::DarkGray)),
    ];

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}
