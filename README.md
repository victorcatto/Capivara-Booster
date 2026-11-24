<div align="center">

# CAPIVARA BOOSTER

### Windows 10/11 system optimization for competitive gaming

[![Latest release](https://img.shields.io/github/v/release/victorcatto/Capivara-Booster?style=for-the-badge&color=00E5AB)](https://github.com/victorcatto/Capivara-Booster/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/victorcatto/Capivara-Booster/release.yml?branch=main&style=for-the-badge&label=release%20build)](https://github.com/victorcatto/Capivara-Booster/actions)
[![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011%20x64-00B8FF?style=for-the-badge&logo=windows)](https://github.com/victorcatto/Capivara-Booster)
[![License](https://img.shields.io/badge/license-MIT-8B949E?style=for-the-badge)](LICENSE)

[Português do Brasil](README.pt-BR.md)

</div>

## What is Capivara Booster?

Capivara Booster is an open-source Windows utility for competitive gamers and power users. It combines system cleanup, privacy controls, latency tuning, game-specific profiles, configuration backups, and rollback in one administrator-controlled desktop application.

The stable Windows release is currently the .NET 8/WPF edition. The Rust/Tauri v2 edition is being developed separately on the `rust` branch and will only replace the stable edition after functional parity and installer validation.

## Download

**[Download the latest official installer](https://github.com/victorcatto/Capivara-Booster/releases/latest/download/CapivaraBooster-Setup.exe)**

See the [release history](https://github.com/victorcatto/Capivara-Booster/releases) for checksums, changelogs, and versioned installers.

### Requirements

- Windows 10 or Windows 11, 64-bit
- Administrator permission for system and registry changes
- .NET 8 Desktop Runtime for the current stable WPF edition

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
- Single administrator identity: `victorcatto93@gmail.com`
- GitHub-based update manifest with SHA-256 verification
- Live execution console hidden by default and available with a toggle

## Safety model

Capivara Booster changes Windows settings only after explicit user action. Restore points and file backups are created before supported destructive operations. Registry values changed by the application are tracked so they can be reverted from Backup & Restore.

Always review an optimization before applying it. Hardware, Windows edition, drivers, antivirus policy, and game updates can change the result of a tweak.

## Build from source

```powershell
git clone https://github.com/victorcatto/Capivara-Booster.git
Set-Location Capivara-Booster

# Stable WPF edition
dotnet restore CapivaraBooster.sln
dotnet build CapivaraBooster.sln -c Release

# Rust/Tauri v2 development branch
Set-Location src-tauri
cargo check
cargo test
```

The Rust branch requires the Rust stable toolchain, the Windows MSVC build tools, Node.js, and the Tauri CLI. Its CI runs on `windows-latest` and validates the frontend, Rust compilation, tests, and NSIS bundle.

## Architecture

- Stable edition: C#/.NET 8, WPF, MVVM, Windows Registry and PowerShell integrations
- Migration edition: Rust, Tauri v2, TypeScript frontend, `winreg`, `windows-rs`, Tokio, DPAPI, and GitHub update compatibility
- Release pipeline: GitHub Actions with Windows build runners and NSIS packaging

Read the [Rust migration blueprint](docs/RUST_MIGRATION_BLUEPRINT.md) for the transition roadmap.

## Project status

- `main`: stable WPF edition and production release pipeline
- `rust`: active migration, feature parity, security review, and installer validation
- v2.0.0 will be released only after the parity checklist, clean installer upgrade, and end-to-end verification pass

## License

Capivara Booster is distributed under the MIT License. See [LICENSE](LICENSE).
