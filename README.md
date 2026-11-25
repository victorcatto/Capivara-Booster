<div align="center">

# CAPIVARA BOOSTER

### Windows 10/11 system optimization for competitive gaming

[![Latest release](https://img.shields.io/github/v/release/victorcatto/Capivara-Booster?style=for-the-badge&color=00E5AB)](https://github.com/victorcatto/Capivara-Booster/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/victorcatto/Capivara-Booster/release.yml?branch=main&style=for-the-badge&label=release%20build)](https://github.com/victorcatto/Capivara-Booster/actions)
[![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011%20x64-00B8FF?style=for-the-badge&logo=windows)](https://github.com/victorcatto/Capivara-Booster)
[![License](https://img.shields.io/badge/license-MIT-8B949E?style=for-the-badge)](LICENSE)

[Português do Brasil](README.pt-BR.md)

**[Official landing page](https://victorcatto.github.io/Capivara-Booster/)**: live version, direct download, and the latest patch notes.

</div>

## What is Capivara Booster?

Capivara Booster is an open-source Windows utility for competitive gamers and power users. It combines system cleanup, privacy controls, latency tuning, game-specific profiles, configuration backups, and rollback in one administrator-controlled desktop application.

The application is built natively in **Rust** with **Tauri v2**: a single lightweight executable, no .NET runtime dependency, direct Windows Registry access, and a modern dark web-based UI.

## Download

**[Download the latest official installer](https://github.com/victorcatto/Capivara-Booster/releases/latest/download/CapivaraBooster-Setup.exe)**

See the [release history](https://github.com/victorcatto/Capivara-Booster/releases) for checksums, changelogs, and versioned installers.

### Requirements

- Windows 10 or Windows 11, 64-bit
- Administrator permission for system and registry changes
- WebView2 Runtime (preinstalled on up-to-date Windows 10/11)

Windows SmartScreen may display a warning because the open-source installer is not code-signed. Verify the release asset and SHA-256 checksum before running it.

## Features

### System and privacy

- Windows restore point before destructive operations
- Registry change tracking and rollback
- UWP debloat with selected package removal
- Telemetry services and scheduled task controls
- Startup item inspection

### Cleanup

- User and system temporary files
- Windows Prefetch and Windows Update download cache
- Recycle Bin and DNS resolver cache
- DirectX, NVIDIA, and AMD shader caches
- Parallel size scanning with per-item selection

### Latency and performance

- HPET and synthetic timer controls
- Ultimate Performance power plan
- Game DVR and fullscreen optimization settings
- Network Throttling Index and SystemResponsiveness
- Multimedia Games scheduling profile

### Game profiles

- **Counter-Strike 2:** IFEO priority/affinity, AppCompat flags, shader cache cleanup, and competitive `autoexec.cfg`
- **VALORANT:** game and Riot Client priority, AppCompat flags, logs, and Defender exclusions
- **League of Legends:** low-spec mode, close-client settings, logs, backups, and Defender exclusions

### Account and updates

- PT-BR and EN-US language switching at runtime
- Local account vault protected by Windows DPAPI
- PBKDF2-HMAC-SHA256 password derivation with random salts
- Registration restricted to Gmail, Outlook, Yahoo, and Yahoo Brasil domains
- Single administrator identity enforced internally and never exposed in the UI
- GitHub-based update manifest with SHA-256 verification
- Live execution console hidden by default and available with a toggle

## Safety model

Capivara Booster changes Windows settings only after explicit user action. Restore points and file backups are created before supported destructive operations. Registry values changed by the application are tracked so they can be reverted from Backup & Restore.

Always review an optimization before applying it. Hardware, Windows edition, drivers, antivirus policy, and game updates can change the result of a tweak.

## Build from source

```powershell
git clone https://github.com/victorcatto/Capivara-Booster.git
Set-Location Capivara-Booster

# Frontend
Set-Location ui
npm install
npm run build

# Backend Rust + instalador NSIS
Set-Location ..\src-tauri
cargo check
cargo test
cargo tauri build --bundles nsis
```

Building requires the Rust stable toolchain, the Windows MSVC build tools, Node.js, and the Tauri CLI. CI runs on `windows-latest` and validates the frontend, Rust compilation, tests, and the NSIS bundle.

## Architecture

- Backend: Rust with Tokio, `winreg`, `windows-rs`, DPAPI (vault/session), SHA-256 update verification, and `serde_json`
- Frontend: TypeScript with Vite, dark cyberpunk theme, runtime PT-BR/EN-US switching
- Packaging: Tauri v2 NSIS installer, per-machine install mode, requireAdministrator manifest
- Release pipeline: GitHub Actions with Windows build runners

## Project status

- Single development tree: all feature work lands on `main`
- Release tags `v2.*` and later build the Rust edition via GitHub Actions
- The v1.x WPF installers remain available on the Releases page for existing installs, and the Rust updater keeps the same `version.json` contract so old clients can still upgrade

## License

Capivara Booster is distributed under the MIT License. See [LICENSE](LICENSE).
