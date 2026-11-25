#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod services;

use commands::AppState;
use services::auth::AuthService;
use services::cleaner::CleanerService;
use services::debloat::DebloatService;
use services::i18n::I18nService;
use services::registry::RegistryService;
use services::restore::RestoreService;
use services::system_boost::SystemBoostService;

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            auth: std::sync::Mutex::new(AuthService::new()),
            registry: std::sync::Mutex::new(RegistryService::new()),
            restore: std::sync::Mutex::new(RestoreService::new()),
            cleaner: std::sync::Mutex::new(CleanerService::new()),
            boost: std::sync::Mutex::new(SystemBoostService::new()),
            debloat: std::sync::Mutex::new(DebloatService::new()),
            i18n: std::sync::Mutex::new(I18nService::new()),
            logs: std::sync::Mutex::new(Vec::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_logs,
            commands::clear_logs,
            commands::get_hpet_status,
            commands::set_hpet_state,
            commands::enable_ultimate_performance,
            commands::get_latency_tweaks,
            commands::apply_latency_tweak,
            commands::revert_latency_tweak,
            commands::clear_standby_list,
            commands::detect_poe,
            commands::apply_poe_profile,
            commands::clear_poe_shader_cache,
            commands::get_auth_state,
            commands::login,
            commands::register,
            commands::logout,
            commands::get_cleaner_items,
            commands::get_bloatware_items,
            commands::clean_item,
            commands::flush_dns,
            commands::apply_system_optimizations,
            commands::revert_system_optimizations,
            commands::revert_registry,
            commands::create_restore_point,
            commands::remove_bloatware,
            commands::disable_telemetry,
            commands::detect_game,
            commands::optimize_game,
            commands::revert_game,
            commands::get_backups,
            commands::restore_backup,
            commands::generate_cs2_autoexec,
            commands::get_startup_items,
            commands::check_updates,
            commands::apply_update,
            commands::get_i18n,
            commands::set_language
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o aplicativo Tauri");
}
