use std::path::Path;
use crate::compat::anyhow::Result;
pub use crate::compat::windows::{expand_env_vars, notify_environment_change};
use crate::compat::winreg::{enums::*, RegKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathScope {
    User,
    System,
}

impl PathScope {
    pub fn name(&self) -> &'static str {
        match self {
            PathScope::User => "User",
            PathScope::System => "System",
        }
    }

    pub fn reg_info(&self) -> (usize, &'static str) {
        match self {
            PathScope::User => (HKEY_CURRENT_USER, "Environment"),
            PathScope::System => (
                HKEY_LOCAL_MACHINE,
                r"System\CurrentControlSet\Control\Session Manager\Environment",
            ),
        }
    }
}

pub fn read_current_paths(scope: PathScope) -> Result<Vec<String>> {
    let (hkey, sub_key) = scope.reg_info();
    let root = RegKey::predef(hkey);
    let key = match root.open_subkey(sub_key) {
        Ok(k) => k,
        Err(_) => return Ok(Vec::new()),
    };

    let path_val = key.get_value("Path").unwrap_or_default();
    let paths: Vec<String> = path_val
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(paths)
}

pub fn write_paths(scope: PathScope, paths: Vec<String>) -> Result<()> {
    let (hkey, sub_key) = scope.reg_info();
    let root = RegKey::predef(hkey);
    let key = root.open_subkey_with_flags(sub_key, KEY_WRITE)?;

    let path_str = paths.join(";");
    key.set_value("Path", &path_str)?;
    notify_environment_change();
    Ok(())
}

pub fn add_path(scope: PathScope, new_path: String) -> Result<bool> {
    let clean = new_path.trim().trim_matches('"').trim_matches('\'').to_string();
    if clean.is_empty() {
        return Ok(false);
    }

    let mut paths = read_current_paths(scope)?;
    if paths.iter().any(|p| is_same_path(p, &clean)) {
        return Ok(false);
    }

    paths.push(clean);
    write_paths(scope, paths)?;
    Ok(true)
}

pub fn remove_path(scope: PathScope, index: usize) -> Result<()> {
    let mut paths = read_current_paths(scope)?;
    if index == 0 || index > paths.len() {
        return Err(crate::compat::anyhow::anyhow!("Invalid path index: {}", index));
    }

    paths.remove(index - 1);
    write_paths(scope, paths)
}

pub fn set_path(scope: PathScope, index: usize, new_path: String) -> Result<String> {
    let clean = new_path.trim().trim_matches('"').trim_matches('\'').to_string();
    if clean.is_empty() {
        return Err(crate::compat::anyhow::anyhow!("Path cannot be empty."));
    }

    let mut paths = read_current_paths(scope)?;
    if index == 0 || index > paths.len() {
        return Err(crate::compat::anyhow::anyhow!("Invalid path index: {}", index));
    }

    let old = std::mem::replace(&mut paths[index - 1], clean);
    write_paths(scope, paths)?;
    Ok(old)
}

pub fn move_path(scope: PathScope, from_idx: usize, to_idx: usize) -> Result<()> {
    let mut paths = read_current_paths(scope)?;
    if from_idx >= paths.len() || to_idx >= paths.len() {
        return Ok(());
    }

    let item = paths.remove(from_idx);
    paths.insert(to_idx, item);
    write_paths(scope, paths)
}

pub fn dedupe_paths(scope: PathScope) -> Result<usize> {
    let paths = read_current_paths(scope)?;
    let mut unique = Vec::new();
    let mut count = 0;

    let system_paths = if scope == PathScope::User {
        read_current_paths(PathScope::System).unwrap_or_default()
    } else {
        Vec::new()
    };

    for p in paths {
        let is_internal_dup = unique.iter().any(|existing: &String| is_same_path(existing, &p));
        let is_system_dup = system_paths.iter().any(|sp| is_same_path(sp, &p));

        if !is_internal_dup && !is_system_dup {
            unique.push(p);
        } else {
            count += 1;
        }
    }

    if count > 0 {
        write_paths(scope, unique)?;
    }
    Ok(count)
}

pub fn merge_paths(scope: PathScope, imported_paths: Vec<String>) -> Result<usize> {
    let mut current = read_current_paths(scope)?;
    let mut count = 0;
    for p in imported_paths {
        let clean = p.trim().trim_matches('"').to_string();
        if clean.is_empty() {
            continue;
        }
        if !current.iter().any(|existing| is_same_path(existing, &clean)) {
            current.push(clean);
            count += 1;
        }
    }
    if count > 0 {
        write_paths(scope, current)?;
    }
    Ok(count)
}

pub fn is_same_path(p1: &str, p2: &str) -> bool {
    let clean1 = expand_env_vars(p1).trim_end_matches(['\\', '/']).to_lowercase();
    let clean2 = expand_env_vars(p2).trim_end_matches(['\\', '/']).to_lowercase();
    clean1 == clean2
}

pub fn has_executables(path_obj: &Path) -> bool {
    const EXE_EXTS: &[&str] = &["exe", "com", "bat", "cmd", "ps1", "vbs", "msc", "js"];
    if let Ok(entries) = std::fs::read_dir(path_obj) {
        entries.filter_map(|e| e.ok()).any(|e| {
            let p = e.path();
            if !p.is_file() {
                return false;
            }
            p.extension()
                .and_then(|s| s.to_str())
                .map(|s| EXE_EXTS.contains(&s.to_lowercase().as_str()))
                .unwrap_or(false)
        })
    } else {
        false
    }
}

pub fn scan_winget_packages() -> Vec<String> {
    let mut found_paths = Vec::new();
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    if local_app_data.is_empty() {
        return found_paths;
    }

    let winget_path = Path::new(&local_app_data)
        .join("Microsoft")
        .join("WinGet")
        .join("Packages");
    if !winget_path.exists() || !winget_path.is_dir() {
        return found_paths;
    }

    let mut stack = vec![winget_path];
    while let Some(current_dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            let mut has_exe = false;
            let mut sub_dirs = Vec::new();

            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if ext.eq_ignore_ascii_case("exe") {
                            has_exe = true;
                        }
                    }
                } else if path.is_dir() {
                    sub_dirs.push(path);
                }
            }

            if has_exe {
                if let Some(path_str) = current_dir.to_str() {
                    found_paths.push(path_str.to_string());
                }
            } else {
                for sd in sub_dirs {
                    stack.push(sd);
                }
            }
        }
    }
    found_paths
}

pub fn spawn_watcher(on_change: impl Fn() + Send + 'static) {
    crate::compat::windows::spawn_registry_watcher(on_change);
}
