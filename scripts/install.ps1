# rsdk installer for PowerShell
# Usage: irm https://github.com/fralalonde/rsdk/releases/latest/download/install.ps1 | iex
#
# Downloads the matching release zip, extracts it to ~/.rsdk/, and installs
# the PowerShell module to the user's PSModulePath so it auto-loads on next
# shell start. Does NOT touch $PROFILE.

$ErrorActionPreference = "Stop"

$REPO = "fralalonde/rsdk"
$RSDK_HOME = Join-Path $HOME ".rsdk"

function Write-Info($msg) {
    Write-Host "rsdk: " -ForegroundColor Cyan -NoNewline
    Write-Host $msg
}

function Write-Warn($msg) {
    Write-Host "rsdk: " -ForegroundColor Yellow -NoNewline
    Write-Host $msg
}

function Write-Fail($msg) {
    Write-Host "rsdk: " -ForegroundColor Red -NoNewline
    Write-Host $msg
    exit 1
}

function Get-ModulePath {
    # Return the standard PSModulePath for user modules.
    # PowerShell Core 6+: $HOME\Documents\PowerShell\Modules\
    # Windows PowerShell 5.1: $HOME\Documents\WindowsPowerShell\Modules\
    if ($IsWindows -or $PSVersionTable.OS -like "*Windows*") {
        $core = Join-Path $HOME "Documents\PowerShell\Modules"
        $legacy = Join-Path $HOME "Documents\WindowsPowerShell\Modules"
        return @($core, $legacy)
    } else {
        # Linux / macOS pwsh
        return @(Join-Path $HOME ".local/share/powershell/Modules")
    }
}

function Get-PlatformInfo {
    $os = if ($IsWindows -or $PSVersionTable.OS -like "*Windows*") { "windows" }
          elseif ($IsMacOS) { "mac" }
          elseif ($IsLinux) { "linux" }
          else { Write-Fail "unsupported OS: $($PSVersionTable.OS)" }

    $arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
        "X64"   { "x64" }
        "Arm64" { "arm64" }
        default { Write-Fail "unsupported arch: $([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)" }
    }

    $target = switch ("$os-$arch") {
        "windows-x64"   { "x86_64-pc-windows-msvc" }
        "windows-arm64" { "aarch64-pc-windows-msvc" }
        "linux-x64"     { "x86_64-unknown-linux-gnu" }
        "mac-arm64"     { "aarch64-apple-darwin" }
        default         { Write-Fail "no release for $os-$arch" }
    }

    @{ os = $os; arch = $arch; target = $target }
}

function Get-LatestReleaseUrl($rustTarget) {
    $apiUrl = "https://api.github.com/repos/$REPO/releases/latest"
    try {
        $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ "User-Agent" = "rsdk-installer" }
    } catch {
        Write-Fail "could not fetch latest release: $_"
    }

    $assetName = "rsdk-$($release.tag_name)-$rustTarget.zip"
    $asset = $release.assets | Where-Object { $_.name -eq $assetName }

    if (-not $asset) {
        Write-Fail "no asset $assetName in release $($release.tag_name)"
    }

    $asset.browser_download_url
}

function Install-Binary($url) {
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "rsdk-install-$(Get-Random)"
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        $zipPath = Join-Path $tmpDir "rsdk.zip"

        Write-Info "downloading $url"
        if ($PSVersionTable.PSEdition -eq "Core") {
            Invoke-WebRequest -Uri $url -OutFile $zipPath
        } else {
            (New-Object Net.WebClient).DownloadFile($url, $zipPath)
        }

        Write-Info "extracting..."
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        # Copy entire extracted contents (binary + powershell/ templates) to ~/.rsdk/
        New-Item -ItemType Directory -Path $RSDK_HOME -Force | Out-Null
        Get-ChildItem -Path $tmpDir | ForEach-Object {
            $dst = Join-Path $RSDK_HOME $_.Name
            if ($_.PSIsContainer) {
                Copy-Item -Path $_.FullName -Destination $dst -Recurse -Force
            } else {
                Copy-Item -Path $_.FullName -Destination $dst -Force
            }
        }

        Write-Info "binary installed at $(Join-Path $RSDK_HOME 'rsdk.exe')"
    } finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

function Install-Module {
    # Patch the module template with the absolute binary path and copy
    # it to every standard PSModulePath that exists.
    $exePath = Join-Path $RSDK_HOME "rsdk.exe"
    $moduleSrc = Join-Path $RSDK_HOME "powershell\Rsdk.psm1"

    if (-not (Test-Path $moduleSrc)) {
        Write-Warn "PowerShell module template not found in release (no pwsh/powershell build?)"
        return
    }

    # Read template, patch the placeholder
    $content = Get-Content -Path $moduleSrc -Raw
    $content = $content -replace 'PUT_RSDK_PATH_HERE', $exePath

    $installed = $false
    foreach ($modDir in Get-ModulePath) {
        $targetDir = Join-Path $modDir "Rsdk"
        New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
        Set-Content -Path (Join-Path $targetDir "Rsdk.psm1") -Value $content
        Write-Info "PowerShell module: $targetDir\Rsdk.psm1"
        $installed = $true
    }

    if ($installed) {
        Write-Info "PowerShell module installed — restart shell or run: Import-Module Rsdk"
    }
}

function Main {
    $platform = Get-PlatformInfo
    Write-Info "platform: $($platform.os)-$($platform.arch) ($($platform.target))  home: $RSDK_HOME"

    $exePath = Join-Path $RSDK_HOME "rsdk.exe"
    if (Test-Path $exePath) {
        Write-Info "reusing existing binary at $exePath"
    } else {
        $url = Get-LatestReleaseUrl $platform.target
        Install-Binary $url
    }

    Install-Module
}

Main
