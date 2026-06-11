# Release Script for Hermes Remote Manager
# Usage: .\scripts\release.ps1 -Version "0.2.0"

param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

Write-Host "=== Releasing Hermes Remote Manager v$Version ===" -ForegroundColor Cyan

# Validate version format
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Invalid version format. Use X.Y.Z (e.g., 0.2.0)"
    exit 1
}

# Check for uncommitted changes
$status = git status --porcelain
if ($status) {
    Write-Error "You have uncommitted changes. Please commit or stash them first."
    exit 1
}

# Update version in Cargo.toml
Write-Host "Updating version in Cargo.toml..." -ForegroundColor Yellow
$cargoContent = Get-Content "src-tauri/Cargo.toml" -Raw
$newCargoContent = $cargoContent -replace 'version = "[^"]*"', "version = `"$Version`""
Set-Content "src-tauri/Cargo.toml" -Value $newCargoContent -NoNewline

# Update version in tauri.conf.json if it exists
if (Test-Path "src-tauri/tauri.conf.json") {
    Write-Host "Updating version in tauri.conf.json..." -ForegroundColor Yellow
    $tauriContent = Get-Content "src-tauri/tauri.conf.json" -Raw
    $newTauriContent = $tauriContent -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
    Set-Content "src-tauri/tauri.conf.json" -Value $newTauriContent -NoNewline
}

# Commit version bump
Write-Host "Committing version bump..." -ForegroundColor Yellow
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to v$Version"

# Create tag
Write-Host "Creating tag v$Version..." -ForegroundColor Yellow
git tag -a "v$Version" -m "Release v$Version"

# Push
Write-Host "Pushing to remote..." -ForegroundColor Yellow
git push origin main
git push origin "v$Version"

Write-Host "=== Release v$Version initiated! ===" -ForegroundColor Green
Write-Host "GitHub Actions will build and create the release automatically." -ForegroundColor Cyan
Write-Host "Monitor at: https://github.com/$(git remote get-url origin | Select-String 'github.com:(.+?)(?:\.git)?$').Matches[0].Groups[1].Value/actions" -ForegroundColor Cyan
