function Invoke-RsdkCommand {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$Command,
        [string[]]$Args
    )

    # Find the path to rsdk.exe dynamically
    $rsdkPath = "PUT_RSDK_PATH_HERE"

    $tempFile = New-TemporaryFile
    $tempFilePath = $tempFile.FullName

    # Build the argument list, appending --shell "powershell"
    $argumentList = @("--shell", "powershell", "--envout", $tempFilePath, $Command) + $Args

    # Run rsdk.exe, capturing output live (tee-like behavior)
    write-host "$rsdkPath $argumentList"
    & $rsdkPath $argumentList

    # Parse the output for environment variable changes and apply them
    if (Test-Path $tempFilePath) {
        $commands = Get-Content -Path $tempFilePath -Raw
        Invoke-Expression $commands
    }
}

# Example command to install an SDK via rsdk.exe
function Rsdk-Install {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$Candidate,

        [string]$Version
    )

    try {
        $args = @($Candidate)
        if ($Version) {
            $args += $Version
        }
        Invoke-RsdkCommand -Command "install" -Args $args
    } catch {
        Write-Error "Failed to install $SDK version $Version. Error: $_"
    }
}

# Command to uninstall an SDK via rsdk.exe
function Rsdk-Uninstall {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$SDK,

        [string]$Version
    )

    try {
        $args = @($SDK)
        if ($Version) {
            $args += $Version
        }
        Invoke-RsdkCommand -Command "uninstall" -Args @($SDK)
    } catch {
        Write-Error "Failed to uninstall $SDK. Error: $_"
    }
}

# Command to set a default SDK via rsdk.exe
function Rsdk-Default {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$SDK,

        [Parameter(Mandatory = $true)]
        [string]$Version
    )

    try {
        Invoke-RsdkCommand -Command "default" -Args @($SDK, $Version)
    } catch {
        Write-Error "Failed to set default SDK $SDK to version $Version. Error: $_"
    }
}

# Command to use a specific SDK version via rsdk.exe
function Rsdk-Use {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$SDK,

        [Parameter(Mandatory = $true)]
        [string]$Version
    )

    try {
        Invoke-RsdkCommand -Command "use" -Args @($SDK, $Version)
    } catch {
        Write-Error "Failed to set SDK $SDK to use version $Version. Error: $_"
    }
}

# Command to flush the SDK cache via rsdk.exe
function Rsdk-Flush {
    [CmdletBinding()]
    param ()

    try {
        Invoke-RsdkCommand -Command "flush" -Args @()
    } catch {
        Write-Error "Failed to flush SDK cache. Error: $_"
    }
}

# Command to list available SDKs via rsdk.exe
function Rsdk-List {
    [CmdletBinding()]
    param (
        [string]$Candidate
    )

    try {
        $args = @()
        if ($Version) {
            $args += $Candidate
        }
        Invoke-RsdkCommand -Command "list" -Args $args
    } catch {
        Write-Error "Failed to list SDKs. Error: $_"
    }
}