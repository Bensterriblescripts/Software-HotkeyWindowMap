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
        const CTRL_D: u32 = 1; // Discord
        const CTRL_G: u32 = 2; // Github Desktop
        const CTRL_1: u32 = 3; // VSCode - Remote LMS-Prod
        const CTRL_2: u32 = 4; // VSCode - Remote LMS-UAT
        let ok: i32 = { RegisterHotKey(std::ptr::null_mut(), CTRL_D as i32, MOD_CONTROL, 'D' as u32) }; if ok == 0 {println!("Failed to register hotkey CTRL_D");}
        let ok: i32 = { RegisterHotKey(std::ptr::null_mut(), CTRL_G as i32, MOD_CONTROL, 'G' as u32) }; if ok == 0 {println!("Failed to register hotkey CTRL_G");}
        let ok: i32 = { RegisterHotKey(std::ptr::null_mut(), CTRL_1 as i32, MOD_CONTROL, '1' as u32) }; if ok == 0 {println!("Failed to register hotkey CTRL_1");}
        let ok: i32 = { RegisterHotKey(std::ptr::null_mut(), CTRL_2 as i32, MOD_CONTROL, '2' as u32) }; if ok == 0 {println!("Failed to register hotkey CTRL_2");}

        /* Applications */
        let mut hotkey: HashMap<u32, Application> = HashMap::new();

        // Discord
        let app = Application {
            label: "Discord".to_string(),
            windowname: " - Discord".to_string(),
            executablepath: "C:\\Users\\Ben\\AppData\\Local\\Discord\\app-1.0.9212\\Discord.exe".to_string(),
            handle: None
        };
        hotkey.insert(CTRL_D, app);

        // Github Desktop
        let app = Application {
            label: "Github Desktop".to_string(),
            windowname: "Github Desktop".to_string(),
            executablepath: "C:\\Users\\Ben\\AppData\\Local\\GitHubDesktop\\GitHubDesktop.exe".to_string(),
            handle: None
        };
        hotkey.insert(CTRL_G, app); 

        // VSCode - Remote LMS-Prod
        let app = Application {
            label: "VSCode".to_string(),
            windowname: "- Visual Studio Code".to_string(),
            executablepath: "code --remote ssh-remote+prod@52.65.216.70 /home/prod".to_string(),
            handle: None
        };
        hotkey.insert(CTRL_1, app);
        // VSCode - Remote LMS-UAT
        let app = Application {
            label: "VSCode".to_string(),
            windowname: "- Visual Studio Code".to_string(),
            executablepath: "code --remote ssh-remote+uat@52.65.141.202 /home/uat".to_string(),
            handle: None
        };
        hotkey.insert(CTRL_2, app);


        /* Message Loop */
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_HOTKEY {
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    if msg.message == WM_HOTKEY {
                        handle_hotkey(&mut hotkey, msg.wParam as u32);
                    }
                }
            }
        }


        UnregisterHotKey(std::ptr::null_mut(), CTRL_D as i32);
        UnregisterHotKey(std::ptr::null_mut(), CTRL_G as i32);
        UnregisterHotKey(std::ptr::null_mut(), CTRL_1 as i32);
        UnregisterHotKey(std::ptr::null_mut(), CTRL_2 as i32);
    }
}
fn handle_hotkey(hotkeys: &mut HashMap<u32, Application>, id: u32) {
    if let Some(app) = hotkeys.get_mut(&id) {
        if app.handle.is_none() {
            if let Ok(hwnd) = handles::find_window(&app.windowname) {
                app.handle = Some(hwnd);
                let _ = handles::set_focus(hwnd);
            } else {
                if app.executablepath.contains("--remote") && !app.executablepath.contains(".exe") { // PATH
                    let _ = std::process::Command::new("powershell").arg(&app.executablepath).spawn().expect("failed to start application");
                    println!("Running command... {}", app.executablepath);
                } else {
                    let _ = std::process::Command::new(&app.executablepath).spawn(); // Executable
                }
            }
        } else if let Err(_) = handles::set_focus(app.handle.unwrap()) {
            if let Ok(hwnd) = handles::find_window(&app.windowname) {
                app.handle = Some(hwnd);
                let _ = handles::set_focus(hwnd);
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

