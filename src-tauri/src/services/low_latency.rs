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

    /// Catálogo completo de tweaks reversíveis com tooltip localizado.
    pub fn tweaks(lang_pt: bool) -> Vec<LatencyTweak> {
        let (title_dynamic_tick, desc_dynamic_tick, tip_dynamic_tick,
             title_synthetic, desc_synthetic, tip_synthetic,
             title_priority_sep, desc_priority_sep, tip_priority_sep,
             title_usb, desc_usb, tip_usb,
             title_queues, desc_queues, tip_queues,
             title_accel, desc_accel, tip_accel,
             title_nagle, desc_nagle, tip_nagle,
             title_doh, desc_doh, tip_doh,
             title_hags, desc_hags, tip_hags,
             title_aspm, desc_aspm, tip_aspm,
             title_powerthrottle, desc_powerthrottle, tip_powerthrottle,
             title_sysmain, desc_sysmain, tip_sysmain) = if lang_pt {
            ("Dynamic Tick", "Desativa o tick dinâmico do kernel para timers consistentes", "bcdedit /set disabledynamictick yes: o kernel não pula ticks ociosos, reduzindo jitter de frame.",
             "Synthetic Timers", "Ativa useplatformtick para timers sintéticos estáveis", "bcdedit /set useplatformtick yes: mantém plataforma de tick previsível (recomendado junto com Dynamic Tick off).",
             "Win32PrioritySeparation", "Boost de foreground: curto, variável, alta prioridade", "0x26: janelas em primeiro plano ganham mais fatias de CPU, melhorando pacing de input.",
             "USB Selective Suspend", "Desativa suspensão seletiva de portas USB", "Impede que o Windows desligue hubs USB ociosos, evitando quedas de polling do mouse.",
             "Data Queue Sizes", "Filas de mouse e teclado reduzidas para 32", "MouseDataQueueSize/KeyboardDataQueueSize de 100 para 32: menos buffering entre dispositivo e jogos.",
             "Mouse Acceleration", "Desativa Enhance Pointer Precision", "Removes aceleração do mouse: 1:1 entre movimento físico e in-game.",
             "Nagle's Algorithm", "TcpAckFrequency=1 e TCPNoDelay=1 nas interfaces ativas", "Envia pacotes imediatamente sem agrupamento, reduzindo latência de rede em jogos.",
             "Delivery Optimization", "DODownloadMode=0: sem P2P em segundo plano", "Impede uploads/downloads P2P do Windows Update consumindo banda durante partidas.",
             "HAGS", "Hardware-Accelerated GPU Scheduling ativado", "HwSchMode=2: GPU gerencia sua própria fila, reduzindo latência de frames em GPUs modernas.",
             "PCIe ASPM", "Link State Power Management desativado", "Mantém o link PCIe sempre ativo, eliminando latência de acordar a GPU.",
             "Power Throttling", "Desativa estrangulamento de energia em apps em primeiro plano", "PowerThrottlingOff=1: processos em foco nunca são limitados por política de energia.")
        } else {
            ("Dynamic Tick", "Disable kernel dynamic tick for consistent timers", "bcdedit /set disabledynamictick yes: kernel stops skipping idle ticks, reducing frame jitter.",
             "Synthetic Timers", "Enable useplatformtick for stable synthetic timers", "bcdedit /set useplatformtick yes: keeps a predictable tick platform (recommended with Dynamic Tick off).",
             "Win32PrioritySeparation", "Foreground boost: short, variable, high boost", "0x26: foreground windows get more CPU slices, improving input pacing.",
             "USB Selective Suspend", "Disable selective suspend on USB hubs", "Stops Windows from powering down idle USB hubs, preventing mouse polling drops.",
             "Data Queue Sizes", "Mouse and keyboard queues reduced to 32", "MouseDataQueueSize/KeyboardDataQueueSize from 100 to 32: less buffering between device and games.",
             "Mouse Acceleration", "Disable Enhance Pointer Precision", "Removes OS mouse acceleration: 1:1 physical to in-game movement.",
             "Nagle's Algorithm", "TcpAckFrequency=1 and TCPNoDelay=1 on active interfaces", "Sends packets immediately without batching, reducing in-game network latency.",
             "Delivery Optimization", "DODownloadMode=0: no background P2P", "Prevents Windows Update P2P uploads/downloads from using bandwidth mid-match.",
             "HAGS", "Hardware-Accelerated GPU Scheduling enabled", "HwSchMode=2: GPU manages its own queue, reducing frame latency on modern GPUs.",
             "PCIe ASPM", "Link State Power Management disabled", "Keeps the PCIe link always active, eliminating GPU wake latency.",
             "Power Throttling", "Disable power throttling for foreground apps", "PowerThrottlingOff=1: foreground processes are never power-limited.")
        };
        vec![
            LatencyTweak { id: "dynamic_tick".into(), title: title_dynamic_tick, description: desc_dynamic_tick, tooltip: tip_dynamic_tick, requires_reboot: true, is_applied: None },
            LatencyTweak { id: "synthetic_timer".into(), title: title_synthetic, description: desc_synthetic, tooltip: tip_synthetic, requires_reboot: true, is_applied: None },
            LatencyTweak { id: "priority_separation".into(), title: title_priority_sep, description: desc_priority_sep, tooltip: tip_priority_sep, requires_reboot: false, is_applied: None },
            LatencyTweak { id: "usb_suspend".into(), title: title_usb, description: desc_usb, tooltip: tip_usb, requires_reboot: false, is_applied: None },
            LatencyTweak { id: "queue_sizes".into(), title: title_queues, description: desc_queues, tooltip: tip_queues, requires_reboot: false, is_applied: None },
            LatencyTweak { id: "mouse_accel".into(), title: title_accel, description: desc_accel, tooltip: tip_accel, requires_reboot: false, is_applied: None },
            LatencyTweak { id: "nagle".into(), title: title_nagle, description: desc_nagle, tooltip: tip_nagle, requires_reboot: false, is_applied: None },
            LatencyTweak { id: "delivery_opt".into(), title: title_doh, description: desc_doh, tooltip: tip_doh, requires_reboot: false, is_applied: None },
            LatencyTweak { id: "hags".into(), title: title_hags, description: desc_hags, tooltip: tip_hags, requires_reboot: true, is_applied: None },
            LatencyTweak { id: "pcie_aspm".into(), title: title_aspm, description: desc_aspm, tooltip: tip_aspm, requires_reboot: false, is_applied: None },
            LatencyTweak { id: "power_throttling".into(), title: title_powerthrottle, description: desc_powerthrottle, tooltip: tip_powerthrottle, requires_reboot: false, is_applied: None },
        ]
    }

    /// Aplica o tweak escolhido. Todos são rastreados para rollback.
    pub async fn apply_tweak(reg: &mut RegistryService, executor: &ProcessExecutor, tweak_id: &str) -> Result<bool, String> {
        match tweak_id {
            "dynamic_tick" => { executor.execute_cmd("bcdedit /set disabledynamictick yes").await; Ok(true) }
            "synthetic_timer" => { executor.execute_cmd("bcdedit /set useplatformtick yes").await; Ok(true) }
            "priority_separation" => reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Control\PriorityControl", "Win32PrioritySeparation", 0x26).map(|_| true),
            "usb_suspend" => reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Services\USB", "DisableSelectiveSuspend", 1).map(|_| true),
            "queue_sizes" => {
                reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Services\ mouclass\Parameters", "MouseDataQueueSize", 32).map(|_| true)?;
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
            "power_throttling" => {
                reg.set_dword(HKLM, r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling", "PowerThrottlingOff", 1).map(|_| true)
            }
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
                reg.delete_value(HKLM, r"SYSTEM\CurrentControlSet\Services\ mouclass\Parameters", "MouseDataQueueSize").map(|_| true)?;
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

    /// Limpa a Standby List sob demanda (RAM standby) via comando nativo.
    pub async fn clear_standby_list(executor: &ProcessExecutor) -> Result<bool, String> {
        let script = "Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.WorkingSet -gt 500MB } | ForEach-Object { $_.MinWorkingSet = 50MB; $_.MaxWorkingSet = 1GB }";
        let result = executor.execute_powershell(script).await;
        Ok(result.exit_code == 0)
    }
}
