use crate::services::process_executor::ProcessExecutor;
use crate::services::registry::{HKCU, HKLM, RegistryService};

pub struct SystemBoostService {
    registry: RegistryService,
    executor: ProcessExecutor,
}

impl SystemBoostService {
    pub fn new() -> Self {
        Self { registry: RegistryService::new(), executor: ProcessExecutor }
    }

    pub async fn is_hpet_disabled() -> bool {
        let executor = ProcessExecutor;
        // cmd.exe engole as chaves de {current}: PowerShell com aspas simples.
        let result = executor.execute_powershell("bcdedit /enum '{current}'").await;
        if result.stdout.trim().is_empty() {
            // Fallback: enumeração simples do loader.
            let fallback = executor.execute_powershell("bcdedit").await;
            return fallback.stdout.to_ascii_lowercase().contains("disabledynamictick");
        }
        let lower = result.stdout.to_ascii_lowercase();
        let tick_active = lower.contains("useplatformtick") && lower.contains("yes");
        let dynamic_off = lower.contains("disabledynamictick") && lower.contains("yes");
        tick_active || dynamic_off
    }

    pub async fn set_hpet_state(disable: bool) -> Result<bool, String> {
        let executor = ProcessExecutor;
        let commands: &[&str] = if disable {
            &["bcdedit /deletevalue useplatformclock", "bcdedit /set useplatformclock false", "bcdedit /set useplatformtick yes", "bcdedit /set disabledynamictick yes"]
        } else {
            &["bcdedit /set useplatformclock true", "bcdedit /deletevalue useplatformtick", "bcdedit /deletevalue disabledynamictick"]
        };
        for command in commands { executor.execute_cmd(command).await; }
        let ps = if disable {
            "Get-PnpDevice -FriendlyName '*High precision event timer*' -ErrorAction SilentlyContinue | Disable-PnpDevice -Confirm:$false -ErrorAction SilentlyContinue"
        } else {
            "Get-PnpDevice -FriendlyName '*High precision event timer*' -ErrorAction SilentlyContinue | Enable-PnpDevice -Confirm:$false -ErrorAction SilentlyContinue"
        };
        executor.execute_powershell(ps).await;
        Ok(true)
    }

    pub async fn enable_ultimate_performance() -> Result<String, String> {
        let executor = ProcessExecutor;
        let result = executor.execute_cmd("powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61").await;
        let guid = extract_guid(&result.stdout).unwrap_or_else(|| "e9a42b02-d5df-448d-aa00-03f14749eb61".into());
        let active = executor.execute_cmd(&format!("powercfg /setactive {guid}")).await;
        if active.exit_code == 0 { Ok(guid) }
        else {
            executor.execute_cmd("powercfg /setactive 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c").await;
            Ok("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c".into())
        }
    }

    pub fn apply_optimizations(&mut self) -> Result<bool, String> {
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_Enabled", 0)?;
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_FSEBehaviorMode", 2)?;
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_HonorUserFSEBehaviorMode", 1)?;
        self.registry.set_dword(HKLM, r"SOFTWARE\Policies\Microsoft\Windows\GameDVR", "AllowGameDVR", 0)?;
        self.registry.set_dword(HKCU, r"Software\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled", 0)?;
        self.registry.set_dword(HKCU, r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications", "GlobalUserDisabled", 1)?;
        self.registry.set_dword(HKCU, r"Software\Microsoft\Windows\CurrentVersion\Search", "BackgroundAppGlobalToggle", 0)?;

        let sys = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
        self.registry.set_dword(HKLM, sys, "NetworkThrottlingIndex", 0xFFFFFFFF)?;
        self.registry.set_dword(HKLM, sys, "SystemResponsiveness", 0)?;
        let games = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";
        self.registry.set_dword(HKLM, games, "Affinity", 0)?;
        self.registry.set_string(HKLM, games, "Background Only", "False")?;
        self.registry.set_dword(HKLM, games, "Clock Rate", 10000)?;
        self.registry.set_dword(HKLM, games, "GPU Priority", 8)?;
        self.registry.set_dword(HKLM, games, "Priority", 6)?;
        self.registry.set_string(HKLM, games, "Scheduling Category", "High")?;
        self.registry.set_string(HKLM, games, "SFIO Priority", "High")?;
        Ok(true)
    }

    pub fn revert_optimizations(&mut self) -> Result<bool, String> {
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_Enabled", 1)?;
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_FSEBehaviorMode", 0)?;
        self.registry.set_dword(HKCU, r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications", "GlobalUserDisabled", 0)?;
        self.registry.set_dword(HKCU, r"Software\Microsoft\Windows\CurrentVersion\Search", "BackgroundAppGlobalToggle", 1)?;
        let sys = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
        self.registry.set_dword(HKLM, sys, "NetworkThrottlingIndex", 10)?;
        self.registry.set_dword(HKLM, sys, "SystemResponsiveness", 20)?;
        let games = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";
        self.registry.set_dword(HKLM, games, "GPU Priority", 8)?;
        self.registry.set_dword(HKLM, games, "Priority", 2)?;
        self.registry.set_string(HKLM, games, "Scheduling Category", "Medium")?;
        self.registry.set_string(HKLM, games, "SFIO Priority", "Normal")?;
        Ok(true)
    }
}

fn extract_guid(value: &str) -> Option<String> {
    value.split_whitespace().find(|part| {
        let p = part.trim_matches(|c: char| !c.is_ascii_hexdigit() && c != '-');
        p.len() == 36 && p.chars().filter(|c| *c == '-').count() == 4 && p.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
    }).map(|p| p.trim_matches(|c: char| !c.is_ascii_hexdigit() && c != '-').to_string())
}
