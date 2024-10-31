# install module locally

param (
    [Parameter(Mandatory = $true)]
    [string]$SourceDirectory,   # Path to the directory containing template files (e.g., .psd1, .psm1)

    [Parameter(Mandatory = $true)]
    [string]$ExePath,

    [string]$ModuleName = "Rsdk"
)

# Get the PowerShell module path for the current user
$modulePath = Join-Path -Path $HOME -ChildPath "Documents\PowerShell\Modules"

# Write-Host "Installing module in $modulePath"

# Determine the final module installation path
if (-not $ModuleName) {
    # If ModuleName is not specified, assume the directory name is the module name
    $ModuleName = (Get-Item -Path $SourceDirectory).BaseName
}
$destinationPath = Join-Path -Path $modulePath -ChildPath $ModuleName

# Write-Host "Copying files to in $destinationPath"

if (Test-Path -Path $destinationPath) {
    Remove-Item -Recurse -Force -Path $destinationPath
}
# suppress useless command output, FML
New-Item -ItemType Directory -Path $destinationPath -Force | Out-Null

$ExePath = $rsdkBinarySource = [System.IO.Path]::GetFullPath($ExePath)

# copy module templates to default windows module dir and fill them out
& ".\build\windows\Module-Template.ps1" -SourceDir $SourceDirectory -DestinationDir $destinationPath -ExePath $ExePath

Write-Host "Module installed in $destinationPath"

# Remove-Module -Name $ModuleName -ErrorAction SilentlyContinue
Import-Module $ModuleName

Write-Host "Remove-Module -Name $ModuleName to remove previous version"
