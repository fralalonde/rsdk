# rsdk installer for PowerShell
# Usage: irm https://github.com/fralalonde/rsdk/releases/latest/download/install.ps1 | iex
#
# This script detects platform, downloads the appropriate release zip,
# extracts the binary, and sets up shell integration.
# Re-running reuses an already-installed binary.

$ErrorActionPreference = "Stop"

$REPO = "fralalonde/rsdk"
$RSDK_HOME = Join-Path $HOME ".rsdk"

function Write-Info($msg) {
    Write-Host "rsdk: " -ForegroundColor Cyan -NoNewline
    Write-Host $msg
}

function Write-Fail($msg) {
    Write-Host "rsdk: " -ForegroundColor Red -NoNewline
    Write-Host $msg
    exit 1
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
        "windows-arm64" { "aarch64-pc-windows-msvc" }  # future-proof
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

        $exeSrc = Join-Path $tmpDir "rsdk.exe"
        if (-not (Test-Path $exeSrc)) {
            Write-Fail "zip is missing rsdk.exe"
        }

        New-Item -ItemType Directory -Path $RSDK_HOME -Force | Out-Null
        $exeDst = Join-Path $RSDK_HOME "rsdk.exe"
        Copy-Item $exeSrc $exeDst -Force

        Write-Info "binary installed at $exeDst"
    } finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

function Install-ShellIntegration {
    $wrapperSrc = Join-Path $RSDK_HOME "powershell\Rsdk.psm1"
    if (Test-Path $wrapperSrc) {
        Write-Info "shell wrapper at $wrapperSrc"
    }

    $profileFile = $PROFILE
    $markerStart = "# >>> rsdk >>>"
    $markerEnd = "# <<< rsdk <<<"

    if (Test-Path $profileFile) {
        $profileContent = Get-Content $profileFile -Raw -ErrorAction SilentlyContinue
        if ($profileContent -match [regex]::Escape($markerStart)) {
            # Remove old snippet
            $lines = Get-Content $profileFile
            $newLines = @()
            $skip = $false
            foreach ($line in $lines) {
                if ($line -match [regex]::Escape($markerStart)) { $skip = $true }
                if (-not $skip) { $newLines += $line }
                if ($line -match [regex]::Escape($markerEnd)) { $skip = $false }
            }
            Set-Content $profileFile ($newLines -join "`n")
        }
    }

    $snippet = @"

$markerStart
`$env:PATH = "$RSDK_HOME;`$env:PATH"
if (Test-Path "$RSDK_HOME\powershell\Rsdk.psm1") {
    Import-Module "$RSDK_HOME\powershell\Rsdk.psm1" -Force
}
$markerEnd

"@

    New-Item -ItemType Directory -Path (Split-Path $profileFile) -Force | Out-Null
    Add-Content -Path $profileFile -Value $snippet

    Write-Info "shell integration written to $profileFile"
    Write-Info "restart PowerShell or run: . `$PROFILE"

    # Source for current session
    $env:PATH = "$RSDK_HOME;$env:PATH"
    if (Test-Path "$RSDK_HOME\powershell\Rsdk.psm1") {
        Import-Module "$RSDK_HOME\powershell\Rsdk.psm1" -Force
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

    Install-ShellIntegration
}

Main
