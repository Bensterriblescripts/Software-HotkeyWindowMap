use std::error::Error;
use winapi::um::winuser::{BringWindowToTop, EnumWindows, FindWindowW, GetWindowLongW, GetWindowTextW, IsWindowVisible, SetForegroundWindow, SetWindowLongW, SetWindowPos, ShowWindow, GWL_STYLE, SWP_FRAMECHANGED, SWP_SHOWWINDOW, SW_SHOW, WS_OVERLAPPEDWINDOW, WS_POPUP, SW_SHOWMAXIMIZED};
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::shared::ntdef::LONG;

use winapi::shared::windef::{HMONITOR, RECT};
use winapi::um::winuser::{MonitorFromWindow, GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO};

pub unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }
        let mut title = [0u16; 256];
        let len = GetWindowTextW(hwnd, title.as_mut_ptr(), 256);
        if len == 0 {
            return 1;
        }
        let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
        
        let chars_copied = GetWindowTextW(hwnd, buffer.as_mut_ptr(), len + 1);
        if chars_copied == 0 {
            return 1;
        }

        let title = String::from_utf16_lossy(&buffer[0..chars_copied as usize]);
        let search_data = &mut *(lparam as *mut (String, Option<HWND>));
        
        if title == search_data.0 {
            search_data.1 = Some(hwnd);
            return 0;
        }
        
        1
    }
}
pub fn find_window(title: &str) -> Result<HWND, Box<dyn Error>> {
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    
    unsafe {
        let hwnd = FindWindowW(std::ptr::null_mut(), title_wide.as_ptr());
        if !hwnd.is_null() {
            return Ok(hwnd);
        }
    }
    let mut search_data = (title.to_string(), None);
    
    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            &mut search_data as *mut _ as LPARAM
        );
    }
    
    if let Some(hwnd) = search_data.1 {
        Ok(hwnd)
    } else {
        Err("Window not found".into())
    }
}
pub fn make_borderless_fullscreen(window_title: &str) -> Result<(), Box<dyn Error>> {
    let hwnd = find_window(window_title)?;

    // Determine the monitor where this window currently resides
    let monitor: HMONITOR = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return Err("Failed to get monitor for window".into());
    }

    // Query monitor work area (or full area). For borderless fullscreen, use rcMonitor.
    let mut mi: MONITORINFO = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        rcWork: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        dwFlags: 0,
    };
    let ok = unsafe { GetMonitorInfoW(monitor, &mut mi as *mut MONITORINFO) };
    if ok == 0 {
        return Err("GetMonitorInfoW failed".into());
    }

    let x = mi.rcMonitor.left;
    let y = mi.rcMonitor.top;
    let width = mi.rcMonitor.right - mi.rcMonitor.left;
    let height = mi.rcMonitor.bottom - mi.rcMonitor.top;

    unsafe {
        let original_style = GetWindowLongW(hwnd, GWL_STYLE);
        let new_style = original_style & !WS_OVERLAPPEDWINDOW as LONG;
        SetWindowLongW(hwnd, GWL_STYLE, new_style);

        // Position and size to the monitor rect the window is on
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            x,
            y,
            width,
            height,
            SWP_FRAMECHANGED | SWP_SHOWWINDOW,
        );

        ShowWindow(hwnd, SW_SHOW);
        Ok(())
    }
}
pub fn make_windowed(window_title: &str) -> Result<(), Box<dyn Error>>{
    let hwnd = find_window(window_title)?;

    // Determine the monitor where this window currently resides
    let monitor: HMONITOR = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return Err("Failed to get monitor for window".into());
    }

    // Query monitor work area (or full area). For borderless fullscreen, use rcMonitor.
    let mut mi: MONITORINFO = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        rcWork: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        dwFlags: 0,
    };
    let ok = unsafe { GetMonitorInfoW(monitor, &mut mi as *mut MONITORINFO) };
    if ok == 0 {
        return Err("GetMonitorInfoW failed".into());
    }

    let x = mi.rcMonitor.left;
    let y = mi.rcMonitor.top;
    let width = mi.rcMonitor.right - mi.rcMonitor.left;
    let height = mi.rcMonitor.bottom - mi.rcMonitor.top;

    unsafe {
        let original_style = GetWindowLongW(hwnd, GWL_STYLE);
        let new_style = (original_style | (WS_OVERLAPPEDWINDOW as LONG)) & !(WS_POPUP as LONG);
        SetWindowLongW(hwnd, GWL_STYLE, new_style);

        // Position and size to the monitor rect the window is on
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            x,
            y,
            width,
            height,
            SWP_FRAMECHANGED | SWP_SHOWWINDOW,
        );

        ShowWindow(hwnd, SW_SHOW);
        Ok(())
    }
}
pub fn set_focus(hwnd: HWND, default_state: i32) -> Result<(), Box<dyn Error>>{
    unsafe {
        if hwnd.is_null() {
            return Err("Window handle is null".into());
        }
        if ShowWindow(hwnd, default_state) == 0 {
            return Err("Failed to show window".into());
        }
        if BringWindowToTop(hwnd) == 0 {
            return Err("Failed to bring window to top".into());
        }
        if SetForegroundWindow(hwnd) == 0 {
            return Err("Failed to set foreground window".into());
        }
    }
    Ok(())
}
