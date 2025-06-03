use std::error::Error;
use winapi::um::winuser::{EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, GetWindowThreadProcessId};
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{BOOL, LPARAM, DWORD};
use winapi::um::winuser::SetForegroundWindow;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::shared::ntdef::{NTSTATUS, PVOID, ULONG};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::libloaderapi::{LoadLibraryA, GetProcAddress, FreeLibrary};
use std::mem;
use regex::Regex;

#[repr(C)]
struct ProcessBasicInformation {
    exit_status: NTSTATUS,
    peb_base_address: PVOID,
    affinity_mask: usize,
    base_priority: i32,
    unique_process_id: usize,
    inherited_from_unique_process_id: usize,
}
#[repr(C)]
struct UnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}
#[repr(C)]
struct RtlUserProcessParameters {
    reserved1: [u8; 16],
    reserved2: [PVOID; 10],
    image_path_name: UnicodeString,
    command_line: UnicodeString,
}
#[repr(C)]
struct Peb {
    reserved1: [u8; 2],
    being_debugged: u8,
    reserved2: [u8; 1],
    reserved3: [PVOID; 2],
    ldr: PVOID,
    process_parameters: *mut RtlUserProcessParameters,
}
type NtQueryInformationProcessFn = unsafe extern "system" fn(
    process_handle: winapi::um::winnt::HANDLE,
    process_information_class: u32,
    process_information: PVOID,
    process_information_length: ULONG,
    return_length: *mut ULONG,
) -> NTSTATUS;

// pub unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
//     unsafe {
//         if IsWindowVisible(hwnd) == 0 {
//             return 1;
//         }
//         let len = GetWindowTextLengthW(hwnd);
//         if len == 0 {
//             return 1;
//         }
//         let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
        
//         let chars_copied = GetWindowTextW(hwnd, buffer.as_mut_ptr(), len + 1);
//         if chars_copied == 0 {
//             return 1;
//         }
//         let title = String::from_utf16_lossy(&buffer[0..chars_copied as usize]);
//         let search_data = &mut *(lparam as *mut (String, Option<HWND>));
//         if title == search_data.0 {
//             search_data.1 = Some(hwnd);
//             return 0;
//         }
        
//         1
//     }
// }
// pub fn find_window_by_title(title: &str) -> Result<HWND, Box<dyn Error>> {

//     // Try finding it directly
//     let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
//     println!("Using the handle...");
//     unsafe {
//         let hwnd = FindWindowW(ptr::null(), title_wide.as_ptr());
//         if !hwnd.is_null() {
//             return Ok(hwnd);
//         }
//     }
    
//     // If that fails, try enumeration
//     let mut search_data = (title.to_string(), None);
//     println!("Iterating through active windows...");
//     unsafe {
//         EnumWindows(
//             Some(enum_windows_callback),
//             &mut search_data as *mut _ as LPARAM
//         );
//     }

//     if let Some(hwnd) = search_data.1 {
//         Ok(hwnd)
//     } else {
//         Err("Window not found".into())
//     }
// }

pub fn list_visible_windows(regex: &Vec<Regex>) -> Result<Vec<(HWND, String, String)>, Box<dyn Error>> {
    let mut windows: Vec<(HWND, String, String)> = Vec::new();
    
    unsafe {
        EnumWindows(
            Some(enum_windows_callback_list),
            &mut windows as *mut _ as LPARAM,
        );
    }

    println!("\n");

    // Filter Window Names
    for window in windows.iter_mut() {

        if window.1.contains("Personal - Microsoft​ Edge") {
            window.1 = "Microsoft Edge - Personal".to_string();
        }
        else if regex[0].is_match(&window.1) {
            window.1 = format!("Microsoft Edge - {}", regex[0].captures(&window.1).unwrap()[2].to_string());
        }

        else if window.1.contains(" - Cursor") {
            window.1 = "Cursor".to_string();
        }

        else if regex[1].is_match(&window.1) {
            let title = regex[2].captures(&window.1).unwrap()[1].to_string();
            let path = regex[1].captures(&window.1).unwrap()[1].to_string();
            window.1 = format!("File Explorer - {}", title);
            window.2 = format!("explorer.exe \"{}\"", path);
        }

        else if regex[3].is_match(&window.1) {
            window.1 = "Discord".to_string();
        }

        else {
            window.1 = window.1.clone();
        }

    }
    
    windows.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
    Ok(windows)
}
pub fn make_focus(window_handle: HWND) -> Result<(), Box<dyn Error>> {
    unsafe {
        SetForegroundWindow(window_handle);
    }
    Ok(())
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

        /* Hardcode to Disable Specific Windows */
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

        // Retrieve the Window Info
        match get_window_info(hwnd) {
            Ok(window_info) => {
                println!("Title: {}: \nPath: {}\n", window_info.title, window_info.command_line);
                window_info.command_line
            }
            Err(_) => {
                println!("Title: {}: \nPath: <Unable to retrieve>\n", title);
                String::new()
            }
        };

        // Fix the type mismatch - cast to the correct 3-tuple type
        let window_list = &mut *(lparam as *mut Vec<(HWND, String, String)>);
        window_list.push((hwnd, title, String::new()));
        
        1
    }
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

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub title: String,
    pub command_line: String,
}
pub fn get_window_info(hwnd: HWND) -> Result<WindowInfo, Box<dyn Error>> {
    unsafe {
        let title_len = GetWindowTextLengthW(hwnd);
        let title = if title_len > 0 {
            let mut buffer: Vec<u16> = vec![0; (title_len + 1) as usize];
            let chars_copied = GetWindowTextW(hwnd, buffer.as_mut_ptr(), title_len + 1);
            if chars_copied > 0 {
                String::from_utf16_lossy(&buffer[0..chars_copied as usize])
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut process_id: DWORD = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id);
        let command_line = get_process_command_line(process_id).unwrap_or_else(|_| String::new());

        Ok(WindowInfo {
            title,
            command_line,
        })
    }
}
pub fn get_process_command_line(process_id: u32) -> Result<String, Box<dyn Error>> {
    unsafe {
        let ntdll = LoadLibraryA(b"ntdll.dll\0".as_ptr() as *const i8);
        if ntdll.is_null() {
            return Err("Failed to load ntdll.dll".into());
        }

        let nt_query_info_proc = GetProcAddress(ntdll, b"NtQueryInformationProcess\0".as_ptr() as *const i8);
        if nt_query_info_proc.is_null() {
            FreeLibrary(ntdll);
            return Err("Failed to get NtQueryInformationProcess".into());
        }

        let nt_query_info_proc: NtQueryInformationProcessFn = mem::transmute(nt_query_info_proc);

        // Open the process
        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            0,
            process_id
        );

        if process_handle.is_null() {
            FreeLibrary(ntdll);
            return Err("Failed to open process".into());
        }

        // Get process basic information
        let mut basic_info: ProcessBasicInformation = mem::zeroed();
        let mut return_length: ULONG = 0;

        let status = nt_query_info_proc(
            process_handle,
            0,
            &mut basic_info as *mut _ as PVOID,
            mem::size_of::<ProcessBasicInformation>() as ULONG,
            &mut return_length,
        );

        if status != STATUS_SUCCESS {
            CloseHandle(process_handle);
            FreeLibrary(ntdll);
            return Err("NtQueryInformationProcess failed".into());
        }

        // Read PEB
        let mut peb: Peb = mem::zeroed();
        let mut bytes_read: usize = 0;

        let read_result = ReadProcessMemory(
            process_handle,
            basic_info.peb_base_address,
            &mut peb as *mut _ as *mut _,
            mem::size_of::<Peb>(),
            &mut bytes_read,
        );

        if read_result == 0 || peb.process_parameters.is_null() {
            CloseHandle(process_handle);
            FreeLibrary(ntdll);
            return Err("Failed to read PEB".into());
        }

        // Read process parameters
        let mut process_params: RtlUserProcessParameters = mem::zeroed();
        let read_result = ReadProcessMemory(
            process_handle,
            peb.process_parameters as *const _,
            &mut process_params as *mut _ as *mut _,
            mem::size_of::<RtlUserProcessParameters>(),
            &mut bytes_read,
        );

        if read_result == 0 {
            CloseHandle(process_handle);
            FreeLibrary(ntdll);
            return Err("Failed to read process parameters".into());
        }

        // Read command line string with bounds checking
        let command_line = if process_params.command_line.length > 0 
            && !process_params.command_line.buffer.is_null() 
            && process_params.command_line.length <= 32768 { // Reasonable upper bound
            
            let buffer_size = (process_params.command_line.length / 2) as usize;
            if buffer_size > 0 && buffer_size <= 16384 { // Additional safety check
                let mut buffer: Vec<u16> = vec![0; buffer_size];
                let read_result = ReadProcessMemory(
                    process_handle,
                    process_params.command_line.buffer as *const _,
                    buffer.as_mut_ptr() as *mut _,
                    process_params.command_line.length as usize,
                    &mut bytes_read,
                );

                if read_result != 0 && bytes_read > 0 {
                    let actual_chars = bytes_read / 2;
                    if actual_chars <= buffer.len() {
                        String::from_utf16_lossy(&buffer[0..actual_chars])
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        CloseHandle(process_handle);
        FreeLibrary(ntdll);

        Ok(command_line)
    }
}
