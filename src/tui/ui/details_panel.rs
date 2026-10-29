use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::language::text::*;
use crate::tui::app::App;

pub fn draw_details_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", DETAILS_TITLE))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let selected_opt = app.selected_item();

    let content: Vec<Line> = if let Some(item) = selected_opt {
        let (hkey, subkey) = app.scope.reg_info();
        let reg_str = if hkey == crate::compat::winreg::enums::HKEY_CURRENT_USER {
            format!("HKCU\\{}", subkey)
        } else {
            format!("HKLM\\{}", subkey)
        };

        let mut lines = Vec::new();

        // Header line: Index & Scope
        lines.push(Line::from(vec![
            Span::styled("Scope: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} PATH", app.scope.name()), Style::default().fg(Color::Cyan).bold()),
            Span::styled("  |  Index: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("#{}", item.original_index), Style::default().fg(Color::White).bold()),
        ]));
        lines.push(Line::raw(""));

        // Path
        lines.push(Line::styled("Target Path:", Style::default().fg(Color::DarkGray)));
        lines.push(Line::styled(format!("  {}", item.path), Style::default().fg(Color::White).bold()));
        lines.push(Line::raw(""));

        // Expanded Path (if differs)
        if item.path != item.expanded {
            lines.push(Line::styled("Expanded Path (Environment):", Style::default().fg(Color::DarkGray)));
            lines.push(Line::styled(format!("  {}", item.expanded), Style::default().fg(Color::Yellow)));
            lines.push(Line::raw(""));
        }

        // Diagnostics Section
        lines.push(Line::styled("Diagnostics:", Style::default().fg(Color::DarkGray)));

        // Exists Check
        if item.diagnostic.exists {
            lines.push(Line::from(vec![
                Span::styled("  ✓ ", Style::default().fg(Color::LightGreen).bold()),
                Span::styled("Folder exists on disk", Style::default().fg(Color::LightGreen)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("  ✖ ", Style::default().fg(Color::Red).bold()),
                Span::styled("Folder does not exist (Broken PATH entry)", Style::default().fg(Color::Red).bold()),
            ]));
        }

        // Executables Check
        if item.diagnostic.has_executables {
            lines.push(Line::from(vec![
                Span::styled("  ✓ ", Style::default().fg(Color::LightGreen).bold()),
                Span::styled("Contains executable files (.exe, .bat, .cmd, etc.)", Style::default().fg(Color::LightGreen)),
            ]));
        } else if item.diagnostic.exists {
            lines.push(Line::from(vec![
                Span::styled("  ▲ ", Style::default().fg(Color::Yellow).bold()),
                Span::styled("No executables found in this directory", Style::default().fg(Color::Yellow)),
            ]));
        }

        // Shadowed Check
        if item.is_shadowed {
            lines.push(Line::from(vec![
                Span::styled("  🛡️ ", Style::default().fg(Color::Magenta)),
                Span::styled("Already present in System PATH (Redundant entry)", Style::default().fg(Color::Magenta)),
            ]));
        }

        lines.push(Line::raw(""));

        // Registry Key Info
        lines.push(Line::styled("Registry Location:", Style::default().fg(Color::DarkGray)));
        lines.push(Line::styled(format!("  {}", reg_str), Style::default().fg(Color::DarkGray)));

        lines
    } else {
        vec![
            Line::raw(""),
            Line::styled(format!("  {}", DETAILS_NO_SELECTION), Style::default().fg(Color::DarkGray)),
        ]
    };

    let para = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}
