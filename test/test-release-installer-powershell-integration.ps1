$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$fixture = Join-Path ([IO.Path]::GetTempPath()) ("rsdk-powershell-test-" + [guid]::NewGuid())
try {
    $moduleDir = Join-Path $fixture '.rsdk\shell\powershell'
    $binDir = Join-Path $fixture '.rsdk\bin'
    New-Item -ItemType Directory -Force -Path $moduleDir, $binDir | Out-Null
    Copy-Item (Join-Path $root 'templates\powershell\Rsdk.psd1') $moduleDir
    Copy-Item (Join-Path $root 'templates\powershell\Rsdk.psm1') $moduleDir
    Copy-Item "$HOME\.rsdk\rsdk.exe" (Join-Path $binDir 'rsdk.exe')

    Import-Module (Join-Path $moduleDir 'Rsdk.psd1') -Force
    if ((Get-Command rsdk -ErrorAction Stop).CommandType -ne 'Alias') { throw 'rsdk alias was not exported.' }
    rsdk --help | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "rsdk --help failed with $LASTEXITCODE" }
    Write-Host 'PowerShell module integration test passed'
}
finally {
    Remove-Module Rsdk -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $fixture -ErrorAction SilentlyContinue
}
