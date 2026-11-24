use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::{HKEY, RegKey};

use crate::models::RegistryValueBackup;

pub const HKCU: HKEY = HKEY_CURRENT_USER;
pub const HKLM: HKEY = HKEY_LOCAL_MACHINE;

pub struct RegistryService {
    tracked: Vec<RegistryValueBackup>,
    store_path: std::path::PathBuf,
}

impl RegistryService {
    pub fn new() -> Self {
        let base = dirs_common_data();
        let folder = base.join("CapivaraBooster");
        let _ = std::fs::create_dir_all(&folder);
        let store_path = folder.join("registry_rollback_store.json");
        let tracked = std::fs::read_to_string(&store_path)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();
        Self { tracked, store_path }
    }

    pub fn tracked_backups(&self) -> &[RegistryValueBackup] {
        &self.tracked
    }

    pub fn set_dword(&mut self, hive: HKEY, sub_key: &str, value_name: &str, value: u32) -> Result<(), String> {
        self.track_current_value(hive, sub_key, value_name)?;
        let base = RegKey::predef(hive);
        let key = base
            .create_subkey(sub_key)
            .map_err(|e| format!("abrir chave {sub_key}: {e}"))?
            .0;
        key.set_value(value_name, &value)
            .map_err(|e| format!("gravar {value_name}: {e}"))
    }

    pub fn set_string(&mut self, hive: HKEY, sub_key: &str, value_name: &str, value: &str) -> Result<(), String> {
        self.track_current_value(hive, sub_key, value_name)?;
        let base = RegKey::predef(hive);
        let key = base
            .create_subkey(sub_key)
            .map_err(|e| format!("abrir chave {sub_key}: {e}"))?
            .0;
        key.set_value(value_name, &value)
            .map_err(|e| format!("gravar {value_name}: {e}"))
    }

    pub fn delete_value(&mut self, hive: HKEY, sub_key: &str, value_name: &str) -> Result<bool, String> {
        self.track_current_value(hive, sub_key, value_name)?;
        let base = RegKey::predef(hive);
        let key = match base.open_subkey_with_flags(sub_key, winreg::enums::KEY_SET_VALUE) {
            Ok(k) => k,
            Err(_) => return Ok(false),
        };
        match key.delete_value(value_name) {
            Ok(()) => Ok(true),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(format!("apagar {value_name}: {e}")),
        }
    }

    pub fn get_string(&self, hive: HKEY, sub_key: &str, value_name: &str) -> Option<String> {
        let key = RegKey::predef(hive).open_subkey(sub_key).ok()?;
        key.get_value(value_name).ok()
    }

    pub fn get_dword(&self, hive: HKEY, sub_key: &str, value_name: &str) -> Option<u32> {
        let key = RegKey::predef(hive).open_subkey(sub_key).ok()?;
        key.get_value(value_name).ok()
    }

    pub fn revert_all_tracked(&mut self) -> Result<usize, String> {
        let list = self.tracked.clone();
        let mut restored = 0;
        for entry in list.iter().rev() {
            let hive = if entry.registry_hive.eq_ignore_ascii_case("LocalMachine") { HKLM } else { HKCU };
            let base = RegKey::predef(hive);
            match (entry.existed_before, entry.original_value.clone()) {
                (false, _) => {
                    if let Ok(key) = base.open_subkey_with_flags(&entry.sub_key, winreg::enums::KEY_SET_VALUE) {
                        let _ = key.delete_value(&entry.value_name);
                        restored += 1;
                    }
                }
                (true, Some(serde_json::Value::Number(n))) => {
                    if let Ok(key) = base.create_subkey(&entry.sub_key) {
                        let v = n.as_u64().unwrap_or(0) as u32;
                        let _ = key.0.set_value(&entry.value_name, &v);
                        restored += 1;
                    }
                }
                (true, Some(serde_json::Value::String(s))) => {
                    if let Ok(key) = base.create_subkey(&entry.sub_key) {
                        let _ = key.0.set_value(&entry.value_name, &s);
                        restored += 1;
                    }
                }
                _ => {}
            }
        }
        self.tracked.clear();
        self.save_tracked();
        Ok(restored)
    }

    fn track_current_value(&mut self, hive: HKEY, sub_key: &str, value_name: &str) -> Result<(), String> {
        let hive_name = if hive == HKLM { "LocalMachine" } else { "CurrentUser" };
        if self.tracked.iter().any(|t| {
            t.registry_hive.eq_ignore_ascii_case(hive_name)
                && t.sub_key.eq_ignore_ascii_case(sub_key)
                && t.value_name.eq_ignore_ascii_case(value_name)
        }) {
            return Ok(());
        }

        let mut entry = RegistryValueBackup {
            registry_hive: hive_name.to_string(),
            sub_key: sub_key.to_string(),
            value_name: value_name.to_string(),
            existed_before: false,
            original_value: None,
            value_kind: None,
        };

        if let Ok(key) = RegKey::predef(hive).open_subkey(sub_key) {
            if let Ok(v) = key.get_value::<u32, _>(value_name) {
                entry.existed_before = true;
                entry.original_value = Some(serde_json::Value::Number(v.into()));
                entry.value_kind = Some("DWord".into());
            } else if let Ok(v) = key.get_value::<String, _>(value_name) {
                entry.existed_before = true;
                entry.original_value = Some(serde_json::Value::String(v));
                entry.value_kind = Some("String".into());
            }
        }

        self.tracked.push(entry);
        self.save_tracked();
        Ok(())
    }

    fn save_tracked(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.tracked) {
            let _ = std::fs::write(&self.store_path, json);
        }
    }
}

/// ProgramData (CommonApplicationData) em Windows.
pub fn dirs_common_data() -> std::path::PathBuf {
    std::env::var("ProgramData")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("C:\\ProgramData"))
}

/// LocalApplicationData.
pub fn dirs_local_data() -> std::path::PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("C:\\Users\\Public\\AppData\\Local"))
}

/// Roaming AppData.
pub fn dirs_roaming_data() -> std::path::PathBuf {
    std::env::var("APPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("C:\\Users\\Public\\AppData\\Roaming"))
}
