use serde::{Serialize, Deserialize};

pub mod dashboard;
pub use dashboard::*;

#[derive(Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_version: String,
    pub hpet_status: String,
    pub power_plan: String,
}

#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    Ok(SystemInfo {
        os_version: "Windows 10/11".to_string(),
        hpet_status: "Detecting...".to_string(),
        power_plan: "Balanced".to_string(),
    })
}

#[tauri::command]
pub fn toggle_console() {
    println!("Console visibility toggled");
}

#[tauri::command]
pub fn change_language(lang: String) {
    println!("Language changed to: {}", lang);
}