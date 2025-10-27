use crate::handles;
use crate::Application;

use std::collections::HashMap;
use windows_sys::Win32::{
    UI::WindowsAndMessaging::*,
    UI::Input::KeyboardAndMouse::*,
};
use winapi::um::winuser::{SW_SHOWMAXIMIZED, SW_NORMAL};

pub const ALT_D: u32 = 1; // Discord
pub const ALT_G: u32 = 2; // GitHub Desktop
pub const ALT_1: u32 = 3; // VSCode - Remote LMS-Prod
pub const ALT_2: u32 = 4; // VSCode - Remote LMS-UAT
pub const ALT_T: u32 = 5; // Terminal
pub const CTRL_T: u32 = 6; // Terminal - Admin

pub fn get_hotkeys() -> (HashMap<u32, Application>, Vec<crate::slint_generatedAppWindow::Hotkey>) {
    let mut hotkeys: HashMap<u32, Application> = HashMap::new();
    let mut slint_hotkeys: Vec<crate::slint_generatedAppWindow::Hotkey> = Vec::new();

    let app = Application { // Discord
        label: "Discord".to_string(),
        windowname: " - Discord".to_string(),
        executablepath: "C:\\Users\\Ben\\AppData\\Local\\Discord\\app-1.0.9212\\Discord.exe".to_string(),
        handle: None,
        attributes: ["Alt + D".to_string(), String::new()],
        default_state: SW_SHOWMAXIMIZED
    };
    hotkeys.insert(ALT_D, app);
    slint_hotkeys.push(crate::slint_generatedAppWindow::Hotkey {
        name: "Discord".into(),
        keybind: "Alt + D".into(),
    });

    let app = Application { // GitHub Desktop
        label: "Github Desktop".to_string(),
        windowname: "Github Desktop".to_string(),
        executablepath: "C:\\Users\\Ben\\AppData\\Local\\GitHubDesktop\\GitHubDesktop.exe".to_string(),
        handle: None,
        attributes: ["Alt + G".to_string(), String::new()],
        default_state: SW_SHOWMAXIMIZED
    };
    hotkeys.insert(ALT_G, app); 
    slint_hotkeys.push(crate::slint_generatedAppWindow::Hotkey {
        name: "Github Desktop".into(),
        keybind: "Alt + G".into(),
    });

    let app = Application { // LMS SSH Environments
        label: "LMS (Production)".to_string(),
        windowname: "- Visual Studio Code".to_string(),
        executablepath: "code --remote ssh-remote+prod@52.65.216.70 /home/prod".to_string(),
        handle: None,
        attributes: ["Alt + 1".to_string(), String::new()],
        default_state: SW_SHOWMAXIMIZED
    };
    hotkeys.insert(ALT_1, app);
    slint_hotkeys.push(crate::slint_generatedAppWindow::Hotkey {
        name: "LMS (Production)".into(),
        keybind: "Alt + 1".into(),
    });

    let app = Application {
        label: "LMS (UAT)".to_string(),
        windowname: "- Visual Studio Code".to_string(),
        executablepath: "code --remote ssh-remote+uat@52.65.141.202 /home/uat".to_string(),
        handle: None,
        attributes: ["Alt + 2".to_string(), String::new()],
        default_state: SW_SHOWMAXIMIZED
    };
    hotkeys.insert(ALT_2, app);
    slint_hotkeys.push(crate::slint_generatedAppWindow::Hotkey {
        name: "LMS (UAT)".into(),
        keybind: "Alt + 2".into(),
    });

    let app = Application { // Terminal - Powershell
        label: "Terminal".to_string(),  
        windowname: "Powershell".to_string(),
        executablepath: "wt".to_string(),
        handle: None,
        attributes: ["Alt + T".to_string(), String::new()],
        default_state: SW_NORMAL
    };
    hotkeys.insert(ALT_T, app);
    slint_hotkeys.push(crate::slint_generatedAppWindow::Hotkey {
        name: "Terminal".into(),
        keybind: "Alt + T".into(),
    });

    let app = Application {
        label: "Terminal - Admin".to_string(),
        windowname: "Administrator: Powershell".to_string(),
        executablepath: "Start-Process wt -Verb RunAs".to_string(),
        handle: None,
        attributes: ["Ctrl + T".to_string(), String::new()],
        default_state: SW_NORMAL
    };
    hotkeys.insert(CTRL_T, app);
    slint_hotkeys.push(crate::slint_generatedAppWindow::Hotkey {
        name: "Terminal - Admin".into(),
        keybind: "Ctrl + T".into(),
    });

    (hotkeys, slint_hotkeys)
}


pub fn spawn_hotkeys_thread(mut hotkeys: std::collections::HashMap<u32, crate::Application>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        unsafe {
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
            let ok = RegisterHotKey(std::ptr::null_mut(), ALT_D as i32, MOD_ALT, 'D' as u32);
            if ok == 0 { println!("Failed to register hotkey ALT_D"); }
            let ok = RegisterHotKey(std::ptr::null_mut(), ALT_G as i32, MOD_ALT, 'G' as u32);
            if ok == 0 { println!("Failed to register hotkey ALT_G"); }
            let ok = RegisterHotKey(std::ptr::null_mut(), ALT_1 as i32, MOD_ALT, '1' as u32);
            if ok == 0 { println!("Failed to register hotkey ALT_1"); }
            let ok = RegisterHotKey(std::ptr::null_mut(), ALT_2 as i32, MOD_ALT, '2' as u32);
            if ok == 0 { println!("Failed to register hotkey ALT_2"); }
            let ok = RegisterHotKey(std::ptr::null_mut(), ALT_T as i32, MOD_ALT, 'T' as u32);
            if ok == 0 { println!("Failed to register hotkey ALT_T"); }
            let ok = RegisterHotKey(std::ptr::null_mut(), CTRL_T as i32, MOD_CONTROL, 'T' as u32);
            if ok == 0 { println!("Failed to register hotkey CTRL_T"); }
        }

        run_hotkeys(&mut hotkeys);
    })
}
fn run_hotkeys(hotkeys: &mut HashMap<u32, Application>) {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_HOTKEY {
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    if msg.message == WM_HOTKEY {
                        handle_hotkey(hotkeys, msg.wParam as u32);
                    }
                }
            }
        }

        for (id, _) in hotkeys.iter_mut() {
            UnregisterHotKey(std::ptr::null_mut(), *id as i32);
        }
    }
}

fn handle_hotkey(hotkeys: &mut HashMap<u32, Application>, id: u32) {
    if let Some(app) = hotkeys.get_mut(&id) {
        if app.handle.is_none() {
            if let Ok(hwnd) = handles::find_window(&app.windowname) {
                app.handle = Some(hwnd as isize);
                let _ = handles::set_focus(hwnd, app.default_state);
            } else {
                if (app.executablepath.contains("--remote") || app.executablepath.contains("Start-Process") || app.executablepath.contains("wt")) && !app.executablepath.contains(".exe") { // PATH
                    let _ = std::process::Command::new("powershell").arg(&app.executablepath).spawn().expect("failed to start application");
                    println!("Running command... {}", app.executablepath);
                } else {
                    let _ = std::process::Command::new(&app.executablepath).spawn(); // Executable
                }
            }
        } else if let Err(_) = handles::set_focus(app.handle.unwrap() as winapi::shared::windef::HWND, app.default_state) {
            if let Ok(hwnd) = handles::find_window(&app.windowname) {
                app.handle = Some(hwnd as isize);
                let _ = handles::set_focus(hwnd, app.default_state);
            } else {
                if app.executablepath.contains("--remote") && !app.executablepath.contains(".exe") {
                    let _ = std::process::Command::new("powershell").arg(&app.executablepath).spawn().expect("failed to start application");
                    println!("Running command... {}", app.executablepath);
                } else {
                    let _ = std::process::Command::new(&app.executablepath).spawn();
                }
            }
        }
    }
}