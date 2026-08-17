param([switch]$Debug)

# Build and install rsdk from source for PowerShell. Mirrors what the release
# installer does for PowerShell (scripts/install.ps1), but uses the local
# build + templates instead of downloading a release archive.
#
# Usage: .\dev\Install-Module.ps1 [-Debug]

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    if ($Debug) {
        cargo build
        $targetDir = 'target/debug'
    } else {
        cargo build --release
        $targetDir = 'target/release'
    }
} finally {
    Pop-Location
}

$rsdkHome = if ($env:RSDK_HOME) { $env:RSDK_HOME } else { Join-Path $HOME '.rsdk' }

# Install binary + module templates (mirror the release layout).
$binDir = Join-Path $rsdkHome 'bin'
$moduleDir = Join-Path $rsdkHome 'shell/powershell'
New-Item -ItemType Directory -Force -Path $binDir, $moduleDir | Out-Null

# cargo names the binary `rsdk` on non-Windows; the module expects `rsdk.exe`.
$builtExe = Join-Path $targetDir 'rsdk.exe'
if (-not (Test-Path $builtExe)) { $builtExe = Join-Path $targetDir 'rsdk' }
Copy-Item -Path $builtExe -Destination (Join-Path $binDir 'rsdk.exe') -Force

Copy-Item -Path (Join-Path $repoRoot 'templates/powershell/Rsdk.psd1') -Destination $moduleDir -Force
Copy-Item -Path (Join-Path $repoRoot 'templates/powershell/Rsdk.psm1') -Destination $moduleDir -Force
Write-Host "Installed rsdk to $rsdkHome"

# Wire $PROFILE exactly like scripts/install.ps1 (guarded block).
$module = Join-Path $moduleDir 'Rsdk.psd1'
$profileDirectory = Split-Path -Parent $PROFILE
New-Item -ItemType Directory -Force -Path $profileDirectory | Out-Null
if (-not (Test-Path $PROFILE)) { New-Item -ItemType File -Path $PROFILE | Out-Null }
$start = '# >>> rsdk initialize >>>'; $end = '# <<< rsdk initialize <<<'
$profileText = Get-Content -Path $PROFILE -Raw
if ($profileText -match [regex]::Escape($start) -or $profileText -match '(?i)Import-Module.*rsdk') {
    Write-Host "rsdk initialization already appears in $PROFILE; it was not modified."
} else {
    Add-Content -Path $PROFILE -Value "`n$start`nImport-Module '$module' -Force`n$end"
    Write-Host "✓ Configured PowerShell via $PROFILE."
}
Write-Host "Run now: Import-Module '$module' -Force"
