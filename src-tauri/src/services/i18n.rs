use std::collections::HashMap;
use std::path::PathBuf;
use serde_json::Value;
use crate::services::registry::dirs_roaming_data;

pub struct I18nService {
 strings: HashMap<String, String>,
 pub current_language: String,
 settings_path: PathBuf,
}

impl I18nService {
 pub fn new() -> Self {
 let folder = dirs_roaming_data().join("CapivaraBooster");
 let _ = std::fs::create_dir_all(&folder);
 let settings_path = folder.join("settings.json");
 let mut svc = Self {
 strings: HashMap::new(),
 current_language: String::new(),
 settings_path,
 };
 let initial = svc.detect_initial_language();
 svc.set_language(&initial);
 svc
 }

 pub fn set_language(&mut self, lang: &str) {
 let target = if lang.to_lowercase().starts_with("pt") { "pt-BR" } else { "en-US" };
 self.current_language = target.to_string();
 self.load_dictionary(target);
 self.save_settings();
 let _ = lang;
 }

 pub fn get(&self, key: &str) -> String {
 self.strings.get(key).cloned().unwrap_or_else(|| format!("[{key}]"))
 }

 pub fn is_portuguese(&self) -> bool { self.current_language == "pt-BR" }

 fn detect_initial_language(&self) -> String {
 if let Ok(json) = std::fs::read_to_string(&self.settings_path) {
 if let Ok(Value::Object(obj)) = serde_json::from_str(&json) {
 if let Some(Value::String(lang)) = obj.get("Language") {
 return lang.clone();
 }
 }
 }
 let ui = sys_locale_or_default();
 if ui.starts_with("pt") { "pt-BR".into() } else { "en-US".into() }
 }

 fn load_dictionary(&mut self, lang: &str) {
 // Tenta pasta do executável e pasta de desenvolvimento
 let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|x| x.to_path_buf()));
 let mut candidates: Vec<PathBuf> = Vec::new();
 if let Some(dir) = exe_dir {
 candidates.push(dir.join("localization").join(format!("{lang}.json")));
 candidates.push(dir.join("../../src-tauri/localization").join(format!("{lang}.json")));
 }
 candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("localization").join(format!("{lang}.json")));

 for candidate in candidates {
 if let Ok(json) = std::fs::read_to_string(&candidate) {
 if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(&json) {
 self.strings.clear();
 for (k, v) in obj {
 if let Value::String(s) = v {
 self.strings.insert(k, s);
 }
 }
 return;
 }
 }
 }
 }

 fn save_settings(&self) {
 let mut obj = serde_json::Map::new();
 obj.insert("Language".into(), serde_json::Value::String(self.current_language.clone()));
 if let Ok(json) = serde_json::to_string_pretty(&Value::Object(obj)) {
 let _ = std::fs::write(&self.settings_path, json);
 }
 }
}

fn sys_locale_or_default() -> String {
 std::env::var("LANG")
 .or_else(|_| std::env::var("LC_ALL"))
 .unwrap_or_else(|_| "en-US".into())
}