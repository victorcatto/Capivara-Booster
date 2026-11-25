use serde::Serialize;
use crate::services::process_executor::ProcessExecutor;

#[derive(Debug, Clone, Serialize)]
pub struct DriverInfo {
    pub device_name: String,
    pub driver_version: String,
    pub driver_date: String,
    pub provider: String,
    pub manufacturer: String,
    pub needs_update: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DriverUpdate {
    pub device_name: String,
    pub download_url: String,
    pub vendor: String,
    pub note: String,
}

pub struct DriverService;

impl DriverService {
    pub fn new() -> Self { Self }

    /// Rastreia dispositivos com driver via WMI (Win32_PnPSignedDriver).
    pub async fn scan(&self, executor: &ProcessExecutor) -> Result<Vec<DriverInfo>, String> {
        let script = r#"Get-CimInstance Win32_PnPSignedDriver -ErrorAction SilentlyContinue | Where-Object { $_.DeviceName -and $_.DriverVersion } | Select-Object DeviceName, DriverVersion, DriverDate, DriverProviderName, Manufacturer | ConvertTo-Json -Compress"#;
        let result = executor.execute_powershell(script).await;
        if result.exit_code != 0 { return Err(result.stderr); }
        let mut drivers: Vec<DriverInfo> = Vec::new();
        let json = result.stdout.trim();
        if json.is_empty() { return Ok(drivers); }
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let items = if value.is_array() { value.as_array().unwrap().clone() } else { vec![value] };
        for item in items {
            let device_name = item.get("DeviceName").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if device_name.is_empty() { continue; }
            drivers.push(DriverInfo {
                device_name,
                driver_version: item.get("DriverVersion").and_then(|v| v.as_str()).unwrap_or("").into(),
                driver_date: item.get("DriverDate").and_then(|v| v.as_str()).unwrap_or("").into(),
                provider: item.get("DriverProviderName").and_then(|v| v.as_str()).unwrap_or("").into(),
                manufacturer: item.get("Manufacturer").and_then(|v| v.as_str()).unwrap_or("").into(),
                needs_update: None,
            });
        }
        drivers.sort_by(|a, b| a.device_name.to_lowercase().cmp(&b.device_name.to_lowercase()));
        drivers.dedup_by(|a, b| a.device_name.eq_ignore_ascii_case(&b.device_name));
        Ok(drivers)
    }

    /// Detecta o fabricante da placa-mãe e gera links oficiais de suporte.
    pub async fn board_vendor_links(&self, executor: &ProcessExecutor) -> Result<Vec<DriverUpdate>, String> {
        let script = "Get-CimInstance Win32_BaseBoard | Select-Object -ExpandProperty Manufacturer; Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name";
        let result = executor.execute_powershell(script).await;
        let stdout = result.stdout.to_lowercase();
        let mut updates = Vec::new();
        let vendors: &[(&str, &str, &str)] = &[
            ("asus", "ASUS", "https://www.asus.com/support/"),
            ("gigabyte", "Gigabyte", "https://www.gigabyte.com/Support"),
            ("msi", "MSI", "https://www.msi.com/support/"),
            ("asrock", "ASRock", "https://www.asrock.com/support/"),
            ("nvidia", "NVIDIA", "https://www.nvidia.com/Download/index.aspx"),
            ("amd", "AMD", "https://www.amd.com/en/support"),
            ("intel", "Intel", "https://www.intel.com/content/www/us/en/download-center/home.html"),
        ];
        for (needle, vendor, url) in vendors {
            if stdout.contains(needle) {
                updates.push(DriverUpdate {
                    device_name: format!("Placa-mãe / GPU {vendor}"),
                    download_url: url.to_string(),
                    vendor: vendor.to_string(),
                    note: "Baixe o driver oficial mais recente direto do fabricante.".into(),
                });
            }
        }
        Ok(updates)
    }

    /// Exporta backup dos drivers atuais (pnputil) para a pasta indicada.
    pub async fn export_backup(&self, executor: &ProcessExecutor, dest: &str) -> Result<String, String> {
        std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
        let quoted = dest.replace('\'', "''");
        let script = format!("pnputil /export-driver * '{}'", quoted);
        let result = executor.execute_powershell(&script).await;
        if result.exit_code != 0 { return Err(result.stderr); }
        Ok(dest.to_string())
    }
}
