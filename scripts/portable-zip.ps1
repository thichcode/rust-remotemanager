param(
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

$root = Split-Path $PSScriptRoot -Parent
$releaseDir = Join-Path $root "src-tauri\target\release"
$bundleDir = Join-Path $releaseDir "hermes-remote-manager-portable"
$outputDir = Join-Path $root "artifacts"

# Tạo thư mục portable
Remove-Item $bundleDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $bundleDir -Force | Out-Null

# Copy exe
Copy-Item (Join-Path $releaseDir "hermes-remote-manager.exe") $bundleDir

# Copy resources folder (chứa app.asar, locales, icons...)
Copy-Item (Join-Path $releaseDir "resources") $bundleDir -Recurse

# Copy các DLL cần thiết (OpenSSL, WebView2 nếu có)
Get-ChildItem $releaseDir -Filter "*.dll" | ForEach-Object {
    Copy-Item $_.FullName $bundleDir
}

# Copy LICENSE nếu có
if (Test-Path (Join-Path $root "LICENSE")) {
    Copy-Item (Join-Path $root "LICENSE") $bundleDir
}

# Tạo thư mục output
New-Item -ItemType Directory -Path $outputDir -Force | Out-Null

# Tạo zip
$zipName = "hermes-remote-manager-v${Version}-portable.zip"
$zipPath = Join-Path $outputDir $zipName

Remove-Item $zipPath -Force -ErrorAction SilentlyContinue
Compress-Archive -Path "$bundleDir\*" -DestinationPath $zipPath

Write-Host "✅ Created: $zipPath"

# Cleanup
Remove-Item $bundleDir -Recurse -Force