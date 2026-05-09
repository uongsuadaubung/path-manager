use super::PathScope;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use std::os::unix::fs as unix_fs;

fn get_bin_dir(scope: PathScope) -> PathBuf {
    match scope {
        PathScope::System => PathBuf::from("/usr/local/bin"),
        PathScope::User => {
            let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
            p.push(".local");
            p.push("bin");
            p
        }
    }
}

pub fn read_current_paths(scope: PathScope) -> Result<Vec<String>> {
    let bin_dir = get_bin_dir(scope);
    let mut paths = std::collections::HashSet::new();
    
    if bin_dir.exists() {
        if let Ok(entries) = fs::read_dir(&bin_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_symlink() {
                    // Đọc đích đến của symlink để dò ra thư mục gốc
                    if let Ok(target) = fs::read_link(&p) {
                        if let Some(parent) = target.parent() {
                            if let Some(parent_str) = parent.to_str() {
                                // Bỏ qua thư mục bin_dir (thư mục Farm) vì nó là trạm trung chuyển, không phải source
                                if Some(parent_str) != bin_dir.to_str() {
                                    paths.insert(parent_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut paths_vec: Vec<String> = paths.into_iter().collect();
    paths_vec.sort(); // Sắp xếp cho gọn gàng trên UI
    Ok(paths_vec)
}

fn create_symlinks(bin_dir: &Path, source_dir: &Path) {
    if !bin_dir.exists() {
        let _ = fs::create_dir_all(bin_dir);
    }
    if !source_dir.exists() || !source_dir.is_dir() {
        return;
    }
    // NGĂN CHẶN TẠO SYMLINK VÒNG LẶP NẾU NGUỒN VÀ ĐÍCH LÀ MỘT
    if source_dir == bin_dir {
        return;
    }
    
    use std::os::unix::fs::PermissionsExt;
    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() {
                if let Ok(metadata) = p.metadata() {
                    // Kiểm tra quyền thực thi (executable)
                    if metadata.permissions().mode() & 0o111 != 0 {
                        if let Some(file_name) = p.file_name() {
                            let symlink_path = bin_dir.join(file_name);
                            
                            // Ghi đè nếu đã tồn tại
                            if symlink_path.exists() || symlink_path.is_symlink() {
                                let _ = fs::remove_file(&symlink_path);
                            }
                            
                            let _ = unix_fs::symlink(&p, &symlink_path);
                        }
                    }
                }
            }
        }
    }
}

pub fn write_paths(scope: PathScope, paths: Vec<String>) -> Result<()> {
    let bin_dir = get_bin_dir(scope);

    if !bin_dir.exists() {
        let _ = fs::create_dir_all(&bin_dir);
    }

    let desired_paths: std::collections::HashSet<String> = paths.into_iter().collect();

    // 1. Quét ~/.local/bin để dọn dẹp các symlink KHÔNG còn nằm trong danh sách mới
    if let Ok(entries) = fs::read_dir(&bin_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_symlink() {
                if let Ok(target) = fs::read_link(&p) {
                    if let Some(parent) = target.parent() {
                        if let Some(parent_str) = parent.to_str() {
                            // Nếu thư mục gốc của symlink này không có trong danh sách paths mới -> Xóa!
                            if !desired_paths.contains(parent_str) {
                                let _ = fs::remove_file(&p);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Tạo symlink mới cho tất cả thư mục có trong danh sách
    for new_p in &desired_paths {
        create_symlinks(&bin_dir, Path::new(new_p));
    }

    Ok(())
}

pub fn is_same_path(p1: &str, p2: &str) -> bool {
    expand_env_vars(p1) == expand_env_vars(p2)
}

pub fn expand_env_vars(path: &str) -> String {
    let mut result = path.to_string();
    let mut i = 0;
    while let Some(start) = result[i..].find('$') {
        let start = i + start;
        let mut end = start + 1;
        let bytes = result.as_bytes();
        
        if end < bytes.len() && bytes[end] == b'{' {
            if let Some(close) = result[end..].find('}') {
                let close = end + close;
                let var_name = &result[end + 1..close];
                if let Ok(val) = std::env::var(var_name) {
                    result.replace_range(start..=close, &val);
                    i = start + val.len();
                } else {
                    i = close + 1;
                }
            } else {
                break;
            }
        } else {
            while end < bytes.len() && (bytes[end] as char).is_alphanumeric() || bytes[end] == b'_' {
                end += 1;
            }
            if end > start + 1 {
                let var_name = &result[start + 1..end];
                if let Ok(val) = std::env::var(var_name) {
                    result.replace_range(start..end, &val);
                    i = start + val.len();
                } else {
                    i = end;
                }
            } else {
                i = start + 1;
            }
        }
    }
    result
}

pub fn scan_winget_packages() -> Vec<String> {
    Vec::new()
}



pub fn has_executables(path_obj: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::read_dir(path_obj).map(|entries| {
        entries.filter_map(|e| e.ok()).any(|e| {
            let p = e.path();
            if !p.is_file() { return false; }
            // Dùng fs::metadata thay vì e.metadata() để đảm bảo luôn follow symlink tới file gốc
            if let Ok(metadata) = std::fs::metadata(&p) {
                metadata.permissions().mode() & 0o111 != 0
            } else {
                false
            }
        })
    }).unwrap_or(false)
}

pub fn spawn_registry_watcher(_on_change: impl Fn() + Send + 'static) {}

pub fn check_sync_needed(scope: PathScope) -> bool {
    let bin_dir = get_bin_dir(scope);
    if !bin_dir.exists() {
        return false;
    }

    let mut symlink_targets = std::collections::HashSet::new();
    let mut parent_dirs = std::collections::HashSet::new();

    if let Ok(entries) = fs::read_dir(&bin_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_symlink() {
                if let Ok(target) = fs::read_link(&p) {
                    symlink_targets.insert(target.clone());
                    if let Some(parent) = target.parent() {
                        if Some(parent.to_str().unwrap_or_default()) != bin_dir.to_str() {
                            parent_dirs.insert(parent.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    use std::os::unix::fs::PermissionsExt;
    for parent_dir in parent_dirs {
        if let Ok(entries) = fs::read_dir(&parent_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() {
                    if let Ok(metadata) = p.metadata() {
                        if metadata.permissions().mode() & 0o111 != 0 {
                            if !symlink_targets.contains(&p) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}
