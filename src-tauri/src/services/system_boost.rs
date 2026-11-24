use crate::services::registry::{HKLM, HKCU, RegistryService};
use crate::commands::ProcessExecutor;

pub struct SystemBoostService {
 registry: RegistryService,
}

impl SystemBoostService {
 pub fn new() -> Self {
 Self {
 registry: RegistryService::new(),
 }
 }

 pub async fn apply_optimizations(&mut self) -> Result<bool, String> {
 // Game DVR disable
 self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_Enabled", 0)?;
 self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_FSEBehaviorMode", 2)?;
 self.registry.set_dword(HKLM, r"SOFTWARE\Policies\Microsoft\Windows\GameDVR", "AllowGameDVR", 0)?;

 // Network throttling
 self.registry.set_dword(HKLM, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile", "NetworkThrottlingIndex", 0xFFFFFFFF)?;
 self.registry.set_dword(HKLM, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile", "SystemResponsiveness", 0)?;

 // Games scheduling priority
 let games = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";
 self.registry.set_dword(HKLM, games, "GPU Priority", 8)?;
 self.registry.set_dword(HKLM, games, "Priority", 6)?;
 self.registry.set_string(HKLM, games, "Scheduling Category", "High")?;

 Ok(true)
 }

 pub async fn revert_optimizations(&mut self) -> Result<bool, String> {
 self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_Enabled", 1)?;
 self.registry.set_dword(HKLM, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile", "NetworkThrottlingIndex", 10)?;
 self.registry.set_dword(HKLM, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile", "SystemResponsiveness", 20)?;
 Ok(true)
 }
}