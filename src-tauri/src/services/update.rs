use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

const VERSION_MANIFEST_URL: &str = "https://raw.githubusercontent.com/victorcatto/Capivara-Booster/main/version.json";

#[derive(Debug, Clone, Deserialize)]
pub struct VersionManifest {
    pub version: String,
    #[serde(default, rename = "releaseDate")]
    pub release_date: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub changelog: String,
    #[serde(default, rename = "installerUrl")]
    pub installer_url: String,
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
            .build().map_err(|e| e.to_string())?;
        let response = client.get(VERSION_MANIFEST_URL).send().await.map_err(|e| e.to_string())?;
        if !response.status().is_success() { return Err(format!("manifesto de atualização retornou HTTP {}", response.status())); }
        let manifest: VersionManifest = response.json().await.map_err(|e| e.to_string())?;
        if parse_semver(&manifest.version) > parse_semver(current_version) { Ok(Some(manifest)) } else { Ok(None) }
    }

    pub async fn download_and_apply(&self, manifest: &VersionManifest) -> Result<(), String> {
        if manifest.installer_url.is_empty() { return Err("URL do instalador ausente no manifesto".into()); }
        let client = reqwest::Client::builder().user_agent("CapivaraBooster-Rust/2.0").build().map_err(|e| e.to_string())?;
        let response = client.get(&manifest.installer_url).send().await.map_err(|e| e.to_string())?;
        if !response.status().is_success() { return Err(format!("download retornou HTTP {}", response.status())); }
        let bytes = response.bytes().await.map_err(|e| e.to_string())?;
        if bytes.len() < 1024 { return Err("instalador baixado é pequeno demais".into()); }
        if !manifest.sha256.is_empty() {
            let actual = hex(&Sha256::digest(&bytes));
            if !actual.eq_ignore_ascii_case(&manifest.sha256) { return Err("SHA-256 do instalador não confere".into()); }
        }
        let path = std::env::temp_dir().join(format!("CapivaraBooster-Setup-v{}.exe", manifest.version));
        let mut file = tokio::fs::File::create(&path).await.map_err(|e| e.to_string())?;
        file.write_all(&bytes).await.map_err(|e| e.to_string())?;
        file.flush().await.map_err(|e| e.to_string())?;
        launch_elevated(&path)?;
        Ok(())
    }
}

#[cfg(windows)]
fn launch_elevated(path: &PathBuf) -> Result<(), String> {
    std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &format!("Start-Process -FilePath '{}' -Verb RunAs -ArgumentList '/SILENT','/CLOSEAPPLICATIONS','/RESTARTAPPLICATIONS'", ps_quote(&path.display().to_string()))])
        .spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
fn launch_elevated(_path: &PathBuf) -> Result<(), String> { Err("atualização automática só é suportada no Windows".into()) }

fn ps_quote(value: &str) -> String { value.replace('\'', "''") }
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn parse_semver(v: &str) -> (u32, u32, u32) {
    let base = v.trim().trim_start_matches(['v', 'V']).split(['-', '+']).next().unwrap_or("0");
    let parts: Vec<&str> = base.split('.').collect();
    (parts.first().and_then(|s| s.parse().ok()).unwrap_or(0), parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0), parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0))
}
