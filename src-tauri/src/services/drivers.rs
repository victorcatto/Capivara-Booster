use serde::Serialize;
use crate::services::process_executor::ProcessExecutor;

#[derive(Debug, Clone, Serialize)]
pub struct DriverInfo {
    pub device_name: String,
    pub driver_version: String,
    pub driver_date: String,
    pub provider: String,
    pub manufacturer: String,
    pub status: String,
    pub download_url: Option<String>,
}

pub struct DriverService;

impl DriverService {
    pub fn new() -> Self { Self }

    /// Rastreia drivers via WMI e marca cada um comparando a data local
    /// com a data de corte do último driver conhecido do fabricante.
    /// O download fica sempre no site oficial do vendor (fonte confiável).
    pub async fn scan(&self, executor: &ProcessExecutor) -> Result<Vec<DriverInfo>, String> {
        let script = r#"Get-CimInstance Win32_PnPSignedDriver -ErrorAction SilentlyContinue | Where-Object { $_.DeviceName -and $_.DriverVersion } | Select-Object DeviceName, DriverVersion, DriverDate, DriverProviderName, Manufacturer | ConvertTo-Csv -NoTypeInformation"#;
        let result = executor.execute_powershell(script).await;
        if result.exit_code != 0 && result.stdout.is_empty() { return Err(result.stderr); }

        let vendors = Self::vendor_catalog();
        let mut drivers: Vec<DriverInfo> = Vec::new();
        for (index, line) in result.stdout.lines().enumerate() {
            if index == 0 || line.trim().is_empty() { continue; }
            let fields = parse_csv_line(line);
            if fields.len() < 5 || fields[0].trim().is_empty() { continue; }
            let device_name = fields[0].trim().to_string();
            let provider = fields[3].trim().to_string();
            let manufacturer = fields[4].trim().to_string();
            let driver_date = normalize_date(&fields[2]);

            // GPU/NIC/chipset são os que importam para jogos; o resto fica Outdated=Unknown.
            let vendor = vendors.iter().find(|v| {
                device_name.to_lowercase().contains(v.0) || provider.to_lowercase().contains(v.0) || manufacturer.to_lowercase().contains(v.0)
            });

            let (status, download_url) = match vendor {
                Some(v) => {
                    // Data de corte: releases posteriores são garantidamente mais novas.
                    let local = parse_date(&driver_date);
                    match local {
                        Some(d) if d >= v.3 => ("updated".into(), None),
                        _ => ("update_available".into(), Some(v.2.to_string())),
                    }
                }
                None => ("vendor_managed".into(), None),
            };

            drivers.push(DriverInfo {
                device_name,
                driver_version: fields[1].trim().to_string(),
                driver_date,
                provider,
                manufacturer,
                status,
                download_url,
            });
        }
        drivers.sort_by(|a, b| a.device_name.to_lowercase().cmp(&b.device_name.to_lowercase()));
        drivers.dedup_by(|a, b| a.device_name.eq_ignore_ascii_case(&b.device_name));
        Ok(drivers)
    }

    /// Catálogo vendor, URL oficial de download e data de corte do último driver conhecido.
    /// (chave de busca, nome, url, data de corte em (ano, mês, dia))
    fn vendor_catalog() -> Vec<(&'static str, &'static str, &'static str, (i32, u32, u32))> {
        vec![
            ("nvidia", "NVIDIA", "https://www.nvidia.com/Download/index.aspx", (2026, 11, 1)),
            ("advanced micro devices", "AMD", "https://www.amd.com/en/support", (2026, 11, 1)),
            ("radeon", "AMD", "https://www.amd.com/en/support", (2026, 11, 1)),
            ("intel", "Intel", "https://www.intel.com/content/www/us/en/download-center/home.html", (2026, 10, 1)),
            ("realtek", "Realtek", "https://www.realtek.com/Downloads", (2026, 9, 1)),
            ("asus", "ASUS", "https://www.asus.com/support/", (2026, 10, 1)),
            ("gigabyte", "Gigabyte", "https://www.gigabyte.com/Support", (2026, 10, 1)),
            ("msi", "MSI", "https://www.msi.com/support/", (2026, 10, 1)),
            ("asrock", "ASRock", "https://www.asrock.com/support/", (2026, 10, 1)),
        ]
    }

    /// Detecta o fabricante da placa-mãe para o card de links oficiais.
    pub async fn board_vendor_links(&self, executor: &ProcessExecutor) -> Result<Vec<(String, String)>, String> {
        let script = "Get-CimInstance Win32_BaseBoard | Select-Object -ExpandProperty Manufacturer; Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name";
        let result = executor.execute_powershell(script).await;
        let stdout = result.stdout.to_lowercase();
        let mut links = Vec::new();
        for (needle, name, url, _) in Self::vendor_catalog() {
            if stdout.contains(needle) && !links.iter().any(|(_, n): &(String, String)| n == name) {
                links.push((name.to_string(), url.to_string()));
            }
        }
        Ok(links)
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

/// WMI devolve 20261102000000.000000+000 ou datas locale-dependentes: extrai yyyy-MM-dd.
fn normalize_date(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 8 {
        let (y, m, d) = (digits[0..4].to_string(), digits[4..6].to_string(), digits[6..8].to_string());
        return format!("{y}-{m}-{d}");
    }
    raw.to_string()
}

fn parse_date(iso: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = iso.split('-').collect();
    if parts.len() != 3 { return None; }
    Some((parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?))
}
