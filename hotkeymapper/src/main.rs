mod windows;
use std::error::Error;
use eframe::egui;
use eframe::egui::style::Visuals;

struct App {
    applications: Vec<String>,
    selected_application: Option<String>,
    keybinds: Vec<[String; 4]>, // [Application, Exe+Arguments, Modifier, Key]
}

// Initialise State
impl App {
    fn new() -> Self {
        let mut app = Self {
            applications: Vec::new(),
            selected_application: None,
            keybinds: Vec::new(),
        };

        if std::env::consts::OS == "windows" {
            if let Ok(applications) = windows::list_visible_windows() {
                app.applications = applications.iter().map(|(_, title)| title.clone()).collect();
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
                columns[0].add_space(10.0);

                /* Left Column */
                columns[0].label("Application");
                egui::ScrollArea::vertical().id_salt("active_windows").show(&mut columns[0], |ui| {
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
                        
                        if ui.add(button).clicked() {
                            if std::env::consts::OS == "windows" {
                                if let Err(e) = windows::make_focus(window) {
                                    println!("Failed to make {} the active window. Error: {}", window, e);
                                }
                                self.selected_application = Some(window.to_string());
                            }
                        }
                    }
                });

                /* Right column */

                // Refresh Button
                if columns[1].add_sized([20.0, 20.0], egui::Button::new("Refresh")).clicked() {
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

                columns[1].add_space(10.0);

                // Keybinds
                columns[1].label("Keybind");
                egui::ScrollArea::vertical().id_salt("active_keybinds").show(&mut columns[1], |ui| {
                });
            });
        });
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Workstation Hotkey Manager",
        options,
        Box::new(|_cc| Ok(Box::new(App::new())))
    );
    
    Ok(())
}