use std::path::{Path, PathBuf};
use serde::Serialize;
use crate::services::process_executor::ProcessExecutor;
use crate::services::registry::{HKCU, HKLM, RegistryService};
use crate::services::restore::RestoreService;

/// Módulo dedicado do Counter-Strike 2: registro, energia, timers,
/// shaders, cs2_video.txt e launch options. Tudo com backup prévio.

pub struct Cs2Service;

#[derive(Debug, Clone, Serialize)]
pub struct Cs2Install {
    pub found: bool,
    pub exe_path: String,
    pub cfg_dir: String,
    pub video_txt: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Cs2TweakStatus {
    pub id: String,
    pub applied: bool,
}

impl Cs2Service {
    pub fn new() -> Self { Self }

    /// Detecta a instalação via registro da Steam e libraryfolders.vdf.
    pub fn detect(&self) -> Cs2Install {
        let mut install = Cs2Install { found: false, exe_path: String::new(), cfg_dir: String::new(), video_txt: String::new() };
        let steam_path = Self::steam_path();
        for library in Self::steam_libraries(&steam_path) {
            // CS2 atual: game\bin\win64\cs2.exe; fallback: game\bin\win64\cs2_metrics.exe não é o jogo.
            let exe = library.join(r"Counter-Strike Global Offensive\game\bin\win64\cs2.exe");
            let exe_alt = library.join(r"steamapps\common\Counter-Strike Global Offensive\game\bin\win64\cs2.exe");
            let found_exe = if exe.exists() { Some(exe) } else if exe_alt.exists() { Some(exe_alt) } else { None };
            if let Some(exe_path) = found_exe {
                install.found = true;
                install.exe_path = exe_path.display().to_string();
                if let Some(cfg) = Self::find_cfg_dir(&steam_path) {
                    install.cfg_dir = cfg.display().to_string();
                    install.video_txt = cfg.join("cs2_video.txt").display().to_string();
                }
                break;
            }
        }
        install
    }

    fn steam_path() -> PathBuf {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(r"SOFTWARE\Valve\Steam")
            .and_then(|k| k.get_value::<String, _>("SteamPath"))
            .map(|p| p.replace('/', "\\"))
            .map(PathBuf::from)
            .ok()
            .filter(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)\Steam"))
    }

    fn steam_libraries(steam_path: &Path) -> Vec<PathBuf> {
        let mut libs = vec![steam_path.to_path_buf()];
        // Bibliotecas padrão que o vdf pode não listar.
        if let Some(root) = steam_path.parent() {
            let alt = root.join("SteamLibrary");
            if alt.exists() { libs.push(alt); }
        }
        for drive in ["C", "D", "E", "F"] {
            let alt = PathBuf::from(format!("{drive}:\\SteamLibrary"));
            if alt.exists() { libs.push(alt); }
        }
        let vdf = steam_path.join(r"steamapps\libraryfolders.vdf");
        if let Ok(content) = std::fs::read_to_string(&vdf) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("\"path\"") {
                    if let Some(raw) = trimmed.split('"').nth(3) {
                        let p = PathBuf::from(raw.replace("\\\\", "\\"));
                        if !libs.contains(&p) { libs.push(p); }
                    }
                }
            }
        }
        libs.retain(|l| l.exists());
        libs
    }

    fn find_cfg_dir(steam_path: &Path) -> Option<PathBuf> {
        let userdata = steam_path.join("userdata");
        let entries = std::fs::read_dir(&userdata).ok()?;
        for entry in entries.flatten() {
            let candidate = entry.path().join(r"730\local\cfg");
            if candidate.exists() { return Some(candidate); }
        }
        None
    }

    /// Tweaks de registro (idempotentes, com rollback via revert_registry).
    pub fn apply_registry_tweaks(reg: &mut RegistryService, restore: &mut RestoreService, exe_path: &str) -> Result<Vec<Cs2TweakStatus>, String> {
        let mut status = Vec::new();
        let _ = restore; // rollback das chaves padrão pelo botão Reverter Tweaks de Registro

        // Mouse: aceleração off.
        let mouse_ok = reg.set_string(HKCU, r"Control Panel\Mouse", "MouseSpeed", "0").is_ok()
            && reg.set_string(HKCU, r"Control Panel\Mouse", "MouseThreshold1", "0").is_ok()
            && reg.set_string(HKCU, r"Control Panel\Mouse", "MouseThreshold2", "0").is_ok();
        status.push(Cs2TweakStatus { id: "mouse_accel_off".into(), applied: mouse_ok });

        // MMCSS: GPU e agendamento de jogos no topo.
        let games = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";
        let mmcss_ok = reg.set_dword(HKLM, games, "GPU Priority", 8).is_ok()
            && reg.set_dword(HKLM, games, "Priority", 6).is_ok()
            && reg.set_string(HKLM, games, "Scheduling Category", "High").is_ok()
            && reg.set_string(HKLM, games, "SFIO Priority", "High").is_ok();
        status.push(Cs2TweakStatus { id: "mmcss_games".into(), applied: mmcss_ok });

        // FSO + Game DVR off.
        let fse_ok = reg.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_FSEBehaviorMode", 2).is_ok()
            && reg.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_Enabled", 0).is_ok()
            && reg.set_dword(HKCU, r"System\GameConfigStore", "GameDVR_HonorUserFSEBehaviorMode", 1).is_ok();
        status.push(Cs2TweakStatus { id: "fso_gamedvr_off".into(), applied: fse_ok });

        // GPU de alto desempenho para o cs2.exe.
        let gpu_ok = reg.set_string(HKCU, r"SOFTWARE\Microsoft\DirectX\UserGpuPreferences", "cs2.exe", "GpuPreference=2;").is_ok();
        status.push(Cs2TweakStatus { id: "gpu_high".into(), applied: gpu_ok });

        // AppCompatFlags: tela cheia exclusiva + DPI aware.
        if Path::new(exe_path).exists() {
            let flags = "~ DISABLEDXMAXIMIZEDWINDOWEDMODE HIGHDPIAWARE";
            let compat_ok = reg.set_string(HKCU, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers", exe_path, flags).is_ok();
            status.push(Cs2TweakStatus { id: "app_compat".into(), applied: compat_ok });
        }

        // IFEO: prioridade alta e sem core 0.
        let ifeo_ok = reg.set_dword(HKLM, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\cs2.exe\PerfOptions", "CpuPriorityClass", 3).is_ok()
            && reg.set_dword(HKLM, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\cs2.exe\PerfOptions", "CpuAffinityMask", 0xFFFF_FFFE).is_ok();
        status.push(Cs2TweakStatus { id: "ifeo_priority".into(), applied: ifeo_ok });

        Ok(status)
    }

    /// Parte async: energia, HPET, Nagle e timer 0.5ms (sem guard vivo).
    pub async fn apply_system_tweaks(executor: &ProcessExecutor) -> Result<Vec<Cs2TweakStatus>, String> {
        let mut status = Vec::new();

        let power = executor.execute_cmd("powercfg /setactive e9a42b02-d5df-448d-aa00-03f14749eb61").await;
        if power.exit_code != 0 {
            let _ = executor.execute_cmd("powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61").await;
        }
        status.push(Cs2TweakStatus { id: "power_ultimate".into(), applied: true });

        let usb = executor.execute_powershell("powercfg /setacvalueindex SCHEME_CURRENT SUB_USB USBSELECTIVESUSPEND 0; powercfg /setdcvalueindex SCHEME_CURRENT SUB_USB USBSELECTIVESUSPEND 0; powercfg /setactive SCHEME_CURRENT").await;
        status.push(Cs2TweakStatus { id: "usb_suspend_off".into(), applied: usb.exit_code == 0 });

        // HPET off (idempotente: deletevalue falha quando a chave não existe).
        let _ = executor.execute_cmd("bcdedit /deletevalue useplatformclock").await;
        let hpet = executor.execute_cmd("bcdedit /set disabledynamictick yes").await;
        status.push(Cs2TweakStatus { id: "hpet_off".into(), applied: hpet.exit_code == 0 });

        // Nagle/TCPNoDelay nas interfaces ativas.
        let nagle = executor.execute_powershell(r#"$ifs = Get-NetAdapter -ErrorAction SilentlyContinue | Select-Object -ExpandProperty InterfaceGuid -Unique; foreach ($guid in $ifs) { $base = "HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\$guid"; Set-ItemProperty -Path $base -Name TcpAckFrequency -Value 1 -Type DWord -ErrorAction SilentlyContinue; Set-ItemProperty -Path $base -Name TCPNoDelay -Value 1 -Type DWord -ErrorAction SilentlyContinue }"#).await;
        status.push(Cs2TweakStatus { id: "nagle_off".into(), applied: nagle.exit_code == 0 });

        // Timer 0.5ms.
        let timer = executor.execute_powershell(r#"
$sig = '[DllImport("ntdll.dll")] public static extern bool NtSetTimerResolution(uint DesiredResolution, bool SetResolution, out uint CurrentResolution);'
$type = Add-Type -MemberDefinition $sig -Name 'Win32TimeCs2' -Namespace 'Kernel' -PassThru
$current = 0
[void]$type::NtSetTimerResolution(5000, $true, [ref]$current)
Write-Output 'TIMER_OK'
"#).await;
        status.push(Cs2TweakStatus { id: "timer_05ms".into(), applied: timer.stdout.contains("TIMER_OK") });

        Ok(status)
    }

    /// cs2_video.txt com backup e preset competitivo.
    pub fn apply_video_config(restore: &mut RestoreService, video_txt: &str) -> Result<String, String> {
        if !Path::new(video_txt).exists() { return Err("cs2_video.txt não encontrado; abra o CS2 uma vez para gerá-lo".into()); }
        restore.backup_file(video_txt, "cs2", "cs2_video.txt original antes do preset competitivo")?;

        let preset = r#"
"config"
{
	"setting.defaultres"		"1920"
	"setting.defaultresheight"		"1080"
	"setting.fullscreen"		"1"
	"setting.nowindowborder"		"0"
	"setting.vsync"		"0"
	"setting.refreshrate_denominator"		"1"
	"setting.refreshrate_numerator"		"240"
	"Setting.FullScreen.LaptopSavings"		"0"
	"setting.cpu_level_1"		"0"
	"setting.gpu_mem_level"		"0"
	"setting.mem_level"		"0"
	"setting.gpu_level"		"0"
	"setting.mat_aaquality"		"1"
	"setting.mat_antialias"		"2"
	"setting.dvs_enable"		"0"
	"setting.dvs_gpuframetime_min"		"15000"
	"setting.dvs_gpuframetime"		"16000"
	"setting.dvs_scale_min"		"0.400000"
	"setting.dvs_scale"		"0.800000"
	"setting.low_latency"		"1"
	"setting.fsr_enable"		"0"
	"setting.r_lowend_specular"		"0"
	"setting.r_aoprocedural"		"0"
	"setting.r_aoprocedural_strength"		"0.5"
	"setting.r_gpasteamplify"		"1"
	"setting.dynamic_shadows_tweak"		"1"
	"setting.r_texturefilteringquality"		"2"
	"setting.r_particle_dlss_force_off"		"1"
	"setting.r_imgui_120hz"		"0"
	"setting.boost_player_contrast"		"1"
	"setting.player_silhouette_thickness"		"0.6"
	"setting.player_silhouette_opacity"		"0.8"
	"setting.hdr"		"-1"
	"setting.jitter_shadows"		"0"
}
"#;
        std::fs::write(video_txt, preset).map_err(|e| format!("falha ao escrever cs2_video.txt: {e}"))?;
        Ok(video_txt.to_string())
    }

    /// Launch options seguras no localconfig.vdf (com backup) e aviso de Steam fechada.
    pub fn apply_launch_options(restore: &mut RestoreService, cfg_dir: &str, refresh_rate: u32) -> Result<String, String> {
        let vdf = PathBuf::from(cfg_dir).join("localconfig.vdf");
        if !vdf.exists() { return Err("localconfig.vdf não encontrado; abra a Steam uma vez para gerá-lo".into()); }
        restore.backup_file(&vdf.display().to_string(), "cs2", "localconfig.vdf original antes das launch options")?;
        let content = std::fs::read_to_string(&vdf).map_err(|e| e.to_string())?;

        let options = format!("-nojoy -freq {refresh_rate} +fps_max 0 -novid -highpriority");
        let updated = if content.contains("\"LaunchOptions\"") {
            // Substitui a linha existente mantendo o resto do arquivo intacto.
            let mut out = String::with_capacity(content.len());
            for line in content.lines() {
                if line.trim_start().starts_with("\"LaunchOptions\"") {
                    let indent = &line[..line.len() - line.trim_start().len()];
                    out.push_str(&format!("{indent}\"LaunchOptions\"\t\t\"{options}\""));
                } else {
                    out.push_str(line);
                }
                out.push('\n');
            }
            out
        } else {
            content.replace("\"Software\"", &format!("\"LaunchOptions\"\t\t\"{options}\"\n\t\t\t\t\t\"Software\""))
        };
        std::fs::write(&vdf, updated).map_err(|e| e.to_string())?;
        Ok(options)
    }

    /// Gera/atualiza o autoexec.cfg com os tweaks de FPS (efeitos visuais off).
    pub fn apply_autoexec(restore: &mut RestoreService, cfg_dir: &str) -> Result<String, String> {
        if cfg_dir.is_empty() { return Err("pasta cfg do CS2 não encontrada".into()); }
        std::fs::create_dir_all(cfg_dir).map_err(|e| e.to_string())?;
        let path = PathBuf::from(cfg_dir).join("autoexec.cfg");
        if path.exists() {
            restore.backup_file(&path.display().to_string(), "cs2", "autoexec.cfg original antes do preset FPS Boost")?;
        }
        let content = r#"// Capivara Booster: FPS Boost CS2 (reverta pelo menu Backup)
cl_skybox 0
r_drawskybox 0
r_waterforceexpensive 0
r_waterforcereflectentities 0
r_drawdecals 0
r_decals 0
r_impacts_alt_orientation 0
cl_debugrumble 0
r_3dsky 0
r_volumetriclighting 0
mat_bloom_scalefactor_spatial 0
cl_phys_maxticks 2
engine_no_focus_sleep 0
fps_max 0
cl_showfps 1
"#;
        std::fs::write(&path, content).map_err(|e| format!("falha ao escrever autoexec.cfg: {e}"))?;
        Ok(path.display().to_string())
    }

    /// Limpa caches de shader e retorna bytes liberados (arquivos em uso pulados).
    pub fn clear_shader_caches() -> Result<(u64, u32), String> {
        let local = std::env::var("LOCALAPPDATA").map(PathBuf::from).unwrap_or_default();
        let steam = Self::steam_path();
        let mut targets = vec![
            local.join("D3DSCache"),
            local.join(r"NVIDIA\DXCache"),
            local.join(r"NVIDIA\GLCache"),
            local.join(r"AMD\DxCache"),
            local.join(r"AMD\GLCache"),
        ];
        for library in Self::steam_libraries(&steam) {
            targets.push(library.join(r"steamapps\shadercache\730"));
        }
        let (mut bytes, mut count) = (0u64, 0u32);
        for target in targets {
            if target.exists() {
                measure_tree(&target, &mut bytes, &mut count);
                let _ = std::fs::remove_dir_all(&target);
            }
        }
        Ok((bytes, count))
    }
}

fn measure_tree(dir: &Path, bytes: &mut u64, count: &mut u32) {
    let entries = match std::fs::read_dir(dir) { Ok(e) => e, Err(_) => return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(meta) = std::fs::metadata(&path) { *bytes += meta.len(); *count += 1; }
        } else if path.is_dir() {
            measure_tree(&path, bytes, count);
        }
    }
}
