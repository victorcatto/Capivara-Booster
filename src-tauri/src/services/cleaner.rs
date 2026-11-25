use std::path::PathBuf;
use crate::models::CleanerItem;
use crate::services::registry::dirs_local_data;

pub struct CleanerService;

#[derive(Debug, Clone, Copy)]
pub enum CleanKind {
    Directory,
    RecycleBin,
    Dns,
}

pub fn clean_kind(item_id: &str) -> CleanKind {
    match item_id {
        "recycle_bin" => CleanKind::RecycleBin,
        "dns" => CleanKind::Dns,
        _ => CleanKind::Directory,
    }
}

/// Alvos que são agregações de várias pastas (como no WPF).
pub fn directories_for(item_id: &str) -> Vec<PathBuf> {
    let local = dirs_local_data();
    let roaming = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_default();
    let windir = std::env::var("WINDIR").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(r"C:\Windows"));
    match item_id {
        "directx_shader" => vec![
            local.join("D3DSCache"),
            local.join(r"Microsoft\DirectX Shader Cache"),
        ],
        "nvidia_shader" => vec![
            local.join(r"NVIDIA\DXCache"),
            local.join(r"NVIDIA\GLCache"),
            roaming.join(r"NVIDIA\ComputeCache"),
        ],
        "amd_shader" => vec![
            local.join(r"AMD\DxCache"),
            local.join(r"AMD\GLCache"),
        ],
        _ => vec![],
    }
}

fn single_targets(item_id: &str) -> Vec<PathBuf> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    match item_id {
        "temp" => vec![std::env::temp_dir()],
        "system_temp" => vec![PathBuf::from(&windir).join("Temp")],
        "prefetch" => vec![PathBuf::from(&windir).join("Prefetch")],
        "windows_update" => vec![PathBuf::from(&windir).join(r"SoftwareDistribution\Download")],
        _ => vec![],
    }
}

impl CleanerService {
    pub fn new() -> Self { Self }

    pub fn get_cleaner_items(&self) -> Vec<CleanerItem> {
        let local = dirs_local_data();
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        vec![
            self.item("temp", "User Temp Files (%TEMP%)", "Temporary files created by applications and installers", std::env::temp_dir().display().to_string()),
            self.item("system_temp", "Windows System Temp", "System-level temporary files", PathBuf::from(&windir).join("Temp").display().to_string()),
            self.item("prefetch", "Windows Prefetch Cache", "Application startup caching files", PathBuf::from(&windir).join("Prefetch").display().to_string()),
            self.item("windows_update", "Windows Update Cache", "Downloaded installation packages", PathBuf::from(&windir).join(r"SoftwareDistribution\Download").display().to_string()),
            self.item("directx_shader", "DirectX Shader Cache", "D3DSCache and DirectX Shader Cache", local.join("D3DSCache").display().to_string()),
            self.item("nvidia_shader", "NVIDIA DX/GL Shader Cache", "NVIDIA DirectX, OpenGL and Compute caches", local.join(r"NVIDIA\DXCache").display().to_string()),
            self.item("amd_shader", "AMD DxCache", "AMD Radeon shader pipeline cache", local.join(r"AMD\DxCache").display().to_string()),
            self.item("recycle_bin", "Recycle Bin", "Files discarded into the Windows Recycle Bin", "RecycleBin".into()),
            self.item("dns", "DNS Resolver Cache", "Flushes stale IP and DNS lookups", "ipconfig /flushdns".into()),
        ]
    }

    fn item(&self, id: &str, name: &str, description: &str, target: String) -> CleanerItem {
        let (bytes, _count, _errors) = Self::measure(id);
        CleanerItem { id: id.into(), display_name: name.into(), description: description.into(), target_path: target, size_bytes: bytes, is_selected: true }
    }

    /// Tamanho somado das pastas do alvo (agregações inclusas).
    pub fn measure(item_id: &str) -> (u64, u32, u32) {
        let mut targets: Vec<PathBuf> = directories_for(item_id);
        if targets.is_empty() { targets = single_targets(item_id); }
        let mut bytes = 0; let mut count = 0; let mut errors = 0;
        for target in targets { measure_recursive(&target, &mut bytes, &mut count, &mut errors); }
        (bytes, count, errors)
    }

    pub fn clean_item(&self, item_id: &str) -> Result<(u64, u32, u32), String> {
        let mut targets: Vec<PathBuf> = directories_for(item_id);
        if targets.is_empty() { targets = single_targets(item_id); }
        let mut bytes = 0; let mut count = 0; let mut errors = 0;
        for target in targets {
            if target.exists() { self.clean_recursive(&target, &mut bytes, &mut count, &mut errors)?; }
        }
        Ok((bytes, count, errors))
    }

    fn clean_recursive(&self, dir: &PathBuf, bytes: &mut u64, count: &mut u32, errors: &mut u32) -> Result<(), String> {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => { *errors += 1; return Ok(()); }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                match std::fs::metadata(&path) {
                    Ok(meta) => {
                        *bytes += meta.len();
                        // Read-only e arquivos em uso: tenta limpar atributo e remover;
                        // se ainda falhar, é arquivo aberto por outro processo (skip, não erro fatal).
                        if std::fs::remove_file(&path).is_ok() {
                            *count += 1;
                        } else {
                            #[cfg(windows)]
                            {
                                use std::os::windows::fs::PermissionsExt;
                                if let Ok(mut perms) = std::fs::metadata(&path).map(|m| m.permissions()) {
                                    if perms.permissions() .mode() & 0o444 != 0 {
                                        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o666));
                                        if std::fs::remove_file(&path).is_ok() { *count += 1; } else { *errors += 1; }
                                        continue;
                                    }
                                }
                            }
                            *errors += 1;
                        }
                    }
                    Err(_) => *errors += 1,
                }
            } else if path.is_dir() {
                self.clean_recursive(&path, bytes, count, errors)?;
                let _ = std::fs::remove_dir(&path);
            }
        }
        Ok(())
    }
}

fn measure_recursive(dir: &PathBuf, bytes: &mut u64, count: &mut u32, errors: &mut u32) {
    let entries = match std::fs::read_dir(dir) { Ok(e) => e, Err(_) => { *errors += 1; return; } };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(meta) = std::fs::metadata(&path) { *bytes += meta.len(); *count += 1; } else { *errors += 1; }
        } else if path.is_dir() {
            measure_recursive(&path, bytes, count, errors);
        }
    }
}
