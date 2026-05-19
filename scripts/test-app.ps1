# Hermes Remote Manager - Automated Test Script
# Run with: powershell -File test-app.ps1

param(
  [switch]$SkipBuild,
  [switch]$Verbose
)

$ErrorActionPreference = "Continue"
$AppName = "hermes-remote-manager"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$RustDir = Join-Path $RepoRoot "src-tauri"
$LogDir = Join-Path $RustDir "target\release\logs"

Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Hermes Remote Manager - Test Suite" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan

$passed = 0
$failed = 0
$skipped = 0

function Test-Step {
  param($Name, $ScriptBlock)
  Write-Host "`n[TEST] $Name" -ForegroundColor Yellow
  try {
    $result = & $ScriptBlock
    if ($result) {
      Write-Host "[PASS] $Name" -ForegroundColor Green
      $script:passed++
    } else {
      Write-Host "[FAIL] $Name" -ForegroundColor Red
      $script:failed++
    }
  } catch {
    Write-Host "[FAIL] $Name : $_" -ForegroundColor Red
    $script:failed++
  }
}

function Test-Rust {
  param($Name, $Command)
  Write-Host "[TEST] $Name" -ForegroundColor Yellow
  Write-Host "  Running: $Command" -ForegroundColor Gray
  $output = Invoke-Expression "$Command 2>&1"
  $exitCode = $LASTEXITCODE
  if ($Verbose) { Write-Host $output }
  if ($exitCode -eq 0) {
    Write-Host "[PASS] $Name (exit 0)" -ForegroundColor Green
    $script:passed++
  } else {
    Write-Host "[FAIL] $Name (exit $exitCode)" -ForegroundColor Red
    if (!$Verbose) {
      Write-Host $output | Select-Object -Last 20
    }
    $script:failed++
  }
}

function Get-LatestLog {
  $logs = Get-ChildItem $LogDir -File -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1
  if ($logs) { return $logs.FullName }
  return $null
}

function Parse-LogForErrors {
  param($LogFile)
  if (!$LogFile -or !(Test-Path $LogFile)) { return @() }
  $errors = @()
  $lines = Get-Content $LogFile -ErrorAction SilentlyContinue
  foreach ($line in $lines) {
    if ($line -match "ERROR|error|panic|panicked") {
      $errors += $line
    }
  }
  return $errors
}

# ── Phase 1: Cargo checks ─────────────────────────────────────────
Write-Host "`n── Phase 1: Rust Compilation" -ForegroundColor Magenta

Test-Rust "cargo check" "cd $RustDir; cargo check 2>&1"

Test-Rust "cargo clippy (lint)" "cd $RustDir; cargo clippy -- -D warnings 2>&1"

if (!$SkipBuild) {
  Test-Rust "cargo build --release" "cd $RustDir; cargo build --release 2>&1"
}

# ── Phase 2: Rust Unit Tests ──────────────────────────────────────
Write-Host "`n── Phase 2: Rust Unit Tests" -ForegroundColor Magenta

Test-Rust "cargo test (unit tests)" "cd $RustDir; cargo test 2>&1"

# ── Phase 3: Frontend Build ───────────────────────────────────────
Write-Host "`n── Phase 3: Frontend Build" -ForegroundColor Magenta

Test-Rust "npm run build" "cd $RepoRoot; npm run build 2>&1"

Test-Rust "TypeScript check (tsc --noEmit)" "cd $RepoRoot; npx tsc --noEmit 2>&1"

# ── Phase 4: Full Tauri Build ─────────────────────────────────────
Write-Host "`n── Phase 4: Full Tauri Build" -ForegroundColor Magenta

if (!$SkipBuild) {
  Test-Rust "npm run tauri build" "cd $RepoRoot; npm run tauri build 2>&1"

  # Check output artifacts
  $exePath = Join-Path $RustDir "target\release\$AppName.exe"
  $msiDir = Join-Path $RustDir "target\release\bundle\msi"
  $nsisDir = Join-Path $RustDir "target\release\bundle\nsis"

  Write-Host "[TEST] Check .exe exists" -ForegroundColor Yellow
  if (Test-Path $exePath) {
    $size = [math]::Round((Get-Item $exePath).Length / 1MB, 1)
    Write-Host "[PASS] .exe exists ($size MB)" -ForegroundColor Green
    $passed++
  } else {
    Write-Host "[FAIL] .exe not found at $exePath" -ForegroundColor Red
    $failed++
  }

  Write-Host "[TEST] Check MSI bundle" -ForegroundColor Yellow
  $msiFiles = Get-ChildItem $msiDir -Filter "*.msi" -ErrorAction SilentlyContinue
  if ($msiFiles) {
    Write-Host "[PASS] MSI found: $($msiFiles.Name)" -ForegroundColor Green
    $passed++
  } else {
    Write-Host "[FAIL] No MSI bundle found" -ForegroundColor Red
    $failed++
  }

  Write-Host "[TEST] Check NSIS bundle" -ForegroundColor Yellow
  $nsisFiles = Get-ChildItem $nsisDir -Filter "*-setup.exe" -ErrorAction SilentlyContinue
  if ($nsisFiles) {
    Write-Host "[PASS] NSIS found: $($nsisFiles.Name)" -ForegroundColor Green
    $passed++
  } else {
    Write-Host "[FAIL] No NSIS installer found" -ForegroundColor Red
    $failed++
  }
}

# ── Phase 5: Log Analysis ──────────────────────────────────────────
Write-Host "`n── Phase 5: Log Analysis" -ForegroundColor Magenta

$logFile = Get-LatestLog
if ($logFile) {
  Write-Host "  Latest log: $logFile" -ForegroundColor Gray
  $errors = Parse-LogForErrors $logFile
  if ($errors.Count -eq 0) {
    Write-Host "[PASS] No errors in log file" -ForegroundColor Green
    $passed++
  } else {
    Write-Host "[WARN] Found $($errors.Count) error lines in log:" -ForegroundColor Yellow
    $errors | Select-Object -First 10 | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
    $skipped++
  }
} else {
  Write-Host "[INFO] No log files found (app may not have been run yet)" -ForegroundColor Gray
  $skipped++
}

# ── Phase 6: Git Status ────────────────────────────────────────────
Write-Host "`n── Phase 6: Git Status" -ForegroundColor Magenta

Push-Location $RepoRoot
$status = git status --porcelain 2>&1
if ($status) {
  Write-Host "[WARN] Uncommitted changes:" -ForegroundColor Yellow
  $status | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
  $skipped++
} else {
  Write-Host "[PASS] Working tree clean" -ForegroundColor Green
  $passed++
}
Pop-Location

# ── Phase 7: Package.json integrity ───────────────────────────────
Write-Host "`n── Phase 7: Package Integrity" -ForegroundColor Magenta

Test-Rust "npm install (check deps)" "cd $RepoRoot; npm install --dry-run 2>&1"

# ── Results ────────────────────────────────────────────────────────
Write-Host "`n═══════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  RESULTS: $passed passed, $failed failed, $skipped skipped" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan

if ($failed -gt 0) {
  Write-Host "`n[FAILED] Tests failed. Fix errors before proceeding." -ForegroundColor Red
  exit 1
} else {
  Write-Host "`n[OK] All automated tests passed!" -ForegroundColor Green
  exit 0
}