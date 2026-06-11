# Generate Changelog Script
# Usage: .\scripts\generate-changelog.ps1 -FromTag "v0.1.0" -ToTag "v0.2.0"

param(
    [string]$FromTag,
    [string]$ToTag = "HEAD"
)

$ErrorActionPreference = "Stop"

Write-Host "=== Generating Changelog ===" -ForegroundColor Cyan

# Get commits
if ($FromTag) {
    Write-Host "Commits from $FromTag to $ToTag..." -ForegroundColor Yellow
    $commits = git log "$FromTag..$ToTag" --pretty=format:"%h %s" --no-merges
} else {
    Write-Host "Last 20 commits..." -ForegroundColor Yellow
    $commits = git log --pretty=format:"%h %s" --no-merges -20
}

# Categorize commits
$features = @()
$fixes = @()
$chores = @()

foreach ($commit in $commits -split "`n") {
    if ($commit -match '^[a-f0-9]+ feat') {
        $features += $commit
    } elseif ($commit -match '^[a-f0-9]+ fix') {
        $fixes += $commit
    } else {
        $chores += $commit
    }
}

# Generate markdown
$changelog = @"
# Changelog

## [Unreleased]

### Added
$(if ($features) { $features | ForEach-Object { "- $_" } | Out-String } else { "- No features added" })

### Fixed
$(if ($fixes) { $fixes | ForEach-Object { "- $_" } | Out-String } else { "- No fixes" })

### Changed
$(if ($chores) { $chores | ForEach-Object { "- $_" } | Out-String } else { "- No changes" })
"@

Write-Host "`nGenerated Changelog:" -ForegroundColor Green
Write-Host $changelog

# Save to file
$changelog | Out-File -FilePath "CHANGELOGGenerated.md" -Encoding utf8
Write-Host "`nSaved to CHANGELOGGenerated.md" -ForegroundColor Cyan
