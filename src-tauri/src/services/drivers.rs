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
    /// CSV em vez de JSON: ConvertTo-Json quebra com listas grandes no PS 5.1.
    pub async fn scan(&self, executor: &ProcessExecutor) -> Result<Vec<DriverInfo>, String> {
        let script = r#"Get-CimInstance Win32_PnPSignedDriver -ErrorAction SilentlyContinue | Where-Object { $_.DeviceName -and $_.DriverVersion } | Select-Object DeviceName, DriverVersion, DriverDate, DriverProviderName, Manufacturer | ConvertTo-Csv -NoTypeInformation"#;
        let result = executor.execute_powershell(script).await;
        if result.exit_code != 0 && result.stdout.is_empty() { return Err(result.stderr); }
        let mut drivers: Vec<DriverInfo> = Vec::new();
        for (index, line) in result.stdout.lines().enumerate() {
            if index == 0 || line.trim().is_empty() { continue; }
            let fields = parse_csv_line(line);
            if fields.len() < 5 { continue; }
            if fields[0].trim().is_empty() { continue; }
            drivers.push(DriverInfo {
                device_name: fields[0].trim().to_string(),
                driver_version: fields[1].trim().to_string(),
                driver_date: fields[2].trim().to_string(),
                provider: fields[3].trim().to_string(),
                manufacturer: fields[4].trim().to_string(),
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

/// Parser de CSV simples, tolerante a vírgulas dentro de aspas.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            if in_quotes && chars.peek() == Some(&'"') {
                current.push('"');
                chars.next();
            } else {
                in_quotes = !in_quotes;
            }
        } else if c == ',' && !in_quotes {
            fields.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
    }
    fields.push(current);
    fields
}
