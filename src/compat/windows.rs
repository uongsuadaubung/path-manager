#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]

use std::ffi::c_void;

pub const HWND_BROADCAST: *mut c_void = 0xffff as *mut c_void;
pub const WM_SETTINGCHANGE: u32 = 0x001A;
pub const SMTO_ABORTIFHUNG: u32 = 0x0002;
pub const REG_NOTIFY_CHANGE_LAST_SET: u32 = 0x00000004;
pub const WAIT_OBJECT_0: u32 = 0x00000000;
pub const INFINITE: u32 = 0xFFFFFFFF;

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageTimeoutA(
        hWnd: *mut c_void,
        Msg: u32,
        wParam: usize,
        lParam: isize,
        fuFlags: u32,
        uTimeout: u32,
        lpdwResult: *mut usize,
    ) -> isize;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn ExpandEnvironmentStringsW(
        lpSrc: *const u16,
        lpDst: *mut u16,
        nSize: u32,
    ) -> u32;

    fn CreateEventW(
        lpEventAttributes: *mut c_void,
        bManualReset: i32,
        bInitialState: i32,
        lpName: *const u16,
    ) -> *mut c_void;

    fn WaitForSingleObject(
        hHandle: *mut c_void,
        dwMilliseconds: u32,
    ) -> u32;

    fn CloseHandle(hObject: *mut c_void) -> i32;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegNotifyChangeKeyValue(
        hKey: *mut c_void,
        bWatchSubtree: i32,
        dwNotifyFilter: u32,
        hEvent: *mut c_void,
        fAsynchronous: i32,
    ) -> i32;
}

pub fn notify_environment_change() {
    let env_str = b"Environment\0";
    unsafe {
        SendMessageTimeoutA(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            env_str.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            std::ptr::null_mut(),
        );
    }
}

pub fn expand_env_vars(path: &str) -> String {
    let utf16_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buffer = vec![0u16; 32768];
    unsafe {
        let size = ExpandEnvironmentStringsW(
            utf16_path.as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        );
        if size > 0 && size <= buffer.len() as u32 {
            String::from_utf16_lossy(&buffer[..(size as usize - 1)])
        } else {
            path.to_string()
        }
    }
}

pub fn spawn_registry_watcher(on_change: impl Fn() + Send + 'static) {
    std::thread::spawn(move || {
        let root = crate::compat::winreg::RegKey::predef(crate::compat::winreg::enums::HKEY_CURRENT_USER);
        if let Ok(key) = root.open_subkey_with_flags("Environment", crate::compat::winreg::enums::KEY_NOTIFY) {
            let h_key = key.raw_handle();
            let event = unsafe { CreateEventW(std::ptr::null_mut(), 1, 0, std::ptr::null()) };
            if event.is_null() {
                return;
            }

            loop {
                let res = unsafe {
                    RegNotifyChangeKeyValue(
                        h_key,
                        0,
                        REG_NOTIFY_CHANGE_LAST_SET,
                        event,
                        1,
                    )
                };
                if res != 0 {
                    break;
                }

                let wait_res = unsafe { WaitForSingleObject(event, INFINITE) };
                if wait_res == WAIT_OBJECT_0 {
                    on_change();
                } else {
                    break;
                }
            }

            unsafe {
                CloseHandle(event);
            }
        }
    });
}
