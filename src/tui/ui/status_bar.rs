use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::tui::app::App;

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);

    let status_color = if app.status_msg.starts_with("Error") || app.status_msg.starts_with("Failed") {
        Color::Red
    } else if app.status_msg.starts_with("Warning") {
        Color::Yellow
    } else {
        Color::LightCyan
    };

    let left_line = Line::from(vec![
        Span::styled("ℹ ", Style::default().fg(Color::Cyan)),
        Span::styled(&app.status_msg, Style::default().fg(status_color)),
    ]);

    let left_para = Paragraph::new(left_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(left_para, chunks[0]);

    let total = app.active_items().len();
    let current = if total == 0 { 0 } else { app.selected_index + 1 };

    let pos_line = Line::from(vec![
        Span::styled("Position: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}/{} ", current, total), Style::default().fg(Color::White).bold()),
    ]);

    let right_para = Paragraph::new(pos_line)
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(right_para, chunks[1]);
}
