function Invoke-Rsdk {
    [CmdletBinding()]
    param (
        [Parameter(
            Mandatory=$False,
            Position = 0
        )]
        [string]$Command,

        [Parameter(
            Mandatory=$False,
            ValueFromRemainingArguments=$true,
            Position = 1
        )]
        [string[]]$Args
    )

    $rsdkPath = "PUT_RSDK_PATH_HERE"

    # Default to launching the TUI when no command is given.
    if (-not $Command) { $Command = "tui" }

    $tempFile = New-TemporaryFile
    $tempFilePath = $tempFile.FullName

    # Build the argument list, appending --shell "powershell"
    $argumentList = @("--shell", "powershell", "--envout", $tempFilePath) + @($Command)

    # Only add $Args if it has non-empty content
    if ($Args -and $Args.Length -gt 0) {
        $argumentList += ($Args)
    }

    # Run rsdk.exe, capturing output live (tee-like behavior)
    & $rsdkPath $argumentList

    # Parse the output for environment variable changes and apply them
    if (Test-Path $tempFilePath) {
        $commands = Get-Content -Path $tempFilePath -Raw
        if (-not [string]::IsNullOrWhiteSpace($commands)) {
            Invoke-Expression $commands
        }
    }
}

# initialize module
Set-Alias -Name rsdk -Value Invoke-Rsdk -Scope Global
Invoke-Rsdk init
