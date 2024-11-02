# Install PowerShell module locally
param (
    [Parameter(Mandatory = $false)]
    [boolean]$Release = $false  # Default to debug mode if not specified
)

$ModuleName = "Rsdk"
$SourceDirectory = "templates\\powershell"

# Build the executable using cargo
if ($Debug) {
    & cargo build --debug
    $ExePath = "target\\debug\\rsdk.exe"
} else {
    & cargo build --release
    $ExePath = "target\\release\\rsdk.exe"
}

# Get the PowerShell module path for the current user
$modulePath = Join-Path -Path $HOME -ChildPath "Documents\PowerShell\Modules"

Write-Host "Installing module in $modulePath"

# Determine the final module installation path
if (-not $ModuleName) {
    # If ModuleName is not specified, assume the directory name is the module name
    $ModuleName = (Get-Item -Path $SourceDirectory).BaseName
}
$destinationPath = Join-Path -Path $modulePath -ChildPath $ModuleName

# Remove any existing module installation to ensure a clean install
if (Test-Path -Path $destinationPath) {
    Write-Host "Removing existing module at $destinationPath"
    Remove-Item -Recurse -Force -Path $destinationPath
}
# Create the module directory without displaying output
New-Item -ItemType Directory -Path $destinationPath -Force | Out-Null

# Resolve the full path of the executable
$ExePath = (Resolve-Path -Path $ExePath).Path

# Copy module templates to the default Windows module directory and replace placeholders
Write-Host "Running module template script with executable path $ExePath"
& ".\build\windows\Module-Template.ps1" -SourceDir $SourceDirectory -DestinationDir $destinationPath -ExePath $ExePath

Write-Host "Module installed in $destinationPath"

# Attempt to load the module
Write-Host "Importing module $ModuleName"
# Remove the module if it's already loaded, to reload the latest version
Remove-Module -Name $ModuleName -ErrorAction SilentlyContinue
Import-Module $ModuleName

Write-Host "You may need to run 'Remove-Module $ModuleName' to reload the module in your session."
