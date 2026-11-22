param (
    [ValidateSet("patch", "minor", "major")]
    [string]$BumpType = "patch",
    [string]$NewVersion = "",
    [string]$Message = "Performance updates and latency optimizations"
)

$ErrorActionPreference = "Stop"

# Ensure Git is in PATH
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    $env:Path = "$env:USERPROFILE\.git-bin\cmd;$env:USERPROFILE\.gh-bin\bin;$env:Path"
}

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   CAPIVARA BOOSTER - ONE-CLICK RELEASE & PUBLISHER       " -ForegroundColor Yellow
Write-Host "==========================================================" -ForegroundColor Cyan

$projectFile = "src\CapivaraBooster\CapivaraBooster.csproj"
$manifestFile = "version.json"

if (-not (Test-Path $projectFile)) {
    Write-Error "Could not locate project file at $projectFile"
}

# 1. Read current version
[xml]$csprojXml = Get-Content $projectFile
$currentVersion = $csprojXml.Project.PropertyGroup.Version
if ([string]::IsNullOrWhiteSpace($currentVersion)) {
    $currentVersion = "1.0.0"
}
Write-Host "Current Version: $currentVersion" -ForegroundColor Gray

# 2. Compute Next Version
if ([string]::IsNullOrWhiteSpace($NewVersion)) {
    $parts = $currentVersion.Split('.')
    $major = [int]$parts[0]
    $minor = if ($parts.Length -gt 1) { [int]$parts[1] } else { 0 }
    $patch = if ($parts.Length -gt 2) { [int]$parts[2] } else { 0 }

    switch ($BumpType) {
        "major" { $major++; $minor = 0; $patch = 0 }
        "minor" { $minor++; $patch = 0 }
        "patch" { $patch++ }
    }
    $targetVersion = "$major.$minor.$patch"
} else {
    $targetVersion = $NewVersion.TrimStart('v','V')
}

Write-Host "Target Release Version: v$targetVersion" -ForegroundColor Green

# 3. Update csproj
(Get-Content $projectFile) -replace "<Version>.*?</Version>", "<Version>$targetVersion</Version>" | Set-Content $projectFile
Write-Host "[OK] Updated $projectFile" -ForegroundColor DarkGreen

# 4. Update version.json
if (Test-Path $manifestFile) {
    $manifest = Get-Content -Raw $manifestFile | ConvertFrom-Json
    $manifest.version = $targetVersion
    $manifest.releaseDate = (Get-Date).ToString("yyyy-MM-dd")
    $manifest.installerUrl = "https://github.com/victorcatto93/Capivara-Booster/releases/download/v$targetVersion/CapivaraBooster-Setup-v$targetVersion.exe"
    $manifest | ConvertTo-Json -Depth 5 | Set-Content $manifestFile
    Write-Host "[OK] Updated $manifestFile" -ForegroundColor DarkGreen
}

# 5. Build and verify
Write-Host "Compiling Release build..." -ForegroundColor Cyan
& "$env:USERPROFILE\.dotnet\dotnet.exe" build CapivaraBooster.sln -c Release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed. Aborting release."
}

# 6. Git commit, tag, and push
Write-Host "Committing and tagging in Git..." -ForegroundColor Cyan
git add .
git commit -m "chore(release): v$targetVersion - $Message"
git tag -a "v$targetVersion" -m "Release v$targetVersion - $Message"

Write-Host "Pushing commit and tags to GitHub..." -ForegroundColor Cyan
git push origin main --tags

Write-Host "==========================================================" -ForegroundColor Green
Write-Host " Successfully pushed v$targetVersion to GitHub!" -ForegroundColor Green
Write-Host " GitHub Actions is now building and publishing the release." -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
