use serde::Serialize;
use std::path::PathBuf;
use crate::services::process_executor::ProcessExecutor;
use crate::services::telemetry::{TelemetryService, LiveTelemetry};

/// Relatório Completo: estado atual do PC em um PDF na Área de Trabalho,
/// com marca Capivara Booster. PDF 1.4 gerado à mão (texto WinAnsi),
/// sem dependências externas: zero risco de quebrar com API de crate.

#[derive(Debug, Clone, Serialize)]
pub struct ReportSection {
    pub title: String,
    pub lines: Vec<String>,
}

pub struct ReportService;

const PAGE_W: f64 = 595.28; // A4 em pontos
const PAGE_H: f64 = 841.89;
const MARGIN: f64 = 48.0;
const LINE_H: f64 = 16.0;

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
Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3" | ForEach-Object { "DISK={0} livre: {1:N0} GB de {2:N0} GB" -f $_.DeviceID, ($_.FreeSpace/1GB), ($_.Size/1GB) }
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
            } else if let Some(rest) = line.strip_prefix("DISK=") {
                temp_lines.push(format!("Disco {}", rest));
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

        // Fluxo de conteúdo: cabeçalho da marca, seções e rodapé.
        let mut ops = String::new();
        let mut y = PAGE_H - MARGIN;
        let mut line = |ops: &mut String, x: f64, yy: f64, size: f64, font: char, text: &str| {
            let safe = text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
            ops.push_str(&format!("BT /{} {} Tf {} {} Td ({}) Tj ET\n", font, size, x, yy, safe));
        };

        // Faixa de marca: retângulo Cyber Mint escuro + título branco.
        ops.push_str(&format!("q 0.00 0.90 0.67 rg {} {} 547.28 70 re f Q\n", MARGIN, PAGE_H - 110.0));
        ops.push_str(&format!("q 0.03 0.07 0.09 rg {} {} 547.28 4 re f Q\n", MARGIN, PAGE_H - 114.0));
        line(&mut ops, MARGIN + 16.0, PAGE_H - 88.0, 24.0, 'F2', "CAPIVARA BOOSTER");
        line(&mut ops, MARGIN + 16.0, PAGE_H - 104.0, 10.5, 'F1', "Relatorio Completo do Sistema: performance e latencia");
        y = PAGE_H - 150.0;

        for section in &sections {
            line(&mut ops, MARGIN, y, 14.0, 'F2', &section.title);
            y -= LINE_H + 2.0;
            for l in &section.lines {
                if y < MARGIN + 40.0 { break; }
                line(&mut ops, MARGIN + 10.0, y, 10.5, 'F1', l);
                y -= LINE_H;
            }
            y -= 8.0;
        }

        line(&mut ops, MARGIN, MARGIN - 16.0, 8.5, 'F1', "Gerado pelo Capivara Booster: github.com/victorcatto/Capivara-Booster");

        let stream = ops;
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> /Contents 6 0 R >>"),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>",
            format!("<< /Length {} >>\nstream\n{}\nendstream", stream.len(), stream),
        ];

        let mut pdf = String::from("%PDF-1.4\n");
        let mut offsets = Vec::new();
        for (i, obj) in objects.iter().enumerate() {
            offsets.push(pdf.len() as u64);
            pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", i + 1, obj));
        }
        let xref_pos = pdf.len() as u64;
        pdf.push_str(&format!("xref\n0 {}\n", objects.len() + 1));
        pdf.push_str("0000000000 65535 f \n");
        for off in offsets {
            pdf.push_str(&format!("{:010} 00000 n \n", off));
        }
        pdf.push_str(&format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF", objects.len() + 1, xref_pos));

        std::fs::write(&path, pdf).map_err(|e| format!("falha ao salvar PDF: {e}"))?;
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
