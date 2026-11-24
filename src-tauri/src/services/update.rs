use serde::Deserialize;

const VERSION_MANIFEST_URL: &str = "https://raw.githubusercontent.com/victorcatto/Capivara-Booster/main/version.json";

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
 pub version: String,
 #[serde(default)]
 pub releaseDate: String,
 #[serde(default)]
 pub title: String,
 #[serde(default)]
 pub changelog: String,
 #[serde(default)]
 pub installerUrl: String,
 #[serde(default)]
 pub sha256: String,
 #[serde(default)]
 pub mandatory: bool,
}

pub struct UpdateService;

impl UpdateService {
 pub fn new() -> Self { Self }

 pub async fn check_for_updates(&self, current_version: &str) -> Result<Option<VersionManifest>, String> {
 let client = reqwest::Client::builder()
 .user_agent("CapivaraBooster-Rust/2.0")
 .timeout(std::time::Duration::from_secs(15))
 .build()
 .map_err(|e| e.to_string())?;

 let manifest: VersionManifest = client
 .get(VERSION_MANIFEST_URL)
 .send()
 .await
 .map_err(|e| e.to_string())?
 .json()
 .await
 .map_err(|e| e.to_string())?;

 let current = parse_semver(current_version);
 let remote = parse_semver(&manifest.version);
 if remote > current {
 Ok(Some(manifest))
 } else {
 Ok(None)
 }
 }
}

fn parse_semver(v: &str) -> (u32, u32, u32) {
 let clean = v.trim().trim_start_matches(['v', 'V']);
 let base = clean.split(['-', '+']).next().unwrap_or("0");
 let parts: Vec<&str> = base.split('.').collect();
 (
 parts.first().and_then(|s| s.parse().ok()).unwrap_or(0),
 parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
 parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
 )
}