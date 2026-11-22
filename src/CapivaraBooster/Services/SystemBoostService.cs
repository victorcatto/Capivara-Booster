using System.IO;
using System.Text.RegularExpressions;
using CapivaraBooster.Models;
using Microsoft.Win32;

namespace CapivaraBooster.Services;

public record StartupItem(string Name, string Path, string Location);

public interface ISystemBoostService
{
    event Action<string>? LogMessage;

    Task<bool> IsHpetDisabledAsync();
    Task<bool> SetHpetStateAsync(bool disableHpet);
    Task<string> EnableUltimatePerformancePowerPlanAsync();
    Task<bool> ApplySystemOptimizationsAsync();
    Task<bool> RevertSystemOptimizationsAsync();
    Task<IReadOnlyList<StartupItem>> GetStartupItemsAsync();
}

public class SystemBoostService : ISystemBoostService
{
    public event Action<string>? LogMessage;
    private readonly IProcessExecutor _executor;
    private readonly IRegistryService _registryService;

    public SystemBoostService(IProcessExecutor executor, IRegistryService registryService)
    {
        _executor = executor;
        _registryService = registryService;
    }

    public async Task<bool> IsHpetDisabledAsync()
    {
        var result = await _executor.ExecuteCommandAsync("bcdedit /enum {current}");
        string output = result.StandardOutput;

        // In Windows BCD:
        // useplatformclock No or absent = HPET clock disabled
        // useplatformtick Yes = Synthetic timers enabled
        bool clockDisabled = output.Contains("useplatformclock        No", StringComparison.OrdinalIgnoreCase)
                             || !output.Contains("useplatformclock", StringComparison.OrdinalIgnoreCase);

        bool tickActive = output.Contains("useplatformtick         Yes", StringComparison.OrdinalIgnoreCase);

        return clockDisabled && tickActive;
    }

    public async Task<bool> SetHpetStateAsync(bool disableHpet)
    {
        if (disableHpet)
        {
            LogMessage?.Invoke("[HPET] Disabling High Precision Event Timer to reduce DPC latency...");

            await _executor.ExecuteCommandAsync("bcdedit /deletevalue useplatformclock");
            await _executor.ExecuteCommandAsync("bcdedit /set useplatformclock false");
            await _executor.ExecuteCommandAsync("bcdedit /set useplatformtick yes");
            await _executor.ExecuteCommandAsync("bcdedit /set disabledynamictick yes");

            string pnpScript = @"
Get-PnpDevice -FriendlyName '*High precision event timer*' -ErrorAction SilentlyContinue | Disable-PnpDevice -Confirm:$false -ErrorAction SilentlyContinue
";
            await _executor.ExecutePowerShellAsync(pnpScript);

            LogMessage?.Invoke("[HPET] HPET disabled in BCD boot configuration & Device Manager. Reboot recommended for changes to take full effect.");
            return true;
        }
        else
        {
            LogMessage?.Invoke("[HPET] Restoring High Precision Event Timer to system default...");

            await _executor.ExecuteCommandAsync("bcdedit /set useplatformclock true");
            await _executor.ExecuteCommandAsync("bcdedit /deletevalue useplatformtick");
            await _executor.ExecuteCommandAsync("bcdedit /deletevalue disabledynamictick");

            string pnpScript = @"
Get-PnpDevice -FriendlyName '*High precision event timer*' -ErrorAction SilentlyContinue | Enable-PnpDevice -Confirm:$false -ErrorAction SilentlyContinue
";
            await _executor.ExecutePowerShellAsync(pnpScript);

            LogMessage?.Invoke("[HPET] HPET restored to default.");
            return true;
        }
    }

    public async Task<string> EnableUltimatePerformancePowerPlanAsync()
    {
        LogMessage?.Invoke("[Power Plan] Unlocking and activating 'Ultimate Performance' power scheme...");

        var dupResult = await _executor.ExecuteCommandAsync("powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61");

        string guid = "e9a42b02-d5df-448d-aa00-03f14749eb61";
        var matches = Regex.Matches(dupResult.StandardOutput, @"([a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12})", RegexOptions.IgnoreCase);
        if (matches.Count > 0)
        {
            // Pick the latest duplicated GUID reported in output
            guid = matches[^1].Value;
        }

        var setResult = await _executor.ExecuteCommandAsync($"powercfg /setactive {guid}");
        if (setResult.IsSuccess)
        {
            LogMessage?.Invoke($"[Power Plan Success] Ultimate Performance power scheme activated ({guid}).");
            return guid;
        }

        // Fallback to standard High Performance GUID
        await _executor.ExecuteCommandAsync("powercfg /setactive 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c");
        LogMessage?.Invoke("[Power Plan] Activated High Performance scheme as fallback.");
        return "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
    }

    public async Task<bool> ApplySystemOptimizationsAsync()
    {
        return await Task.Run(() =>
        {
            LogMessage?.Invoke("[System Tweaks] Applying latency, network, and gaming optimizations...");

            // 1. Disable Game DVR and Fullscreen Optimizations globally
            _registryService.SetDword(RegistryHive.CurrentUser, @"System\GameConfigStore", "GameDVR_Enabled", 0);
            _registryService.SetDword(RegistryHive.CurrentUser, @"System\GameConfigStore", "GameDVR_FSEBehaviorMode", 2);
            _registryService.SetDword(RegistryHive.CurrentUser, @"System\GameConfigStore", "GameDVR_HonorUserFSEBehaviorMode", 1);
            _registryService.SetDword(RegistryHive.CurrentUser, @"System\GameConfigStore", "GameDVR_DXGIHonorFSEWindowsCompatible", 1);
            _registryService.SetDword(RegistryHive.CurrentUser, @"System\GameConfigStore", "GameDVR_EFSEFeatureFlags", 0);

            _registryService.SetDword(RegistryHive.LocalMachine, @"SOFTWARE\Policies\Microsoft\Windows\GameDVR", "AllowGameDVR", 0);
            _registryService.SetDword(RegistryHive.CurrentUser, @"Software\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled", 0);

            // 2. Disable Background Apps globally
            _registryService.SetDword(RegistryHive.CurrentUser, @"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications", "GlobalUserDisabled", 1);
            _registryService.SetDword(RegistryHive.CurrentUser, @"Software\Microsoft\Windows\CurrentVersion\Search", "BackgroundAppGlobalToggle", 0);

            // 3. Network Throttling Index & System Responsiveness
            string sysProfile = @"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
            _registryService.SetDword(RegistryHive.LocalMachine, sysProfile, "NetworkThrottlingIndex", unchecked((int)0xFFFFFFFF));
            _registryService.SetDword(RegistryHive.LocalMachine, sysProfile, "SystemResponsiveness", 0);

            // 4. Multimedia Games Scheduling Priority
            string gamesProfile = @"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";
            _registryService.SetDword(RegistryHive.LocalMachine, gamesProfile, "Affinity", 0);
            _registryService.SetString(RegistryHive.LocalMachine, gamesProfile, "Background Only", "False");
            _registryService.SetDword(RegistryHive.LocalMachine, gamesProfile, "Clock Rate", 10000);
            _registryService.SetDword(RegistryHive.LocalMachine, gamesProfile, "GPU Priority", 8);
            _registryService.SetDword(RegistryHive.LocalMachine, gamesProfile, "Priority", 6);
            _registryService.SetString(RegistryHive.LocalMachine, gamesProfile, "Scheduling Category", "High");
            _registryService.SetString(RegistryHive.LocalMachine, gamesProfile, "SFIO Priority", "High");

            LogMessage?.Invoke("[System Tweaks] All global performance and network tweaks applied successfully.");
            return true;
        });
    }

    public async Task<bool> RevertSystemOptimizationsAsync()
    {
        return await Task.Run(() =>
        {
            LogMessage?.Invoke("[System Tweaks] Reverting system tweaks to Windows defaults...");

            _registryService.SetDword(RegistryHive.CurrentUser, @"System\GameConfigStore", "GameDVR_Enabled", 1);
            _registryService.SetDword(RegistryHive.CurrentUser, @"System\GameConfigStore", "GameDVR_FSEBehaviorMode", 0);
            _registryService.DeleteValue(RegistryHive.LocalMachine, @"SOFTWARE\Policies\Microsoft\Windows\GameDVR", "AllowGameDVR");

            _registryService.SetDword(RegistryHive.CurrentUser, @"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications", "GlobalUserDisabled", 0);
            _registryService.SetDword(RegistryHive.CurrentUser, @"Software\Microsoft\Windows\CurrentVersion\Search", "BackgroundAppGlobalToggle", 1);

            string sysProfile = @"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
            _registryService.SetDword(RegistryHive.LocalMachine, sysProfile, "NetworkThrottlingIndex", 10);
            _registryService.SetDword(RegistryHive.LocalMachine, sysProfile, "SystemResponsiveness", 20);

            string gamesProfile = @"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";
            _registryService.SetDword(RegistryHive.LocalMachine, gamesProfile, "GPU Priority", 8);
            _registryService.SetDword(RegistryHive.LocalMachine, gamesProfile, "Priority", 2);
            _registryService.SetString(RegistryHive.LocalMachine, gamesProfile, "Scheduling Category", "Medium");
            _registryService.SetString(RegistryHive.LocalMachine, gamesProfile, "SFIO Priority", "Normal");

            LogMessage?.Invoke("[System Tweaks] System tweaks reverted to defaults.");
            return true;
        });
    }

    public async Task<IReadOnlyList<StartupItem>> GetStartupItemsAsync()
    {
        return await Task.Run(() =>
        {
            var list = new List<StartupItem>();
            string runKey = @"Software\Microsoft\Windows\CurrentVersion\Run";

            // 1. HKCU Run
            try
            {
                using var key = Registry.CurrentUser.OpenSubKey(runKey);
                if (key != null)
                {
                    foreach (var name in key.GetValueNames())
                    {
                        var val = key.GetValue(name)?.ToString() ?? string.Empty;
                        list.Add(new StartupItem(name, val, "HKCU Run"));
                    }
                }
            }
            catch { }

            // 2. HKLM Run (64-bit)
            try
            {
                using var baseKey = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64);
                using var key = baseKey.OpenSubKey(runKey);
                if (key != null)
                {
                    foreach (var name in key.GetValueNames())
                    {
                        var val = key.GetValue(name)?.ToString() ?? string.Empty;
                        list.Add(new StartupItem(name, val, "HKLM Run (64-bit)"));
                    }
                }
            }
            catch { }

            // 3. HKLM Run (32-bit WOW64)
            try
            {
                using var baseKey = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry32);
                using var key = baseKey.OpenSubKey(runKey);
                if (key != null)
                {
                    foreach (var name in key.GetValueNames())
                    {
                        var val = key.GetValue(name)?.ToString() ?? string.Empty;
                        if (!list.Any(x => x.Name == name))
                        {
                            list.Add(new StartupItem(name, val, "HKLM Run (32-bit)"));
                        }
                    }
                }
            }
            catch { }

            // 4. User Startup Folder (.lnk shortcuts)
            try
            {
                string startupFolder = Environment.GetFolderPath(Environment.SpecialFolder.Startup);
                if (Directory.Exists(startupFolder))
                {
                    foreach (var file in Directory.EnumerateFiles(startupFolder, "*.lnk"))
                    {
                        string name = Path.GetFileNameWithoutExtension(file);
                        list.Add(new StartupItem(name, file, "Startup Folder"));
                    }
                }
            }
            catch { }

            return (IReadOnlyList<StartupItem>)list.AsReadOnly();
        });
    }
}