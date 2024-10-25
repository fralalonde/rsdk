# Helper function to find rsdk.exe dynamically
function Get-RsdkPath {
    # Try to locate rsdk.exe in the system's PATH
    $rsdkPath = (Get-Command rsdk.exe -ErrorAction SilentlyContinue).Path

    # If rsdk.exe is not found in the PATH, throw an error
    if (-not $rsdkPath) {
        throw "Could not find rsdk.exe. Please ensure it is installed and in your PATH."
    }

    return $rsdkPath
}

# Helper function to invoke rsdk.exe, tee the output to the console, and parse environment variables
function Invoke-RsdkCommand {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory=$true)]
        [string]$Command,

        [string[]]$Args
    )

    # Find the path to rsdk.exe dynamically
    $rsdkPath = Get-RsdkPath

    # Build the argument list, appending --shell "powershell"
    $argumentList = @($Command) + $Args + "--shell" + "powershell"

    # Create a temporary file to capture the output
    $tempFile = New-TemporaryFile

    # Run rsdk.exe, capturing its output to a temporary file and displaying it live (tee-like behavior)
    $process = Start-Process -FilePath $rsdkPath -ArgumentList $argumentList `
                             -RedirectStandardOutput $tempFile -NoNewWindow -PassThru -Wait

    # After the process finishes, read the output and display it live as it is read
    $output = Get-Content -Path $tempFile | Tee-Object -Variable teeOutput

    # Parse the output for environment variable changes and apply them
    foreach ($line in $teeOutput) {
        if ($line -match "^\$env:([A-Za-z_]+)='([^']+)'") {
            $varName = $matches[1]
            $varValue = $matches[2]

            # Apply the environment variable
            Set-Item -Path "Env:$varName" -Value $varValue
            Write-Host "Set environment variable $varName to $varValue"
        }
    }

    # Clean up the temporary file
    Remove-Item $tempFile
}

# Example command to install an SDK via rsdk.exe
function Rsdk-Install {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory=$true)]
        [string]$SDK,

        [Parameter(Mandatory=$true)]
        [string]$Version
    )

    Invoke-RsdkCommand -Command "install" -Args @($SDK, $Version)
}

# Command to uninstall an SDK via rsdk.exe
function Rsdk-Uninstall {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory=$true)]
        [string]$SDK
    )

    Invoke-RsdkCommand -Command "uninstall" -Args @($SDK)
}

# Command to set a default SDK via rsdk.exe
function Rsdk-Default {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory=$true)]
        [string]$SDK,

        [Parameter(Mandatory=$true)]
        [string]$Version
    )

    Invoke-RsdkCommand -Command "default" -Args @($SDK, $Version)
}

# Command to use a specific SDK version via rsdk.exe
function Rsdk-Use {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory=$true)]
        [string]$SDK,

        [Parameter(Mandatory=$true)]
        [string]$Version
    )

    Invoke-RsdkCommand -Command "use" -Args @($SDK, $Version)
}

# Command to flush the SDK cache via rsdk.exe
function Rsdk-Flush {
    [CmdletBinding()]
    param ()

    Invoke-RsdkCommand -Command "flush" -Args @()
}

# Command to list available SDKs via rsdk.exe
function Rsdk-List {
    [CmdletBinding()]
    param ()

    Invoke-RsdkCommand -Command "list" -Args @()
}
