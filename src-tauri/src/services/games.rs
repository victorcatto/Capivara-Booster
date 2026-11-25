use std::path::{Path, PathBuf};
use serde_json::{json, Value};
use crate::models::GameDetectionResult;
use crate::services::process_executor::ProcessExecutor;
use crate::services::registry::{HKCU, HKLM, RegistryService};
use crate::services::restore::RestoreService;

pub struct GameOptimizerService;

impl GameOptimizerService {
    pub fn new() -> Self { Self }

    pub fn detect_game(&self, game_id: &str, reg: &RegistryService) -> GameDetectionResult {
        match game_id.to_lowercase().as_str() {
            "cs2" => self.detect_cs2(reg),
            "valorant" => self.detect_valorant(),
            "lol" => self.detect_lol(),
            _ => GameDetectionResult { game_id: game_id.into(), found: false, executable_path: String::new(), install_directory: String::new() },
        }
    }

    fn detect_cs2(&self, reg: &RegistryService) -> GameDetectionResult {
        let steam = reg.get_string(HKCU, r"Software\Valve\Steam", "SteamPath")
            .or_else(|| reg.get_string(HKLM, r"SOFTWARE\Valve\Steam", "InstallPath"))
            .or_else(|| reg.get_string(HKLM, r"SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath"))
            .unwrap_or_default();
        let mut libraries = Vec::new();
        if !steam.is_empty() {
            let root = PathBuf::from(steam.replace('/', "\\"));
            if root.exists() {
                libraries.push(root.clone());
                if let Ok(vdf) = std::fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf")) {
                    for section in vdf.split("\"path\"").skip(1) {
                        if let Some(path) = quoted_value(section) {
                            let path = PathBuf::from(path.replace("\\\\", "\\").replace('/', "\\"));
                            if path.exists() && !libraries.contains(&path) { libraries.push(path); }
                        }
                    }
                }
            }
        }
        for p in [r"C:\Program Files (x86)\Steam", r"D:\SteamLibrary", r"E:\SteamLibrary", r"F:\SteamLibrary", r"G:\SteamLibrary"] {
            let path = PathBuf::from(p);
            if path.exists() && !libraries.contains(&path) { libraries.push(path); }
        }
        for lib in libraries {
            let exe = lib.join("steamapps/common/Counter-Strike Global Offensive/game/bin/win64/cs2.exe");
            if exe.exists() { return detected("cs2", &exe); }
        }
        missing("cs2")
    }

    fn detect_valorant(&self) -> GameDetectionResult {
        let riot_json = PathBuf::from(std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".into())).join("Riot Games/RiotClientInstalls.json");
        if let Ok(text) = std::fs::read_to_string(riot_json) {
            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                if let Some(live) = value.get("rc_live").and_then(Value::as_str) {
                    if let Some(parent) = Path::new(live).parent() {
                        let exe = parent.join(r"..")
                            .join("VALORANT/live/ShooterGame/Binaries/Win64/VALORANT-Win64-Shipping.exe");
                        if exe.exists() { return detected("valorant", &exe); }
                    }
                }
            }
        }
        for drive in ['C', 'D', 'E', 'F', 'G'] {
            let exe = PathBuf::from(format!(r"{drive}:\Riot Games\VALORANT\live\ShooterGame\Binaries\Win64\VALORANT-Win64-Shipping.exe"));
            if exe.exists() { return detected("valorant", &exe); }
        }
        missing("valorant")
    }

    fn detect_lol(&self) -> GameDetectionResult {
        let riot_json = PathBuf::from(std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".into())).join("Riot Games/RiotClientInstalls.json");
        if let Ok(text) = std::fs::read_to_string(riot_json) {
            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                // Riot JSON uses nested structure: league_of_legends.live (nested object)
                if let Some(league) = value.get("league_of_legends").and_then(|v| v.get("live")) {
                    if let Some(live_str) = league.as_str() {
                        let exe = PathBuf::from(live_str).join("League of Legends.exe");
                        if exe.exists() { return detected("lol", &exe); }
                    }
                }
            }
        }
        for drive in ['C', 'D', 'E', 'F', 'G'] {
            let exe = PathBuf::from(format!(r"{drive}:\Riot Games\League of Legends\League of Legends.exe"));
            if exe.exists() { return detected("lol", &exe); }
        }
        missing("lol")
    }

    pub fn generate_cs2_autoexec() -> String {
        [
            "// Capivara Booster - Counter-Strike 2 Esports Competitive Config",
            "rate \"786432\"", "cl_net_buffer_ticks \"0\"", "cl_updaterate \"128\"",
            "cl_interp \"0.015625\"", "cl_interp_ratio \"1\"", "fps_max \"0\"",
            "fps_max_ui \"120\"", "engine_low_latency_sleep_after_client_tick \"true\"",
            "r_drawtracers_firstperson \"0\"", "r_show_build_info \"false\"", "cl_autohelp \"false\"",
            "gameinstructor_enable \"false\"", "snd_headphone_eq \"0\"", "snd_spatialize_roundoff \"1\"",
            "snd_mixahead \"0.011\"", "alias \"+jumpaction\" \"+jump;\"", "alias \"-jumpaction\" \"-jump;\"",
            "alias \"+throwaction\" \"-attack; -attack2;\"", "alias \"+jumpthrow\" \"+jumpaction; +throwaction;\"",
            "alias \"-jumpthrow\" \"-jumpaction;\"", "bind \"v\" \"+jumpthrow\"", "m_pitch \"0.022\"",
            "m_yaw \"0.022\"", "host_writeconfig", "echo \"[CapivaraBooster] CS2 autoexec loaded\"",
        ].join("\r\n") + "\r\n"
    }

    pub fn apply_priority_affinity(reg: &mut RegistryService, exe: &str, priority: u32, no_core0: bool) -> Result<(), String> {
        let key = format!(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\{exe}\PerfOptions");
        reg.set_dword(HKLM, &key, "CpuPriorityClass", priority)?;
        if no_core0 { reg.set_dword(HKLM, &key, "CpuAffinityMask", 0xFFFF_FFFE)?; }
        Ok(())
    }

    /// Otimizações adicionais de CS2: prioridade do processo vivo, GPU priority
    /// no perfil Multimedia e exclusões do Defender para o executável do jogo.
    pub fn prepare_cs2(reg: &mut RegistryService, exe_path: &str) -> Result<String, String> {
        Self::apply_priority_affinity(reg, "cs2.exe", 3, true)?;
        if Path::new(exe_path).exists() { Self::apply_app_compat(reg, exe_path)?; }
        let games = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";
        reg.set_dword(HKLM, games, "GPU Priority", 8)?;
        reg.set_dword(HKLM, games, "Priority", 6)?;
        reg.set_string(HKLM, games, "Scheduling Category", "High")?;
        let exe_dir = Path::new(exe_path).parent().unwrap_or(Path::new(""));
        let steam_library = exe_dir.ancestors().nth(3).unwrap_or(Path::new(""));
        Ok(format!(
            "foreach ($p in @('cs2.exe','steam.exe')) {{ Add-MpPreference -ExclusionProcess $p -ErrorAction SilentlyContinue }}; Add-MpPreference -ExclusionPath '{}' -ErrorAction SilentlyContinue; Add-MpPreference -ExclusionPath '{}' -ErrorAction SilentlyContinue",
            ps_quote(&steam_library.join("shadercache").display().to_string()),
            ps_quote(&steam_library.display().to_string())
        ))
    }

    /// Limpa o shader cache do Steam do CS2 (shadercache/730).
    pub fn clear_cs2_shader_cache(exe_path: &str) -> Result<(u64, u32), String> {
        let exe_dir = Path::new(exe_path).parent().ok_or("caminho do CS2 inválido")?;
        let library = exe_dir.ancestors().nth(3).ok_or("biblioteca Steam inválida")?;
        let cache = library.join("shadercache").join("730");
        if !cache.exists() { return Ok((0, 0)); }
        let (mut bytes, mut count, _errors) = (0u64, 0u32, 0u32);
        measure_tree(&cache, &mut bytes, &mut count);
        std::fs::remove_dir_all(&cache).map_err(|e| format!("não foi possível limpar o shader cache: {e}"))?;
        Ok((bytes, count))
    }

    /// Define prioridade em tempo real para o cs2.exe em execução agora.
    pub async fn boost_live_cs2(executor: &ProcessExecutor) -> Result<bool, String> {
        let script = "Get-Process cs2 -ErrorAction SilentlyContinue | ForEach-Object { $_.PriorityClass = 'High' }";
        let result = executor.execute_powershell(script).await;
        Ok(result.exit_code == 0)
    }

    pub fn apply_app_compat(reg: &mut RegistryService, exe_path: &str) -> Result<(), String> {
        let flags = "~ DISABLEDXMAXIMIZEDWINDOWEDMODE HIGHDPIAWARE";
        reg.set_string(HKCU, r"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers", exe_path, flags)
    }

    pub fn prepare_valorant(reg: &mut RegistryService, exe_path: &str) -> Result<String, String> {
        Self::apply_priority_affinity(reg, "VALORANT-Win64-Shipping.exe", 3, true)?;
        Self::apply_priority_affinity(reg, "RiotClientServices.exe", 1, false)?;
        if Path::new(exe_path).exists() { Self::apply_app_compat(reg, exe_path)?; }
        let root = Path::new(exe_path).parent().and_then(|p| p.ancestors().nth(4)).unwrap_or(Path::new(r"C:\Riot Games"));
        let vanguard = PathBuf::from(std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into())).join("Riot Vanguard");
        Ok(format!("Add-MpPreference -ExclusionPath '{}' -ErrorAction SilentlyContinue; Add-MpPreference -ExclusionPath '{}' -ErrorAction SilentlyContinue; foreach ($p in @('vgk.sys','vgc.exe','VALORANT-Win64-Shipping.exe','RiotClientServices.exe')) {{ Add-MpPreference -ExclusionProcess $p -ErrorAction SilentlyContinue }}", ps_quote(&vanguard.display().to_string()), ps_quote(&root.display().to_string())))
    }

    pub fn prepare_lol(reg: &mut RegistryService, restore: &mut RestoreService, exe_path: &str) -> Result<String, String> {
        Self::apply_priority_affinity(reg, "League of Legends.exe", 3, false)?;
        if Path::new(exe_path).exists() { Self::apply_app_compat(reg, exe_path)?; }
        let root = Path::new(exe_path).parent().ok_or("caminho de instalação do LoL inválido")?;
        let config = root.join("Config");
        std::fs::create_dir_all(&config).map_err(|e| e.to_string())?;
        let game_cfg = config.join("game.cfg");
        let persisted = config.join("PersistedSettings.json");
        if game_cfg.exists() { restore.backup_file(&game_cfg.display().to_string(), "LoL", "Original LoL game.cfg")?; }
        patch_game_cfg(&game_cfg)?;
        if persisted.exists() {
            restore.backup_file(&persisted.display().to_string(), "LoL", "Original LoL PersistedSettings.json")?;
            patch_persisted_settings(&persisted)?;
        }
        let logs = root.join("Logs");
        if logs.exists() { std::fs::remove_dir_all(logs).map_err(|e| e.to_string())?; }
        Ok(format!("Add-MpPreference -ExclusionPath '{}' -ErrorAction SilentlyContinue; foreach ($p in @('League of Legends.exe','LeagueClient.exe','LeagueClientUx.exe')) {{ Add-MpPreference -ExclusionProcess $p -ErrorAction SilentlyContinue }}", ps_quote(&root.display().to_string())))
    }

    pub fn revert_game(reg: &mut RegistryService, restore: &mut RestoreService, game_id: &str, exe_path: &str) -> Result<usize, String> {
        let exe_name = Path::new(exe_path).file_name().and_then(|n| n.to_str()).unwrap_or(match game_id { "cs2" => "cs2.exe", "valorant" => "VALORANT-Win64-Shipping.exe", _ => "League of Legends.exe" });
        let ifeo = format!(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\{exe_name}\PerfOptions");
        reg.delete_value(HKLM, &ifeo, "CpuPriorityClass")?;
        reg.delete_value(HKLM, &ifeo, "CpuAffinityMask")?;
        reg.delete_value(HKCU, r"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers", exe_path)?;
        if game_id == "valorant" {
            let riot = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\RiotClientServices.exe\PerfOptions";
            reg.delete_value(HKLM, riot, "CpuPriorityClass")?;
        }
        restore.restore_category(game_id)
    }
}

fn detected(game_id: &str, path: &Path) -> GameDetectionResult {
    GameDetectionResult { game_id: game_id.into(), found: true, executable_path: path.display().to_string(), install_directory: path.parent().unwrap_or(Path::new("")).display().to_string() }
}
fn missing(game_id: &str) -> GameDetectionResult {
    GameDetectionResult { game_id: game_id.into(), found: false, executable_path: String::new(), install_directory: String::new() }
}
fn quoted_value(value: &str) -> Option<String> {
    let start = value.find('"')? + 1;
    let end = value[start..].find('"')? + start;
    Some(value[start..end].to_string())
}
fn ps_quote(value: &str) -> String { value.replace('\'', "''") }

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

fn patch_game_cfg(path: &Path) -> Result<(), String> {
    let mut lines = if path.exists() { std::fs::read_to_string(path).map_err(|e| e.to_string())?.lines().map(str::to_string).collect::<Vec<_>>() } else { Vec::new() };
    let mut general_start = lines.iter().position(|l| l.trim().eq_ignore_ascii_case("[General]"));
    if general_start.is_none() { lines.push("[General]".into()); general_start = Some(lines.len() - 1); }
    let start = general_start.unwrap();
    let end = lines.iter().enumerate().skip(start + 1).find(|(_, l)| l.trim().starts_with('[')).map(|(i, _)| i).unwrap_or(lines.len());
    let mut section = lines[start + 1..end].to_vec();
    for (key, value) in [("LowSpecMode", "1"), ("CloseClient", "1"), ("WaitForVerticalSync", "0")] {
        if let Some(line) = section.iter_mut().find(|l| l.trim().split('=').next().unwrap_or("").trim().eq_ignore_ascii_case(key)) { *line = format!("{key}={value}"); }
        else { section.push(format!("{key}={value}")); }
    }
    lines.splice(start + 1..end, section);
    std::fs::write(path, lines.join("\r\n")).map_err(|e| e.to_string())
}

fn patch_persisted_settings(path: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut root: Value = serde_json::from_str(&content).map_err(|e| format!("PersistedSettings.json inválido: {e}"))?;
    let files = root.get_mut("files").and_then(Value::as_array_mut).ok_or("PersistedSettings.json não contém files")?;
    for file in files {
        let Some(sections) = file.get_mut("sections").and_then(Value::as_array_mut) else { continue };
        let general = sections.iter_mut().find(|s| s.get("name").and_then(Value::as_str) == Some("General"));
        let section = if let Some(s) = general { s } else { sections.push(json!({"name":"General","settings":[]})); sections.last_mut().unwrap() };
        let settings = section.get_mut("settings").and_then(Value::as_array_mut).ok_or("seção General sem settings válidos")?;
        for (name, value) in [("LowSpecMode", "1"), ("CloseClientDuringGame", "1"), ("WaitForVerticalSync", "0")] {
            if let Some(item) = settings.iter_mut().find(|s| s.get("name").and_then(Value::as_str) == Some(name)) { item["value"] = json!(value); }
            else { settings.push(json!({"name":name,"value":value})); }
        }
    }
    std::fs::write(path, serde_json::to_vec_pretty(&root).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
