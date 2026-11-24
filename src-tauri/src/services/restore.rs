use std::path::PathBuf;
use chrono::Utc;
use crate::models::BackupRecord;
use crate::services::registry::dirs_common_data;

pub struct RestoreService {
 backup_root: PathBuf,
 backups: Vec<BackupRecord>,
}

impl RestoreService {
 pub fn new() -> Self {
 let root = dirs_common_data().join("CapivaraBooster").join("Backups");
 let _ = std::fs::create_dir_all(&root);
 let manifest = root.join("backups_manifest.json");
 let backups = std::fs::read_to_string(&manifest)
 .ok()
 .and_then(|json| serde_json::from_str(&json).ok())
 .unwrap_or_default();
 Self { backup_root: root, backups }
 }

 pub fn get_backups(&self) -> &[BackupRecord] { &self.backups }

 pub fn backup_file(&mut self, source: &str, category: &str, description: &str) -> Result<Option<BackupRecord>, String> {
 let src = PathBuf::from(source);
 if !src.exists() { return Ok(None); }

 let file_name = src.file_name().and_then(|n| n.to_str()).unwrap_or("file");
 let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
 let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("");
 let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
 let dest_name = format!("{stem}_{timestamp}.{ext}");

 let category_dir = self.backup_root.join(category);
 std::fs::create_dir_all(&category_dir).map_err(|e| e.to_string())?;
 let dest = category_dir.join(&dest_name);
 std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;

 let record = BackupRecord {
 id: uuid_v4_simple(),
 name: file_name.into(),
 category: category.into(),
 original_path: source.into(),
 backup_path: dest.display().to_string(),
 description: description.into(),
 created_at: Utc::now().to_rfc3339(),
 };

 self.backups.insert(0, record.clone());
 self.save_manifest();
 Ok(Some(record))
 }

 pub fn restore_file(&mut self, backup_id: &str) -> Result<bool, String> {
 let record = match self.backups.iter().find(|b| b.id == backup_id) {
 Some(r) => r.clone(),
 None => return Err(format!("backup {backup_id} não encontrado")),
 };
 let dest = PathBuf::from(&record.original_path);
 if let Some(parent) = dest.parent() {
 let _ = std::fs::create_dir_all(parent);
 }
 std::fs::copy(&record.backup_path, &dest).map_err(|e| e.to_string())?;
 Ok(true)
 }

 fn save_manifest(&self) {
 let manifest = self.backup_root.join("backups_manifest.json");
 if let Ok(json) = serde_json::to_string_pretty(&self.backups) {
 let _ = std::fs::write(manifest, json);
 }
 }
}

fn uuid_v4_simple() -> String {
 use sha2::{Digest, Sha256};
 let mut hasher = Sha256::new();
 hasher.update(format!("{:?}|{}", std::time::SystemTime::now(), Utc::now().to_rfc3339()));
 let digest = hasher.finalize();
 digest.iter().map(|b| format!("{b:02x}")).collect::<String>()[..16].to_string()
}