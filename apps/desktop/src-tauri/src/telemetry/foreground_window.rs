use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessWindowInfo {
    pub process_id: u32,
    pub executable_name: String,
    pub executable_path: String,
    pub window_title: Option<String>,
}

#[cfg(target_os = "windows")]
pub fn get_foreground_process_info() -> Option<ProcessWindowInfo> {
    use std::path::Path;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }

        let process_handle: HANDLE = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            0,
            pid,
        );

        let mut exec_path = String::new();
        if !process_handle.is_null() {
            let mut buf = [0u16; 1024];
            let len = K32GetModuleFileNameExW(process_handle, std::ptr::null_mut(), buf.as_mut_ptr(), buf.len() as u32);
            if len > 0 {
                exec_path = String::from_utf16_lossy(&buf[..len as usize]);
            }
            CloseHandle(process_handle);
        }

        let exec_name = if !exec_path.is_empty() {
            Path::new(&exec_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.exe")
                .to_string()
        } else {
            "unknown.exe".to_string()
        };

        // Window Title
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
        let title = if title_len > 0 {
            Some(String::from_utf16_lossy(&title_buf[..title_len as usize]))
        } else {
            None
        };

        Some(ProcessWindowInfo {
            process_id: pid,
            executable_name: exec_name,
            executable_path: exec_path,
            window_title: title,
        })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_foreground_process_info() -> Option<ProcessWindowInfo> {
    Some(ProcessWindowInfo {
        process_id: 1000,
        executable_name: "Code.exe".to_string(),
        executable_path: "/usr/bin/code".to_string(),
        window_title: Some("FocusDrive Agent - Visual Studio Code".to_string()),
    })
}
