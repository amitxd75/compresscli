# CompressCLI PowerShell Completion Setup Script for Windows
# Run this script in PowerShell to enable tab completion for compresscli

$ErrorActionPreference = "Stop"

Write-Host "Setting up CompressCLI completions for PowerShell..." -ForegroundColor Cyan

# Determine PowerShell Profile Path
$ProfilePath = $PROFILE.CurrentUserAllHosts
if (-not $ProfilePath) {
    $ProfilePath = $PROFILE
}

$ProfileDir = Split-Path -Parent $ProfilePath
if (-not (Test-Path $ProfileDir)) {
    New-Item -ItemType Directory -Path $ProfileDir -Force | Out-Null
}

if (-not (Test-Path $ProfilePath)) {
    New-Item -ItemType File -Path $ProfilePath -Force | Out-Null
}

# Generate PowerShell completion script using compresscli
$CompletionScript = compresscli completions powershell

# Append completion to profile if not already present
$Marker = "# CompressCLI PowerShell Completion"
$ProfileContent = Get-Content $ProfilePath -Raw -ErrorAction SilentlyContinue

if ($ProfileContent -notmatch $Marker) {
    Add-Content -Path $ProfilePath -Value "`n$Marker`n$CompletionScript"
    Write-Host "CompressCLI completions added to $ProfilePath" -ForegroundColor Green
    Write-Host "Please restart your PowerShell session to apply completion changes." -ForegroundColor Yellow
} else {
    Write-Host "CompressCLI completions already exist in $ProfilePath." -ForegroundColor Green
}
