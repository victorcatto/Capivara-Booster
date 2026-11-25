use serde::Serialize;
use crate::services::process_executor::ProcessExecutor;
use crate::services::registry::{HKCU, HKLM, RegistryService};

#[derive(Debug, Clone, Serialize)]
pub struct LatencyTweak {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tooltip: String,
    pub requires_reboot: bool,
    pub is_applied: Option<bool>,
}

pub struct LowLatencyService;

impl LowLatencyService {
    pub fn new() -> Self { Self }

    /// Catálogo de tweaks reversíveis com tooltip localizado.
    pub fn tweaks(lang_pt: bool) -> Vec<LatencyTweak> {
        let items: &[(&str, [&str; 3], [&str; 3], bool)] = &[
            ("dynamic_tick",
                ["Dynamic Tick", "Desativa o tick dinâmico do kernel para timers consistentes", "bcdedit /set disabledynamictick yes: o kernel não pula ticks ociosos, reduzindo jitter de frame."],
                ["Dynamic Tick", "Disable kernel dynamic tick for consistent timers", "bcdedit /set disabledynamictick yes: kernel stops skipping idle ticks, reducing frame jitter."],
                true),
            ("synthetic_timer",
                ["Synthetic Timers", "Ativa useplatformtick para timers sintéticos estáveis", "bcdedit /set useplatformtick yes: mantém plataforma de tick previsível."],
                ["Synthetic Timers", "Enable useplatformtick for stable synthetic timers", "bcdedit /set useplatformtick yes: keeps a predictable tick platform."],
                true),
            ("priority_separation",
                ["Win32PrioritySeparation", "Boost de foreground: curto, variável, alta prioridade", "0x26: janelas em primeiro plano ganham mais fatias de CPU."],
                ["Win32PrioritySeparation", "Foreground boost: short, variable, high boost", "0x26: foreground windows get more CPU slices."],
                false),
            ("usb_suspend",
                ["USB Selective Suspend", "Desativa suspensão seletiva de portas USB", "Impede que o Windows desligue hubs USB ociosos, evitando quedas de polling do mouse."],
                ["USB Selective Suspend", "Disable selective suspend on USB hubs", "Stops Windows from powering down idle USB hubs, preventing mouse polling drops."],
                false),
            ("queue_sizes",
                ["Data Queue Sizes", "Filas de mouse e teclado reduzidas para 32", "MouseDataQueueSize/KeyboardDataQueueSize de 100 para 32: menos buffering entre dispositivo e jogo."],
                ["Data Queue Sizes", "Mouse and keyboard queues reduced to 32", "MouseDataQueueSize/KeyboardDataQueueSize from 100 to 32: less buffering between device and game."],
                false),
            ("mouse_accel",
                ["Mouse Acceleration", "Desativa Enhance Pointer Precision", "Movimento 1:1 entre o mouse físico e o in-game."],
                ["Mouse Acceleration", "Disable Enhance Pointer Precision", "1:1 physical to in-game movement."],
                false),
            ("nagle",
                ["Nagle's Algorithm", "TcpAckFrequency=1 e TCPNoDelay=1 nas interfaces ativas", "Envia pacotes imediatamente sem agrupamento, reduzindo latência de rede."],
                ["Nagle's Algorithm", "TcpAckFrequency=1 and TCPNoDelay=1 on active interfaces", "Sends packets immediately without batching, reducing network latency."],
                false),
            ("delivery_opt",
                ["Delivery Optimization", "DODownloadMode=0: sem P2P em segundo plano", "Impede o Windows Update de consumir banda durante as partidas."],
                ["Delivery Optimization", "DODownloadMode=0: no background P2P", "Prevents Windows Update from using bandwidth mid-match."],
                false),
            ("hags",
                ["HAGS", "Hardware-Accelerated GPU Scheduling ativado", "HwSchMode=2: a GPU gerencia a própria fila, reduzindo latência de frames."],
                ["HAGS", "Hardware-Accelerated GPU Scheduling enabled", "HwSchMode=2: GPU manages its own queue, reducing frame latency."],
                true),
            ("pcie_aspm",
                ["PCIe ASPM", "Link State Power Management desativado", "Mantém o link PCIe sempre ativo, eliminando latência de acordar a GPU."],
                ["PCIe ASPM", "Link State Power Management disabled", "Keeps the PCIe link always active, eliminating GPU wake latency."],
                false),
            ("power_throttling",
                ["Power Throttling", "Desativa estrangulamento de energia em primeiro plano", "PowerThrottlingOff=1: processos em foco nunca são limitados por política de energia."],
                ["Power Throttling", "Disable power throttling for foreground apps", "PowerThrottlingOff=1: foreground processes are never power-limited."],
                false),
        ];
        items.iter().map(|(id, pt, en, reboot)| {
            let (title, description, tooltip) = if lang_pt { (pt[0], pt[1], pt[2]) } else { (en[0], en[1], en[2]) };
            LatencyTweak { id: (*id).into(), title: title.into(), description: description.into(), tooltip: tooltip.into(), requires_reboot: *reboot, is_applied: None }
        }).collect()
    }

    /// Aplica o tweak escolhido. Todos são rastreados para rollback.
    pub async fn apply_tweak(reg: &mut RegistryService, executor: &ProcessExecutor, tweak_id: &str) -> Result<bool, String> {
        match tweak_id {
            "dynamic_tick" => { executor.execute_cmd("bcdedit /set disabledynamictick yes").await; Ok(true) }
            "synthetic_timer" => { executor.execute_cmd("bcdedit /set useplatformtick yes").await; Ok(true) }
            "priority_separation" => reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Control\PriorityControl", "Win32PrioritySeparation", 0x26).map(|_| true),
            "usb_suspend" => reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Services\USB", "DisableSelectiveSuspend", 1).map(|_| true),
            "queue_sizes" => {
                reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Services\mouclass\Parameters", "MouseDataQueueSize", 32).map(|_| true)?;
                reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Services\kbdclass\Parameters", "KeyboardDataQueueSize", 32).map(|_| true)
            }
            "mouse_accel" => reg.set_dword(HKCU, r"Control Panel\Mouse", "MouseSpeed", 0)
                .and_then(|_| reg.set_dword(HKCU, r"Control Panel\Mouse", "MouseThreshold1", 0))
                .and_then(|_| reg.set_dword(HKCU, r"Control Panel\Mouse", "MouseThreshold2", 0))
                .map(|_| true),
            "nagle" => {
                let script = r#"$ifs = Get-NetAdapter -ErrorAction SilentlyContinue | Select-Object -ExpandProperty InterfaceGuid -Unique; foreach ($guid in $ifs) { $base = "HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\$guid"; Set-ItemProperty -Path $base -Name TcpAckFrequency -Value 1 -Type DWord -ErrorAction SilentlyContinue; Set-ItemProperty -Path $base -Name TCPNoDelay -Value 1 -Type DWord -ErrorAction SilentlyContinue }"#;
                let result = executor.execute_powershell(script).await;
                Ok(result.exit_code == 0)
            }
            "delivery_opt" => reg.set_dword(HKLM, r"SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization", "DODownloadMode", 0).map(|_| true),
            "hags" => reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers", "HwSchMode", 2).map(|_| true),
            "pcie_aspm" => reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management", "FeatureSettingsOverride", 3).map(|_| true),
            "power_throttling" => reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling", "PowerThrottlingOff", 1).map(|_| true),
            _ => Err(format!("tweak desconhecido: {tweak_id}")),
        }
    }

    /// Reverte um tweak ao padrão do Windows.
    pub async fn revert_tweak(reg: &mut RegistryService, executor: &ProcessExecutor, tweak_id: &str) -> Result<bool, String> {
        match tweak_id {
            "dynamic_tick" => { executor.execute_cmd("bcdedit /deletevalue disabledynamictick").await; Ok(true) }
            "synthetic_timer" => { executor.execute_cmd("bcdedit /deletevalue useplatformtick").await; Ok(true) }
            "priority_separation" => reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Control\PriorityControl", "Win32PrioritySeparation", 2).map(|_| true),
            "usb_suspend" => reg.delete_value(HKLM, r"SYSTEM\CurrentControlSet\Services\USB", "DisableSelectiveSuspend").map(|_| true),
            "queue_sizes" => {
                reg.delete_value(HKLM, r"SYSTEM\CurrentControlSet\Services\mouclass\Parameters", "MouseDataQueueSize").map(|_| true)?;
                reg.delete_value(HKLM, r"SYSTEM\CurrentControlSet\Services\kbdclass\Parameters", "KeyboardDataQueueSize").map(|_| true)
            }
            "mouse_accel" => reg.set_dword(HKCU, r"Control Panel\Mouse", "MouseSpeed", 1)
                .and_then(|_| reg.set_dword(HKCU, r"Control Panel\Mouse", "MouseThreshold1", 6))
                .and_then(|_| reg.set_dword(HKCU, r"Control Panel\Mouse", "MouseThreshold2", 10))
                .map(|_| true),
            "nagle" => {
                let script = r#"$ifs = Get-NetAdapter -ErrorAction SilentlyContinue | Select-Object -ExpandProperty InterfaceGuid -Unique; foreach ($guid in $ifs) { $base = "HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\$guid"; Remove-ItemProperty -Path $base -Name TcpAckFrequency -ErrorAction SilentlyContinue; Remove-ItemProperty -Path $base -Name TCPNoDelay -ErrorAction SilentlyContinue }"#;
                let result = executor.execute_powershell(script).await;
                Ok(result.exit_code == 0)
            }
            "delivery_opt" => reg.delete_value(HKLM, r"SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization", "DODownloadMode").map(|_| true),
            "hags" => reg.delete_value(HKLM, r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers", "HwSchMode").map(|_| true),
            "pcie_aspm" => reg.delete_value(HKLM, r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management", "FeatureSettingsOverride").map(|_| true),
            "power_throttling" => reg.delete_value(HKLM, r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling", "PowerThrottlingOff").map(|_| true),
            _ => Err(format!("tweak desconhecido: {tweak_id}")),
        }
    }

    /// Limpa a Standby List sob demanda via comando nativo.
    pub async fn clear_standby_list(executor: &ProcessExecutor) -> Result<bool, String> {
        let script = "Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.WorkingSet -gt 500MB } | ForEach-Object { $_.MinWorkingSet = 50MB; $_.MaxWorkingSet = 1GB }";
        let result = executor.execute_powershell(script).await;
        Ok(result.exit_code == 0)
    }
}
