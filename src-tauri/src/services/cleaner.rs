use std::path::PathBuf;
use crate::models::CleanerItem;
use crate::services::registry::dirs_local_data;

pub struct CleanerService;

impl CleanerService {
    pub fn new() -> Self { Self }

    pub fn get_cleaner_items(&self) -> Vec<CleanerItem> {
        let local = dirs_local_data();
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        vec![
            item("temp", "User Temp Files (%TEMP%)", "Temporary files created by applications and installers", std::env::temp_dir()),
            item("system_temp", "Windows System Temp", "System temporary files", PathBuf::from(&windir).join("Temp")),
            item("prefetch", "Windows Prefetch Cache", "Application startup cache", PathBuf::from(&windir).join("Prefetch")),
            item("windows_update", "Windows Update Cache", "Downloaded Windows Update packages", PathBuf::from(&windir).join("SoftwareDistribution/Download")),
            item("directx_shader", "DirectX Shader Cache", "D3DSCache and DirectX Shader Cache", local.join("D3DSCache")),
            item("nvidia_shader", "NVIDIA DX/GL Shader Cache", "NVIDIA DirectX, OpenGL and Compute caches", local.join("NVIDIA/DXCache")),
            item("amd_shader", "AMD DxCache", "AMD Radeon shader pipeline cache", local.join("AMD/DxCache")),
            item("recycle_bin", "Recycle Bin", "Files discarded into the Recycle Bin", PathBuf::from("RecycleBin")),
            item("dns", "DNS Resolver Cache", "Flushes stale IP and DNS lookups", PathBuf::from("ipconfig /flushdns")),
        ]
    }

    pub fn clean_directory_contents(&self, path: &str) -> Result<(u64, u32, u32), String> {
        let p = PathBuf::from(path);
        if !p.exists() { return Ok((0, 0, 0)); }
        let mut bytes = 0; let mut count = 0; let mut errors = 0;
        self.clean_recursive(&p, &mut bytes, &mut count, &mut errors)?;
        Ok((bytes, count, errors))
    }

    fn clean_recursive(&self, dir: &PathBuf, bytes: &mut u64, count: &mut u32, errors: &mut u32) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| format!("não foi possível ler {}: {e}", dir.display()))? {
            match entry {
                Ok(e) => {
                    let path = e.path();
                    if path.is_file() {
                        match std::fs::metadata(&path) {
                            Ok(meta) => { *bytes += meta.len(); if std::fs::remove_file(&path).is_ok() { *count += 1; } else { *errors += 1; } }
                            Err(_) => *errors += 1,
                        }
                    } else if path.is_dir() {
                        self.clean_recursive(&path, bytes, count, errors)?;
                        let _ = std::fs::remove_dir(&path);
                    }
                }
                Err(_) => *errors += 1,
            }
        }
        Ok(())
    }
}

fn item(id: &str, name: &str, description: &str, path: PathBuf) -> CleanerItem {
    CleanerItem { id: id.into(), display_name: name.into(), description: description.into(), target_path: path.display().to_string(), size_bytes: 0, is_selected: true }
}
