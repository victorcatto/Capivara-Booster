# 🦫 CAPIVARA BOOSTER - Windows 10/11 Esports & Low-Latency Tuning Suite

![Capivara Booster](src/CapivaraBooster/Assets/logo.png)

**Capivara Booster** is an enterprise-grade desktop utility engineered in **C# (.NET 8 WPF)** targeting Windows 10 and Windows 11. Designed by senior Windows systems engineers, DevOps specialists, and esports optimization professionals, it delivers deep OS debloating, hardware timer tuning (HPET), cache purges, and engine/network optimizations for competitive titles (**Counter-Strike 2**, **Valorant**, and **League of Legends**).

---

## 🎨 Visual Identity & Cyberpunk Aesthetic

- **Primary Background**: Deep Matte Charcoal (`#0F1015`)
- **Card / Panel Surfaces**: Dark Slate (`#181A22`) with subtle glowing borders (`#262A38` and active `#00E5FF`)
- **Primary Accent (Cyan Neon)**: `#00E5FF` (active navigation tabs, primary cyber action buttons, toggle switches, and progress indicators)
- **Secondary Accent (Electric Purple)**: `#9D00FF` (hover transitions, badges, and gradient accents from `#00E5FF` to `#9D00FF`)
- **Primary Text**: Crisp Off-White (`#F4F4F6`)
- **Secondary Text**: Muted Cool Gray (`#8E92A4`)
- **Console Typography**: Clean monospace (`Cascadia Code`, `Consolas`) with color-coded severity tags (`[CMD]`, `[OK]`, `[WARN]`, `[ERROR]`, `[INFO]`)

---

## 🌐 Full Internationalization (i18n — PT-BR & EN-US)

Capivara Booster features a dynamic localization engine ([LocalizationService.cs](file:///c:/Users/User/Documents/PROJETO2/src/CapivaraBooster/Services/LocalizationService.cs)):
- **Zero hardcoded UI strings**: All user-facing texts, alerts, descriptions, and diagnostics are loaded from [pt-BR.json](file:///c:/Users/User/Documents/PROJETO2/src/CapivaraBooster/Localization/pt-BR.json) and [en-US.json](file:///c:/Users/User/Documents/PROJETO2/src/CapivaraBooster/Localization/en-US.json).
- **Dynamic Real-Time Language Switcher**: A segmented control `[ PT-BR | EN-US ]` in the top-right header updates every open tab, label, and dialog **instantly without restarting the app**.
- **Auto-Detection**: On first run, it detects the Windows UI culture (`pt-BR` defaults to Portuguese; other locales default to English) and persists user choices in `%AppData%\CapivaraBooster\settings.json`.

---

## 🔐 Authentication, Domain Allowlist & Single Admin Rule

Implemented in [AuthService.cs](file:///c:/Users/User/Documents/PROJETO2/src/CapivaraBooster/Services/AuthService.cs):
- **Allowed Registration Domains Only**:
  Registration is strictly permitted for:
  - `@gmail.com`
  - `@outlook.com`
  - `@yahoo.com.br`
  - `@yahoo.com`
  *(Any other domain is rejected with a localized error message).*
- **Single Administrator Rule**:
  - `victorcatto93@gmail.com` is assigned the **Master Administrator** role (`UserRole.Admin`).
  - All other users receive standard user privileges (`UserRole.StandardUser`).
- **Encrypted Local Vault & DPAPI Sessions**: Passwords hashed with PBKDF2 (SHA-256, 100,000 iterations); persistent session tokens encrypted via Windows DPAPI.

---

## 🚀 Automated Versioning, GitHub Actions CI/CD & Self-Updater

### 1. In-App Self-Updater ([UpdateService.cs](file:///c:/Users/User/Documents/PROJETO2/src/CapivaraBooster/Services/UpdateService.cs))
- Checks GitHub Releases API / `version.json` on startup and on demand.
- When an update is detected, displays a notification banner with release notes.
- Downloads the installer in the background with real-time percentage progress.
- Verifies SHA-256 binary integrity before execution.
- Launches the installer (`/SILENT /CLOSEAPPLICATIONS`) and performs a smooth restart.

### 2. GitHub Actions Pipeline ([.github/workflows/release.yml](file:///c:/Users/User/Documents/PROJETO2/.github/workflows/release.yml))
Whenever a tag (`v*`) is pushed from VS Code:
1. Builds .NET 8 WPF binary in Release mode.
2. Compiles Inno Setup installer (`CapivaraBooster-Setup-vX.X.X.exe`).
3. Computes SHA-256 checksum and updates `version.json`.
4. Creates a tagged GitHub Release with the installer asset.

### 3. One-Click VS Code Publisher ([publish-update.ps1](file:///c:/Users/User/Documents/PROJETO2/publish-update.ps1))
Run from terminal or VS Code Task (`Ctrl+Shift+B`):
```powershell
# Bump patch (1.0.0 -> 1.0.1), commit, tag, and push to GitHub
.\publish-update.ps1 -BumpType patch -Message "Optimized network scheduler"

# Bump minor (1.0.0 -> 1.1.0)
.\publish-update.ps1 -BumpType minor -Message "Added new CS2 subtick features"
```

---

## 📦 Windows Installer ([installer/CapivaraBooster.iss](file:///c:/Users/User/Documents/PROJETO2/installer/CapivaraBooster.iss))

- Requires Administrator privileges (`PrivilegesRequired=admin`).
- Installs to `{autopf}\CapivaraBooster`.
- Bundles application binaries, dark theme assets, Capivara emblem, and localization JSON files.
- Creates Start Menu and Desktop shortcuts.
- Registers clean Windows Add/Remove Programs entry with icon and versioning.
- Supports unattended/silent upgrades over previous versions (`CloseApplications=yes`).

---

## 📁 Repository Structure

```
c:\Users\User\Documents\PROJETO2\
├── .github/
│   └── workflows/
│       └── release.yml           # GitHub Actions CI/CD Release Pipeline
├── .vscode/
│   └── tasks.json                # VS Code Build & One-Click Release Tasks
├── installer/
│   └── CapivaraBooster.iss       # Inno Setup Windows Installer Script
├── Assets/
│   ├── app.ico                   # Executable application icon
│   └── logo.png                  # High-resolution Capivara Booster emblem
├── src/
│   └── CapivaraBooster/
│       ├── CapivaraBooster.csproj # .NET 8 WPF Project Definition
│       ├── app.manifest          # Admin elevation (requireAdministrator)
│       ├── App.xaml / .cs        # DI Root & global exception handlers
│       ├── MainWindow.xaml / .cs # Main shell, navbar, console dock & auth modal
│       ├── Localization/
│       │   ├── en-US.json        # English translation dictionary
│       │   └── pt-BR.json        # Portuguese (Brazil) translation dictionary
│       ├── Common/               # ObservableObject, RelayCommand, LogItem
│       ├── Models/               # UserAccount, UpdateInfo, CleanerItem, etc.
│       ├── Services/             # AuthService, UpdateService, SystemBoostService, etc.
│       ├── ViewModels/           # MainViewModel, DashboardViewModel, etc.
│       ├── Views/                # Dashboard, Debloat, Cleaner, SystemBoost, etc.
│       └── Styles/               # DarkTheme.xaml & Controls.xaml
├── .gitignore                    # Comprehensive ignore rules
├── CapivaraBooster.sln           # Visual Studio & .NET Solution
├── publish-update.ps1            # Automated version bump & git release script
├── version.json                  # Remote manifest for in-app Self-Updater
└── README.md
```

---

## 🛠️ Quick Start & Build

```powershell
# Restore dependencies & compile
dotnet build CapivaraBooster.sln -c Release

# Publish app binaries
dotnet publish src/CapivaraBooster/CapivaraBooster.csproj -c Release -r win-x64 --self-contained false -o ./publish/app
```
