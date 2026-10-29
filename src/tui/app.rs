use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use ratatui::layout::Rect;
use crate::language::text::*;
use crate::logic::{self, PathScope};

#[derive(Debug, Clone, Default)]
pub struct PathDiagnostic {
    pub exists: bool,
    pub has_executables: bool,
}

#[derive(Debug, Clone)]
pub struct PathItem {
    pub original_index: usize, // 1-based index in the registry
    pub path: String,
    pub expanded: String,
    pub diagnostic: PathDiagnostic,
    pub is_shadowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Searching,
    AddModal,
    EditModal,
    DeleteModal,
    HelpModal,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutAreas {
    pub screen: Rect,
    pub tabs: Rect,
    pub search: Rect,
    pub table: Rect,
    pub details: Rect,
    pub footer: Rect,
    pub table_start_idx: usize,
    pub table_visible_count: usize,
}

pub struct App {
    pub scope: PathScope,
    pub user_paths: Vec<PathItem>,
    pub system_paths: Vec<PathItem>,
    pub selected_index: usize,
    pub search_query: String,
    pub input_buffer: String,
    pub status_msg: String,
    pub mode: AppMode,
    pub should_quit: bool,
    pub needs_refresh: Arc<AtomicBool>,
    pub layout: LayoutAreas,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let needs_refresh = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&needs_refresh);

        // Spawn real-time Registry change watcher
        logic::spawn_watcher(move || {
            flag.store(true, Ordering::SeqCst);
        });

        let mut app = Self {
            scope: PathScope::User,
            user_paths: Vec::new(),
            system_paths: Vec::new(),
            selected_index: 0,
            search_query: String::new(),
            input_buffer: String::new(),
            status_msg: STATUS_READY.to_string(),
            mode: AppMode::Normal,
            should_quit: false,
            needs_refresh,
            layout: LayoutAreas::default(),
        };

        // Scan winget packages on launch
        let winget_paths = logic::scan_winget_packages();
        if !winget_paths.is_empty() {
            if let Ok(count) = logic::merge_paths(PathScope::User, winget_paths) {
                if count > 0 {
                    app.status_msg = format!("Auto-added {} WinGet package path(s) to User PATH.", count);
                }
            }
        }

        app.refresh_all();
        app
    }

    pub fn run_diagnostic(path: &str) -> (String, PathDiagnostic) {
        let expanded = logic::expand_env_vars(path);
        let path_obj = Path::new(&expanded);
        let exists = path_obj.exists();
        let mut has_exec = true;
        if exists && path_obj.is_dir() {
            has_exec = logic::has_executables(path_obj);
        }
        (expanded, PathDiagnostic { exists, has_executables: has_exec })
    }

    pub fn refresh_all(&mut self) {
        let u_raw = logic::read_current_paths(PathScope::User).unwrap_or_default();
        let s_raw = logic::read_current_paths(PathScope::System).unwrap_or_default();

        let s_items: Vec<PathItem> = s_raw
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                let (expanded, diag) = Self::run_diagnostic(&p);
                PathItem {
                    original_index: i + 1,
                    path: p,
                    expanded,
                    diagnostic: diag,
                    is_shadowed: false,
                }
            })
            .collect();

        let u_items: Vec<PathItem> = u_raw
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                let (expanded, diag) = Self::run_diagnostic(&p);
                let is_shadowed = s_items.iter().any(|sp| logic::is_same_path(&sp.path, &p));
                PathItem {
                    original_index: i + 1,
                    path: p,
                    expanded,
                    diagnostic: diag,
                    is_shadowed,
                }
            })
            .collect();

        self.user_paths = u_items;
        self.system_paths = s_items;
        self.adjust_selection();
    }

    pub fn active_items(&self) -> Vec<&PathItem> {
        let list = match self.scope {
            PathScope::User => &self.user_paths,
            PathScope::System => &self.system_paths,
        };

        if self.search_query.is_empty() {
            list.iter().collect()
        } else {
            let q = self.search_query.to_lowercase();
            list.iter()
                .filter(|item| {
                    item.path.to_lowercase().contains(&q)
                        || item.expanded.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn selected_item(&self) -> Option<&PathItem> {
        let items = self.active_items();
        items.get(self.selected_index).copied()
    }

    pub fn adjust_selection(&mut self) {
        let count = self.active_items().len();
        if count == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= count {
            self.selected_index = count - 1;
        }
    }

    pub fn select_next(&mut self) {
        let count = self.active_items().len();
        if count > 0 {
            if self.selected_index + 1 < count {
                self.selected_index += 1;
            } else {
                self.selected_index = 0; // wrap around
            }
        }
    }

    pub fn select_prev(&mut self) {
        let count = self.active_items().len();
        if count > 0 {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = count - 1; // wrap around
            }
        }
    }

    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn select_last(&mut self) {
        let count = self.active_items().len();
        if count > 0 {
            self.selected_index = count - 1;
        }
    }

    pub fn set_scope(&mut self, scope: PathScope) {
        if self.scope != scope {
            self.scope = scope;
            self.selected_index = 0;
            self.adjust_selection();
        }
    }

    pub fn toggle_scope(&mut self) {
        let new_scope = match self.scope {
            PathScope::User => PathScope::System,
            PathScope::System => PathScope::User,
        };
        self.set_scope(new_scope);
    }

    pub fn open_add_modal(&mut self) {
        self.input_buffer.clear();
        self.mode = AppMode::AddModal;
    }

    pub fn open_edit_modal(&mut self) {
        if let Some(item) = self.selected_item() {
            self.input_buffer = item.path.clone();
            self.mode = AppMode::EditModal;
        }
    }

    pub fn open_delete_modal(&mut self) {
        if self.selected_item().is_some() {
            self.mode = AppMode::DeleteModal;
        }
    }

    pub fn apply_add(&mut self) {
        let input = self.input_buffer.trim().to_string();
        if !input.is_empty() {
            match logic::add_path(self.scope, input.clone()) {
                Ok(true) => {
                    self.refresh_all();
                    self.status_msg = format!("{}{}", STATUS_ADDED, input);
                    self.select_last();
                }
                Ok(false) => {
                    self.status_msg = CLI_PATH_EXISTS.to_string();
                }
                Err(e) => {
                    self.status_msg = format!("{}{}", STATUS_ERROR_PREFIX, e);
                }
            }
        }
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
    }

    pub fn apply_edit(&mut self) {
        let input = self.input_buffer.trim().to_string();
        if let Some(item) = self.selected_item() {
            let orig_idx = item.original_index;
            if !input.is_empty() {
                match logic::set_path(self.scope, orig_idx, input) {
                    Ok(_) => {
                        self.refresh_all();
                        self.status_msg = STATUS_UPDATED.to_string();
                    }
                    Err(e) => {
                        self.status_msg = format!("{}{}", STATUS_ERROR_PREFIX, e);
                    }
                }
            }
        }
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
    }

    pub fn apply_delete(&mut self) {
        if let Some(item) = self.selected_item() {
            let orig_idx = item.original_index;
            match logic::remove_path(self.scope, orig_idx) {
                Ok(_) => {
                    self.refresh_all();
                    self.status_msg = STATUS_DELETED.to_string();
                }
                Err(e) => {
                    self.status_msg = format!("{}{}", STATUS_ERROR_PREFIX, e);
                }
            }
        }
        self.mode = AppMode::Normal;
    }

    pub fn move_up(&mut self) {
        if !self.search_query.is_empty() {
            self.status_msg = "Cannot reorder while search filter is active.".to_string();
            return;
        }
        if let Some(item) = self.selected_item() {
            let idx = item.original_index - 1;
            if idx > 0 {
                if let Ok(()) = logic::move_path(self.scope, idx, idx - 1) {
                    self.refresh_all();
                    self.selected_index = idx - 1;
                    self.status_msg = STATUS_REORDERED.to_string();
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        if !self.search_query.is_empty() {
            self.status_msg = "Cannot reorder while search filter is active.".to_string();
            return;
        }
        let total = match self.scope {
            PathScope::User => self.user_paths.len(),
            PathScope::System => self.system_paths.len(),
        };
        if let Some(item) = self.selected_item() {
            let idx = item.original_index - 1;
            if idx + 1 < total {
                if let Ok(()) = logic::move_path(self.scope, idx, idx + 1) {
                    self.refresh_all();
                    self.selected_index = idx + 1;
                    self.status_msg = STATUS_REORDERED.to_string();
                }
            }
        }
    }

    pub fn dedupe(&mut self) {
        match logic::dedupe_paths(self.scope) {
            Ok(count) => {
                self.refresh_all();
                if count > 0 {
                    self.status_msg = format!("Cleaned {} duplicate/redundant path(s).", count);
                } else {
                    self.status_msg = STATUS_NO_DUPLICATES.to_string();
                }
            }
            Err(e) => {
                self.status_msg = format!("{}{}", STATUS_ERROR_PREFIX, e);
            }
        }
    }

    pub fn backup(&mut self) {
        let file = match self.scope {
            PathScope::User => "user_path_backup.txt",
            PathScope::System => "system_path_backup.txt",
        };
        if let Ok(paths) = logic::read_current_paths(self.scope) {
            let content = paths.join("\n");
            if let Ok(()) = std::fs::write(file, content) {
                self.status_msg = format!("Backup saved to {}", file);
            } else {
                self.status_msg = format!("Failed to write backup to {}", file);
            }
        }
    }

    pub fn restore(&mut self) {
        let file = match self.scope {
            PathScope::User => "user_path_backup.txt",
            PathScope::System => "system_path_backup.txt",
        };
        if let Ok(content) = std::fs::read_to_string(file) {
            let imported: Vec<String> = content.lines().filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_string()).collect();
            match logic::merge_paths(self.scope, imported) {
                Ok(count) => {
                    self.refresh_all();
                    self.status_msg = format!("Restored/merged {} path(s) from {}", count, file);
                }
                Err(e) => {
                    self.status_msg = format!("{}{}", STATUS_ERROR_PREFIX, e);
                }
            }
        } else {
            self.status_msg = format!("Backup file '{}' not found in current directory.", file);
        }
    }
}
