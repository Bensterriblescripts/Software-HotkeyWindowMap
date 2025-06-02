mod windows;
use std::error::Error;
use eframe::egui;

struct App {
    applications: Vec<String>,
    selected_application: Option<String>,
    keybinds: Vec<[&'static str; 4]>, // [ApplicationName, ExecutablePath+Arguments, Modifier, Key]
}

// Initialise State
impl App {
    fn new() -> Self {
        let mut app = Self {
            applications: Vec::new(),
            selected_application: None,
            keybinds: Vec::new(),
        };

        // Preload Application List
        if std::env::consts::OS == "windows" {
            if let Ok(applications) = windows::list_visible_windows() {
                app.applications = applications.iter().map(|(_, title)| title.clone()).collect();
            }
        }

        // Keybinds
        app.keybinds.push(["main.rs - hotkeymapper - Visual Studio Code", "cursor.exe", "", "U"]);

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
                            match windows::list_visible_windows() {
                                Ok(applications) => {
                                    self.applications = applications.iter()
                                        .filter_map(|(_, title)| {
                                            if title.trim().is_empty() {
                                                None
                                            }
                                            else {
                                                Some(title.clone())
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

                columns[0].add_space(20.0);
                columns[0].label("Application");
                
                columns[1].add_space(20.0);
                columns[1].label("Keybind");

                /* Left Column */
                egui::ScrollArea::vertical().id_salt("active_windows").show(&mut columns[0], |ui| {

                    // List of Active Windows
                    for window in &self.applications {
                        let is_selected = self.selected_application.as_ref() == Some(window);
                        let button = egui::Button::new(window)
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
                                if let Err(e) = windows::make_focus(window) {
                                    println!("Failed to make {} the active window. Error: {}", window, e);
                                }
                                self.selected_application = Some(window.to_string());
                            }
                        }
                    }
                });

                /* Right Column */
                egui::ScrollArea::vertical().id_salt("active_keybinds").show(&mut columns[1], |ui| {

                    // List of Associated Keybinds
                    for application in &self.applications {
                        if let Some(keybind) = self.keybinds.iter().find(|keybind| keybind[0] == application) {
                            if let Some(_key) = keybind.iter().find(|key| *key == application) {
                                if keybind[2].is_empty() {
                                    ui.add_sized([20.0, 20.0], egui::Button::new(keybind[3]).fill(egui::Color32::TRANSPARENT));
                                }
                                else {
                                    ui.add_sized([20.0, 20.0], egui::Button::new(keybind[2].to_string() + " + " + keybind[3]).fill(egui::Color32::TRANSPARENT));
                                }
                            }
                            else {
                                ui.add_sized([20.0, 20.0], egui::Button::new("~").fill(egui::Color32::TRANSPARENT));
                            }
                        }
                        else {
                            ui.add_sized([20.0, 20.0], egui::Button::new("~").fill(egui::Color32::TRANSPARENT));
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