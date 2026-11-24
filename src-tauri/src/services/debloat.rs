use crate::models::BloatwareItem;

pub struct DebloatService;

impl DebloatService {
 pub fn new() -> Self { Self }

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