#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logic;
mod ui;

use clap::{Parser, Subcommand};
use colored::*;
use logic::*;
use ui::PathApp;

#[derive(Parser)]
#[command(name = "path-mgr")]
#[command(about = "🚀 Trình quản lý User & System PATH chuyên nghiệp cho Windows", long_about = None)]
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

    let args = Args::parse();
    let scope = if args.system { PathScope::System } else { PathScope::User };

    if args.command.is_some() {
        match args.command.unwrap() {
            Commands::List => {
                let paths = read_current_paths(scope)?;
                println!("\n{} ({})\n", "Danh sách PATH hiện tại:".cyan().bold(), if args.system { "System".red() } else { "User".green() });
                for (i, p) in paths.iter().enumerate() {
                    println!("{:>2}. {}", (i + 1).to_string().yellow(), p);
                }
                println!("\n{} {} đường dẫn.", "Tổng cộng:".bold(), paths.len());
            }
            Commands::Add { path } => {
                match add_path(scope, path.clone()) {
                    Ok(true) => println!("{} Đã thêm thành công: {}", "Thành công:".green().bold(), path),
                    Ok(false) => println!("{} Đường dẫn đã tồn tại trong PATH.", "Thông báo:".yellow()),
                    Err(e) => println!("{} {}", "Lỗi:".red().bold(), e),
                }
            }
            Commands::Remove { index } => {
                match remove_path(scope, index) {
                    Ok(_) => println!("{} Đã xóa đường dẫn tại vị trí {}.", "Thành công:".green().bold(), index),
                    Err(e) => println!("{} {}", "Lỗi:".red().bold(), e),
                }
            }
            Commands::Set { index, new_path } => {
                match set_path(scope, index, new_path.clone()) {
                    Ok(old) => {
                        println!("{} Đã cập nhật thành công.", "Thành công:".green().bold());
                        println!("   {} {}", "Từ:".blue(), old);
                        println!("   {} {}", "Thành:".green(), new_path);
                    }
                    Err(e) => println!("{} {}", "Lỗi:".red().bold(), e),
                }
            }
            Commands::Dedupe => {
                match dedupe_paths(scope) {
                    Ok(count) => println!("{} Đã loại bỏ {} đường dẫn trùng lặp.", "Thành công:".green().bold(), count),
                    Err(e) => println!("{} {}", "Lỗi:".red().bold(), e),
                }
            }
            Commands::Backup { file } => {
                let paths = read_current_paths(scope)?;
                let content = paths.join("\n");
                std::fs::write(&file, content)?;
                println!("{} Đã sao lưu PATH vào file: {}", "Thành công:".green().bold(), file);
            }
            Commands::Restore { file, override_path } => {
                let content = std::fs::read_to_string(&file)?;
                let imported_paths: Vec<String> = content.lines().filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
                
                if override_path {
                    write_paths(scope, imported_paths)?;
                    println!("{} Đã ghi đè hoàn toàn PATH từ file.", "Thành công:".green().bold());
                } else {
                    let count = merge_paths(scope, imported_paths)?;
                    println!("{} Đã hợp nhất thêm {} đường dẫn mới.", "Thành công:".green().bold(), count);
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
