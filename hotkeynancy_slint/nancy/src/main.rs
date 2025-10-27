#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod handles;
mod hotkeys;
mod borderless;

use std::error::Error;
use std::collections::HashMap;

slint::include_modules!();

#[derive(Clone)]
struct Application {
    label: String,
    windowname: String,
    executablepath: String,
    handle: Option<isize>,
    attributes: [String; 2], // [0] = hotkey, [1] = borderless
    default_state: i32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let (hotkeys, slint_hotkeys): (HashMap<u32, Application>, Vec<Hotkey>) = hotkeys::get_hotkeys();
    let (borderless_applications, slint_borderless_applications): (HashMap<u32, Application>, Vec<BorderlessApplication>) = borderless::get_borderless_applications();

    ui.set_hotkeys(slint::ModelRc::new(slint::VecModel::from(slint_hotkeys)));
    ui.set_borderless_applications(slint::ModelRc::new(slint::VecModel::from(slint_borderless_applications)));

    hotkeys::spawn_hotkeys_thread(hotkeys);
    // borderless: spawn_borderless_thread(borderless_applications);

    ui.run()?;

    Ok(())
}
