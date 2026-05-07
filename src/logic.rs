use anyhow::Result;

#[cfg(unix)]
use std::path::PathBuf;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathScope {
    User,
    System,
}

#[cfg(windows)]
pub fn get_registry_info(scope: PathScope) -> (RegKey, String) {
    let hk = RegKey::predef(match scope {
        PathScope::User => HKEY_CURRENT_USER,
        PathScope::System => HKEY_LOCAL_MACHINE,
    });
    let sub_key = match scope {
        PathScope::User => "Environment".to_string(),
        PathScope::System => r"System\CurrentControlSet\Control\Session Manager\Environment".to_string(),
    };
    (hk, sub_key)
}

pub fn read_current_paths(scope: PathScope) -> Result<Vec<String>> {
    #[cfg(windows)]
    {
        let (hk, sub_key) = get_registry_info(scope);
        let key = hk.open_subkey(&sub_key)?;
        let path: String = key.get_value("Path").unwrap_or_default();
        
        Ok(path.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect())
    }

    #[cfg(unix)]
    {
        let file_path = match scope {
            PathScope::System => PathBuf::from("/etc/environment"),
            PathScope::User => {
                let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
                p.push(".profile");
                p
            }
        };

        if !file_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&file_path)?;
        
        // Trên Linux, PATH thường được định nghĩa dạng: PATH="/usr/local/sbin:/usr/local/bin:..."
        // Chúng ta cần tìm dòng bắt đầu bằng PATH=
        for line in content.lines() {
            if line.starts_with("PATH=") {
                let val = line.trim_start_matches("PATH=").trim_matches('"');
                return Ok(val.split(':')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect());
            }
        }
        Ok(Vec::new())
    }
}

pub fn write_paths(scope: PathScope, paths: Vec<String>) -> Result<()> {
    #[cfg(windows)]
    {
        let (hk, sub_key) = get_registry_info(scope);
        let key = hk.open_subkey_with_flags(&sub_key, KEY_WRITE)?;
        let path_string = paths.join(";");
        key.set_value("Path", &path_string)?;
        notify_system();
        Ok(())
    }

    #[cfg(unix)]
    {
        let file_path = match scope {
            PathScope::System => PathBuf::from("/etc/environment"),
            PathScope::User => {
                let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
                p.push(".profile");
                p
            }
        };

        let path_string = format!("PATH=\"{}\"", paths.join(":"));
        
        let content = if file_path.exists() {
            std::fs::read_to_string(&file_path)?
        } else {
            String::new()
        };

        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut found = false;
        for line in lines.iter_mut() {
            if line.starts_with("PATH=") {
                *line = path_string.clone();
                found = true;
                break;
            }
        }

        if !found {
            lines.push(path_string);
        }

        std::fs::write(&file_path, lines.join("\n") + "\n")?;
        Ok(())
    }
}

pub fn is_same_path(p1: &str, p2: &str) -> bool {
    #[cfg(windows)]
    {
        expand_env_vars(p1).to_lowercase() == expand_env_vars(p2).to_lowercase()
    }
    #[cfg(unix)]
    {
        expand_env_vars(p1) == expand_env_vars(p2)
    }
}

pub fn add_path(scope: PathScope, new_path: String) -> Result<bool> {
    let mut paths = read_current_paths(scope)?;
    if paths.iter().any(|p| is_same_path(p, &new_path)) {
        return Ok(false);
    }
    paths.push(new_path);
    write_paths(scope, paths)?;
    Ok(true)
}

pub fn remove_path(scope: PathScope, index: usize) -> Result<()> {
    let mut paths = read_current_paths(scope)?;
    if index == 0 || index > paths.len() {
        return Err(anyhow::anyhow!("Số thứ tự không hợp lệ."));
    }
    paths.remove(index - 1);
    write_paths(scope, paths)
}

pub fn set_path(scope: PathScope, index: usize, new_path: String) -> Result<String> {
    let mut paths = read_current_paths(scope)?;
    if index == 0 || index > paths.len() {
        return Err(anyhow::anyhow!("Số thứ tự không hợp lệ."));
    }
    let old = std::mem::replace(&mut paths[index - 1], new_path);
    write_paths(scope, paths)?;
    Ok(old)
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
        let is_internal_duplicate = unique.iter().any(|existing: &String| is_same_path(existing, &p));
        let is_system_duplicate = system_paths.iter().any(|sp| is_same_path(sp, &p));

        if !is_internal_duplicate && !is_system_duplicate {
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
        if !current.iter().any(|existing| is_same_path(existing, &p)) {
            current.push(p);
            count += 1;
        }
    }
    if count > 0 {
        write_paths(scope, current)?;
    }
    Ok(count)
}

pub fn expand_env_vars(path: &str) -> String {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Environment::ExpandEnvironmentStringsW;
        let utf16_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let mut buffer = vec![0u16; 32768];
        unsafe {
            let size = ExpandEnvironmentStringsW(utf16_path.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32);
            if size > 0 && size <= buffer.len() as u32 {
                String::from_utf16_lossy(&buffer[..(size as usize - 1)])
            } else {
                path.to_string()
            }
        }
    }

    #[cfg(unix)]
    {
        // Tối ưu: Chỉ tìm và thay thế các biến xuất hiện trong chuỗi thay vì lặp qua toàn bộ môi trường
        let mut result = path.to_string();
        let mut i = 0;
        while let Some(start) = result[i..].find('$') {
            let start = i + start;
            let mut end = start + 1;
            let bytes = result.as_bytes();
            
            if end < bytes.len() && bytes[end] == b'{' {
                // Trường hợp ${VAR}
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
                // Trường hợp $VAR
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
}

#[cfg(windows)]
pub fn notify_system() {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let env_str = "Environment\0";
    unsafe {
        SendMessageTimeoutA(HWND_BROADCAST, WM_SETTINGCHANGE, 0, env_str.as_ptr() as isize, SMTO_ABORTIFHUNG, 5000, std::ptr::null_mut());
    }
}
