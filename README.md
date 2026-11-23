<div align="center">

# 🦫 CAPIVARA BOOSTER

### Windows 10 & 11 Esports, Debloat & Low-Latency Tuning Suite

[![Latest Release](https://img.shields.io/github/v/release/victorcatto/Capivara-Booster?style=for-the-badge&color=00E5FF&label=LATEST%20RELEASE)](https://github.com/victorcatto/Capivara-Booster/releases/latest)
[![Windows 10 & 11](https://img.shields.io/badge/PLATFORM-WINDOWS%2010%20%7C%2011%20(64--BIT)-9D00FF?style=for-the-badge&logo=windows)](https://github.com/victorcatto/Capivara-Booster)
[![Language Support](https://img.shields.io/badge/LANGUAGES-PT--BR%20%7C%20EN--US-00E5FF?style=for-the-badge)](https://github.com/victorcatto/Capivara-Booster)
[![.NET 8 WPF](https://img.shields.io/badge/.NET-8.0%20WPF-9D00FF?style=for-the-badge&logo=dotnet)](https://github.com/victorcatto/Capivara-Booster)
[![License](https://img.shields.io/badge/LICENSE-MIT-00E5FF?style=for-the-badge)](LICENSE)

<br/>

<img src="Assets/logo.png" width="160" alt="Capivara Booster Emblem" style="border-radius: 20px; box-shadow: 0 0 25px rgba(0, 229, 255, 0.4);" />

<br/>

[ **🇧🇷 Versão em Português (Brasil)** ](README.pt-BR.md) &nbsp;&bull;&nbsp; [ **🇺🇸 English Version** ](README.md)

---

</div>

<br/>

<!-- ================= DIRECT DOWNLOAD CARD FOR NON-TECHNICAL USERS ================= -->

<div align="center">
<table>
<tr>
<td align="center" style="background-color: #141722; border: 2px solid #00E5FF; border-radius: 14px; padding: 24px 32px;">

## ⚡ Quick 1-Click Direct Download

### **[ 🚀 DOWNLOAD CAPIVARA BOOSTER (LATEST INSTALLER) ](https://github.com/victorcatto/Capivara-Booster/releases/latest/download/CapivaraBooster-Setup.exe)**

**Version:** `v1.1.2` *(Latest Stable Release)* &nbsp;|&nbsp; **File:** `CapivaraBooster-Setup.exe` (~15 MB)  
*Or browse all release assets: [GitHub Releases](https://github.com/victorcatto/Capivara-Booster/releases/latest)*

<br/>

> **Simple 3-Step Setup for Non-Technical Users:**
> 1. Click the **[Download](https://github.com/victorcatto/Capivara-Booster/releases/latest/download/CapivaraBooster-Setup.exe)** link above.
> 2. Run the downloaded `CapivaraBooster-Setup.exe` installer.
> 3. If Windows SmartScreen displays *"Windows protected your PC"*, click **"More info"** &rarr; **"Run anyway"** *(this is normal for specialized open-source utilities requiring Administrator privileges to optimize hardware timers and Windows registry settings)*.

</td>
</tr>
</table>
</div>

<br/>

---

## 📖 Overview

**Capivara Booster** is an enterprise-grade Windows optimization, debloating, and esports performance desktop utility developed in **C# (.NET 8 WPF)**. Engineered specifically for competitive gaming, it eliminates OS background stutter, purges GBs of stale caches, optimizes system timers (HPET), and injects tailored engine tweaks for **Counter-Strike 2 (CS2)**, **Valorant**, and **League of Legends (LoL)**.

Built around a sleek **Cyberpunk Dark Theme** derived from its official Capivara emblem, it includes real-time bilingual switching (**PT-BR / EN-US**), a safe rollback engine, and an automated background Self-Updater.

---

## 🎨 Visual Identity & Cyberpunk Aesthetic

- **Deep Matte Charcoal (`#0F1015`)**: Base application background.
- **Dark Slate Card Surfaces (`#181A22`)**: With subtle cyber glow borders (`#262A38` and active `#00E5FF`).
- **Cyan Neon Accent (`#00E5FF`)**: Primary action buttons, active navigation indicators, and progress bars.
- **Electric Purple Accent (`#9D00FF`)**: Hover transitions, badges, and gradient highlights.
- **Crisp Off-White (`#F4F4F6`)** & **Cool Gray (`#8E92A4`)**: High-contrast, fatigue-free typography.
- **Real-Time Monospace Console**: High-performance color-coded terminal dock virtualized for 144Hz displays.

---

## ⚡ Core Features

### 1. 🧹 Windows Debloater & Privacy Engine
- **Safe UWP Removal**: One-click removal of bloatware apps (Cortana, Xbox Game Bar, MSN Weather, MSN News, Get Help, Tips, Solitaire, Feedback Hub, Mixed Reality Portal, 3D Viewer) using an ultra-fast batched PowerShell execution engine (~85% faster than sequential uninstallers).
- **Telemetry & Diagnostic Tracking Disabler**: Disables telemetry services (`DiagTrack`, `dmwappushservice`, `standardcollector`) and groups policy tracking keys.
- **Scheduled Tasks Optimizer**: Deactivates background Customer Experience Improvement Program (CEIP) and Compatibility Appraiser daemons.

### 2. 🧼 Deep Junk, Temp & Shader Cache Purger
- **User & System Temp**: Cleans `%temp%`, `C:\Windows\Temp`, and `C:\Windows\Prefetch`.
- **Windows Update Download Cache**: Safely purges `C:\Windows\SoftwareDistribution\Download` with guaranteed Windows Update service restart (`net start wuauserv`).
- **Recycle Bin & Network Flush**: Empties Recycle Bin and flushes Windows DNS cache (`ipconfig /flushdns`).
- **GPU Shader Caches**: Purges compiled DirectX (`D3DSCache`), NVIDIA DX/GL/Compute caches, and AMD Radeon DxCache to resolve micro-stutters and outdated shader conflicts.
- **Asynchronous Parallel Scanning**: Utilizes multi-threaded directory crawlers to calculate cache sizes in milliseconds without freezing the UI.

### 3. 🚀 System Latency & Hardware DPC Tuning
- **HPET Control (High Precision Event Timer)**: Disables HPET in BCD boot settings (`bcdedit /deletevalue useplatformclock`, `useplatformtick yes`) and Device Manager PnP to reduce DPC latency and frame-time spikes.
- **Ultimate Performance Power Plan**: Automatically unlocks and activates the hidden Windows `Ultimate Performance` power scheme (`e9a42b02-d5df-448d-aa00-03f14749eb61`).
- **Global Gaming Tweaks**: Disables Game DVR, eliminates Fullscreen Optimizations globally, turns off background apps, sets Network Throttling Index to maximum throughput (`0xFFFFFFFF`), and grants 100% responsiveness to multimedia game tasks (`SystemResponsiveness = 0`).
- **Startup Programs Manager**: Inspects startup entries across 64-bit Run, 32-bit WOW64 Run, and user Startup shortcuts.

### 4. 🎮 Esports Game Optimizer Modules

#### 🎯 Counter-Strike 2 (CS2)
- **High CPU Priority & Affinity**: Sets `cs2.exe` to High CPU Priority (`CpuPriorityClass = 3`) via Registry IFEO and optionally isolates Core 0 (`CpuAffinityMask = 0xFFFFFFFE`) for better thread scheduling.
- **Executable Flags**: Enforces High DPI awareness and disables fullscreen optimizations via `AppCompatFlags\Layers`.
- **Shader Cache Reset**: Safely purges Steam AppID `730` shader cache.
- **Competitive `autoexec.cfg` Generation**: Generates an optimized autoexec with de-subtick jumpthrow binds, 128-tick rate (`786432`), audio latency buffers (`snd_mixahead 0.011`), and low-latency tick sleep.
- **Recommended Launch Options**: Instant copy for `-high -novid +fps_max 0 -nojoy +exec autoexec.cfg`.

#### 🏹 Valorant
- **Process Scheduling**: Sets `VALORANT-Win64-Shipping.exe` to High Priority while dynamically throttling `RiotClientServices.exe` to Low/Idle Priority to prevent background CPU interrupts.
- **Log & Crash Purge**: Cleans local crash dumps, logs, and Riot webcache (`%localappdata%\VALORANT\Saved`).
- **Defender Exclusions**: Adds exclusions for Riot Vanguard (`vgk.sys`, `vgc.exe`) and installation directories to eliminate real-time antivirus disk scanning latency.

#### ⚔️ League of Legends
- **Chromium Client Control**: Automatically patches `game.cfg` and `PersistedSettings.json` with `LowSpecMode=1` and `CloseClientDuringGame=1` so the heavy client closes during match gameplay, freeing up to 1.5 GB of RAM.
- **Logs Cleanup & Defender Exclusions**: Clears game logs and adds Windows Defender exclusions for the Riot Games folder.

### 5. 🛡️ Safety, Backup & Rollback
- **System Restore Point**: Automatically creates a Windows System Restore Point (`Checkpoint-Computer`) before executing debloating, registry, or system changes.
- **Config Backups**: Creates dated `.bak` archives before modifying game configuration files.
- **1-Click Registry Rollback**: Maintains an immutable rollback store to revert all tweaks back to default Windows values.

### 6. 🌐 Full Internationalization (i18n — PT-BR & EN-US)
- **Zero Hardcoded Strings**: All labels, descriptions, and diagnostics are loaded dynamically from JSON dictionaries ([en-US.json](src/CapivaraBooster/Localization/en-US.json) and [pt-BR.json](src/CapivaraBooster/Localization/pt-BR.json)).
- **Real-Time Switching**: Switch between Portuguese and English instantly from the top-right header with zero application restarts.
- **Auto-Detection**: Automatically detects Windows system locale on first launch and saves preferences in `%AppData%\CapivaraBooster\settings.json`.

---

## 🔰 Step-by-Step Beginner's Guide

### Step 1: Download & Install
1. Download **[CapivaraBooster-Setup.exe](https://github.com/victorcatto/Capivara-Booster/releases/latest/download/CapivaraBooster-Setup.exe)** from the direct download button above.
2. Double-click the installer. If prompted by Windows UAC, click **Yes** to grant Administrator permissions.
3. Follow the installation wizard and launch **Capivara Booster** from your Desktop or Start Menu.

### Step 2: Account Login / Registration
1. In the top-right corner of the application, click **Login / Register**.
2. To create an account, select **Sign Up** and enter your email and password (minimum 6 characters).
   > **Note on Supported Email Providers:**  
   > For security and abuse prevention, registration is strictly allowed for:  
   > &bull; `@gmail.com`  
   > &bull; `@outlook.com`  
   > &bull; `@yahoo.com.br`  
   > &bull; `@yahoo.com`  
3. Once logged in, your session is saved securely via Windows DPAPI encryption.

### Step 3: Create a Restore Point & Optimize
1. On the **Dashboard**, click **Quick Boost** to apply recommended baseline optimizations, or navigate through the tabs:
   - **Debloat**: Select bloatware apps to safely remove and disable telemetry.
   - **Cleaner**: Click **Scan**, check the junk/shader caches you wish to purge, and click **Clean Selected**.
   - **System Boost**: Toggle **HPET** off to lower latency, activate the **Ultimate Performance** power plan, and apply global network tweaks.
   - **Game Optimizer**: Detect installed games (CS2, Valorant, LoL) and apply customized performance profiles.
2. If you ever wish to revert any tweak, head to **Backup & Restore** and click **Revert Registry Tweaks** or restore backed-up game configs.

### Step 4: Automatic Updates
- Capivara Booster checks for updates automatically in the background. When a new version is released, an update banner will appear at the top. Click **Update Now** to download and install the update seamlessly.

---

## 🛠️ Developer & Build Instructions

### Prerequisites
- **Windows 10 or 11 (64-bit)**
- **.NET 8 SDK** (`net8.0-windows`)
- *(Optional for installer compilation)*: **Inno Setup 6**

### Building from Source

```powershell
# Clone the repository
git clone https://github.com/victorcatto/Capivara-Booster.git
cd Capivara-Booster

# Restore dependencies
dotnet restore CapivaraBooster.sln

# Compile Release build
dotnet build CapivaraBooster.sln -c Release

# Publish self-contained executable folder
dotnet publish src/CapivaraBooster/CapivaraBooster.csproj -c Release -r win-x64 --self-contained false -o ./publish/app
```

### Automated 1-Click Release Script

To release a new version from VS Code:
```powershell
# Bump version, compile, commit, tag, and push to trigger GitHub Actions CI/CD:
.\publish-update.ps1 -BumpType patch -Message "Performance optimizations"
```

---

## 🔒 Security & Privacy Policy

- **No Data Harvesting**: Capivara Booster runs locally on your machine. No telemetry, browsing data, or keystrokes are transmitted.
- **Single Administrator Role**: Administrator privileges within the app are strictly restricted to `victorcatto93@gmail.com`. Standard accounts receive standard user privileges with no client-side elevation.
- **Encrypted Local Storage**: Session tokens and credentials are encrypted using the Windows Data Protection API (DPAPI).
- **Clean Uninstallation**: The uninstaller cleanly removes all application files and registry modifications.

---

## 📜 License

Distributed under the **MIT License**. See `LICENSE` for more information.

<div align="center">
<b>Capivara Booster &copy; 2026 &mdash; Built with ❤️ for competitive gamers worldwide.</b>
</div>