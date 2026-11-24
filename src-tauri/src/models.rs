use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RegistryValueBackup {
    pub registry_hive: String,
    pub sub_key: String,
    pub value_name: String,
    pub existed_before: bool,
    #[serde(default)]
    pub original_value: Option<serde_json::Value>,
    #[serde(default)]
    pub value_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BackupRecord {
    pub id: String,
    pub name: String,
    pub category: String,
    pub original_path: String,
    pub backup_path: String,
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleaningSummary {
    pub total_bytes_cleaned: u64,
    pub files_deleted: u32,
    pub errors_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanerItem {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub target_path: String,
    pub size_bytes: u64,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BloatwareItem {
    pub id: String,
    pub name: String,
    pub package_pattern: String,
    pub description: String,
    pub category: String,
    pub is_selected: bool,
    pub is_removed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartupItem {
    pub name: String,
    pub path: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameDetectionResult {
    pub game_id: String,
    pub found: bool,
    pub executable_path: String,
    pub install_directory: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemTweakItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub is_applied: bool,
}
