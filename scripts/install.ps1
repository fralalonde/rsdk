[CmdletBinding()]
param(
    [string]$Version = $env:RSDK_VERSION,
    [switch]$Yes,
    [switch]$NoModifyShell,
    [string]$DownloadBaseUrl = $env:RSDK_DOWNLOAD_BASE_URL
)
$ErrorActionPreference = 'Stop'
$repository = if ($env:RSDK_REPOSITORY) { $env:RSDK_REPOSITORY } else { 'fralalonde/rsdk' }
$rsdkHome = if ($env:RSDK_HOME) { $env:RSDK_HOME } else { Join-Path $HOME '.rsdk' }
if (-not $Version) {
    $latest = Invoke-RestMethod -Uri "https://api.github.com/repos/$repository/releases/latest"
    $Version = $latest.tag_name.TrimStart('v')
}
if (-not $Version) { throw 'Unable to determine rsdk version; pass -Version VERSION.' }
$asset = "rsdk-$Version-windows-x86_64.zip"
if (-not $DownloadBaseUrl) { $DownloadBaseUrl = "https://github.com/$repository/releases/download/v$Version" }
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("rsdk-install-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
    $archive = Join-Path $tmp $asset
    Invoke-WebRequest -UseBasicParsing -Uri "$DownloadBaseUrl/$asset" -OutFile $archive
    try {
        $checksums = Join-Path $tmp 'checksums.txt'
        Invoke-WebRequest -UseBasicParsing -Uri "$DownloadBaseUrl/checksums.txt" -OutFile $checksums
        $expected = ((Get-Content $checksums | Where-Object { $_ -match ([regex]::Escape($asset) + '$') }) -split '\s+')[0]
        if ($expected -and (Get-FileHash -Algorithm SHA256 $archive).Hash.ToLowerInvariant() -ne $expected.ToLowerInvariant()) { throw 'Archive checksum does not match checksums.txt.' }
    } catch [System.Net.WebException] { }
    Expand-Archive -Path $archive -DestinationPath $tmp -Force
    foreach ($relative in @('bin/rsdk.exe','shell/bash/rsdk.bash','shell/zsh/rsdk.zsh','shell/fish/rsdk.fish','shell/powershell/Rsdk.psd1','shell/powershell/Rsdk.psm1','VERSION','checksums.txt')) {
        if (-not (Test-Path (Join-Path $tmp "rsdk/$relative"))) { throw "Release archive is missing rsdk/$relative" }
    }
    New-Item -ItemType Directory -Force -Path $rsdkHome | Out-Null
    # Preserve installed tools and cache when updating the release runtime.
    Remove-Item -Recurse -Force (Join-Path $rsdkHome 'bin'), (Join-Path $rsdkHome 'shell') -ErrorAction SilentlyContinue
    Remove-Item -Force (Join-Path $rsdkHome 'VERSION'), (Join-Path $rsdkHome 'checksums.txt') -ErrorAction SilentlyContinue
    Move-Item (Join-Path $tmp 'rsdk/bin'), (Join-Path $tmp 'rsdk/shell'), (Join-Path $tmp 'rsdk/VERSION'), (Join-Path $tmp 'rsdk/checksums.txt') -Destination $rsdkHome
    Write-Host "Installed rsdk $Version to $rsdkHome"
} finally { if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp } }
$module = Join-Path $rsdkHome 'shell/powershell/Rsdk.psd1'
if (-not $NoModifyShell) {
    $modify = $Yes
    if (-not $modify) { $modify = (Read-Host "Configure rsdk in $PROFILE now? [y/N]") -match '^(?i:y|yes)$' }
    if ($modify) {
        $profileDirectory = Split-Path -Parent $PROFILE
        New-Item -ItemType Directory -Force -Path $profileDirectory | Out-Null
        if (-not (Test-Path $PROFILE)) { New-Item -ItemType File -Path $PROFILE | Out-Null }
        $start = '# >>> rsdk initialize >>>'; $end = '# <<< rsdk initialize <<<'
        $profileText = Get-Content -Path $PROFILE -Raw
        if ($profileText -match [regex]::Escape($start) -or $profileText -match '(?i)(Import-Module|\.).*rsdk|rsdk.*init') {
            Write-Host "rsdk initialization already appears in $PROFILE; it was not modified."
            Write-Host "Standard block:`n$start`nImport-Module '$module' -Force`n$end"
        } else {
            Add-Content -Path $PROFILE -Value "`n$start`nImport-Module '$module' -Force`n$end"
        }
        Write-Host "Run now: Import-Module '$module' -Force"
    } else { Write-Host "Shell configuration not modified. Run now: Import-Module '$module' -Force" }
} else { Write-Host "Shell configuration not modified. Run now: Import-Module '$module' -Force" }
