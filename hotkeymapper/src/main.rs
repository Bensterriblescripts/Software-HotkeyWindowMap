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

    regex: Vec<Regex>,
}
struct Application {
    handle: HWND,
    label: String,
    path: String,
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

            regex: Vec::new(),
        };

        // Regex
        app.regex.push(Regex::new(r"^(.*?) - (.*?) - Microsoft​ Edge$").unwrap());
        app.regex.push(Regex::new(r"^(.*?) - File Explorer$").unwrap());

        // Preload Application List
        if std::env::consts::OS == "windows" {
            if let Ok(applications) = windows::list_visible_windows(&app.regex) {
                app.applications = applications.iter().map(|(handle, title, path)| Application { handle: *handle, label: title.to_string(), path: path.to_string() }).collect();
            }
        }

        // Keybinds
        app.keybinds.push(["main.rs - hotkeymapper - Cursor", "cursor.exe", "", "U"]);

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
                                            if path.trim().is_empty() {
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
                        
                        // Key Bound to Application (By Title)
                        if let Some(keybind) = self.keybinds.iter().find(|keybind| keybind[0] == &application.label) {
                            if keybind[0] == self.keybind_selected {
                                println!("Clicked on the selected keybind");
                            }
                            
                            let keybind_text = if keybind[2].is_empty() {
                                format!("{}", keybind[3])
                            } 
                            else {
                                format!("{}+{}", keybind[2], keybind[3])
                            };
                            let button = egui::Button::new(&keybind_text);
                            let new_button = ui.add_sized([20.0, 20.0], button.fill(egui::Color32::TRANSPARENT));
                            if new_button.clicked() {
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
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 600.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Workstation Hotkey Manager",
        options,
        Box::new(|_cc| Ok(Box::new(App::new())))
    );
    
    Ok(())
}