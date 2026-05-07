use winreg::enums::*;
use winreg::RegKey;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathScope {
    User,
    System,
}

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
    let (hk, sub_key) = get_registry_info(scope);
    let key = hk.open_subkey(&sub_key)?;
    let path: String = key.get_value("Path").unwrap_or_default();
    
    Ok(path.split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect())
}

pub fn write_paths(scope: PathScope, paths: Vec<String>) -> Result<()> {
    let (hk, sub_key) = get_registry_info(scope);
    let key = hk.open_subkey_with_flags(&sub_key, KEY_WRITE)?;
    let path_string = paths.join(";");
    key.set_value("Path", &path_string)?;
    notify_system();
    Ok(())
}

/// So sánh hai đường dẫn một cách thông minh (đã giải mã biến môi trường)
pub fn is_same_path(p1: &str, p2: &str) -> bool {
    expand_env_vars(p1).to_lowercase() == expand_env_vars(p2).to_lowercase()
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

pub fn notify_system() {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let env_str = "Environment\0";
    unsafe {
        SendMessageTimeoutA(HWND_BROADCAST, WM_SETTINGCHANGE, 0, env_str.as_ptr() as isize, SMTO_ABORTIFHUNG, 5000, std::ptr::null_mut());
    }
}
