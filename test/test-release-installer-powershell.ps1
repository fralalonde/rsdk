$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
foreach ($path in @('scripts/install.ps1', 'templates/powershell/Rsdk.psd1', 'templates/powershell/Rsdk.psm1')) {
    if (-not (Test-Path (Join-Path $root $path))) { throw "Missing $path" }
}
$module = Get-Content (Join-Path $root 'templates/powershell/Rsdk.psm1') -Raw
if ($module -notmatch '\$PSScriptRoot' -or $module -notmatch 'Export-ModuleMember -Function Invoke-Rsdk -Alias rsdk' -or $module -notmatch 'finally') { throw 'PowerShell module lacks self-relative, exported, cleanup integration.' }
if ($module -notmatch 'completions powershell' -or $module -notmatch '\$LASTEXITCODE') { throw 'PowerShell module lacks binary-generated completions registration.' }
$installer = Get-Content (Join-Path $root 'scripts/install.ps1') -Raw
if ($installer -notmatch '\[switch\]\$Yes' -or $installer -notmatch '\[switch\]\$NoModifyShell' -or $installer -notmatch '# >>> rsdk initialize >>>') { throw 'PowerShell installer lacks noninteractive managed-profile support.' }
if ($installer -notmatch 'Get-Command nu') { throw 'PowerShell installer lacks nushell binary detection.' }
if ($installer -notmatch "source '\`$rsdkHome") { throw 'PowerShell installer must single-quote the nushell source path (nu processes backslash escapes in double-quoted strings).' }
Write-Host 'PowerShell release installer static checks passed'
