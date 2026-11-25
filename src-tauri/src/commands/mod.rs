use serde::Serialize;
use tauri::State;
use std::sync::Mutex;

use crate::models::{BloatwareItem, CleanerItem, CleaningSummary, GameDetectionResult, StartupItem};
use crate::services::auth::AuthService;
use crate::services::cleaner::CleanerService;
use crate::services::debloat::DebloatService;
use crate::services::games::GameOptimizerService;
use crate::services::i18n::I18nService;
use crate::services::process_executor::ProcessExecutor;
use crate::services::registry::{RegistryService, HKCU, HKLM};
use crate::services::restore::RestoreService;
use crate::services::system_boost::SystemBoostService;
use crate::services::update::UpdateService;

pub struct AppState {
 pub auth: Mutex<AuthService>,
 pub registry: Mutex<RegistryService>,
 pub restore: Mutex<RestoreService>,
 pub cleaner: Mutex<CleanerService>,
 pub boost: Mutex<SystemBoostService>,
 pub debloat: Mutex<DebloatService>,
 pub i18n: Mutex<I18nService>,
 pub logs: Mutex<Vec<String>>,
}

fn log(state: &AppState, msg: &str) {
 let mut logs = state.logs.lock().unwrap();
 logs.push(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
 if logs.len() > 1000 { logs.remove(0); }
}

/// Bloqueia comandos de sistema quando não há sessão autenticada.
fn require_auth(state: &AppState) -> Result<(), String> {
 let auth = state.auth.lock().unwrap();
 if auth.is_authenticated() { Ok(()) } else { Err("login obrigatório: autentique-se para usar as otimizações".into()) }
}

#[derive(Serialize)]
pub struct LogPayload {
 pub items: Vec<String>,
}

#[tauri::command]
pub fn get_logs(state: State<AppState>) -> LogPayload {
 LogPayload { items: state.logs.lock().unwrap().clone() }
}

#[tauri::command]
pub fn clear_logs(state: State<AppState>) {
 state.logs.lock().unwrap().clear();
}

#[derive(Serialize)]
pub struct AuthPayload {
 pub email: Option<String>,
 pub display_name: Option<String>,
 pub role: Option<String>,
 pub authenticated: bool,
 pub is_admin: bool,
}

#[tauri::command]
pub fn get_auth_state(state: State<AppState>) -> AuthPayload {
 let auth = state.auth.lock().unwrap();
 match auth.current() {
 Some(u) => AuthPayload {
 email: Some(u.email.clone()),
 display_name: Some(u.display_name.clone()),
 role: Some(u.role.clone()),
 authenticated: true,
 is_admin: u.role == "Admin",
 },
 None => AuthPayload { email: None, display_name: None, role: None, authenticated: false, is_admin: false },
 }
}

#[tauri::command]
pub fn login(state: State<AppState>, email: String, password: String) -> Result<AuthPayload, String> {
 let mut auth = state.auth.lock().unwrap();
 let user = auth.login(&email, &password)?;
 log(&state, &format!("[Auth] Login efetuado: {}", user.email));
 Ok(AuthPayload {
 email: Some(user.email),
 display_name: Some(user.display_name),
 role: Some(user.role),
 authenticated: true,
 is_admin: auth.is_admin(),
 })
}

#[tauri::command]
pub fn register(state: State<AppState>, email: String, password: String) -> Result<AuthPayload, String> {
 let mut auth = state.auth.lock().unwrap();
 let user = auth.register(&email, &password)?;
 log(&state, &format!("[Auth] Conta criada: {}", user.email));
 Ok(AuthPayload {
 email: Some(user.email),
 display_name: Some(user.display_name),
 role: Some(user.role),
 authenticated: true,
 is_admin: auth.is_admin(),
 })
}

#[tauri::command]
pub fn logout(state: State<AppState>) {
 let mut auth = state.auth.lock().unwrap();
 auth.logout();
 log(&state, "[Auth] Sessão encerrada");
}

#[tauri::command]
pub fn get_cleaner_items(state: State<AppState>) -> Vec<CleanerItem> {
 let cleaner = state.cleaner.lock().unwrap();
 cleaner.get_cleaner_items()
}

#[tauri::command]
pub fn get_bloatware_items(state: State<AppState>) -> Vec<BloatwareItem> {
 let debloat = state.debloat.lock().unwrap();
 debloat.get_default_bloatware_list()
}

#[tauri::command]
pub async fn clean_item(state: State<'_, AppState>, item_id: String) -> Result<CleaningSummary, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 if item_id == "dns" {
  let result = executor.execute_cmd("ipconfig /flushdns").await;
  log(&state, "[Cleaner] Cache DNS limpo");
  return Ok(CleaningSummary { total_bytes_cleaned: 0, files_deleted: if result.exit_code == 0 { 1 } else { 0 }, errors_count: if result.exit_code == 0 { 0 } else { 1 } });
 }
 if item_id == "recycle_bin" {
  let result = executor.execute_powershell("Clear-RecycleBin -Force -ErrorAction SilentlyContinue").await;
  log(&state, "[Cleaner] Lixeira esvaziada");
  return Ok(CleaningSummary { total_bytes_cleaned: 0, files_deleted: if result.exit_code == 0 { 1 } else { 0 }, errors_count: if result.exit_code == 0 { 0 } else { 1 } });
 }
 if item_id == "windows_update" {
  executor.execute_cmd("net stop wuauserv").await;
 }
 let (bytes, count, errors) = {
  let cleaner = state.cleaner.lock().unwrap();
  let result = cleaner.clean_item(&item_id)?;
  result
 };
 if item_id == "windows_update" { executor.execute_cmd("net start wuauserv").await; }
 log(&state, &format!("[Cleaner OK] {item_id}: {count} arquivos limpos, {errors} em uso pulados"));
 Ok(CleaningSummary { total_bytes_cleaned: bytes, files_deleted: count, errors_count: errors })
}

#[tauri::command]
pub async fn flush_dns(state: State<'_, AppState>) -> Result<bool, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 let result = executor.execute_cmd("ipconfig /flushdns").await;
 log(&state, "[Cleaner] DNS flush executado");
 Ok(result.exit_code == 0)
}

#[tauri::command]
pub fn get_latency_tweaks(state: State<AppState>) -> Vec<crate::services::low_latency::LatencyTweak> {
 let lang_pt = state.i18n.lock().unwrap().is_portuguese();
 crate::services::low_latency::LowLatencyService::tweaks(lang_pt)
}

#[tauri::command]
pub async fn apply_latency_tweak(state: State<'_, AppState>, tweak_id: String) -> Result<bool, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 // Registry tweaks são síncronos e rápidos: guard solta antes de qualquer await.
 let registry_part = {
 let mut registry = state.registry.lock().unwrap();
 crate::services::low_latency::LowLatencyService::apply_registry_tweak(&mut registry, &tweak_id)
 };
 let bcd_part = if registry_part.is_ok() {
 crate::services::low_latency::LowLatencyService::apply_bcd_tweak(&executor, &tweak_id).await
 } else { Ok(true) };
 let result = registry_part.and(bcd_part);
 match &result {
 Ok(_) => log(&state, &format!("[Latency] {tweak_id} aplicado")),
 Err(e) => log(&state, &format!("[Latency Erro] {tweak_id}: {e}")),
 }
 result
}

#[tauri::command]
pub async fn revert_latency_tweak(state: State<'_, AppState>, tweak_id: String) -> Result<bool, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 let registry_part = {
 let mut registry = state.registry.lock().unwrap();
 crate::services::low_latency::LowLatencyService::revert_registry_tweak(&mut registry, &tweak_id)
 };
 let bcd_part = if registry_part.is_ok() {
 crate::services::low_latency::LowLatencyService::revert_bcd_tweak(&executor, &tweak_id).await
 } else { Ok(true) };
 let result = registry_part.and(bcd_part);
 log(&state, &format!("[Latency] {tweak_id} revertido"));
 result
}

#[tauri::command]
pub async fn clear_standby_list(state: State<'_, AppState>) -> Result<bool, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 let result = crate::services::low_latency::LowLatencyService::clear_standby_list(&executor).await;
 log(&state, "[Latency] Standby list otimizada");
 result
}

#[tauri::command]
pub fn detect_poe(state: State<AppState>) -> Vec<crate::services::poe::PoeInstall> {
 crate::services::poe::PoeService::new().detect()
}

#[tauri::command]
pub async fn apply_poe_profile(state: State<'_, AppState>, game: String, options: Option<crate::services::poe::PoeOptions>) -> Result<String, String> {
 require_auth(&state)?;
 let opts = options.unwrap_or_default();
 let executor = ProcessExecutor;
 // .ini e registro são rápidos: guards soltos antes do await do Nagle.
 let file_part = {
 let mut registry = state.registry.lock().unwrap();
 let mut restore = state.restore.lock().unwrap();
 crate::services::poe::PoeService::apply_file_profile(&mut registry, &mut restore, &game, &opts)
 };
 let result = if let Ok(path) = &file_part {
 crate::services::poe::PoeService::apply_network_tweaks(&executor).await?;
 file_part
 } else {
 file_part
 };
 match &result {
 Ok(path) => log(&state, &format!("[PoE] Perfil {game} aplicado: {path}")),
 Err(e) => log(&state, &format!("[PoE Erro] {e}")),
 }
 result
}

#[tauri::command]
pub async fn clear_poe_shader_cache(state: State<'_, AppState>) -> Result<(u64, u32), String> {
 require_auth(&state)?;
 let result = crate::services::poe::PoeService::clear_shader_cache()?;
 log(&state, &format!("[PoE Cache] Shader cache limpo: {} arquivos", result.1));
 Ok(result)
}

#[tauri::command]
pub async fn boost_live_poe(state: State<'_, AppState>, game: String) -> Result<bool, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 let result = crate::services::poe::PoeService::boost_live_poe(&executor, &game).await;
 log(&state, &format!("[PoE] Boost ao vivo: {game}"));
 result
}

#[tauri::command]
pub fn detect_cs2() -> crate::services::cs2::Cs2Install {
 crate::services::cs2::Cs2Service::new().detect()
}

#[tauri::command]
pub async fn apply_cs2_profile(state: State<'_, AppState>) -> Result<Vec<crate::services::cs2::Cs2TweakStatus>, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 let install = crate::services::cs2::Cs2Service::new().detect();
 if !install.found { return Err("Counter-Strike 2 não encontrado na Steam".into()); }
 // Registro com guards curtos; sistema/energia async sem guard.
 let reg_part = {
 let mut registry = state.registry.lock().unwrap();
 let mut restore = state.restore.lock().unwrap();
 crate::services::cs2::Cs2Service::apply_registry_tweaks(&mut registry, &mut restore, &install.exe_path)
 };
 let sys_part = crate::services::cs2::Cs2Service::apply_system_tweaks(&executor).await;
 let mut all = reg_part?;
 all.extend(sys_part?);
 log(&state, &format!("[CS2] Perfil aplicado: {} tweaks", all.iter().filter(|s| s.applied).count()));
 Ok(all)
}

#[tauri::command]
pub async fn apply_cs2_configs(state: State<'_, AppState>, refreshRate: u32) -> Result<Vec<String>, String> {
 require_auth(&state)?;
 let install = crate::services::cs2::Cs2Service::new().detect();
 if !install.found { return Err("Counter-Strike 2 não encontrado na Steam".into()); }
 let mut results = Vec::new();
 {
 let mut restore = state.restore.lock().unwrap();
 results.push(crate::services::cs2::Cs2Service::apply_video_config(&mut restore, &install.video_txt)?);
 results.push(crate::services::cs2::Cs2Service::apply_autoexec(&mut restore, &install.cfg_dir)?);
 results.push(crate::services::cs2::Cs2Service::apply_launch_options(&mut restore, &install.cfg_dir, refreshRate.max(60).min(360))?);
 }
 log(&state, &format!("[CS2 OK] Configs aplicadas: {}", results.len()));
 Ok(results)
}

#[tauri::command]
pub async fn clear_cs2_caches(state: State<'_, AppState>) -> Result<(u64, u32), String> {
 require_auth(&state)?;
 let result = crate::services::cs2::Cs2Service::clear_shader_caches()?;
 log(&state, &format!("[CS2 Cache] {:.1} MB em {} arquivos", result.0 as f64 / 1_048_576.0, result.1));
 Ok(result)
}

#[tauri::command]
pub fn get_app_version() -> String {
 env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn get_live_telemetry() -> Result<crate::services::telemetry::LiveTelemetry, String> {
 let executor = ProcessExecutor;
 crate::services::telemetry::TelemetryService::new().snapshot(&executor).await
}

#[tauri::command]
pub async fn purge_standby_list() -> Result<bool, String> {
 let executor = ProcessExecutor;
 crate::services::telemetry::TelemetryService::clear_standby(&executor).await
}

#[tauri::command]
pub async fn get_hardware_profile(state: State<'_, AppState>) -> Result<crate::services::hardware::HardwareProfile, String> {
 let executor = ProcessExecutor;
 crate::services::hardware::HardwareService::new().profile(&executor).await
}

#[tauri::command]
pub async fn scan_drivers(state: State<'_, AppState>) -> Result<Vec<crate::services::drivers::DriverInfo>, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 crate::services::drivers::DriverService::new().scan(&executor).await
}

#[tauri::command]
pub async fn get_driver_vendors(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
 let executor = ProcessExecutor;
 crate::services::drivers::DriverService::new().board_vendor_links(&executor).await
}

#[tauri::command]
pub async fn export_driver_backup(state: State<'_, AppState>, dest: String) -> Result<String, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 let result = crate::services::drivers::DriverService::new().export_backup(&executor, &dest).await;
 log(&state, &format!("[Drivers] Backup exportado: {dest}"));
 result
}

#[tauri::command]
pub async fn update_driver_device(state: State<'_, AppState>, deviceName: String) -> Result<String, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 log(&state, &format!("[Drivers] Atualizando via Windows Update: {deviceName}"));
 let result = crate::services::drivers::DriverService::new().update_via_windows_update(&executor, &deviceName).await;
 match result {
 Ok(msg) => {
  log(&state, &format!("[Drivers OK] {msg}"));
  Ok(msg)
 }
 Err(e) => {
  // Fallback: baixar e instalar direto do fabricante (Intel/NVIDIA/AMD/Realtek).
  log(&state, &format!("[Drivers] Windows Update sem pacote; tentando o fabricante..."));
  match crate::services::drivers::DriverService::new().download_and_install_from_vendor(&executor, &deviceName).await {
  Ok(msg) => {
   log(&state, &format!("[Drivers OK] {msg}"));
   Ok(msg)
  }
  Err(e2) => {
   log(&state, &format!("[Drivers Erro] {e2}"));
   Err(e2)
  }
  }
 }
 }
}

#[tauri::command]
pub async fn get_hpet_status(_state: State<'_, AppState>) -> Result<bool, String> {
 Ok(crate::services::system_boost::SystemBoostService::is_hpet_disabled().await)
}

#[tauri::command]
pub async fn set_hpet_state(state: State<'_, AppState>, disable: bool) -> Result<bool, String> {
 require_auth(&state)?;
 let result = crate::services::system_boost::SystemBoostService::set_hpet_state(disable).await;
 log(&state, if disable { "[HPET] Desativado" } else { "[HPET] Restaurado" });
 result
}

#[tauri::command]
pub async fn enable_ultimate_performance(state: State<'_, AppState>) -> Result<String, String> {
 require_auth(&state)?;
 let result = crate::services::system_boost::SystemBoostService::enable_ultimate_performance().await;
 if let Ok(ref guid) = result { log(&state, &format!("[Power Plan] Ativo: {guid}")); }
 result
}

#[tauri::command]
pub async fn apply_system_optimizations(state: State<'_, AppState>) -> Result<bool, String> {
 require_auth(&state)?;
 let result = {
 let mut boost = state.boost.lock().unwrap();
 boost.apply_optimizations()
 };
 match &result {
 Ok(_) => log(&state, "[Boost] Otimizações aplicadas"),
 Err(e) => log(&state, &format!("[Boost Erro] {e}")),
 }
 result
}

#[tauri::command]
pub async fn revert_system_optimizations(state: State<'_, AppState>) -> Result<bool, String> {
 require_auth(&state)?;
 let result = {
 let mut boost = state.boost.lock().unwrap();
 boost.revert_optimizations()
 };
 log(&state, "[Boost] Ajustes revertidos ao padrão");
 result
}

#[tauri::command]
pub async fn revert_registry(state: State<'_, AppState>) -> Result<usize, String> {
 require_auth(&state)?;
 let mut registry = state.registry.lock().unwrap();
 let n = registry.revert_all_tracked()?;
 log(&state, &format!("[Rollback] {n} valores restaurados"));
 Ok(n)
}

#[tauri::command]
pub async fn create_restore_point(state: State<'_, AppState>, description: String) -> Result<bool, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 let safe_description = powershell_single_quote(&description);
 let script = format!(
 "Enable-ComputerRestore -Drive ($env:SystemDrive + '\\') -ErrorAction SilentlyContinue; Checkpoint-Computer -Description '{safe_description}' -RestorePointType 'MODIFY_SETTINGS' -ErrorAction Stop"
 );
 let result = executor.execute_powershell(&script).await;
 log(&state, &format!("[Restore] Ponto de restauração: {}", if result.exit_code == 0 { "criado" } else { "falhou" }));
 Ok(result.exit_code == 0)
}

#[tauri::command]
pub async fn remove_bloatware(state: State<'_, AppState>, items: Vec<String>) -> Result<u32, String> {
 require_auth(&state)?;
 if items.is_empty() { return Ok(0); }
 let safe_patterns = items.iter().map(|p| powershell_single_quote(p)).collect::<Vec<_>>();
 let patterns = safe_patterns.join("','");
 let script = format!(
 "$patterns = @('{patterns}'); foreach ($p in $patterns) {{ Get-AppxPackage -Name \"*$p*\" -AllUsers -ErrorAction SilentlyContinue | Remove-AppxPackage -AllUsers -ErrorAction SilentlyContinue; Get-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue | Where-Object {{ $_.DisplayName -like \"*$p*\" }} | Remove-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue }}; Write-Output 'DEBLOAT_DONE'"
 );
 let executor = ProcessExecutor;
 let result = executor.execute_powershell(&script).await;
 log(&state, &format!("[Debloat] Remoção em lote concluída ({} pacotes)", items.len()));
 Ok(if result.stdout.contains("DEBLOAT_DONE") { items.len() as u32 } else { 0 })
}

fn powershell_single_quote(value: &str) -> String {
 value.replace('\'', "''")
}

#[tauri::command]
pub async fn disable_telemetry(state: State<'_, AppState>) -> Result<bool, String> {
 require_auth(&state)?;
 {
 let mut registry = state.registry.lock().unwrap();
 registry.set_dword(HKLM, r"SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry", 0)?;
 registry.set_dword(HKLM, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection", "AllowTelemetry", 0)?;
 registry.set_dword(HKCU, r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo", "Enabled", 0)?;
 }
 let executor = ProcessExecutor;
 let services = "foreach ($svc in @('DiagTrack','dmwappushservice','diagnosticshub.standardcollector.service')) { Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue; Set-Service -Name $svc -StartupType Disabled -ErrorAction SilentlyContinue }";
 executor.execute_powershell(services).await;
 let debloat = crate::services::debloat::DebloatService::new();
 debloat.disable_scheduled_telemetry_tasks().await?;
 log(&state, "[Debloat] Telemetria, serviços e tarefas de diagnóstico desativados");
 Ok(true)
}

#[tauri::command]
pub async fn detect_game(state: State<'_, AppState>, game_id: String) -> Result<GameDetectionResult, String> {
 let result = {
 let registry = state.registry.lock().unwrap();
 let games = GameOptimizerService::new();
 games.detect_game(&game_id, &registry)
 };
 log(&state, &format!("[Detection] {}: {}", game_id, if result.found { "encontrado" } else { "não encontrado" }));
 Ok(result)
}

#[tauri::command]
pub async fn optimize_game(state: State<'_, AppState>, game_id: String, exe_path: String) -> Result<bool, String> {
 require_auth(&state)?;
 let executor = ProcessExecutor;
 match game_id.to_lowercase().as_str() {
 "cs2" => {
 let script = {
 let mut registry = state.registry.lock().unwrap();
 GameOptimizerService::prepare_cs2(&mut registry, &exe_path)?
 };
 let result = executor.execute_powershell(&script).await;
 if result.exit_code != 0 { return Err(result.stderr); }
 if !exe_path.is_empty() {
 let path = std::path::Path::new(&exe_path);
 let cfg = path.parent().and_then(|p| p.ancestors().nth(3)).map(|p| p.join("csgo/cfg/autoexec.cfg")).ok_or("caminho de CS2 inválido")?;
 if let Some(parent) = cfg.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
 std::fs::write(&cfg, GameOptimizerService::generate_cs2_autoexec()).map_err(|e| e.to_string())?;
 }
 GameOptimizerService::boost_live_cs2(&executor).await;
 }
 "valorant" => {
 let script = {
 let mut registry = state.registry.lock().unwrap();
 GameOptimizerService::prepare_valorant(&mut registry, &exe_path)?
 };
 let result = executor.execute_powershell(&script).await;
 if result.exit_code != 0 { return Err(result.stderr); }
 }
 "lol" => {
 let script = {
 let mut registry = state.registry.lock().unwrap();
 let mut restore = state.restore.lock().unwrap();
 GameOptimizerService::prepare_lol(&mut registry, &mut restore, &exe_path)?
 };
 let result = executor.execute_powershell(&script).await;
 if result.exit_code != 0 { return Err(result.stderr); }
 }
 _ => return Err("jogo não suportado".into()),
 }
 log(&state, &format!("[Games] Perfil {game_id} aplicado"));
 Ok(true)
}

#[tauri::command]
pub async fn clear_cs2_shader_cache(state: State<'_, AppState>, exe_path: String) -> Result<(u64, u32), String> {
 require_auth(&state)?;
 let result = GameOptimizerService::clear_cs2_shader_cache(&exe_path)?;
 log(&state, &format!("[CS2 Cache] Shader cache limpo: {} bytes, {} arquivos", result.0, result.1));
 Ok(result)
}

#[tauri::command]
pub async fn revert_game(state: State<'_, AppState>, game_id: String, exe_path: String) -> Result<usize, String> {
 require_auth(&state)?;
 let mut registry = state.registry.lock().unwrap();
 let mut restore = state.restore.lock().unwrap();
 let restored = GameOptimizerService::revert_game(&mut registry, &mut restore, &game_id, &exe_path)?;
 log(&state, &format!("[Games] Perfil {game_id} revertido"));
 Ok(restored)
}

#[tauri::command]
pub fn get_backups(state: State<AppState>) -> Vec<crate::models::BackupRecord> {
 state.restore.lock().unwrap().get_backups().to_vec()
}

#[tauri::command]
pub fn restore_backup(state: State<AppState>, backup_id: String) -> Result<bool, String> {
 state.restore.lock().unwrap().restore_file(&backup_id)
}

#[tauri::command]
pub async fn generate_cs2_autoexec(state: State<'_, AppState>, cs2_exe_path: String) -> Result<String, String> {
 let path = std::path::Path::new(&cs2_exe_path);
 let cfg_dir = path
 .parent()
 .and_then(|p| p.ancestors().nth(3))
 .map(|p| p.join("csgo").join("cfg"));
 let cfg_dir = match cfg_dir { Some(d) => d, None => return Err("caminho inválido".into()) };
 std::fs::create_dir_all(&cfg_dir).map_err(|e| e.to_string())?;
 let autoexec = cfg_dir.join("autoexec.cfg");
 let content = GameOptimizerService::generate_cs2_autoexec();
 std::fs::write(&autoexec, &content).map_err(|e| e.to_string())?;
 log(&state, &format!("[CS2] autoexec.cfg gerado em {}", autoexec.display()));
 Ok(autoexec.display().to_string())
}

#[tauri::command]
pub async fn get_startup_items(state: State<'_, AppState>) -> Result<Vec<StartupItem>, String> {
 let mut items = Vec::new();
 if let Ok(key) = winreg::RegKey::predef(HKCU).open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
 for (name, _value) in key.enum_values().flatten() {
 if let Ok(v) = key.get_value::<String, _>(&name) {
 items.push(StartupItem { name, path: v, location: "HKCU Run".into() });
 }
 }
 }
 log(&state, &format!("[Startup] {} itens encontrados", items.len()));
 Ok(items)
}

#[derive(Serialize)]
pub struct UpdateCheckPayload {
 pub available: bool,
 pub version: Option<String>,
 pub title: Option<String>,
 pub changelog: Option<String>,
 pub installer_url: Option<String>,
 pub sha256: Option<String>,
}

#[tauri::command]
pub async fn check_updates(state: State<'_, AppState>) -> Result<UpdateCheckPayload, String> {
 let updater = UpdateService::new();
 let result = updater.check_for_updates(env!("CARGO_PKG_VERSION")).await;
 match result {
 Ok(Some(m)) => {
 log(&state, &format!("[Updater] Nova versão disponível: {}", m.version));
 Ok(UpdateCheckPayload {
 available: true,
 version: Some(m.version),
 title: Some(m.title),
 changelog: Some(m.changelog),
 installer_url: Some(m.installer_url),
 sha256: Some(m.sha256),
 })
 }
 Ok(None) => {
 log(&state, "[Updater] Aplicativo atualizado");
 Ok(UpdateCheckPayload { available: false, version: None, title: None, changelog: None, installer_url: None, sha256: None })
 }
 Err(e) => {
 log(&state, &format!("[Updater Erro] {e}"));
 Err(e)
 }
 }
}

#[tauri::command]
pub async fn apply_update(state: State<'_, AppState>, version: String, installer_url: String, sha256: String) -> Result<crate::services::update::DownloadOutcome, String> {
 let updater = UpdateService::new();
 let manifest = crate::services::update::VersionManifest {
  version: version.clone(),
  release_date: String::new(),
  title: String::new(),
  changelog: String::new(),
  installer_url,
  sha256,
  mandatory: false,
 };
 let outcome = updater.download_and_apply(&manifest).await?;
 log(&state, &format!("[Updater OK] Download concluído ({:.1} MB) e instalador v{} iniciado", outcome.bytes_downloaded as f64 / 1_048_576.0, version));
 Ok(outcome)
}

#[derive(Serialize)]
pub struct I18nPayload {
 pub language: String,
 pub strings: std::collections::HashMap<String, String>,
}

#[tauri::command]
pub fn get_i18n(state: State<AppState>) -> I18nPayload {
 let i18n = state.i18n.lock().unwrap();
 I18nPayload { language: i18n.current_language.clone(), strings: i18n.strings.clone() }
}

#[tauri::command]
pub fn set_language(state: State<AppState>, lang: String) -> I18nPayload {
 let mut i18n = state.i18n.lock().unwrap();
 i18n.set_language(&lang);
 I18nPayload { language: i18n.current_language.clone(), strings: i18n.strings.clone() }
}
