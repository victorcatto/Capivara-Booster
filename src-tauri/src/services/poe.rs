use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::services::process_executor::ProcessExecutor;
use crate::services::registry::{HKCU, HKLM, RegistryService};
use crate::services::restore::RestoreService;

const POE1_EXES: &[&str] = &["PathOfExile.exe", "PathOfExile_x64.exe", "PathOfExileSteam.exe", "PathOfExile_x64Steam.exe"];
const POE2_EXES: &[&str] = &["PathOfExile2.exe", "PathOfExile2Steam.exe", "PathOfExile_x64Steam.exe"];

#[derive(Debug, Clone, Serialize)]
pub struct PoeInstall {
    pub game: String,
    pub found: bool,
    pub config_path: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoeOptions {
    pub global_illumination_off: bool,
    pub shadows_min: bool,
    pub screen_shake_off: bool,
    pub dynamic_culling: bool,
    pub dynamic_resolution: bool,
    pub renderer_dx12: bool,
    pub sound_channels_low: bool,
    pub mute_effects_keep_loot: bool,
    pub lockstep: bool,
}

impl Default for PoeOptions {
    fn default() -> Self {
        Self {
            global_illumination_off: true,
            shadows_min: true,
            screen_shake_off: true,
            dynamic_culling: true,
            dynamic_resolution: true,
            renderer_dx12: true,
            sound_channels_low: true,
            mute_effects_keep_loot: false,
            lockstep: true,
        }
    }
}

pub struct PoeService;

impl PoeService {
    pub fn new() -> Self { Self }

    fn documents_dir() -> PathBuf {
        std::env::var("USERPROFILE").map(|u| PathBuf::from(u).join("Documents")).unwrap_or_else(|_| PathBuf::from(r"C:\Users\Public\Documents"))
    }

    pub fn detect(&self) -> Vec<PoeInstall> {
        let docs = Self::documents_dir();
        vec![
            PoeInstall {
                game: "poe1".into(),
                found: docs.join(r"My Games\Path of Exile\production_Config.ini").exists(),
                config_path: docs.join(r"My Games\Path of Exile\production_Config.ini").display().to_string(),
                source: "Documents".into(),
            },
            PoeInstall {
                game: "poe2".into(),
                found: docs.join(r"My Games\Path of Exile 2\poe2_production_Config.ini").exists(),
                config_path: docs.join(r"My Games\Path of Exile 2\poe2_production_Config.ini").display().to_string(),
                source: "Documents".into(),
            },
        ]
    }

    /// Parte síncrona do perfil: .ini com backup, GPU preference. Sem await.
    pub fn apply_file_profile(reg: &mut RegistryService, restore: &mut RestoreService, game: &str, options: &PoeOptions) -> Result<String, String> {
        let installs = Self::new().detect();
        let install = installs.iter().find(|i| i.game == game && i.found).ok_or_else(|| format!("Path of Exile {game} não encontrado"))?;
        let config = PathBuf::from(&install.config_path);
        restore.backup_file(&config.display().to_string(), game, "Original PoE config antes do perfil competitivo")?;

        let mut ini = read_ini(&config)?;
        let pos = ini.iter().position(|(name, _)| name.eq_ignore_ascii_case("DISPLAY"));
        let pos = match pos {
            Some(p) => p,
            None => { ini.push(("DISPLAY".to_string(), Vec::new())); ini.len() - 1 }
        };
        let graphics = &mut ini[pos];

        set_option(graphics, "global_illumination_type", if options.global_illumination_off { "0" } else { "2" });
        set_option(graphics, "shadow_type", if options.shadows_min { "no_shadows" } else { "all_low" });
        set_option(graphics, "screen_shake", if options.screen_shake_off { "false" } else { "true" });
        set_option(graphics, "dynamic_culling", if options.dynamic_culling { "true" } else { "false" });
        set_option(graphics, "dynamic_resolution", if options.dynamic_resolution { "true" } else { "false" });
        set_option(graphics, "renderer_type", if options.renderer_dx12 { "DirectX12" } else { "Vulkan" });
        set_option(graphics, "sound_channel_count", if options.sound_channels_low { "low" } else { "high" });
        if options.mute_effects_keep_loot {
            set_option(graphics, "sound_effects_volume", "false");
            set_option(graphics, "ambient_sound_volume", "false");
            set_option(graphics, "item_filter_sound_volume", "true");
        }
        set_option(graphics, "networking_type", if options.lockstep { "lockstep" } else { "predictive" });

        write_ini(&config, &ini)?;

        let exe_names: &[&str] = if game == "poe2" { POE2_EXES } else { POE1_EXES };
        for exe in exe_names {
            let key = r"SOFTWARE\Microsoft\DirectX\UserGpuPreferences".to_string();
            reg.set_string(HKLM, &key, exe, "GpuPreference=2;")?;
        }

        Ok(config.display().to_string())
    }

    /// Parte async: Nagle nas interfaces ativas. Chamada sem guard vivo.
    pub async fn apply_network_tweaks(executor: &ProcessExecutor) -> Result<(), String> {
        let script = r#"$ifs = Get-NetAdapter -ErrorAction SilentlyContinue | Select-Object -ExpandProperty InterfaceGuid -Unique; foreach ($guid in $ifs) { $base = "HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\$guid"; Set-ItemProperty -Path $base -Name TcpAckFrequency -Value 1 -Type DWord -ErrorAction SilentlyContinue; Set-ItemProperty -Path $base -Name TCPNoDelay -Value 1 -Type DWord -ErrorAction SilentlyContinue }"#;
        let _ = executor.execute_powershell(script).await;
        Ok(())
    }

    /// Limpa os caches de shader do PoE/PoE2 (as pastas são reconstruídas pelo jogo).
    pub fn clear_shader_cache() -> Result<(u64, u32), String> {
        let appdata = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_default();
        let local = std::env::var("LOCALAPPDATA").map(PathBuf::from).unwrap_or_default();
        let targets = [
            appdata.join(r"Path of Exile\ShaderCacheD3D12"),
            appdata.join(r"Path of Exile\ShaderCacheVulkan"),
            appdata.join(r"Path of Exile 2\ShaderCacheD3D12"),
            appdata.join(r"Path of Exile 2\ShaderCacheVulkan"),
            local.join(r"NVIDIA\DXCache"),
            local.join(r"NVIDIA\GLCache"),
        ];
        let (mut bytes, mut count) = (0u64, 0u32);
        for target in targets {
            if target.exists() {
                measure_tree(&target, &mut bytes, &mut count);
                let _ = std::fs::remove_dir_all(&target);
            }
        }
        Ok((bytes, count))
    }
}

fn set_option(section: &mut (String, Vec<(String, String)>), key: &str, value: &str) {
    let key_lower = key.to_lowercase();
    if let Some(entry) = section.1.iter_mut().find(|(k, _)| *k == key_lower) {
        entry.1 = value.to_string();
    } else {
        section.1.push((key_lower, value.to_string()));
    }
}

/// Lê o .ini preservando ordem de seções e linhas de comentário.
type IniDoc = Vec<(String, Vec<(String, String)>)>;

fn read_ini(path: &Path) -> Result<IniDoc, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("não foi possível ler {}: {e}", path.display()))?;
    let mut doc: IniDoc = Vec::new();
    let mut current: Option<(String, Vec<(String, String)>)> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some(section) = current.take() { doc.push(section); }
            current = Some((trimmed.trim_matches(|c| c == '[' || c == ']').to_string(), Vec::new()));
        } else if let Some((_, entries)) = current.as_mut() {
            if let Some((key, value)) = trimmed.split_once('=') {
                entries.push((key.trim().to_lowercase(), value.trim().to_string()));
            } else {
                entries.push((format!("__comment_{}", entries.len()), line.to_string()));
            }
        }
    }
    if let Some(section) = current.take() { doc.push(section); }
    Ok(doc)
}

fn write_ini(path: &Path, doc: &IniDoc) -> Result<(), String> {
    let mut out = String::new();
    for (name, entries) in doc {
        out.push_str(&format!("[{name}]\r\n"));
        for (key, value) in entries {
            if key.starts_with("__comment_") { out.push_str(value); }
            else { out.push_str(&format!("{key}={value}")); }
            out.push_str("\r\n");
        }
        out.push_str("\r\n");
    }
    std::fs::write(path, out).map_err(|e| format!("não foi possível escrever {}: {e}", path.display()))
}

fn measure_tree(dir: &Path, bytes: &mut u64, count: &mut u32) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(meta) = std::fs::metadata(&path) { *bytes += meta.len(); *count += 1; }
        } else if path.is_dir() {
            measure_tree(&path, bytes, count);
        }
    }
}
