use crate::services::registry::{HKLM, HKCU, RegistryService};

pub struct SystemBoostService {
    registry: RegistryService,
}

impl SystemBoostService {
    pub fn new() -> Self {
        Self {
            registry: RegistryService::new(),
        }
    }

    pub fn apply_optimizations(&mut self) -> Result<bool, String> {
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_Enabled", 0)?;
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_FSEBehaviorMode", 2)?;
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_HonorUserFSEBehaviorMode", 1)?;
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_DXGIHonorFSEWindowsCompatible", 1)?;
        self.registry.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_EFSEFeatureFlags", 0)?;
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
