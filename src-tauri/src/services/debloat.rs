use crate::models::BloatwareItem;
use crate::services::process_executor::ProcessExecutor;

pub struct DebloatService;

impl DebloatService {
    pub fn new() -> Self { Self }

    /// Lista completa dos 15 pacotes UWP do WPF, com os mesmos padrões.
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
        ];
        packages.iter().map(|(id, name, pattern, description, selected)| BloatwareItem {
            id: (*id).into(), name: (*name).into(), package_pattern: (*pattern).into(),
            description: (*description).into(), category: "UWP".into(),
            is_selected: *selected, is_removed: false,
        }).collect()
    }

    pub async fn disable_scheduled_telemetry_tasks(&self) -> Result<bool, String> {
        let executor = ProcessExecutor;
        let script = r#"$tasks = @(
 '\Microsoft\Windows\Customer Experience Improvement Program\Consolidator',
 '\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip',
 '\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser',
 '\Microsoft\Windows\Application Experience\ProgramDataUpdater',
 '\Microsoft\Windows\Autochk\Proxy'
); foreach ($task in $tasks) { $parent = Split-Path $task -Parent; $leaf = Split-Path $task -Leaf; Disable-ScheduledTask -TaskPath ($parent + '\') -TaskName $leaf -ErrorAction SilentlyContinue }"#;
        let result = executor.execute_powershell(script).await;
        Ok(result.exit_code == 0)
    }
}
