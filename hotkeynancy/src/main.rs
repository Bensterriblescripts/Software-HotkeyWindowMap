mod handles;

use std::collections::HashMap;
use windows_sys::Win32::{
    UI::WindowsAndMessaging::*,
    UI::Input::KeyboardAndMouse::*,
};
use winapi::shared::windef::HWND;


struct Application {
    label: String,
    windowname: String,
    executablepath: String,
    handle: Option<HWND>,
}

fn main() {
    unsafe {
        /* Hotkeys */
        const CTRL_D: u32 = 1;
        const CTRL_G: u32 = 2;
        let _ok = { RegisterHotKey(std::ptr::null_mut(), CTRL_D as i32, MOD_CONTROL, 'D' as u32) };
        let _ok = { RegisterHotKey(std::ptr::null_mut(), CTRL_G as i32, MOD_CONTROL, 'G' as u32) };

        /* Application Map */
        let mut hotkeys: HashMap<u32, Application> = HashMap::new();
        let app = Application {
            label: "Discord".to_string(),
            windowname: " - Discord".to_string(),
            executablepath: "C:\\Users\\Ben\\AppData\\Local\\Discord\\app-1.0.9212\\Discord.exe".to_string(),
            handle: None
        };
        hotkeys.insert(CTRL_D, app);
        let app = Application {
            label: "Github Desktop".to_string(),
            windowname: "Github Desktop".to_string(),
            executablepath: "C:\\Users\\Ben\\AppData\\Local\\GitHubDesktop\\GitHubDesktop.exe".to_string(),
            handle: None
        };
        hotkeys.insert(CTRL_G, app);

        /* Message Loop */
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_HOTKEY && msg.wParam == CTRL_D as usize {
                if let Some(app) = hotkeys.get_mut(&CTRL_D) {
                    if app.handle.is_none() {
                        println!("Finding handle by title");
                        if let Ok(hwnd) = handles::find_window(&app.windowname) {
                            app.handle = Some(hwnd);
                            println!("Handle found, setting focus with new handle");
                            if let Err(e) = handles::set_focus(hwnd) {
                                println!("Failed to set focus: {}", e);
                            }
                        } else {
                            println!("Starting application");
                            std::process::Command::new(&app.executablepath)
                            .spawn()
                            .expect("failed to start application");
                        }
                    } else {
                        println!("Setting focus for existing handle");
                        if let Err(e) = handles::set_focus(app.handle.unwrap()) {
                            println!("Failed to set focus: {}", e);
                        }
                    }
                }
            } else if let Some(app) = hotkeys.get_mut(&CTRL_G) {
                if app.handle.is_none() {
                    println!("Finding handle by title");
                    if let Ok(hwnd) = handles::find_window(&app.windowname) {
                        app.handle = Some(hwnd);
                        println!("Handle found, setting focus with new handle");
                        if let Err(e) = handles::set_focus(hwnd) {
                            println!("Failed to set focus: {}", e);
                        }
                    } else {
                        println!("Starting application");
                        std::process::Command::new(&app.executablepath)
                        .spawn()
                        .expect("failed to start application");
                    }
                } else {
                    println!("Setting focus for existing handle");
                    if let Err(e) = handles::set_focus(app.handle.unwrap()) {
                        println!("Failed to set focus: {}", e);
                    }
                }
            }
        }

        UnregisterHotKey(std::ptr::null_mut(), 1);
    }
}

