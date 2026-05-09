#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod i18n;
mod logic;
mod ui;

use clap::{Parser, Subcommand};
use colored::*;
use logic::*;
use ui::PathApp;

#[derive(Parser)]
#[command(name = "path-mgr")]
#[command(about = "🚀 Trình quản lý User & System PATH chuyên nghiệp", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Thao tác trên System PATH (yêu cầu quyền Admin) thay vì User PATH
    #[arg(short, long, global = true)]
    system: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// ✨ Liệt kê tất cả đường dẫn
    List,
    /// ➕ Thêm đường dẫn mới
    Add { path: String },
    /// 🗑 Xóa đường dẫn theo số thứ tự
    Remove { index: usize },
    /// ✏️ Cập nhật đường dẫn tại vị trí cụ thể
    Set { index: usize, new_path: String },
    /// 🧹 Loại bỏ các đường dẫn trùng lặp
    Dedupe,
    /// 📤 Sao lưu danh sách PATH ra file
    Backup {
        #[arg(short, long, default_value = "path_backup.txt")]
        file: String,
    },
    /// 📥 Khôi phục hoặc hợp nhất PATH từ file
    Restore {
        #[arg(short, long, default_value = "path_backup.txt")]
        file: String,
        /// Ghi đè hoàn toàn PATH hiện tại (mặc định là Hợp nhất)
        #[arg(short, long)]
        override_path: bool,
    },
}

#[cfg(target_os = "windows")]
fn attach_console() {
    use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn main() -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    attach_console();

    // Khắc phục lỗi tương thích Vulkan trên Wayland/Linux bằng cách mặc định dùng OpenGL
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WGPU_BACKEND").is_err() {
            unsafe { std::env::set_var("WGPU_BACKEND", "gl") };
        }
    }

    let args = Args::parse();
    let scope = if args.system { PathScope::System } else { PathScope::User };

    if args.command.is_some() {
        use crate::i18n::{t, t_args};
        match args.command.unwrap() {
            Commands::List => {
                let paths = read_current_paths(scope)?;
                println!("\n{} ({})\n", t("cli_list_current").cyan().bold(), if args.system { t("cli_system").red() } else { t("cli_user").green() });
                for (i, p) in paths.iter().enumerate() {
                    println!("{:>2}. {}", (i + 1).to_string().yellow(), p);
                }
                println!("\n{} {} {}", t("cli_total").bold(), paths.len(), t("cli_paths"));
            }
            Commands::Add { path } => {
                match add_path(scope, path.clone()) {
                    Ok(true) => println!("{} {}", t("cli_success").green().bold(), path),
                    Ok(false) => println!("{} {}", t("cli_info").yellow(), t("cli_path_exists")),
                    Err(e) => println!("{} {}", t("cli_error").red().bold(), e),
                }
            }
            Commands::Remove { index } => {
                match remove_path(scope, index) {
                    Ok(_) => println!("{} {}", t("cli_success").green().bold(), t_args("cli_deleted_at", &index.to_string())),
                    Err(e) => println!("{} {}", t("cli_error").red().bold(), e),
                }
            }
            Commands::Set { index, new_path } => {
                match set_path(scope, index, new_path.clone()) {
                    Ok(old) => {
                        println!("{}", t("cli_updated_success").green().bold());
                        println!("   {} {}", t("cli_from").blue(), old);
                        println!("   {} {}", t("cli_to").green(), new_path);
                    }
                    Err(e) => println!("{} {}", t("cli_error").red().bold(), e),
                }
            }
            Commands::Dedupe => {
                match dedupe_paths(scope) {
                    Ok(count) => println!("{} {}", t("cli_success").green().bold(), t_args("cli_deduped_count", &count.to_string())),
                    Err(e) => println!("{} {}", t("cli_error").red().bold(), e),
                }
            }
            Commands::Backup { file } => {
                let paths = read_current_paths(scope)?;
                let content = paths.join("\n");
                std::fs::write(&file, content)?;
                println!("{} {}", t("cli_success").green().bold(), t_args("cli_backed_up_to", &file));
            }
            Commands::Restore { file, override_path } => {
                let content = std::fs::read_to_string(&file)?;
                let imported_paths: Vec<String> = content.lines().filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
                
                if override_path {
                    write_paths(scope, imported_paths)?;
                    println!("{}", t("cli_restored_overwrite").green().bold());
                } else {
                    let count = merge_paths(scope, imported_paths)?;
                    println!("{} {}", t("cli_success").green().bold(), t_args("cli_restored_merge", &count.to_string()));
                }
            }
        }
    } else {
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([900.0, 650.0])
                .with_min_inner_size([800.0, 500.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Path Manager",
            options,
            Box::new(|cc| Ok(Box::new(PathApp::new(cc)))),
        ).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }

    Ok(())
}
