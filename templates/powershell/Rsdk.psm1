$script:RsdkBinary = Join-Path (Join-Path $PSScriptRoot '..\..\bin') 'rsdk.exe'

function Invoke-Rsdk {
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)] [string] $Command = '--help',
        [Parameter(ValueFromRemainingArguments = $true, Position = 1)] [string[]] $Arguments
    )
    $temporaryFile = New-TemporaryFile
    try {
        $argumentList = @('--shell', 'powershell', '--envout', $temporaryFile.FullName, $Command) + @($Arguments)
        & $script:RsdkBinary @argumentList
        $exitCode = $LASTEXITCODE
        if ((Test-Path -LiteralPath $temporaryFile.FullName) -and (Get-Item -LiteralPath $temporaryFile.FullName).Length -gt 0) {
            Invoke-Expression (Get-Content -LiteralPath $temporaryFile.FullName -Raw)
        }
        return $exitCode
    } finally {
        Remove-Item -LiteralPath $temporaryFile.FullName -Force -ErrorAction SilentlyContinue
    }
}

Set-Alias -Name rsdk -Value Invoke-Rsdk -Scope Script
Export-ModuleMember -Function Invoke-Rsdk -Alias rsdk
