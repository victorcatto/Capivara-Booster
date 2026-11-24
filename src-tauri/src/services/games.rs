use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::models::GameDetectionResult;
use crate::services::registry::{HKCU, HKLM, RegistryService};
use crate::services::process_executor::ProcessExecutor;

pub struct GameOptimizerService;

fn steam_candidates(cs2: &Path) -> Vec<PathBuf> {
 Vec::new()
}

impl GameOptimizerService {
 pub fn new() -> Self { Self }

 pub fn detect_game(&self, game_id: &str, reg: &RegistryService) -> GameDetectionResult {
 match game_id.to_lowercase().as_str() {
 "cs2" => self.detect_cs2(reg),
 "valorant" => self.detect_valorant(),
 "lol" => self.detect_lol(),
 _ => GameDetectionResult {
 game_id: game_id.into(),
 found: false,
 executable_path: String::new(),
 install_directory: String::new(),
 },
 }
 }

 fn detect_cs2(&self, reg: &RegistryService) -> GameDetectionResult {
 let steam_path = reg
 .get_string(HKCU, r"Software\Valve\Steam", "SteamPath")
 .or_else(|| reg.get_string(HKLM, r"SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath"))
 .unwrap_or_default();

 let mut libs: Vec<PathBuf> = Vec::new();
 if !steam_path.is_empty() {
 let sp = PathBuf::from(&steam_path);
 if sp.exists() {
 libs.push(sp.clone());
 let vdf = sp.join("steamapps").join("libraryfolders.vdf");
 if let Ok(content) = std::fs::read_to_string(&vdf) {
 for cap in content.split("\"path\"").skip(1) {
 if let Some(start) = cap.find('"') {
 if let Some(end_rel) = cap[start + 1..].find('"') {
 let p = &cap[start + 1..start + 1 + end_rel];
 let pb = PathBuf::from(p.replace("\\\\", "\\"));
 if pb.exists() && !libs.contains(&pb) {
 libs.push(pb);
 }
 }
 }
 }
 }
 }
 }
 for p in ["C:\\Program Files (x86)\\Steam", "D:\\SteamLibrary", "E:\\SteamLibrary"] {
 let pb = PathBuf::from(p);
 if pb.exists() && !libs.contains(&pb) { libs.push(pb); }
 }

 for lib in libs {
 let exe = lib.join("steamapps").join("common").join("Counter-Strike Global Offensive").join("game").join("bin").join("win64").join("cs2.exe");
 if exe.exists() {
 return GameDetectionResult {
 game_id: "cs2".into(),
 found: true,
 executable_path: exe.display().to_string(),
 install_directory: exe.parent().map(|p| p.display().to_string()).unwrap_or_default(),
 };
 }
 }
 GameDetectionResult { game_id: "cs2".into(), found: false, executable_path: String::new(), install_directory: String::new() }
 }

 fn detect_valorant(&self) -> GameDetectionResult {
 let candidates = [
 r"C:\Riot Games\VALORANT\live\ShooterGame\Binaries\Win64\VALORANT-Win64-Shipping.exe",
 r"D:\Riot Games\VALORANT\live\ShooterGame\Binaries\Win64\VALORANT-Win64-Shipping.exe",
 r"E:\Riot Games\VALORANT\live\ShooterGame\Binaries\Win64\VALORANT-Win64-Shipping.exe",
 ];
 for c in candidates {
 if Path::new(c).exists() {
 return GameDetectionResult {
 game_id: "valorant".into(),
 found: true,
 executable_path: c.into(),
 install_directory: Path::new(c).parent().map(|p| p.display().to_string()).unwrap_or_default(),
 };
 }
 }
 GameDetectionResult { game_id: "valorant".into(), found: false, executable_path: String::new(), install_directory: String::new() }
 }

 fn detect_lol(&self) -> GameDetectionResult {
 let candidates = [
 r"C:\Riot Games\League of Legends\League of Legends.exe",
 r"D:\Riot Games\League of Legends\League of Legends.exe",
 r"E:\Riot Games\League of Legends\League of Legends.exe",
 ];
 for c in candidates {
 if Path::new(c).exists() {
 return GameDetectionResult {
 game_id: "lol".into(),
 found: true,
 executable_path: c.into(),
 install_directory: Path::new(c).parent().map(|p| p.display().to_string()).unwrap_or_default(),
 };
 }
 }
 GameDetectionResult { game_id: "lol".into(), found: false, executable_path: String::new(), install_directory: String::new() }
 }

 pub fn generate_cs2_autoexec() -> String {
 let mut s = String::new();
 s.push_str("// Capivara Booster - CS2 Esports Competitive Config\n");
 s.push_str("rate \"786432\"\n");
 s.push_str("cl_updaterate \"128\"\n");
 s.push_str("cl_interp \"0.015625\"\n");
 s.push_str("cl_interp_ratio \"1\"\n");
 s.push_str("fps_max \"0\"\n");
 s.push_str("engine_low_latency_sleep_after_client_tick \"true\"\n");
 s.push_str("snd_mixahead \"0.011\"\n");
 s.push_str("host_writeconfig\n");
 s
 }

 pub async fn add_defender_exclusion(executor: &ProcessExecutor, path: &str) -> Result<bool, String> {
 let script = format!("Add-MpPreference -ExclusionPath '{}' -ErrorAction SilentlyContinue", path);
 let result = executor.execute_powershell(&script).await;
 Ok(result.exit_code == 0)
 }
}