use super::PathScope;
use anyhow::Result;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

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

pub fn is_same_path(p1: &str, p2: &str) -> bool {
    expand_env_vars(p1).to_lowercase() == expand_env_vars(p2).to_lowercase()
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

pub fn scan_winget_packages() -> Vec<String> {
    let mut found_paths = Vec::new();
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    if local_app_data.is_empty() {
        return found_paths;
    }

    let winget_path = Path::new(&local_app_data).join("Microsoft").join("WinGet").join("Packages");
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



pub fn has_executables(path_obj: &Path) -> bool {
    let exe_exts = ["exe", "com", "bat", "cmd", "ps1", "vbs", "msc", "js"];
    std::fs::read_dir(path_obj).map(|entries| {
        entries.filter_map(|e| e.ok()).any(|e| {
            let p = e.path();
            if !p.is_file() { return false; }
            p.extension()
                .and_then(|s| s.to_str())
                .map(|s| exe_exts.contains(&s.to_lowercase().as_str()))
                .unwrap_or(false)
        })
    }).unwrap_or(false)
}

pub fn spawn_registry_watcher(on_change: impl Fn() + Send + 'static) {
    std::thread::spawn(move || {
        use windows_sys::Win32::System::Registry::{
            RegOpenKeyExW, RegNotifyChangeKeyValue, HKEY_CURRENT_USER, KEY_NOTIFY,
            REG_NOTIFY_CHANGE_LAST_SET,
        };
        use windows_sys::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
        use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
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
                                on_change();
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

pub fn check_sync_needed(_scope: PathScope) -> bool {
    false
}
