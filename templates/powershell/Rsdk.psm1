function Invoke-Rsdk {
    [CmdletBinding()]
    param (
        [Parameter(
            Mandatory=$True,
            Position = 0
        )]
        [string]$Command,

        [Parameter(
            Mandatory=$False,
            ValueFromRemainingArguments=$true,
            Position = 1
        )][string[]]
        [string[]]$Args
    )

    # Find the path to rsdk.exe dynamically
    $rsdkPath = "PUT_RSDK_PATH_HERE"

    $tempFile = New-TemporaryFile
    $tempFilePath = $tempFile.FullName

    # Build the argument list, appending --shell "powershell"
    $argumentList = @("--shell", "powershell", "--envout", $tempFilePath) + @($Command)

    # Only add $Args if it has non-empty content
    if ($Args -and $Args.Length -gt 0) {
        $argumentList += ($Args)
    }

    # Run rsdk.exe, capturing output live (tee-like behavior)
    # write-host "$rsdkPath $argumentList"
    & $rsdkPath $argumentList

    # Parse the output for environment variable changes and apply them
    if (Test-Path $tempFilePath) {
        $commands = Get-Content -Path $tempFilePath -Raw
        if (-not [string]::IsNullOrWhiteSpace($commands)) {
            # write-host "envout contains $commands"
            Invoke-Expression $commands
        }
    }
}

# initialize module
Set-Alias -Name rsdk -Value Invoke-Rsdk -Scope Global
Invoke-Rsdk attach

# Example command to install an SDK via rsdk.exe
function Install-Rsdk {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$Candidate,

        [string]$Version
    )

    $args = @($Candidate)
    if ($Version) {
        $args += $Version
    }
    Invoke-Rsdk -Command "install" -Args $args
}

# Command to uninstall an SDK via rsdk.exe
function Uninstall-Rsdk {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$Candidate,

        [string]$Version
    )

    $args = @($Candidate)
    if ($Version) {
        $args += $Version
    }
    Invoke-Rsdk -Command "uninstall" -Args $args
}

# Command to use a specific SDK version via rsdk.exe
function Select-Rsdk {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$Candidate,

        [Parameter(Mandatory = $true)]
        [string]$Version,

        [Parameter(Mandatory = $false)]
        [switch]$Default
    )

    if ($Default) {
        Invoke-Rsdk -Command "default" -Args @($Candidate, $Version)
        Write-Host "Set $Candidate version $Version as the default."
    }

    Invoke-Rsdk -Command "use" -Args @($Candidate, $Version)
    Write-Host "Using $Candidate version $Version for the current session."
}

# Command to flush the SDK cache via rsdk.exe
function Reset-Rsdk {
    [CmdletBinding()]
    param ()

    Invoke-Rsdk -Command "flush" -Args @()
}

# Command to list available SDKs via rsdk.exe
function Show-Rsdk {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $false)]
        [string]$Candidate,

        [Parameter(Mandatory = $false)]
        [switch]$Installed
    )

    $args = @()
    if ($Candidate) {
        $args += $Candidate
    }
    if ($Installed) {
        $args += '--installed'
    }
    Invoke-Rsdk -Command "list" -Args $args
}
