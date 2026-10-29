use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::language::text::*;
use crate::logic::PathScope;
use crate::tui::app::App;

pub fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let (scope_name, scope_style) = match app.scope {
        PathScope::User => ("User PATH", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
        PathScope::System => ("System PATH", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    };

    let title_line = Line::from(vec![
        Span::styled(format!(" {} ", APP_TITLE), Style::default().bg(Color::Cyan).fg(Color::Black).bold()),
        Span::raw("  "),
        Span::styled("● ", scope_style),
        Span::styled(scope_name, scope_style),
        Span::raw("  "),
        Span::styled(format!("v{}", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::DarkGray)),
    ]);

    let left_para = Paragraph::new(title_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(left_para, chunks[0]);

    let counts_line = Line::from(vec![
        Span::styled("User: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} ", app.user_paths.len()), Style::default().fg(Color::LightGreen).bold()),
        Span::raw("| "),
        Span::styled("System: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} ", app.system_paths.len()), Style::default().fg(Color::Yellow).bold()),
        Span::raw("  "),
        Span::styled("🪟 Windows Registry", Style::default().fg(Color::White)),
    ]);

    let right_para = Paragraph::new(counts_line)
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(right_para, chunks[1]);
}
