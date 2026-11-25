use serde::Serialize;
use std::path::PathBuf;
use crate::services::process_executor::ProcessExecutor;
use crate::services::telemetry::{TelemetryService, LiveTelemetry};

/// Relatório Completo: estado atual do PC em um PDF na Área de Trabalho,
/// com marca Capivara Booster (paleta Cyber Mint / Volt Cyan).

#[derive(Debug, Clone, Serialize)]
pub struct ReportSection {
    pub title: String,
    pub lines: Vec<String>,
}

pub struct ReportService;

impl ReportService {
    pub fn new() -> Self { Self }

    pub async fn collect(&self, executor: &ProcessExecutor) -> Result<Vec<ReportSection>, String> {
        let mut sections = Vec::new();

        let telemetry: LiveTelemetry = TelemetryService::new().snapshot(executor).unwrap_or(LiveTelemetry {
            cpu_name: "N/A".into(), cpu_load_percent: 0, ram_total_gb: 0.0, ram_used_gb: 0.0,
            gpu_name: "N/A".into(), power_plan: "N/A".into(), uptime_hours: 0,
        });
        sections.push(ReportSection {
            title: "PROCESSADOR E MEMORIA".into(),
            lines: vec![
                format!("CPU: {}", telemetry.cpu_name),
                format!("Carga atual da CPU: {}%", telemetry.cpu_load_percent),
                format!("RAM em uso: {:.1} GB de {:.1} GB", telemetry.ram_used_gb, telemetry.ram_total_gb),
                format!("Uptime: {} horas", telemetry.uptime_hours),
            ],
        });

        let temps = executor.execute_powershell(
            r#"
$cpu = Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction SilentlyContinue | Select-Object -First 1
if ($cpu) { $ct = [math]::Round(($cpu.CurrentTemperature / 10) - 273.15, 1); Write-Output "CPU_TEMP=$ct" } else { Write-Output "CPU_TEMP=N/A" }
$g = Get-CimInstance Win32_VideoController | Select-Object -First 1
if ($g) { Write-Output ("GPU_DRIV=" + $g.DriverVersion); Write-Output ("GPU_DATE=" + $g.DriverDate) }
$disk = Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3" | ForEach-Object { "{0} livre: {1:N0} GB de {2:N0} GB" -f $_.DeviceID, ($_.FreeSpace/1GB), ($_.Size/1GB) }
$disk
"#,
        ).await;
        let mut temp_lines = Vec::new();
        for line in temps.stdout.lines() {
            if line.starts_with("CPU_TEMP=") {
                temp_lines.push(format!("Temperatura CPU: {}", line.trim_start_matches("CPU_TEMP=").replace("N/A", "indisponivel (requer sensor do fabricante)")));
            } else if line.starts_with("GPU_DRIV=") {
                temp_lines.push(format!("Driver GPU: {}", line.trim_start_matches("GPU_DRIV=")));
            } else if line.starts_with("GPU_DATE=") {
                temp_lines.push(format!("Data do driver GPU: {}", line.trim_start_matches("GPU_DATE=")));
            } else if line.contains("livre:") {
                temp_lines.push(format!("Disco {}", line));
            }
        }
        if !temp_lines.is_empty() {
            sections.push(ReportSection { title: "TEMPERATURAS, GPU E DISCOS".into(), lines: temp_lines });
        }

        let hpet = executor.execute_powershell("bcdedit").await;
        let lower = hpet.stdout.to_ascii_lowercase();
        let hpet_state = if lower.contains("disabledynamictick") && lower.contains("yes") { "otimizado (dynamic tick off)" } else { "padrao do Windows" };
        sections.push(ReportSection {
            title: "ENERGIA E LATENCIA".into(),
            lines: vec![
                format!("Plano de energia ativo: {}", telemetry.power_plan),
                format!("Timers do sistema: {}", hpet_state),
            ],
        });

        let hw = crate::services::hardware::HardwareService::new().profile(executor).await.unwrap_or(crate::services::hardware::HardwareProfile {
            motherboard_vendor: "N/A".into(), motherboard_product: "N/A".into(), bios_vendor: "N/A".into(),
            bios_version: "N/A".into(), cpu_name: "N/A".into(), gpu_names: vec![], ram_gb: 0, os_name: "N/A".into(), os_build: "N/A".into(),
        });
        sections.push(ReportSection {
            title: "PLATAFORMA".into(),
            lines: vec![
                format!("Placa-mae: {} {}", hw.motherboard_vendor, hw.motherboard_product),
                format!("BIOS: {} {}", hw.bios_vendor, hw.bios_version),
                format!("Windows: {} (build {})", hw.os_name, hw.os_build),
                format!("GPU(s): {}", if hw.gpu_names.is_empty() { "N/A".into() } else { hw.gpu_names.join(", ") }),
            ],
        });

        Ok(sections)
    }

    pub async fn generate_pdf(&self, executor: &ProcessExecutor) -> Result<PathBuf, String> {
        let sections = self.collect(executor).await?;
        let desktop = Self::desktop_dir();
        std::fs::create_dir_all(&desktop).map_err(|e| e.to_string())?;
        let path = desktop.join(format!("CapivaraBooster-Relatorio-{}.pdf", chrono_stamp()));

        // printpdf 0.7: pages e text builder nativos (sem manipular estruturas internas).
        let (mut doc, page1, regular) = printpdf::PdfDocument::new(
            "Capivara Booster - Relatorio Completo do Sistema",
            printpdf::mm(210.0),
            printpdf::mm(297.0),
        );
        let bold = doc.add_builtin_font(printpdf::BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;
        let font_regular = doc.add_builtin_font(printpdf::BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
        let _ = regular;

        let mut writer = doc.start_page();
        writer.set_font(&font_regular, 24.0);
        writer.left(18.0, printpdf::mm(229.0), printpdf::mm(60.0), "CAPIVARA BOOSTER");
        writer.set_font(&font_regular, 10.0);
        writer.left(18.0, printpdf::mm(110.0), printpdf::mm(184.0), "Relatorio Completo do Sistema: performance e latencia");
        let mut y = 265.0;
        for section in &sections {
            writer.set_font(&bold, 13.0);
            writer.left(18.0, printpdf::mm(0.0), printpdf::mm(0.0), &section.title);
            y -= 10.0;
            writer.set_font(&font_regular, 10.0);
            for line in &section.lines {
                writer.left(18.0, printpdf::mm(0.0), printpdf::mm(0.0), line);
                y -= 7.0;
            }
            y -= 4.0;
        }
        writer.set_font(&font_regular, 8.0);
        writer.left(18.0, printpdf::mm(120.0), printpdf::mm(120.0), "Gerado pelo Capivara Booster (github.com/victorcatto/Capivara-Booster)");
        let _ = (page1, &mut y);
        doc.save(&path).map_err(|e| format!("falha ao salvar PDF: {e}"))?;
        Ok(path)
    }

    fn desktop_dir() -> PathBuf {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders") {
            if let Ok(raw) = key.get_value::<String, _>("Desktop") {
                let expanded = std::env::var("USERPROFILE").map(|u| raw.replace("%USERPROFILE%", &u)).unwrap_or(raw);
                let p = PathBuf::from(expanded);
                if p.exists() { return p; }
            }
        }
        std::env::var("USERPROFILE").map(|u| PathBuf::from(u).join("Desktop")).unwrap_or_else(|_| PathBuf::from(r"C:\Users\Public\Desktop"))
    }
}

fn chrono_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86400;
    let (y, m, d) = civil_from_days(days as i64);
    let time = secs % 86400;
    format!("{y:04}-{m:02}-{d:02}_{:02}{:02}", time / 3600, (time % 3600) / 60)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365*yoe + yoe/4 - yoe/100);
    let mp = (5*doy + 2)/153;
    let d = (doy - (153*mp+2)/5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
