pub mod app;
pub mod ui;

use std::io::stdout;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::compat::anyhow::Result;
use crate::language::text::*;
use crate::logic::PathScope;
pub use app::{App, AppMode};

pub fn run() -> Result<()> {
    // Set panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            crossterm::cursor::Show
        );
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    execute!(
        stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
        crossterm::cursor::Hide
    )?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_loop(&mut terminal, &mut app);

    // Restore terminal
    let _ = disable_raw_mode();
    let _ = execute!(
        stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::cursor::Show
    );

    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Check background Registry change signal
        if app.needs_refresh.swap(false, Ordering::SeqCst) {
            app.refresh_all();
            if app.mode == AppMode::Normal {
                app.status_msg = STATUS_REGISTRY_CHANGED.to_string();
            }
        }

        terminal.draw(|f| ui::draw_ui(f, app))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Paste(text) => handle_paste(app, &text),
                Event::Key(key) if key.kind == KeyEventKind::Press => handle_key_event(app, key),
                Event::Mouse(mouse) => handle_mouse_event(app, mouse),
                _ => {}
            }
        }
    }

    Ok(())
}

fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        // Scroll wheel: Navigate list up/down
        MouseEventKind::ScrollUp => {
            app.select_prev();
        }
        MouseEventKind::ScrollDown => {
            app.select_next();
        }

        // Left Click
        MouseEventKind::Down(MouseButton::Left) => {
            let x = mouse.column;
            let y = mouse.row;

            // 1. If in Help Modal: any click closes help
            if app.mode == AppMode::HelpModal {
                app.mode = AppMode::Normal;
                return;
            }

            // 2. If in Add/Edit Modal: click outside closes
            if app.mode == AppMode::AddModal || app.mode == AppMode::EditModal {
                let modal_x = (app.layout.screen.width * 65) / 100;
                let modal_y = (app.layout.screen.height * 25) / 100;
                let start_x = (app.layout.screen.width.saturating_sub(modal_x)) / 2;
                let start_y = (app.layout.screen.height.saturating_sub(modal_y)) / 2;
                let modal_rect = Rect::new(start_x, start_y, modal_x, modal_y);
                if !rect_contains(modal_rect, x, y) {
                    app.mode = AppMode::Normal;
                    app.input_buffer.clear();
                }
                return;
            }

            // 3. Normal / Searching mode
            // Check Click on Scope Tabs
            if rect_contains(app.layout.tabs, x, y) {
                let rel_x = x.saturating_sub(app.layout.tabs.x);
                let user_tab_end = 22; // rough width of "[1] User PATH (X)"
                if rel_x < user_tab_end {
                    app.set_scope(PathScope::User);
                } else if rel_x < user_tab_end + 25 {
                    app.set_scope(PathScope::System);
                }
                return;
            }

            // Check Click on Search Bar
            if rect_contains(app.layout.search, x, y) {
                app.mode = AppMode::Searching;
                return;
            }

            // Check Click on Path Table Rows
            if rect_contains(app.layout.table, x, y) {
                let header_offset = 2; // border + table header row
                let table_top = app.layout.table.y + header_offset;
                if y >= table_top && y < table_top + (app.layout.table_visible_count as u16) {
                    let clicked_row = (y - table_top) as usize;
                    let target_index = app.layout.table_start_idx + clicked_row;
                    if target_index < app.active_items().len() {
                        app.selected_index = target_index;
                    }
                }
                return;
            }
        }
        _ => {}
    }
}

fn handle_paste(app: &mut App, text: &str) {
    let clean = text.trim().trim_matches('"').trim_matches('\'');
    match app.mode {
        AppMode::AddModal | AppMode::EditModal => {
            app.input_buffer.push_str(clean);
        }
        AppMode::Searching => {
            app.search_query.push_str(clean);
            app.adjust_selection();
        }
        _ => {}
    }
}

fn handle_key_event(app: &mut App, key: KeyEvent) {
    match app.mode {
        AppMode::AddModal => handle_input_modal_key(app, key, true),
        AppMode::EditModal => handle_input_modal_key(app, key, false),
        AppMode::DeleteModal => handle_delete_modal_key(app, key.code),
        AppMode::HelpModal => handle_help_modal_key(app, key.code),
        AppMode::Searching => handle_search_key(app, key.code),
        AppMode::Normal => handle_normal_key(app, key),
    }
}

fn handle_input_modal_key(app: &mut App, key: KeyEvent, is_add: bool) {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
            app.input_buffer.clear();
        }
        KeyCode::Enter => {
            if is_add {
                app.apply_add();
            } else {
                app.apply_edit();
            }
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char('u') | KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input_buffer.clear();
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input_buffer.push(c);
        }
        _ => {}
    }
}

fn handle_delete_modal_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            app.apply_delete();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
}

fn handle_help_modal_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Enter => {
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
}

fn handle_search_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.search_query.clear();
            app.mode = AppMode::Normal;
            app.adjust_selection();
        }
        KeyCode::Enter | KeyCode::Down | KeyCode::Up => {
            app.mode = AppMode::Normal;
            app.adjust_selection();
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.adjust_selection();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.adjust_selection();
        }
        _ => {}
    }
}

fn handle_normal_key(app: &mut App, key: KeyEvent) {
    match key.code {
        // Quit
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,

        // Help
        KeyCode::Char('?') | KeyCode::Char('h') => app.mode = AppMode::HelpModal,

        // Reordering (match first before simple arrow keys)
        KeyCode::Char('K') => app.move_up(),
        KeyCode::Char('J') => app.move_down(),
        KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => app.move_up(),
        KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => app.move_down(),

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::PageUp => {
            for _ in 0..5 {
                app.select_prev();
            }
        }
        KeyCode::PageDown => {
            for _ in 0..5 {
                app.select_next();
            }
        }
        KeyCode::Home => app.select_first(),
        KeyCode::End => app.select_last(),

        // Switch Scope Tabs
        KeyCode::Char('1') => app.set_scope(PathScope::User),
        KeyCode::Char('2') => app.set_scope(PathScope::System),
        KeyCode::Tab => app.toggle_scope(),

        // Search / Filter
        KeyCode::Char('/') => {
            app.mode = AppMode::Searching;
        }
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                app.search_query.clear();
                app.adjust_selection();
            }
        }

        // Actions
        KeyCode::Char('a') => app.open_add_modal(),
        KeyCode::Char('e') => app.open_edit_modal(),
        KeyCode::Char('d') | KeyCode::Delete => app.open_delete_modal(),
        KeyCode::Char('c') => app.dedupe(),
        KeyCode::Char('b') => app.backup(),
        KeyCode::Char('i') => app.restore(),
        KeyCode::Char('r') => {
            app.refresh_all();
            app.status_msg = STATUS_REFRESHED.to_string();
        }

        _ => {}
    }
}
