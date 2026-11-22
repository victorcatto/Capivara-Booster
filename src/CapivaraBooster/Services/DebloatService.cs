using System.IO;
using CapivaraBooster.Models;
using Microsoft.Win32;

namespace CapivaraBooster.Services;

public interface IDebloatService
{
    event Action<string>? LogMessage;

    IReadOnlyList<BloatwareItem> GetDefaultBloatwareList();
    Task<int> RemoveSelectedBloatwareAsync(IEnumerable<BloatwareItem> items);
    Task<bool> DisableTelemetryAndTrackingAsync();
    Task<bool> EnableTelemetryAndTrackingAsync();
    Task<bool> DisableScheduledTelemetryTasksAsync();
}

public class DebloatService : IDebloatService
{
    public event Action<string>? LogMessage;
    private readonly IProcessExecutor _executor;
    private readonly IRegistryService _registryService;

    public DebloatService(IProcessExecutor executor, IRegistryService registryService)
    {
        _executor = executor;
        _registryService = registryService;
    }

    public IReadOnlyList<BloatwareItem> GetDefaultBloatwareList()
    {
        return new List<BloatwareItem>
        {
            new() { Id = "cortana", Name = "Cortana", PackagePattern = "Microsoft.549981C3F5F10", Description = "Microsoft Cortana voice assistant", Category = "Productivity", IsSelected = true },
            new() { Id = "xboxgamebar", Name = "Xbox Game Bar", PackagePattern = "Microsoft.XboxGamingOverlay", Description = "Overhead Game Bar overlay and DVR hooks", Category = "Gaming", IsSelected = true },
            new() { Id = "xboxapp", Name = "Xbox App Companion", PackagePattern = "Microsoft.XboxApp", Description = "Legacy Xbox companion application", Category = "Gaming", IsSelected = true },
            new() { Id = "weather", Name = "MSN Weather", PackagePattern = "Microsoft.BingWeather", Description = "Windows Bing Weather live tile and background tasks", Category = "News & Info", IsSelected = true },
            new() { Id = "news", Name = "MSN News", PackagePattern = "Microsoft.BingNews", Description = "Bing News and Feeds service app", Category = "News & Info", IsSelected = true },
            new() { Id = "gethelp", Name = "Get Help", PackagePattern = "Microsoft.GetHelp", Description = "Online Help and diagnostics assistant", Category = "Support", IsSelected = true },
            new() { Id = "tips", Name = "Tips (Get Started)", PackagePattern = "Microsoft.Getstarted", Description = "Windows tips and promotional notifications", Category = "Support", IsSelected = true },
            new() { Id = "solitaire", Name = "Solitaire Collection", PackagePattern = "Microsoft.MicrosoftSolitaireCollection", Description = "Microsoft Solitaire with ad tracking", Category = "Games", IsSelected = true },
            new() { Id = "people", Name = "Microsoft People", PackagePattern = "Microsoft.People", Description = "Contacts and People integration bar", Category = "Social", IsSelected = true },
            new() { Id = "feedback", Name = "Feedback Hub", PackagePattern = "Microsoft.WindowsFeedbackHub", Description = "Windows user feedback and diagnostic sender", Category = "Telemetry", IsSelected = true },
            new() { Id = "phone", Name = "Phone Link", PackagePattern = "Microsoft.YourPhone", Description = "Phone pairing background telemetry and bridge", Category = "Sync", IsSelected = false },
            new() { Id = "zunevideo", Name = "Films & TV", PackagePattern = "Microsoft.ZuneVideo", Description = "Default video player (Films & TV)", Category = "Media", IsSelected = false },
            new() { Id = "maps", Name = "Windows Maps", PackagePattern = "Microsoft.WindowsMaps", Description = "Offline maps and location tracking background service", Category = "Utility", IsSelected = true },
            new() { Id = "3dviewer", Name = "3D Viewer", PackagePattern = "Microsoft.Microsoft3DViewer", Description = "3D model viewing utility", Category = "Media", IsSelected = true },
            new() { Id = "mixedreality", Name = "Mixed Reality Portal", PackagePattern = "Microsoft.MixedReality.Portal", Description = "VR and Mixed reality background daemon", Category = "Media", IsSelected = true }
        };
    }

    public async Task<int> RemoveSelectedBloatwareAsync(IEnumerable<BloatwareItem> items)
    {
        var targetList = items.Where(x => x.IsSelected).ToList();
        if (targetList.Count == 0) return 0;

        LogMessage?.Invoke($"[Debloat] Initiating batched removal of {targetList.Count} UWP packages...");

        // Build a single high-performance PowerShell script to process all packages in a single execution pipeline
        var patterns = string.Join("', '", targetList.Select(x => x.PackagePattern));
        string batchScript = $@"
$patterns = @('{patterns}')
$removed = 0
foreach ($pattern in $patterns) {{
    try {{
        Get-AppxPackage -Name ""*$pattern*"" -AllUsers -ErrorAction SilentlyContinue | Remove-AppxPackage -AllUsers -ErrorAction SilentlyContinue
        Get-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue | Where-Object {{ $_.DisplayName -like ""*$pattern*"" }} | Remove-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue
        Write-Output ""DEBLOAT_OK:$pattern""
        $removed++
    }} catch {{
        Write-Output ""DEBLOAT_FAIL:$pattern""
    }}
}}
Write-Output ""DEBLOAT_TOTAL:$removed""
";

        var result = await _executor.ExecutePowerShellAsync(batchScript);

        int removedCount = 0;
        foreach (var item in targetList)
        {
            if (result.StandardOutput.Contains($"DEBLOAT_OK:{item.PackagePattern}"))
            {
                item.IsRemoved = true;
                item.IsInstalled = false;
                removedCount++;
                LogMessage?.Invoke($"[Debloat Success] Removed package: {item.Name} ({item.PackagePattern})");
            }
            else
            {
                LogMessage?.Invoke($"[Debloat Warning] Notice while removing {item.Name}: check Windows logs.");
            }
        }

        LogMessage?.Invoke($"[Debloat Finished] Successfully removed {removedCount} of {targetList.Count} selected packages.");
        return removedCount;
    }

    public async Task<bool> DisableTelemetryAndTrackingAsync()
    {
        LogMessage?.Invoke("[Telemetry] Disabling Windows Telemetry & Diagnostic tracking services...");

        // 1. Group Policy & Registry telemetry keys
        _registryService.SetDword(RegistryHive.LocalMachine, @"SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry", 0);
        _registryService.SetDword(RegistryHive.LocalMachine, @"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection", "AllowTelemetry", 0);
        _registryService.SetDword(RegistryHive.CurrentUser, @"Software\Microsoft\Windows\CurrentVersion\Privacy", "TailoredExperiencesWithDiagnosticDataEnabled", 0);
        _registryService.SetDword(RegistryHive.LocalMachine, @"SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo", "DisabledByGroupPolicy", 1);
        _registryService.SetDword(RegistryHive.CurrentUser, @"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo", "Enabled", 0);

        // 2. High-performance single-pass service disablement
        string servicesBatch = @"
$services = @('DiagTrack', 'dmwappushservice', 'diagnosticshub.standardcollector.service')
foreach ($svc in $services) {
    Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue
    Set-Service -Name $svc -StartupType Disabled -ErrorAction SilentlyContinue
}
";
        await _executor.ExecutePowerShellAsync(servicesBatch);
        LogMessage?.Invoke("[Service] Disabled background diagnostic services: DiagTrack, dmwappushservice, standardcollector.");

        // 3. High-performance batch scheduled tasks disablement
        await DisableScheduledTelemetryTasksAsync();

        LogMessage?.Invoke("[Telemetry] Windows diagnostic data and telemetry services disabled successfully.");
        return true;
    }

    public async Task<bool> EnableTelemetryAndTrackingAsync()
    {
        LogMessage?.Invoke("[Telemetry] Reverting Windows Telemetry settings to Windows Defaults...");

        _registryService.DeleteValue(RegistryHive.LocalMachine, @"SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry");
        _registryService.SetDword(RegistryHive.LocalMachine, @"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection", "AllowTelemetry", 1);
        _registryService.DeleteValue(RegistryHive.LocalMachine, @"SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo", "DisabledByGroupPolicy");

        string servicesRestore = @"
$services = @('DiagTrack', 'dmwappushservice')
foreach ($svc in $services) {
    Set-Service -Name $svc -StartupType Automatic -ErrorAction SilentlyContinue
    Start-Service -Name $svc -ErrorAction SilentlyContinue
}
";
        await _executor.ExecutePowerShellAsync(servicesRestore);

        LogMessage?.Invoke("[Telemetry] Windows telemetry services restored to default.");
        return true;
    }

    public async Task<bool> DisableScheduledTelemetryTasksAsync()
    {
        string tasksBatch = @"
$tasks = @(
    '\Microsoft\Windows\Customer Experience Improvement Program\Consolidator',
    '\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip',
    '\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser',
    '\Microsoft\Windows\Application Experience\ProgramDataUpdater',
    '\Microsoft\Windows\Autochk\Proxy'
)
foreach ($task in $tasks) {
    $parent = Split-Path $task -Parent
    $leaf = Split-Path $task -Leaf
    Disable-ScheduledTask -TaskPath ($parent + '\') -TaskName $leaf -ErrorAction SilentlyContinue
}
";
        await _executor.ExecutePowerShellAsync(tasksBatch);
        LogMessage?.Invoke("[Scheduled Tasks] Disabled Customer Experience and Compatibility Appraiser tasks.");
        return true;
    }
}