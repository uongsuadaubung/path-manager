use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::logic::PathScope;
use crate::tui::app::{App, AppMode};

pub fn draw_nav_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Left: Scope Tabs
    let user_selected = app.scope == PathScope::User;
    let system_selected = app.scope == PathScope::System;

    let user_tab_style = if user_selected {
        Style::default().bg(Color::LightGreen).fg(Color::Black).bold()
    } else {
        Style::default().fg(Color::Gray)
    };

    let system_tab_style = if system_selected {
        Style::default().bg(Color::Yellow).fg(Color::Black).bold()
    } else {
        Style::default().fg(Color::Gray)
    };

    let tabs_line = Line::from(vec![
        Span::styled(format!(" [1] User PATH ({}) ", app.user_paths.len()), user_tab_style),
        Span::raw(" "),
        Span::styled(format!(" [2] System PATH ({}) ", app.system_paths.len()), system_tab_style),
        Span::raw("  "),
        Span::styled("[Tab] Switch", Style::default().fg(Color::DarkGray)),
    ]);

    let tabs_para = Paragraph::new(tabs_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(tabs_para, chunks[0]);

    // Right: Search Bar
    let is_searching = app.mode == AppMode::Searching;
    let search_border_color = if is_searching {
        Color::Yellow
    } else if !app.search_query.is_empty() {
        Color::Green
    } else {
        Color::DarkGray
    };

    let search_content = if is_searching {
        Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(Color::Yellow)),
            Span::styled(&app.search_query, Style::default().fg(Color::White).bold()),
            Span::styled("█", Style::default().fg(Color::Yellow)),
            Span::styled(" (Enter/Esc to finish)", Style::default().fg(Color::DarkGray)),
        ])
    } else if !app.search_query.is_empty() {
        Line::from(vec![
            Span::styled("🔍 Filter: ", Style::default().fg(Color::Green)),
            Span::styled(&app.search_query, Style::default().fg(Color::White).bold()),
            Span::styled(" (Press Esc to clear)", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled("🔍 Press '/' to filter paths...", Style::default().fg(Color::DarkGray)),
        ])
    };

    let search_para = Paragraph::new(search_content)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(search_border_color)),
        );
    f.render_widget(search_para, chunks[1]);
}
