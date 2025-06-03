mod windows;
use std::error::Error;
use eframe::egui;
use regex::Regex;
use winapi::shared::windef::HWND;

struct App {
    applications: Vec<Application>,
    selected_application: Option<String>,

    keybinds: Vec<[&'static str; 4]>, // [ApplicationName, ExecutablePath+Arguments, Modifier, Key]
    keybind_selected: String,
    keybind_selected_modifier: String,
    keybind_selected_key: String,

    // Modal state
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
    application: String,
    executable: String,
    modifier: String,
    key: String,
}

// Initialise State
impl App {
    fn new() -> Self {
        let mut app = Self {
            applications: Vec::new(),
            selected_application: None,
            keybinds: Vec::new(),

            keybind_selected: String::new(),
            keybind_selected_modifier: String::new(),
            keybind_selected_key: String::new(),

            // Modal state
            show_keybind_modal: false,
            editing_app_name: String::new(),

            regex: Vec::new(),
        };

        // Regex
        app.regex.push(Regex::new(r"^(.*?) - (.*?) - Microsoft​ Edge$").unwrap());
        app.regex.push(Regex::new(r"^(.*?) - File Explorer$").unwrap());
        app.regex.push(Regex::new(r"^.*\\(.*?) - File Explorer$").unwrap());
        app.regex.push(Regex::new(r"^(.*?) - Discord$").unwrap());

        // Preload Application List
        if std::env::consts::OS == "windows" {
            if let Ok(applications) = windows::list_visible_windows(&app.regex) {
                app.applications = applications.iter().map(|(handle, title, path)| Application { handle: *handle, label: title.to_string(), path: path.to_string() }).collect();
            }
        }

        // Keybinds
        app.keybinds.push(["Cursor", "cursor.exe", "Ctrl", "U"]);

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

                    // List of Associated Keybinds
                    for application in &self.applications {
                        
                        // Keybind Found
                        if let Some(keybind) = self.keybinds.iter().find(|keybind| keybind[0] == &application.label) {
                            
                            let keybind_text = if keybind[2].is_empty() { format!("{}", keybind[3]) }       // Character Only
                            else { format!("{} + {}", keybind[2], keybind[3]) };                                    // Modifier + Character

                            // Change Button
                            let button = egui::Button::new(&keybind_text);
                            let new_button = ui.add_sized([20.0, 20.0], button.fill(egui::Color32::TRANSPARENT));
                            if new_button.clicked() {
                                self.show_keybind_modal = true;
                                self.editing_app_name = application.label.clone();
                                self.keybind_selected = keybind[1].to_string();
                                self.keybind_selected_modifier = keybind[2].to_string();
                                self.keybind_selected_key = keybind[3].to_string();
                            }
                        }

                        else {
                            let button = egui::Button::new("~");
                            ui.add_sized([20.0, 20.0], button.fill(egui::Color32::TRANSPARENT));
                        }
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
                                // Update the keybind
                                if let Some(keybind) = self.keybinds.iter_mut().find(|kb| kb[0] == self.editing_app_name) {
                                    keybind[2] = Box::leak(self.keybind_selected_modifier.clone().into_boxed_str());
                                    keybind[3] = Box::leak(self.keybind_selected_key.clone().into_boxed_str());
                                }
                                self.show_keybind_modal = false;
                            }

                            if ui.button("Cancel").clicked() {
                                self.show_keybind_modal = false;
                            }
                        });
                    });
                });
        }
    }
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