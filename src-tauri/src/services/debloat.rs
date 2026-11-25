use crate::models::BloatwareItem;
use crate::services::process_executor::ProcessExecutor;

pub struct DebloatService;

impl DebloatService {
    pub fn new() -> Self { Self }

    /// Lista completa de 25 pacotes UWP com padrões oficiais.
    /// selected: seguro remover sem quebrar recursos do Windows.
    pub fn get_default_bloatware_list(&self) -> Vec<BloatwareItem> {
        let packages: &[(&str, &str, &str, &str, bool)] = &[
            ("cortana", "Cortana", "Microsoft.549981C3F5F10", "Microsoft Cortana voice assistant", true),
            ("xboxgamebar", "Xbox Game Bar", "Microsoft.XboxGamingOverlay", "Overhead Game Bar overlay and DVR hooks", true),
            ("xboxapp", "Xbox App Companion", "Microsoft.XboxApp", "Legacy Xbox companion application", true),
            ("weather", "MSN Weather", "Microsoft.BingWeather", "Windows Bing Weather live tile and background tasks", true),
            ("news", "MSN News", "Microsoft.BingNews", "Bing News and Feeds service app", true),
            ("gethelp", "Get Help", "Microsoft.GetHelp", "Online Help and diagnostics assistant", true),
            ("tips", "Tips (Get Started)", "Microsoft.Getstarted", "Windows tips and promotional notifications", true),
            ("solitaire", "Solitaire Collection", "Microsoft.MicrosoftSolitaireCollection", "Microsoft Solitaire with ad tracking", true),
            ("people", "Microsoft People", "Microsoft.People", "Contacts and People integration bar", true),
            ("feedback", "Feedback Hub", "Microsoft.WindowsFeedbackHub", "Windows user feedback and diagnostic sender", true),
            ("phone", "Phone Link", "Microsoft.YourPhone", "Phone pairing background telemetry and bridge", false),
            ("zunevideo", "Films & TV", "Microsoft.ZuneVideo", "Default video player (Films & TV)", false),
            ("maps", "Windows Maps", "Microsoft.WindowsMaps", "Offline maps and location tracking background service", true),
            ("3dviewer", "3D Viewer", "Microsoft.Microsoft3DViewer", "3D model viewing utility", true),
            ("mixedreality", "Mixed Reality Portal", "Microsoft.MixedReality.Portal", "VR and Mixed reality background daemon", true),
            ("copilot", "Copilot", "Microsoft.Copilot", "AI assistant web view preinstalled on Windows 11", true),
            ("clipchamp", "Clipchamp", "Microsoft.Clipchamp", "Video editor with cloud upload prompts", true),
            ("onenote", "OneNote for Windows 10", "Microsoft.Office.OneNote", "Legacy OneNote UWP companion", true),
            ("camera", "Windows Camera", "Microsoft.WindowsCamera", "Camera app with OneDrive upload prompts", false),
            ("alarms", "Alarms & Clock", "Microsoft.WindowsAlarms", "Alarms, timers and world clock", false),
            ("sketchpad", "Snip & Sketch", "Microsoft.ScreenSketch", "Screenshot annotation tool", false),
            ("soundrecorder", "Sound Recorder", "Microsoft.SoundRecorder", "Voice recorder utility", false),
            ("spotify", "Spotify", "SpotifyAB.SpotifyMusic", "Preinstalled Spotify PWA wrapper", true),
            ("disney", "Disney+ EXP", "Disney.37853FC22B2CE", "Preinstalled Disney+ streaming app", true),
            ("tiktok", "TikTok", "BytedanceInc.TikTok", "Preinstalled TikTok PWA wrapper", true),
        ];
        packages.iter().map(|(id, name, pattern, description, selected)| BloatwareItem {
            id: (*id).into(), name: (*name).into(), package_pattern: (*pattern).into(),
            description: (*description).into(), category: "UWP".into(),
            is_selected: *selected, is_removed: false,
        }).collect()
    }

    /// Verifica quais padrões estão realmente instalados nesta máquina
    /// (a UI marca "Removido" a partir do estado real, não de memória).
    pub async fn query_installed(&self, executor: &ProcessExecutor, patterns: &[String]) -> Result<Vec<String>, String> {
        let safe_patterns = patterns.iter().map(|p| p.replace('\'', "''")).collect::<Vec<_>>();
        let joined = safe_patterns.join("','");
        let script = format!(
            r#"$patterns = @('{joined}')
$installed = @()
foreach ($p in $patterns) {{
  $found = Get-AppxPackage -Name "*$p*" -ErrorAction SilentlyContinue
  if ($found) {{ $installed += $p }}
}}
$installed -join '|'
"#);
        let result = executor.execute_powershell(&script).await;
        let out = result.stdout.trim();
        Ok(if out.is_empty() { Vec::new() } else { out.split('|').map(|s| s.to_string()).collect() })
    }

    /// Reinstala um pacote UWP do repositório do Windows (desfazer remoção).
    pub async fn restore_package(&self, executor: &ProcessExecutor, pattern: &str) -> Result<bool, String> {
        let safe = pattern.replace('\'', "''");
        let script = format!(
            r#"$ErrorActionPreference = 'Stop'
$pkg = Get-AppxPackage -Name "*{safe}*" -AllUsers -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $pkg) {{
  $prov = Get-AppxPackage -AllUsers -ErrorAction SilentlyContinue | Where-Object {{ $_.Name -like "*{safe}*" }} | Select-Object -First 1
  if ($prov) {{ Add-AppxPackage -Register -DisableDevelopmentMode ($prov.InstallLocation + "\AppxManifest.xml") -ErrorAction SilentlyContinue }}
}} else {{
  Add-AppxPackage -Register -DisableDevelopmentMode ($pkg.InstallLocation + "\AppxManifest.xml") -ErrorAction SilentlyContinue
}}
Write-Output 'RESTORE_DONE'
"#);
        let result = executor.execute_powershell(&script).await;
        Ok(result.stdout.contains("RESTORE_DONE"))
    }

    pub async fn disable_scheduled_telemetry_tasks(&self) -> Result<bool, String> {
        let executor = ProcessExecutor;
        let script = r#"$tasks = @(
 '\Microsoft\Windows\Customer Experience Improvement Program\Consolidator',
 '\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip',
 '\Microsoft\Windows\Customer Experience Improvement Program\KernelCeipTask',
 '\Microsoft\Windows\Customer Experience Improvement Program\Uploader',
 '\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser',
 '\Microsoft\Windows\Application Experience\ProgramDataUpdater',
 '\Microsoft\Windows\Application Experience\StartupAppTask',
 '\Microsoft\Windows\Autochk\Proxy',
 '\Microsoft\Windows\CloudExperienceHost\CreateObjectTask',
 '\Microsoft\Windows\Feedback\Siuf\DmClient'
); foreach ($task in $tasks) { $parent = Split-Path $task -Parent; $leaf = Split-Path $task -Leaf; Disable-ScheduledTask -TaskPath ($parent + '\') -TaskName $leaf -ErrorAction SilentlyContinue }"#;
        let result = executor.execute_powershell(script).await;
        Ok(result.exit_code == 0)
    }
}
