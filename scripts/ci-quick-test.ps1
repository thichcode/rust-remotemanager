# Quick validation script - run in CI or locally
# Run with: powershell -File ci-quick-test.ps1

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ErrorActionPreference = "Stop"

Write-Host "[1/4] TypeScript check..." -ForegroundColor Cyan
npm run build 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "[FAIL] TypeScript failed" -ForegroundColor Red; exit 1 }
Write-Host "[OK] TypeScript passed" -ForegroundColor Green

Write-Host "[2/4] Rust check..." -ForegroundColor Cyan
cd src-tauri; cargo check 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "[FAIL] cargo check failed" -ForegroundColor Red; exit 1 }
Write-Host "[OK] Rust check passed" -ForegroundColor Green

Write-Host "[3/4] Rust unit tests..." -ForegroundColor Cyan
cargo test 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "[FAIL] cargo test failed" -ForegroundColor Red; exit 1 }
Write-Host "[OK] Rust tests passed" -ForegroundColor Green

Write-Host "[4/4] Tauri build..." -ForegroundColor Cyan
cd ..; npm run tauri build 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "[FAIL] tauri build failed" -ForegroundColor Red; exit 1 }
Write-Host "[OK] Tauri build passed" -ForegroundColor Green

Write-Host ""
Write-Host "[ALL DONE] CI quick test passed!" -ForegroundColor Green
exit 0