use crate::Application;
use std::collections::HashMap;
use winapi::um::winuser::SW_SHOWMAXIMIZED;

pub fn get_borderless_applications() -> (HashMap<u32, Application>, Vec<crate::slint_generatedAppWindow::BorderlessApplication>) {
    let mut borderless_applications: HashMap<u32, Application> = HashMap::new();
    let mut slint_borderless_applications: Vec<crate::slint_generatedAppWindow::BorderlessApplication> = Vec::new();
    let app = Application {
        label: "Dark Souls 2".to_string(),
        windowname: "Dark Souls 2".to_string(),
        executablepath: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Dark Souls II\\DarkSoulsII.exe".to_string(),
        handle: None,
        attributes: [String::new(), "Always".to_string()],
        default_state: SW_SHOWMAXIMIZED
    };
    borderless_applications.insert(1, app);
    slint_borderless_applications.push(crate::slint_generatedAppWindow::BorderlessApplication {
        name: "Dark Souls 2".into(),
        borderless_type: "Always".into(),
    });
    (borderless_applications, slint_borderless_applications)
}
