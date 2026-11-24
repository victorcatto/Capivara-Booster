use std::io;
use std::path::PathBuf;
use crate::models::CleanerItem;
use crate::services::registry::dirs_local_data;

pub struct CleanerService;

impl CleanerService {
 pub fn new() -> Self { Self }

 pub fn get_cleaner_items(&self) -> Vec<CleanerItem> {
 let local = dirs_local_data();
 vec![
 CleanerItem {
 id: "temp".into(),
 display_name: "User Temp Files (%TEMP%)".into(),
 description: "Temporary files created by applications and installers".into(),
 target_path: std::env::temp_dir().display().to_string(),
 size_bytes: 0,
 is_selected: true,
 },
 CleanerItem {
 id: "prefetch".into(),
 display_name: "Windows Prefetch Cache".into(),
 description: "Application startup caching files in C:\\Windows\\Prefetch".into(),
 target_path: "C:\\Windows\\Prefetch".into(),
 size_bytes: 0,
 is_selected: true,
 },
 CleanerItem {
 id: "dns".into(),
 display_name: "DNS Resolver Cache".into(),
 description: "Flushes stale IP/DNS lookups".into(),
 target_path: "ipconfig /flushdns".into(),
 size_bytes: 0,
 is_selected: true,
 },
 CleanerItem {
 id: "dx_shader".into(),
 display_name: "DirectX Shader Cache".into(),
 description: "Compiled DirectX shader cache (D3DSCache)".into(),
 target_path: local.join("D3DSCache").display().to_string(),
 size_bytes: 0,
 is_selected: true,
 },
 ]
 }

 pub fn flush_dns(&self) -> Result<bool, String> {
 // Executado via commands module; aqui apenas sinaliza sucesso
 Ok(true)
 }

 pub fn clean_directory_contents(&self, path: &str) -> Result<(u64, u32, u32), String> {
 let p = PathBuf::from(path);
 if !p.exists() { return Ok((0, 0, 0)); }
 let mut bytes: u64 = 0;
 let mut count: u32 = 0;
 let mut errors: u32 = 0;
 self.clean_recursive(&p, &mut bytes, &mut count, &mut errors)?;
 Ok((bytes, count, errors))
 }

 fn clean_recursive(&self, dir: &PathBuf, bytes: &mut u64, count: &mut u32, errors: &mut u32) -> Result<(), String> {
 match std::fs::read_dir(dir) {
 Ok(entries) => {
 for entry in entries {
 match entry {
 Ok(e) => {
 let path = e.path();
 if path.is_file() {
 match std::fs::metadata(&path) {
 Ok(meta) => {
 *bytes += meta.len();
 if let Err(_) = std::fs::remove_file(&path) {
 *errors += 1;
 } else {
 *count += 1;
 }
 }
 Err(_) => *errors += 1,
 }
 } else if path.is_dir() {
 self.clean_recursive(&path, bytes, count, errors)?;
 let _ = std::fs::remove_dir_all(&path);
 }
 }
 Err(_) => *errors += 1,
 }
 }
 }
 Err(e) => return Err(format!("não foi possível ler {}: {}", dir.display(), e)),
 }
 Ok(())
 }
}