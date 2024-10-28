# install module locally

param (
    [Parameter(Mandatory = $true)]
    [string]$SourceDirectory,   # Path to the directory containing template files (e.g., .psd1, .psm1)

    [Parameter(Mandatory = $true)]
    [string]$ExePath,

    [string]$ModuleName = "Rsdk"
)

# Get the PowerShell module path for the current user
$targetModulePath = Join-Path -Path $HOME -ChildPath "Documents\PowerShell\Modules"

# Determine the final module installation path
if (-not $ModuleName) {
    # If ModuleName is not specified, assume the directory name is the module name
    $ModuleName = (Get-Item -Path $SourceDirectory).BaseName
}
$destinationPath = Join-Path -Path $targetModulePath -ChildPath $ModuleName

if (Test-Path -Path $destinationPath) {
    Remove-Item -Recurse -Force -Path $destinationPath
}
New-Item -ItemType Directory -Path $destinationPath -Force

# copy module templates to default windows module dir and fill them out
& "$PSScriptRoot\Module-Template.ps1" -SourceDirectory $SourceDirectory -DestinationDirectory $destinationPath -ExePath $ExePath

Write-Host "Module installed in $destinationPath"

Remove-Module -Name $ModuleName -ErrorAction SilentlyContinue
Import-Module $ModuleName

Write-Host "Module $ModuleName imported"
