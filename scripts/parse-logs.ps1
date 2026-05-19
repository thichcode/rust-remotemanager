# Hermes Log Parser
# Parse and analyze Rust logs for errors/warnings
# Run with: powershell -File parse-logs.ps1

param(
  [string]$LogPath = "src-tauri\target\release\logs",
  [switch]$ShowAll,
  [switch]$ShowDebug,
  [int]$LastNHours = 24
)

$ErrorActionPreference = "Continue"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$LogDir = Join-Path $RepoRoot $LogPath

if (!(Test-Path $LogDir)) {
  Write-Host "[ERROR] Log directory not found: $LogDir" -ForegroundColor Red
  exit 1
}

$cutoff = (Get-Date).AddHours(-$LastNHours)
$logFiles = Get-ChildItem $LogDir -File | Where-Object { $_.LastWriteTime -gt $cutoff } | Sort-Object LastWriteTime -Descending

if ($logFiles.Count -eq 0) {
  Write-Host "[INFO] No log files found in last $LastNHours hours" -ForegroundColor Yellow
  exit 0
}

Write-Host "Found $($logFiles.Count) log file(s)" -ForegroundColor Cyan
Write-Host ""

$allErrors = @()
$allWarnings = @()
$allInfo = @()
$sessionMap = @{}

foreach ($log in $logFiles) {
  Write-Host "=== $($log.Name) ===" -ForegroundColor Gray
  $lines = Get-Content $log.FullName

  foreach ($line in $lines) {
    # Parse session IDs
    if ($line -match "([a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12})") {
      $sid = $Matches[1]
      if (!$sessionMap.ContainsKey($sid)) {
        $sessionMap[$sid] = $line
      }
    }

    # Parse by level
    if ($line -match "^\S+ \S+ (ERROR|error)") {
      $allErrors += $line
      if ($ShowAll) { Write-Host "  [E] $line" -ForegroundColor Red }
    } elseif ($line -match "^\S+ \S+ WARN") {
      $allWarnings += $line
      if ($ShowAll) { Write-Host "  [W] $line" -ForegroundColor Yellow }
    } elseif ($line -match "^\S+ \S+ DEBUG") {
      $allInfo += $line
      if ($ShowDebug) { Write-Host "  [D] $line" -ForegroundColor Gray }
    } elseif ($ShowAll) {
      Write-Host "  $line" -ForegroundColor White
    }
  }
  Write-Host ""
}

# Summary
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  SUMMARY" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Log files: $($logFiles.Count)"
Write-Host "  Errors: $($allErrors.Count)"
Write-Host "  Warnings: $($allWarnings.Count)"
Write-Host "  Debug lines: $($allInfo.Count)"
Write-Host "  Unique sessions: $($sessionMap.Count)"

if ($sessionMap.Count -gt 0) {
  Write-Host "  Sessions:" -ForegroundColor White
  $sessionMap.GetEnumerator() | ForEach-Object {
    $firstLine = $_.Value
    if ($firstLine.Length -gt 80) { $firstLine = $firstLine.Substring(0, 80) + "..." }
    Write-Host "    $($_.Key): $firstLine" -ForegroundColor Gray
  }
}

Write-Host ""

if ($allErrors.Count -gt 0) {
  Write-Host "── Top Errors ──" -ForegroundColor Red
  $allErrors | Select-Object -First 10 | ForEach-Object {
    Write-Host "  $_" -ForegroundColor Red
  }
}

if ($allWarnings.Count -gt 0) {
  Write-Host "── Top Warnings ──" -ForegroundColor Yellow
  $allWarnings | Select-Object -First 10 | ForEach-Object {
    Write-Host "  $_" -ForegroundColor Yellow
  }
}

Write-Host ""
if ($allErrors.Count -eq 0) {
  Write-Host "[OK] No errors found!" -ForegroundColor Green
  exit 0
} else {
  Write-Host "[WARN] Found $($allErrors.Count) errors. Review above." -ForegroundColor Yellow
  exit 1
}