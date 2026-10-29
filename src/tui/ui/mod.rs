pub mod details_panel;
pub mod footer;
pub mod header;
pub mod modals;
pub mod nav_bar;
pub mod path_table;
pub mod status_bar;

pub use details_panel::*;
pub use footer::*;
pub use header::*;
pub use modals::*;
pub use nav_bar::*;
pub use path_table::*;
pub use status_bar::*;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::tui::app::{App, AppMode};

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs & Search
            Constraint::Min(8),    // Table & Details
            Constraint::Length(3), // Status bar
            Constraint::Length(2), // Footer
        ])
        .split(size);

    let nav_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    // Main split: Left 58% (Table), Right 42% (Details)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(58),
            Constraint::Percentage(42),
        ])
        .split(chunks[2]);

    // Record layout areas for mouse click hit testing
    let inner_height = main_chunks[0].height.saturating_sub(3) as usize;
    let page_size = inner_height.max(1);
    let total_count = app.active_items().len();
    let start_idx = if app.selected_index >= page_size {
        app.selected_index - page_size + 1
    } else {
        0
    };
    let visible_count = (start_idx + page_size).min(total_count).saturating_sub(start_idx);

    app.layout.screen = size;
    app.layout.tabs = nav_chunks[0];
    app.layout.search = nav_chunks[1];
    app.layout.table = main_chunks[0];
    app.layout.details = main_chunks[1];
    app.layout.footer = chunks[4];
    app.layout.table_start_idx = start_idx;
    app.layout.table_visible_count = visible_count;

    draw_header(f, app, chunks[0]);
    draw_nav_bar(f, app, chunks[1]);
    draw_path_table(f, app, main_chunks[0]);
    draw_details_panel(f, app, main_chunks[1]);
    draw_status_bar(f, app, chunks[3]);
    draw_footer(f, app, chunks[4]);

    // Draw active modal if any
    match app.mode {
        AppMode::AddModal => draw_add_modal(f, app),
        AppMode::EditModal => draw_edit_modal(f, app),
        AppMode::DeleteModal => draw_delete_modal(f, app),
        AppMode::HelpModal => draw_help_modal(f, app),
        _ => {}
    }
}
