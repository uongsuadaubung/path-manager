use eframe::egui;
use crate::logic::*;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

enum AppEvent {
    AddPath(String),
    Restore(Vec<String>),
}

pub struct PathApp {
    user_paths: Vec<String>,
    system_paths: Vec<String>,
    new_path_input: String,
    status_msg: String,
    search_query: String,
    editing_index: Option<usize>, 
    edit_input: String,
    needs_refresh: Arc<AtomicBool>,
    async_event_rx: std::sync::mpsc::Receiver<AppEvent>,
    async_event_tx: std::sync::mpsc::Sender<AppEvent>,
}

impl PathApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let needs_refresh = Arc::new(AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::channel();
        spawn_registry_watcher(cc.egui_ctx.clone(), Arc::clone(&needs_refresh));

        // Tự động đảm bảo các đường dẫn mặc định của Windows luôn tồn tại
        let _ = Self::ensure_defaults();

        Self {
            user_paths: read_current_paths(PathScope::User).unwrap_or_default(),
            system_paths: read_current_paths(PathScope::System).unwrap_or_default(),
            new_path_input: String::new(),
            status_msg: "Sẵn sàng.".to_string(),
            search_query: String::new(),
            editing_index: None,
            edit_input: String::new(),
            needs_refresh,
            async_event_rx: rx,
            async_event_tx: tx,
        }
    }

    fn ensure_defaults() -> Result<(), anyhow::Error> {
        let defaults = vec![
            r"%USERPROFILE%\AppData\Local\Microsoft\WindowsApps".to_string(),
        ];
        for d in defaults {
            // add_path đã có sẵn logic kiểm tra trùng lặp thông minh
            let _ = add_path(PathScope::User, d);
        }
        Ok(())
    }

    fn refresh_all(&mut self) {
        self.user_paths = read_current_paths(PathScope::User).unwrap_or_default();
        self.system_paths = read_current_paths(PathScope::System).unwrap_or_default();
    }
}

fn spawn_registry_watcher(ctx: egui::Context, needs_refresh: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        use windows_sys::Win32::System::Registry::{
            RegOpenKeyExW, RegNotifyChangeKeyValue, HKEY_CURRENT_USER, KEY_NOTIFY,
            REG_NOTIFY_CHANGE_LAST_SET,
        };
        use windows_sys::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
        use windows_sys::Win32::Foundation::{WAIT_OBJECT_0};
        const INFINITE: u32 = 0xFFFFFFFF;

        let sub_key: Vec<u16> = "Environment\0".encode_utf16().collect();
        let mut h_key = std::ptr::null_mut();
        
        unsafe {
            if RegOpenKeyExW(HKEY_CURRENT_USER as _, sub_key.as_ptr(), 0, KEY_NOTIFY, &mut h_key) == 0 {
                let event = CreateEventW(std::ptr::null(), 1, 0, std::ptr::null());
                if !event.is_null() {
                    loop {
                        if RegNotifyChangeKeyValue(h_key, 0, REG_NOTIFY_CHANGE_LAST_SET, event, 1) == 0 {
                            if WaitForSingleObject(event, INFINITE) == WAIT_OBJECT_0 {
                                needs_refresh.store(true, Ordering::SeqCst);
                                ctx.request_repaint();
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    });
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    if let Ok(tahoma_data) = std::fs::read("C:\\Windows\\Fonts\\tahoma.ttf") {
        fonts.font_data.insert("tahoma".to_owned(), egui::FontData::from_owned(tahoma_data).into());
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "tahoma".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "tahoma".to_owned());
    }
    
    if let Ok(emoji_data) = std::fs::read("C:\\Windows\\Fonts\\seguiemj.ttf") {
        fonts.font_data.insert("emoji".to_owned(), egui::FontData::from_owned(emoji_data).into());
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().push("emoji".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("emoji".to_owned());
    }

    ctx.set_fonts(fonts);
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len { return path.to_string(); }
    let parts: Vec<&str> = path.split('\\').collect();
    if parts.len() < 2 { return format!("{}...", &path[..max_len - 3]); }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    let mid_len = max_len.saturating_sub(first.len() + last.len() + 5);
    if mid_len > 0 {
        format!("{}\\{}...\\{}", first, ".".repeat(mid_len.min(3)), last)
    } else {
        format!("{}\\{}", first, last)
    }
}

impl eframe::App for PathApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.needs_refresh.swap(false, Ordering::SeqCst) {
            self.refresh_all();
            if self.editing_index.is_none() {
                self.status_msg = "🔄 Đã cập nhật từ Registry.".to_string();
            }
        }

        while let Ok(event) = self.async_event_rx.try_recv() {
            match event {
                AppEvent::AddPath(path) => {
                    let _ = add_path(PathScope::User, path);
                    self.refresh_all();
                    self.status_msg = "✅ Đã thêm mới.".to_string();
                }
                AppEvent::Restore(paths) => {
                    if let Ok(count) = merge_paths(PathScope::User, paths) {
                        self.refresh_all();
                        self.status_msg = format!("📥 Đã khôi phục {} đường dẫn mới.", count);
                    }
                }
            }
        }

        egui::Panel::bottom("status_bar").show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.status_msg).small().color(egui::Color32::LIGHT_BLUE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("User: {} | System: {}", self.user_paths.len(), self.system_paths.len()));
                });
            });
            ui.add_space(3.0);
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
            
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                ui.heading(egui::RichText::new("🚀 Path Manager").strong().size(28.0));
                
                ui.horizontal(|ui| {
                    ui.label("🔍 Tìm kiếm:");
                    ui.add(egui::TextEdit::singleline(&mut self.search_query).desired_width(250.0));
                    ui.add_space(10.0);
                    if ui.button("🧹 Dọn trùng").clicked() {
                        let _ = dedupe_paths(PathScope::User);
                        self.refresh_all();
                        self.status_msg = "✅ Đã dọn trùng & bản sao hệ thống.".to_string();
                    }
                    if ui.button("🔄 Làm mới").clicked() {
                        self.refresh_all();
                        self.status_msg = "♻️ Đã làm mới.".to_string();
                    }
                    if ui.button("📤 Sao lưu").clicked() {
                        let paths = self.user_paths.clone();
                        std::thread::spawn(move || {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("path_backup.txt")
                                .save_file() {
                                let content = paths.join("\n");
                                let _ = std::fs::write(path, content);
                            }
                        });
                        self.status_msg = "📤 Đang chọn nơi lưu...".to_string();
                    }
                    if ui.button("📥 Khôi phục").clicked() {
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
                    ui.label("✨ Thêm mới:");
                    ui.add(egui::TextEdit::singleline(&mut self.new_path_input).hint_text("C:\\bin;...").desired_width(350.0));
                    if ui.button("➕ Thêm").clicked() && !self.new_path_input.is_empty() {
                        match add_path(PathScope::User, self.new_path_input.clone()) {
                            Ok(_) => {
                                self.refresh_all();
                                self.new_path_input.clear();
                                self.status_msg = "✅ Đã thêm mới.".to_string();
                            }
                            Err(e) => self.status_msg = format!("❌ Lỗi: {}", e),
                        }
                    }
                    if ui.button("📁 Chọn thư mục").clicked() {
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
                    ui.heading(egui::RichText::new("📍 User PATH").color(egui::Color32::LIGHT_GREEN).size(18.0));
                    ui.add_space(5.0);

                    for i in 0..self.user_paths.len() {
                        let path = self.user_paths[i].clone();
                        if !query.is_empty() && !path.to_lowercase().contains(&query) { continue; }

                        let is_system_default = self.system_paths.iter().any(|sp| {
                            is_same_path(sp, &path)
                        });
                        let item_id = egui::Id::new("user_item").with(i);
                        
                        ui.horizontal(|ui| {
                            if is_system_default {
                                ui.label(egui::RichText::new("🛡️")).on_hover_text("Bản sao hệ thống (Đã khóa)");
                                ui.label(egui::RichText::new(format!("{:>2}.", i + 1)).monospace().color(egui::Color32::DARK_GRAY));
                                let display_path = truncate_path(&path, 85);
                                ui.label(egui::RichText::new(display_path).color(egui::Color32::DARK_GRAY).italics());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_enabled(false, egui::Button::new("📝 Sửa"));
                                    ui.add_enabled(false, egui::Button::new("🗑 Xóa"));
                                    if ui.button("📋 Copy").clicked() {
                                        ctx.copy_text(path.clone());
                                        self.status_msg = "✅ Đã copy.".to_string();
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
                                        if ui.button("💾 Lưu").clicked() { to_save = Some((i, self.edit_input.clone())); }
                                        if ui.button("🚫 Hủy").clicked() { self.editing_index = None; }
                                    } else {
                                        let expanded = expand_env_vars(&path);
                                        let path_obj = std::path::Path::new(&expanded);
                                        let exists = path_obj.exists();
                                        
                                        let has_executables = if exists && path_obj.is_dir() {
                                            let exe_exts = ["exe", "com", "bat", "cmd", "ps1", "vbs", "msc", "js"];
                                            std::fs::read_dir(path_obj).map(|entries| {
                                                entries.filter_map(|e| e.ok()).any(|e| {
                                                    let p = e.path();
                                                    p.is_file() && p.extension()
                                                        .and_then(|s| s.to_str())
                                                        .map(|s| exe_exts.contains(&s.to_lowercase().as_str()))
                                                        .unwrap_or(false)
                                                })
                                            }).unwrap_or(false)
                                        } else {
                                            true
                                        };

                                        let display_path = truncate_path(&path, 75);
                                        let text = if !exists {
                                            egui::RichText::new(format!("⚠️ {}", display_path)).color(egui::Color32::RED)
                                        } else if !has_executables {
                                            egui::RichText::new(format!("No Exec: {}", display_path)).color(egui::Color32::GOLD).italics()
                                        } else {
                                            egui::RichText::new(display_path).color(egui::Color32::LIGHT_GRAY)
                                        };

                                        ui.add(egui::Label::new(text).truncate()).on_hover_text(format!("{}\n{}", path, if !has_executables { "(Không chứa file thực thi)" } else { "" }));

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("🗑 Xóa").clicked() { to_delete = Some(i + 1); }
                                            if ui.button("📝 Sửa").clicked() {
                                                self.editing_index = Some(i + 1);
                                                self.edit_input = path.clone();
                                            }
                                            if ui.button("📋 Copy").clicked() {
                                                ctx.copy_text(path.clone());
                                                self.status_msg = "✅ Đã copy.".to_string();
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
                    ui.heading(egui::RichText::new("🖥️ System PATH").color(egui::Color32::GOLD).size(18.0));
                    ui.add_space(5.0);

                    for (i, path) in self.system_paths.iter().enumerate() {
                        if !query.is_empty() && !path.to_lowercase().contains(&query) { continue; }

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{:>2}.", i + 1)).monospace().color(egui::Color32::GRAY));
                            let display_path = truncate_path(path, 85);
                            ui.label(egui::RichText::new(display_path).color(egui::Color32::GRAY));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("📋 Copy").clicked() {
                                    ctx.copy_text(path.clone());
                                    self.status_msg = "✅ Đã copy.".to_string();
                                }
                            });
                        });
                    }
                });
            });

            if let Some((from, to)) = move_data {
                let path = self.user_paths.remove(from);
                self.user_paths.insert(to, path);
                let _ = write_paths(PathScope::User, self.user_paths.clone());
                self.status_msg = "🎯 Đã sắp xếp lại.".to_string();
            }

            if let Some((idx, new_val)) = to_save {
                match set_path(PathScope::User, idx + 1, new_val) {
                    Ok(_) => { self.refresh_all(); self.editing_index = None; self.status_msg = "✅ Đã cập nhật.".to_string(); }
                    Err(e) => self.status_msg = format!("❌ Lỗi: {}", e),
                }
            }
            if let Some(idx) = to_delete {
                match remove_path(PathScope::User, idx) {
                    Ok(_) => { self.refresh_all(); self.status_msg = "✅ Đã xóa.".to_string(); }
                    Err(e) => self.status_msg = format!("❌ Lỗi: {}", e),
                }
            }
        });
    }
}
