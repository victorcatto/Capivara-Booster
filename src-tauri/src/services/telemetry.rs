use serde::Serialize;
use crate::services::process_executor::ProcessExecutor;

#[derive(Debug, Clone, Serialize)]
pub struct LiveTelemetry {
    pub cpu_name: String,
    pub cpu_load_percent: u32,
    pub ram_total_gb: f64,
    pub ram_used_gb: f64,
    pub gpu_name: String,
    pub power_plan: String,
    pub uptime_hours: u64,
}

pub struct TelemetryService;

impl TelemetryService {
    pub fn new() -> Self { Self }

    /// Telemetria ao vivo em UMA consulta só (menos janelas de processo, mais rápido).
    /// CSV: o PS 5.1 corrompe JSON grande; uma linha pipe-separada é estável.
    pub async fn snapshot(&self, executor: &ProcessExecutor) -> Result<LiveTelemetry, String> {
        let script = r#"
$cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1)
$load = [math]::Round($cpu.LoadPercentage, 0)
$os = Get-CimInstance Win32_OperatingSystem
$ramTotal = [math]::Round($os.TotalVisibleMemorySize / 1MB, 1)
$ramFree = [math]::Round($os.FreePhysicalMemory / 1MB, 1)
$gpu = (Get-CimInstance Win32_VideoController | Select-Object -First 1).Name
$plan = (powercfg /getactivescheme) -replace '.*:\s+', ''
$uptimeH = [math]::Round(($os.LocalDateTime - $os.LastBootUpTime).TotalHours, 0)
Write-Output "$($cpu.Name)|$load|$ramTotal|$ramFree|$gpu|$plan|$uptimeH"
"#;
        let result = executor.execute_powershell(script).await;
        let line = result.stdout.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        let f: Vec<&str> = line.split('|').collect();
        if f.len() < 7 { return Err("telemetria indisponível".into()); }
        Ok(LiveTelemetry {
            cpu_name: f[0].trim().to_string(),
            cpu_load_percent: f[1].trim().parse().unwrap_or(0),
            ram_total_gb: f[2].trim().parse().unwrap_or(0.0),
            ram_used_gb: (f[2].trim().parse::<f64>().unwrap_or(0.0) - f[3].trim().parse::<f64>().unwrap_or(0.0)).max(0.0),
            gpu_name: f[4].trim().to_string(),
            power_plan: f[5].trim().to_string(),
            uptime_hours: f[6].trim().parse().unwrap_or(0),
        })
    }

    /// Limpa a Standby List sem ferramenta externa (RAM em cache).
    pub async fn clear_standby(executor: &ProcessExecutor) -> Result<bool, String> {
        let script = r#"
$sig = '[DllImport("ntdll.dll")] public static extern uint NtSetSystemInformation(int InfoClass, ref int Info, int Length);'
$type = Add-Type -MemberDefinition $sig -Name 'Win32Mem' -Namespace 'Kernel' -PassThru
$info = 0
[void]$type::NtSetSystemInformation(80, [ref]$info, 4)
Write-Output 'STANDBY_CLEARED'
"#;
        let result = executor.execute_powershell(script).await;
        Ok(result.stdout.contains("STANDBY_CLEARED"))
    }
}
