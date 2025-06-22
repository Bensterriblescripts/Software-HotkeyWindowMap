pub async fn create_config_path(config_path: &str) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(config_path)?;
    println!("Created the Configuration Folder Path");
    Ok(())
}
pub async fn create_config_file(config_file_path: &str) -> Result<(), Box<dyn Error>> {
    std::fs::write(config_file_path, "")?;
    println!("Created the Configuration File Path");
    Ok(())
}
pub async fn load_config(config_path: &str) -> Result<Vec<Keybind>, Box<dyn Error>> {
    let file_contents = std::fs::read_to_string(config_path)?;
    if file_contents.is_empty() { return Ok(Vec::new()); }


    println!("Loaded the Configuration Keybinds");

    Ok(keybinds)
}
pub async fn save_config(config_path: &str, keybinds: &Vec<[String; 4]>) -> Result<(), Box<dyn Error>> {
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