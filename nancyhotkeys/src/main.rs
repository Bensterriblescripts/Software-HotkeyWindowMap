mod windows;

use eframe::egui;
use regex::Regex;
use winapi::shared::windef::HWND;
use winapi::um::winuser::*;

use std::error::Error;

struct App {
    active_keybinds: Vec<Keybind>,
    active_applications: Vec<Application>,

    show_keybindwindow: bool,
    update_keybind: Keybind,
    regex: Vec<Regex>,
}
#[derive(Debug, Clone, Default)]
struct Keybind {
    label: String,
    path: String,
    application_label: String,
    modifier: String,
    modifiercode: i32,
    key: String,
    keycode: i32,
}
struct Application {
    handle: HWND,
    label: String,
    path: String,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            active_keybinds: Vec::new(),
            active_applications: Vec::new(),

            show_keybindwindow: false,
            update_keybind: Keybind::default(),

            regex: Vec::new(),
        };

        // Regex
        app.regex.push(Regex::new(r"^(.*?) - (.*?) - Microsoft​ Edge$").unwrap());
        app.regex.push(Regex::new(r"^(.*?) - File Explorer$").unwrap());
        app.regex.push(Regex::new(r"^.*\\(.*?) - File Explorer$").unwrap());
        app.regex.push(Regex::new(r"^(.*?) - Discord$").unwrap());

        // Preload Applications
        if std::env::consts::OS == "windows" {
            if let Ok(applications) = windows::list_visible_windows(&app.regex) {
                app.active_applications = applications.iter().map(|(handle, title, path)| Application { handle: *handle, label: title.to_string(), path: path.to_string() }).collect();
                for application in &app.active_applications {
                    if let Some(keybind) = app.active_keybinds.iter().find(|keybind| keybind.label == application.label) {
                        let modifiercode = get_key_code(&keybind.modifier);
                        let keycode = get_key_code(&keybind.key);
                        app.active_keybinds.push(Keybind { label: application.label.clone(), path: application.path.clone(), application_label: application.label.clone(), modifier: keybind.modifier.clone(), modifiercode: modifiercode, key: keybind.key.clone(), keycode: keycode });
                    }
                }
            }
        }

        app
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.add_space(20.0);
            ui.columns(2, |columns| {

                /* Refresh Button */
                columns[0].heading("Active Windows");
                egui::Frame::new().show(&mut columns[1], |ui| {
                    if ui.add_sized([100.0, 20.0], egui::Button::new("Refresh")).clicked() {
                        if std::env::consts::OS == "windows" {
                            match windows::list_visible_windows(&self.regex) {
                                Ok(applications) => {
                                    self.active_applications = applications.iter()
                                        .filter_map(|(handle, title, path)| {
                                            if title.trim().is_empty() {
                                                None
                                            }
                                            else {
                                                Some(Application { handle: *handle, label: title.to_string(), path: path.to_string() })
                                            }
                                        }).collect();
                                }
                                Err(e) => {
                                    println!("Failed to refresh active windows. Error: {}", e);
                                }
                            }
                        }
                    }
                });

                columns[0].add_space(40.0);
                columns[0].label("Application");
                columns[1].add_space(40.0);
                columns[1].label("Keybind");

                /* Left Column */
                egui::ScrollArea::vertical().id_salt("active_windows").show(&mut columns[0], |ui| {

                    // Active Applications
                    for window in &self.active_applications {
                        let is_selected = self.update_keybind.label == window.label;
                        let button = egui::Button::new(window.label.to_string())
                            .fill(
                                if is_selected {
                                    egui::Color32::BLACK
                                } 
                                else {
                                    egui::Color32::TRANSPARENT
                                }
                            );
                        if ui.add_sized([20.0, 20.0], button).clicked() {
                            if std::env::consts::OS == "windows" {
                                if let Err(e) = windows::make_focus(window.handle) {
                                    println!("Failed to make {} the active window. Error: {}", window.path, e);
                                }
                                self.update_keybind.label = window.label.to_string();
                                self.update_keybind.path = window.path.to_string();
                            }
                        }
                    }
                });

                /* Right Column */
                egui::ScrollArea::vertical().id_salt("active_keybinds").show(&mut columns[1], |ui| {
                    for application in &self.active_applications {
                        if let Some(keybind) = self.active_keybinds.iter().find(|keybind| keybind.label == application.label) {
                            let buttonlabel = set_keybindbuttonlabel(&keybind.label, &keybind.key, &keybind.modifier);
                            if ui.add_sized([20.0, 20.0], egui::Button::new(buttonlabel).fill(egui::Color32::TRANSPARENT)).clicked() {
                                self.show_keybindwindow = true;
                            }
                        }
                        else {
                            let buttonlabel = set_keybindbuttonlabel(&"", &"~", &"");
                            if ui.add_sized([20.0, 20.0], egui::Button::new(buttonlabel).fill(egui::Color32::TRANSPARENT)).clicked() {
                                self.show_keybindwindow = true;
                            }
                        }
                    }
                });
            });
        });

        fn set_keybindbuttonlabel(label: &str, key: &str, modifier: &str) -> String {
            let buttonlabel: String = if modifier.is_empty() { key.to_string() }
            else { format!("{} + {}", modifier, key) };
            buttonlabel
        }
    }
}

fn get_key_code(key: &str) -> i32 {
    match key {
        "A" => 0x1E,
        "B" => 0x30,
        "C" => 0x2E,
        "CTRL"  => VK_CONTROL,
        "ALT" => VK_MENU,
        "SHIFT" => VK_SHIFT,
        "WIN" => VK_LWIN,
        _ => 0x00,
    }
}
fn get_code_key(code: i32) -> String {
    match code {
        VK_CONTROL => "CTRL".to_string(),
        VK_MENU => "ALT".to_string(),
        VK_SHIFT => "SHIFT".to_string(),
        VK_LWIN => "WIN".to_string(),
        _ => "~".to_string(),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Nancy Hotkey Manager",
        options,
        Box::new(|_cc| Ok(Box::new(App::new())))
    );
    
    Ok(())
}