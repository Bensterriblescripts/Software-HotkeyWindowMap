use std::error::Error;
use winapi::um::winuser::{EnumWindows, GetWindowTextLengthW, GetWindowTextW, 
                          IsWindowVisible, FindWindowW};
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::um::winuser::SetForegroundWindow;
use std::ptr;

pub unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }
        let len = GetWindowTextLengthW(hwnd);
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

pub fn find_window_by_title(title: &str) -> Result<HWND, Box<dyn Error>> {

    // Try finding it directly
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    println!("Using the handle...");
    unsafe {
        let hwnd = FindWindowW(ptr::null(), title_wide.as_ptr());
        if !hwnd.is_null() {
            return Ok(hwnd);
        }
    }
    
    // If that fails, try enumeration
    let mut search_data = (title.to_string(), None);
    println!("Iterating through active windows...");
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

pub fn list_visible_windows() -> Result<Vec<(HWND, String)>, Box<dyn Error>> {
    let mut windows: Vec<(HWND, String)> = Vec::new();
    
    unsafe {
        EnumWindows(
            Some(enum_windows_callback_list),
            &mut windows as *mut _ as LPARAM
        );
    }

    println!("\n");
    
    windows.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
    Ok(windows)
}

unsafe extern "system" fn enum_windows_callback_list(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }

        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return 1;
        }

        let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
        let chars_copied = GetWindowTextW(hwnd, buffer.as_mut_ptr(), len + 1);
        if chars_copied == 0 {
            return 1;
        }
        

        /* Hardcode to Disable Windows */
        let title = String::from_utf16_lossy(&buffer[0..chars_copied as usize]);
        if title == "Workstation Hotkey Manager" {
            return 1;
        }
        else if title == "Windows Input Experience" {
            return 1;
        }
        else if title == "Program Manager" {
            return 1;
        }
        else if title == "Settings" {
            return 1;
        }
        else if title == "Task Manager" {
            return 1;
        }
        else if title == "Windows Shell Experience Host" {
            return 1;
        }
        

        println!("{}", title);

        let window_list = &mut *(lparam as *mut Vec<(HWND, String)>);
        window_list.push((hwnd, title));
        
        1
    }
}

pub fn make_focus(window_title: &str) -> Result<(), Box<dyn Error>> {
    let hwnd = find_window_by_title(window_title)?;
    unsafe {
        SetForegroundWindow(hwnd);
    }
    println!("Made {} the active window\n", window_title);
    Ok(())
}
// pub fn make_borderless_fullscreen(window_title: &str) -> Result<(), Box<dyn Error>> {
//     let hwnd = find_window_by_title(window_title)?;
    
//     unsafe {
//         let original_style = GetWindowLongW(hwnd, GWL_STYLE);
//         let new_style = original_style & !WS_OVERLAPPEDWINDOW as LONG;
//         SetWindowLongW(hwnd, GWL_STYLE, new_style);
        
//         // Get screen dimensions
//         let screen_width = GetSystemMetrics(0);
//         let screen_height = GetSystemMetrics(1);
        
//         // Resize window to fill the screen
//         SetWindowPos(
//             hwnd,
//             ptr::null_mut(),
//             0, 0,
//             screen_width, screen_height,
//             SWP_FRAMECHANGED | SWP_SHOWWINDOW
//         );
        
//         ShowWindow(hwnd, SW_SHOW);
        
//         Ok(())
//     }
// }
