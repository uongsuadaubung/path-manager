use eframe::egui;
use crate::logic::*;
use crate::i18n::{t, t_args};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

enum AppEvent {
    AddPath(String),
    Restore(Vec<String>),
}

#[derive(Clone, Default)]
struct PathDiagnostic {
    exists: bool,
    has_executables: bool,
}

#[derive(Clone)]
struct PathInfo {
    path: String,
    diagnostic: PathDiagnostic,
}

pub struct PathApp {
    user_paths: Vec<PathInfo>,
    system_paths: Vec<PathInfo>,
    new_path_input: String,
    status_msg: String,
    search_query: String,
    editing_index: Option<usize>, 
    edit_input: String,
    needs_sync: bool,
    needs_refresh: Arc<AtomicBool>,
    async_event_rx: std::sync::mpsc::Receiver<AppEvent>,
    async_event_tx: std::sync::mpsc::Sender<AppEvent>,
}

impl PathApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let needs_refresh = Arc::new(AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::channel();
        
        let needs_refresh_clone = Arc::clone(&needs_refresh);
        let ctx_clone = cc.egui_ctx.clone();
        spawn_registry_watcher(move || {
            needs_refresh_clone.store(true, Ordering::SeqCst);
            ctx_clone.request_repaint();
        });

        let mut status_msg = t("ready");
        let winget_paths = scan_winget_packages();
        if !winget_paths.is_empty() {
            if let Ok(count) = merge_paths(PathScope::User, winget_paths) {
                if count > 0 {
                    status_msg = t_args("auto_added_winget", &count.to_string());
                }
            }
        }

        let mut app = Self {
            user_paths: Vec::new(),
            system_paths: Vec::new(),
            new_path_input: String::new(),
            status_msg,
            search_query: String::new(),
            editing_index: None,
            edit_input: String::new(),
            needs_sync: false,
            needs_refresh,
            async_event_rx: rx,
            async_event_tx: tx,
        };
        app.refresh_all();
        app
    }


    fn run_diagnostic(path: &str) -> PathDiagnostic {
        let expanded = expand_env_vars(path);
        let path_obj = std::path::Path::new(&expanded);
        let exists = path_obj.exists();
        
        let mut has_executables_flag = true;
        if exists && path_obj.is_dir() {
            has_executables_flag = has_executables(path_obj);
        }
        PathDiagnostic { exists, has_executables: has_executables_flag }
    }

    fn refresh_all(&mut self) {
        let u_paths = read_current_paths(PathScope::User).unwrap_or_default();
        let s_paths = read_current_paths(PathScope::System).unwrap_or_default();

        self.user_paths = u_paths.into_iter().map(|p| PathInfo {
            diagnostic: Self::run_diagnostic(&p),
            path: p,
        }).collect();

        self.system_paths = s_paths.into_iter().map(|p| PathInfo {
            diagnostic: Self::run_diagnostic(&p),
            path: p,
        }).collect();
        
        self.needs_sync = check_sync_needed(PathScope::User);
    }
}



fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Nhúng thẳng font Inter (hỗ trợ Tiếng Việt cực tốt) vào file exe/binary để ứng dụng chạy mượt trên mọi OS
    let font_data = include_bytes!("../assets/Inter-Regular.ttf");

    fonts.font_data.insert(
        "inter_font".to_owned(),
        egui::FontData::from_static(font_data).into(),
    );
    
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "inter_font".to_owned());
    }
    
    ctx.set_fonts(fonts);
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len { return path.to_string(); }
    let sep = std::path::MAIN_SEPARATOR;
    let parts: Vec<&str> = path.split(sep).collect();
    if parts.len() < 2 { return format!("{}...", &path[..max_len - 3]); }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    let mid_len = max_len.saturating_sub(first.len() + last.len() + 5);
    if mid_len > 0 {
        format!("{}{}{}...{}{}", first, sep, ".".repeat(mid_len.min(3)), sep, last)
    } else {
        format!("{}{}{}", first, sep, last)
    }
}

impl eframe::App for PathApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.needs_refresh.swap(false, Ordering::SeqCst) {
            self.refresh_all();
            if self.editing_index.is_none() {
                self.status_msg = t("updated");
            }
        }

        while let Ok(event) = self.async_event_rx.try_recv() {
            match event {
                AppEvent::AddPath(path) => {
                    let _ = add_path(PathScope::User, path);
                    self.refresh_all();
                    self.status_msg = t("added_new");
                }
                AppEvent::Restore(paths) => {
                    if let Ok(count) = merge_paths(PathScope::User, paths) {
                        self.refresh_all();
                        self.status_msg = t_args("restored_paths", &count.to_string());
                    }
                }
            }
        }

        egui::Panel::bottom("status_bar").show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.status_msg).small().color(egui::Color32::LIGHT_BLUE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}: {} | {}: {}", t("cli_user"), self.user_paths.len(), t("cli_system"), self.system_paths.len()));
                });
            });
            ui.add_space(3.0);
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
            
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new(t("title_path_manager")).strong().size(28.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut current_lang = crate::i18n::get_lang();
                        egui::ComboBox::from_id_salt("lang_selector")
                            .selected_text(if current_lang == "vi" { "🇻🇳 Tiếng Việt" } else { "🇬🇧 English" })
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut current_lang, "vi".to_string(), "🇻🇳 Tiếng Việt").changed() {
                                    crate::i18n::set_lang("vi");
                                    self.status_msg = t("ready");
                                }
                                if ui.selectable_value(&mut current_lang, "en".to_string(), "🇬🇧 English").changed() {
                                    crate::i18n::set_lang("en");
                                    self.status_msg = t("ready");
                                }
                            });
                    });
                });
                
                ui.horizontal(|ui| {
                    ui.label(t("search_label"));
                    ui.add(egui::TextEdit::singleline(&mut self.search_query).desired_width(250.0));
                    ui.add_space(10.0);
                    if ui.button(t("btn_dedupe")).clicked() {
                        let _ = dedupe_paths(PathScope::User);
                        self.refresh_all();
                        self.status_msg = t("msg_deduped");
                    }
                    if ui.button(t("btn_refresh")).clicked() {
                        self.refresh_all();
                        self.status_msg = t("msg_refreshed");
                    }
                    #[cfg(unix)]
                    if self.needs_sync {
                        if ui.button(egui::RichText::new(t("btn_sync_farm")).color(egui::Color32::GOLD)).clicked() {
                            let current_paths: Vec<String> = self.user_paths.iter().map(|p| p.path.clone()).collect();
                            let _ = write_paths(PathScope::User, current_paths);
                            self.refresh_all();
                            self.status_msg = t("msg_farm_synced");
                        }
                    }
                    if ui.button(t("btn_backup")).clicked() {
                        let paths: Vec<String> = self.user_paths.iter().map(|p| p.path.clone()).collect();
                        std::thread::spawn(move || {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("path_backup.txt")
                                .save_file() {
                                let content = paths.join("\n");
                                let _ = std::fs::write(path, content);
                            }
                        });
                        self.status_msg = t("msg_backing_up");
                    }
                    if ui.button(t("btn_restore")).clicked() {
                        let tx = self.async_event_tx.clone();
                        let ctx = ctx.clone();
                        std::thread::spawn(move || {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                if let Ok(content) = std::fs::read_to_string(path) {
                                    let imported: Vec<String> = content.lines()
                                        .filter(|s| !s.is_empty())
                                        .map(|s| s.to_string())
                                        .collect();
                                    let _ = tx.send(AppEvent::Restore(imported));
                                    ctx.request_repaint();
                                }
                            }
                        });
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(t("add_new_label"));
                    ui.add(egui::TextEdit::singleline(&mut self.new_path_input).hint_text(t("add_path_hint")).desired_width(350.0));
                    if ui.button(t("btn_add")).clicked() && !self.new_path_input.is_empty() {
                        match add_path(PathScope::User, self.new_path_input.clone()) {
                            Ok(_) => {
                                self.refresh_all();
                                self.new_path_input.clear();
                                self.status_msg = t("added_new");
                            }
                            Err(e) => self.status_msg = t_args("msg_error", &e.to_string()),
                        }
                    }
                    if ui.button(t("btn_pick_folder")).clicked() {
                        let tx = self.async_event_tx.clone();
                        let ctx = ctx.clone();
                        std::thread::spawn(move || {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                let _ = tx.send(AppEvent::AddPath(path.display().to_string()));
                                ctx.request_repaint();
                            }
                        });
                    }
                });
            });

            ui.separator();

            let mut to_delete = None;
            let mut to_save = None;
            let mut move_data = None;

            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                let query = self.search_query.to_lowercase();
                
                // USER PATH
                ui.group(|ui| {
                    ui.heading(egui::RichText::new(t("user_path_heading")).color(egui::Color32::LIGHT_GREEN).size(18.0));
                    ui.add_space(5.0);

                    for i in 0..self.user_paths.len() {
                        let path_info = self.user_paths[i].clone();
                        let path = &path_info.path;
                        if !query.is_empty() && !path.to_lowercase().contains(&query) { continue; }

                        let is_system_duplicate = self.system_paths.iter().any(|sp| {
                            is_same_path(&sp.path, path)
                        });
                        let item_id = egui::Id::new("user_item").with(i);
                        
                        ui.horizontal(|ui| {
                            if is_system_duplicate {
                                ui.label(egui::RichText::new("🛡️")).on_hover_text(t("system_duplicate_hover"));
                                ui.label(egui::RichText::new(format!("{:>2}.", i + 1)).monospace().color(egui::Color32::DARK_GRAY));
                                let display_path = truncate_path(path, 85);
                                ui.label(egui::RichText::new(display_path).color(egui::Color32::DARK_GRAY).italics());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_enabled(false, egui::Button::new(t("btn_edit")));
                                    ui.add_enabled(false, egui::Button::new(t("btn_delete")));
                                    if ui.button(t("btn_copy")).clicked() {
                                        ctx.copy_text(path.clone());
                                        self.status_msg = t("msg_copied");
                                    }
                                });
                            } else {
                                let (_rect, dropped_payload) = ui.dnd_drop_zone::<usize, _>(egui::Frame::NONE, |ui| {
                                    let inner_response = ui.dnd_drag_source::<usize, _>(item_id, i, |ui: &mut egui::Ui| {
                                        ui.label(egui::RichText::new("☰").color(egui::Color32::GRAY).strong());
                                    });

                                    if inner_response.response.drag_started() {
                                        inner_response.response.dnd_set_drag_payload(i);
                                    }

                                    ui.label(egui::RichText::new(format!("{:>2}.", i + 1)).monospace().color(egui::Color32::GRAY));
                                    
                                    if self.editing_index == Some(i + 1) {
                                        ui.add(egui::TextEdit::singleline(&mut self.edit_input).desired_width(500.0));
                                        if ui.button(t("btn_save")).clicked() { to_save = Some((i, self.edit_input.clone())); }
                                        if ui.button(t("btn_cancel")).clicked() { self.editing_index = None; }
                                    } else {
                                        let exists = path_info.diagnostic.exists;
                                        let has_executables = path_info.diagnostic.has_executables;

                                        let display_path = truncate_path(path, 75);
                                        let text = if !exists {
                                            egui::RichText::new(format!("⚠️ {}", display_path)).color(egui::Color32::RED)
                                        } else if !has_executables {
                                            egui::RichText::new(format!("No Exec: {}", display_path)).color(egui::Color32::GOLD).italics()
                                        } else {
                                            egui::RichText::new(display_path).color(egui::Color32::LIGHT_GRAY)
                                        };

                                        ui.add(egui::Label::new(text).truncate()).on_hover_text(format!("{}\n{}", path, if !has_executables { t("no_executables_hover") } else { String::new() }));

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button(t("btn_delete")).clicked() { to_delete = Some(i + 1); }
                                            if ui.button(t("btn_edit")).clicked() {
                                                self.editing_index = Some(i + 1);
                                                self.edit_input = path.clone();
                                            }
                                            if ui.button(t("btn_copy")).clicked() {
                                                ctx.copy_text(path.clone());
                                                self.status_msg = t("msg_copied");
                                            }
                                        });
                                    }
                                });
                                if let Some(source_idx) = dropped_payload {
                                    if *source_idx != i { move_data = Some((*source_idx, i)); }
                                }
                            }
                        });
                    }
                });

                ui.add_space(20.0);

                // SYSTEM PATH
                ui.group(|ui| {
                    ui.heading(egui::RichText::new(t("system_path_heading")).color(egui::Color32::GOLD).size(18.0));
                    ui.add_space(5.0);

                    for (i, path_info) in self.system_paths.iter().enumerate() {
                        let path = &path_info.path;
                        if !query.is_empty() && !path.to_lowercase().contains(&query) { continue; }

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{:>2}.", i + 1)).monospace().color(egui::Color32::GRAY));
                            let display_path = truncate_path(path, 85);
                            ui.label(egui::RichText::new(display_path).color(egui::Color32::GRAY));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(t("btn_copy")).clicked() {
                                    ctx.copy_text(path.clone());
                                    self.status_msg = t("msg_copied");
                                }
                            });
                        });
                    }
                });
            });

            if let Some((from, to)) = move_data {
                let mut current_paths: Vec<String> = self.user_paths.iter().map(|p| p.path.clone()).collect();
                let path = current_paths.remove(from);
                current_paths.insert(to, path);
                let _ = write_paths(PathScope::User, current_paths);
                self.refresh_all();
                self.status_msg = t("msg_reordered");
            }

            if let Some((idx, new_val)) = to_save {
                match set_path(PathScope::User, idx + 1, new_val) {
                    Ok(_) => { self.refresh_all(); self.editing_index = None; self.status_msg = t("updated"); }
                    Err(e) => self.status_msg = t_args("msg_error", &e.to_string()),
                }
            }
            if let Some(idx) = to_delete {
                match remove_path(PathScope::User, idx) {
                    Ok(_) => { self.refresh_all(); self.status_msg = t("msg_deleted"); }
                    Err(e) => self.status_msg = t_args("msg_error", &e.to_string()),
                }
            }
        });
    }
}
