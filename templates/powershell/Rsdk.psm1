function Invoke-RsdkCommand {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$Command,
        [string[]]$Args
    )

    # Find the path to rsdk.exe dynamically
    $rsdkPath = "PUT_RSDK_PATH_HERE"

    # Build the argument list, appending --shell "powershell"
    $argumentList = @("--shell", "powershell", $Command) + $Args

    # Run rsdk.exe, capturing output live (tee-like behavior)
    $output = & $rsdkPath $argumentList
    #| Tee-Object -Variable teeOutput

    # Parse the output for environment variable changes and apply them
    foreach ($line in $teeOutput) {
        if ($line -match '^#cmdmagic#([A-Za-z_]+)="(.*)"') {
            $varName = $matches[1]
            $varValue = $matches[2]

            # Apply the environment variable
            Set-Item -Path "Env:$varName" -Value $varValue
            Write-Host "Set environment variable $varName to $varValue"
        }
        else {
            Write-Host $line
        }
    }
}

# Example command to install an SDK via rsdk.exe
function Rsdk-Install {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$SDK,

        [Parameter(Mandatory = $true)]
        [string]$Version
    )

    try {
        Invoke-RsdkCommand -Command "install" -Args @($SDK, $Version)
    } catch {
        Write-Error "Failed to install $SDK version $Version. Error: $_"
    }
}

# Command to uninstall an SDK via rsdk.exe
function Rsdk-Uninstall {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$SDK
    )

    try {
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
        [Parameter(Mandatory = $false)]
        [string]$Candidate
    )

    try {
        Invoke-RsdkCommand -Command "list" -Args @()
    } catch {
        Write-Error "Failed to list SDKs. Error: $_"
    }
}