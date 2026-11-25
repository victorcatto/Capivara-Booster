use serde::Serialize;
use crate::services::process_executor::ProcessExecutor;

#[derive(Debug, Clone, Serialize)]
pub struct HardwareProfile {
    pub motherboard_vendor: String,
    pub motherboard_product: String,
    pub bios_vendor: String,
    pub bios_version: String,
    pub cpu_name: String,
    pub gpu_names: Vec<String>,
    pub ram_gb: u64,
    pub os_name: String,
    pub os_build: String,
}

pub struct HardwareService;

impl HardwareService {
    pub fn new() -> Self { Self }

    /// Perfil completo do hardware: fonte única de verdade para o Driver Booster.
    /// Consultas separadas porque o PS 5.1 não serializa objetos aninhados com confiabilidade.
    pub async fn profile(&self, executor: &ProcessExecutor) -> Result<HardwareProfile, String> {
        let board = executor.execute_powershell(
            "Get-CimInstance Win32_BaseBoard | ForEach-Object { \"$($_.Manufacturer)|$($_.Product)\" }; Get-CimInstance Win32_BIOS | ForEach-Object { \"$($_.Manufacturer)|$($_.SMBIOSBIOSVersion)\" }",
        ).await;
        let cpu = executor.execute_powershell(
            "Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name",
        ).await;
        let gpu = executor.execute_powershell(
            "Get-CimInstance Win32_VideoController | ForEach-Object { $_.Name }",
        ).await;
        let ram = executor.execute_powershell(
            "[math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB, 0)",
        ).await;
        let os = executor.execute_powershell(
            "$o = Get-CimInstance Win32_OperatingSystem; \"$($o.Caption)|$($o.BuildNumber)\"",
        ).await;

        let board_lines: Vec<&str> = board.stdout.lines().filter(|l| !l.trim().is_empty()).collect();
        let (mb_vendor, mb_product) = board_lines.first()
            .map(|l| split_pipe(l))
            .unwrap_or(("Desconhecido".into(), "Desconhecido".into()));
        let (bios_vendor, bios_version) = board_lines.get(1)
            .map(|l| split_pipe(l))
            .unwrap_or(("Desconhecido".into(), "Desconhecido".into()));
        let (os_name, os_build) = os.stdout.trim()
            .split_once('|')
            .map(|(a, b)| (a.trim().to_string(), b.trim().to_string()))
            .unwrap_or(("Windows".into(), String::new()));

        Ok(HardwareProfile {
            motherboard_vendor: mb_vendor,
            motherboard_product: mb_product,
            bios_vendor,
            bios_version,
            cpu_name: cpu.stdout.trim().to_string(),
            gpu_names: gpu.stdout.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect(),
            ram_gb: ram.stdout.trim().parse().unwrap_or(0),
            os_name,
            os_build,
        })
    }

    /// URL de suporte correta para o hardware detectado (placa-mãe exata quando possível).
    pub fn support_url(&self, profile: &HardwareProfile) -> Option<(String, String)> {
        let vendor = profile.motherboard_vendor.to_lowercase();
        let product = profile.motherboard_product.trim();
        let base: Option<(&str, &str)> = if vendor.contains("asus") {
            Some(("ASUS", "https://www.asus.com/supportonly/"))
        } else if vendor.contains("gigabyte") || vendor.contains("gigabyte technology") {
            Some(("Gigabyte", "https://www.gigabyte.com/Motherboard/"))
        } else if vendor.contains("msi") {
            Some(("MSI", "https://www.msi.com/search?query="))
        } else if vendor.contains("asrock") {
            Some(("ASRock", "https://www.asrock.com/support/"))
        } else {
            None
        };
        base.map(|(name, url)| {
            // Busca pelo modelo exato da placa quando existe (ex.: TUF-GAMING-B650-PLUS).
            let slug = product.to_uppercase().replace(' ', "-").replace('/', "-");
            (name.to_string(), format!("{url}{slug}"))
        })
    }
}

fn split_pipe(line: &str) -> (String, String) {
    match line.trim().split_once('|') {
        Some((a, b)) => (a.trim().to_string(), b.trim().to_string()),
        None => (line.trim().to_string(), String::new()),
    }
}
