use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::app::App;

pub fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let key_style = Style::default().fg(Color::Yellow).bold();
    let desc_style = Style::default().fg(Color::Gray);
    let sep = Span::raw("   ");

    let line1 = Line::from(vec![
        Span::styled("[↑/↓/k/j]", key_style),
        Span::styled(" Navigate", desc_style),
        sep.clone(),
        Span::styled("[1/2/Tab]", key_style),
        Span::styled(" Scope", desc_style),
        sep.clone(),
        Span::styled("[a]", key_style),
        Span::styled(" Add", desc_style),
        sep.clone(),
        Span::styled("[e]", key_style),
        Span::styled(" Edit", desc_style),
        sep.clone(),
        Span::styled("[d/Del]", key_style),
        Span::styled(" Delete", desc_style),
        sep.clone(),
        Span::styled("[K/J]", key_style),
        Span::styled(" Move", desc_style),
    ]);

    let line2 = Line::from(vec![
        Span::styled("[/]", key_style),
        Span::styled(" Search", desc_style),
        sep.clone(),
        Span::styled("[c]", key_style),
        Span::styled(" Dedupe", desc_style),
        sep.clone(),
        Span::styled("[b]", key_style),
        Span::styled(" Backup", desc_style),
        sep.clone(),
        Span::styled("[i]", key_style),
        Span::styled(" Import", desc_style),
        sep.clone(),
        Span::styled("[r]", key_style),
        Span::styled(" Refresh", desc_style),
        sep.clone(),
        Span::styled("[?]", key_style),
        Span::styled(" Help", desc_style),
        sep.clone(),
        Span::styled("[q]", Style::default().fg(Color::Red).bold()),
        Span::styled(" Quit", desc_style),
    ]);

    let para = Paragraph::new(vec![line1, line2]);
    f.render_widget(para, area);
}
