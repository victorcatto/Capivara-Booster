use crate::models::BloatwareItem;

use crate::services::process_executor::ProcessExecutor;

pub struct DebloatService;

impl DebloatService {
 pub fn new() -> Self { Self }

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

 pub fn get_default_bloatware_list(&self) -> Vec<BloatwareItem> {
 vec![
 BloatwareItem {
 id: "cortana".into(),
 name: "Cortana".into(),
 package_pattern: "Microsoft.549981C3F5F10".into(),
 description: "Microsoft Cortana voice assistant".into(),
 category: "Productivity".into(),
 is_selected: true,
 is_removed: false,
 },
 BloatwareItem {
 id: "xboxgamebar".into(),
 name: "Xbox Game Bar".into(),
 package_pattern: "Microsoft.XboxGamingOverlay".into(),
 description: "Overhead Game Bar overlay and DVR hooks".into(),
 category: "Gaming".into(),
 is_selected: true,
 is_removed: false,
 },
 BloatwareItem {
 id: "weather".into(),
 name: "MSN Weather".into(),
 package_pattern: "Microsoft.BingWeather".into(),
 description: "Windows Bing Weather live tile".into(),
 category: "News & Info".into(),
 is_selected: true,
 is_removed: false,
 },
 ]
 }
}