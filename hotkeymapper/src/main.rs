mod windows;
mod keybinds;

use std::error::Error;
use std::path::Path;
use std::env;

use eframe::egui;
use regex::Regex;
use winapi::shared::windef::HWND;
use serde::{Deserialize, Serialize};

struct App {
    applications: Vec<Application>,
    selected_application: Option<String>,
    config_path: String,

    // Keybinds
    active_keybinds: Vec<[String; 4]>, // [ApplicationName, ExecutablePath+Arguments, Modifier, Key]
    keybind_selected: String,   
    keybind_selected_modifier: String,
    keybind_selected_key: String,

    // Change Keybinds
    show_keybind_modal: bool,
    editing_app_name: String,

    regex: Vec<Regex>,
}

struct Application {
    handle: HWND,
    label: String,
    path: String,
}

struct Keybind {
    application: Application,
    modifier: String,
    modifiercode: i32,
    key: String,
    keycode: i32,
}
#[derive(Serialize, Deserialize)]
struct ConfigKeybind {
    application_label: String,
    application_path: String,
    modifier: String,
    key: String,
}

// Initialise State
impl App {
    fn new() -> Self {
        let mut app = Self {
            applications: Vec::new(),
            selected_application: None,
            config_path: String::new(),
            
            active_keybinds: Vec::new(),
            keybind_selected: String::new(),
            keybind_selected_modifier: String::new(),
            keybind_selected_key: String::new(),

            show_keybind_modal: false,
            editing_app_name: String::new(),

            regex: Vec::new(),
        };


        /* Configuration File */

        // Retrieve the File
        app.config_path = match env::var("LOCALAPPDATA") {
            Ok(local_path) => {

                // Retrieve the Folder
                let local_path = format!("{}\\Nancy Configuration", local_path.as_str());
                let dir_path = Path::new(&local_path);
                if !dir_path.exists() || !dir_path.is_dir() {
                    create_config_path(&local_path).unwrap();
                    local_path.to_string();
                }

                // Retrieve the File
                let file_path_string = format!("{}\\config.json", local_path);
                let file_path = Path::new(&file_path_string);
                if !file_path.exists() || !file_path.is_file() {
                    create_config_file(&file_path_string).unwrap();
                    file_path_string
                }
                else { file_path_string }
                
            }
            Err(error) => {
                panic!("Unable to locate and create the configuration path. Error: {}", error);
            }
        };

        // Retrieve the Keybinds
        app.active_keybinds = load_config(&app.config_path).unwrap_or_default();

        /* End Configuration File */


        // Regex
        app.regex.push(Regex::new(r"^(.*?) - (.*?) - Microsoft​ Edge$").unwrap());
        app.regex.push(Regex::new(r"^(.*?) - File Explorer$").unwrap());
        app.regex.push(Regex::new(r"^.*\\(.*?) - File Explorer$").unwrap());
        app.regex.push(Regex::new(r"^(.*?) - Discord$").unwrap());

        // Preload Application List
        if std::env::consts::OS == "windows" {
            if let Ok(applications) = windows::list_visible_windows(&app.regex) {
                app.applications = applications.iter().map(|(handle, title, path)| Application { handle: *handle, label: title.to_string(), path: path.to_string() }).collect();
                for application in &app.applications {
                    if let Some(keybind) = app.active_keybinds.iter().find(|keybind| keybind[0] == application.label) {
                        let modifiercode = keybinds::get_key_code(&keybind[2]);
                        let keycode = keybinds::get_key_code(&keybind[3]);
                        app.active_keybinds.push([application.label.clone(), application.path.clone(), keybind[2].clone(), keybind[3].clone(), modifiercode, keycode]);
                    }
                }w
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

                columns[0].heading("Active Windows");

                // Refresh Button
                egui::Frame::new().show(&mut columns[1], |ui| {
                    if ui.add_sized([100.0, 20.0], egui::Button::new("Refresh")).clicked() {
                        if std::env::consts::OS == "windows" {
                            match windows::list_visible_windows(&self.regex) {
                                Ok(applications) => {
                                    self.applications = applications.iter()
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

                    // List of Active Windows
                    for window in &self.applications {
                        let is_selected = self.selected_application.as_ref() == Some(&window.label);
                        let button = egui::Button::new(window.label.to_string())
                            .fill(
                                if is_selected {
                                    egui::Color32::GRAY
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
                                self.selected_application = Some(window.path.to_string());
                            }
                        }
                    }
                });

                /* Right Column */
                egui::ScrollArea::vertical().id_salt("active_keybinds").show(&mut columns[1], |ui| {

                    let mut refresh_windows = false;
                    let mut refresh_keybinds = false;

                    // List of Associated Keybinds
                    for application in &self.applications {
                        
                        // Keybind Found
                        if let Some(keybind) = self.active_keybinds.iter().find(|keybind| keybind[0] == application.label) {
                            
                            let keybind_text = if keybind[2].is_empty() { format!("{}", keybind[3]) }               // Character Only
                            else { format!("{} + {}", keybind[2], keybind[3]) };                                            // Modifier + Character

                            // Change Button
                            let button = egui::Button::new(&keybind_text);
                            let new_button = ui.add_sized([20.0, 20.0], button.fill(egui::Color32::TRANSPARENT));
                            if new_button.clicked() {
                                self.show_keybind_modal = true;
                                self.editing_app_name = application.label.clone();
                                self.keybind_selected = keybind[1].to_string();
                                self.keybind_selected_modifier = keybind[2].to_string();
                                self.keybind_selected_key = keybind[3].to_string();
                                refresh_windows = true;
                                refresh_keybinds = true;
                            }
                        }

                        // No Keybind Found
                        else {
                            let button = egui::Button::new("~");
                            let new_button = ui.add_sized([20.0, 20.0], button.fill(egui::Color32::TRANSPARENT));
                            if new_button.clicked() {
                                self.show_keybind_modal = true;
                                self.editing_app_name = application.label.clone();
                                refresh_windows = true;
                                refresh_keybinds = true;
                            }
                        }
                    }

                    // Handle Window Refresh
                    if refresh_windows {
                        if std::env::consts::OS == "windows" {
                            match windows::list_visible_windows(&self.regex) {
                                Ok(applications) => {
                                    self.applications = applications.iter()
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
                    // Handle Keybind Refresh
                    if refresh_keybinds {
                        for application in &self.applications {
                            if let Some(keybind) = self.active_keybinds.iter_mut().find(|keybind| keybind[0] == application.label) {
                                keybind[1] = application.path.clone();
                                keybind[2] = "".to_string();
                                keybind[3] = "".to_string();
                            }
                        }
                        save_config(&self.config_path, &self.active_keybinds).unwrap();
                    }
                });
            });
        });

        // Keybind editing modal
        if self.show_keybind_modal {
            egui::Window::new("Edit Keybind")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(format!("Editing keybind for: {}", self.editing_app_name));
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label("Modifier:");
                            egui::ComboBox::from_id_salt("modifier_combo")
                                .selected_text(&self.keybind_selected_modifier)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.keybind_selected_modifier, "".to_string(), "None");
                                    ui.selectable_value(&mut self.keybind_selected_modifier, "Ctrl".to_string(), "Ctrl");
                                    ui.selectable_value(&mut self.keybind_selected_modifier, "Alt".to_string(), "Alt");
                                    ui.selectable_value(&mut self.keybind_selected_modifier, "Shift".to_string(), "Shift");
                                    ui.selectable_value(&mut self.keybind_selected_modifier, "Win".to_string(), "Win");
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Key:");
                            ui.text_edit_singleline(&mut self.keybind_selected_key);
                        });

                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                if let Some(keybind) = self.active_keybinds.iter_mut().find(|kb| kb[0] == self.editing_app_name) {
                                    keybind[2] = self.keybind_selected_modifier.clone();
                                    keybind[3] = self.keybind_selected_key.clone();
                                    println!("Stored {} with the binding {} and modifier {}", keybind[0], keybind[3], keybind[2]);
                                    save_config(&self.config_path, &self.active_keybinds).unwrap();
                                }
                                else {
                                    self.active_keybinds.push([self.editing_app_name.clone(), "".to_string(), self.keybind_selected_modifier.clone(), self.keybind_selected_key.clone()]);
                                }
                                self.show_keybind_modal = false;
                            }

                            if ui.button("Cancel").clicked() {
                                self.show_keybind_modal = false;
                            }
                        });
                    });
                }
            );
        }
    }
}

/* Configuration File */
fn create_config_path(config_path: &str) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(config_path)?;
    println!("Created the Configuration Folder Path");
    Ok(())
}
fn create_config_file(config_file_path: &str) -> Result<(), Box<dyn Error>> {
    std::fs::write(config_file_path, "")?;
    println!("Created the Configuration File Path");
    Ok(())
}
fn load_config(config_path: &str) -> Result<Vec<[String; 4]>, Box<dyn Error>> {
    let file_contents = std::fs::read_to_string(config_path)?;
    if file_contents.is_empty() { return Ok(Vec::new()); }

    let config: Vec<ConfigKeybind> = serde_json::from_str(&file_contents)?;
    let keybinds: Vec<[String; 4]> = config.iter().map(|keybind| [
        keybind.application_label.clone(), 
        keybind.application_path.clone(), 
        keybind.modifier.clone(), 
        keybind.key.clone()
    ]).collect();

    println!("Loaded the Configuration Keybinds");

    Ok(keybinds)
}
fn save_config(config_path: &str, keybinds: &Vec<[String; 4]>) -> Result<(), Box<dyn Error>> {
    let config: Vec<ConfigKeybind> = keybinds.iter().map(|keybind| ConfigKeybind {
        application_label: keybind[0].clone(),
        application_path: keybind[1].clone(),
        modifier: keybind[2].clone(),
        key: keybind[3].clone()
    }).collect();

    let encoded = serde_json::to_string(&config)?;
    std::fs::write(config_path, encoded)?;

    println!("Saved Current Keybinds to Configuration");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Workstation Hotkey Manager",
        options,
        Box::new(|_cc| Ok(Box::new(App::new())))
    );
    
    Ok(())
}