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

# Copy the PowerShell module files except rsdk.psm1 from the source to the destination
Write-Host "Copying module files from $SourceDirectory to $destinationPath"
Copy-Item -Path "$SourceDirectory\*" -Destination $destinationPath -Recurse -Exclude "Rsdk.psm1"


$rsdkBinarySource = [System.IO.Path]::GetFullPath($ExePath)

# Generate rsdk.psm1 from template with the correct path to rsdk.exe
$psm1TemplatePath = Join-Path -Path $SourceDirectory -ChildPath "Rsdk.psm1"
$psm1DestinationPath = Join-Path -Path $destinationPath -ChildPath "Rsdk.psm1"
$rsdkPathEscaped = $rsdkBinarySource # -replace '\\', '\\\\'  # Escape backslashes for PowerShell

Write-Host "Generating rsdk.psm1 with rsdk.exe path $rsdkPathEscaped"
$templateContent = Get-Content -Path $psm1TemplatePath -Raw
$updatedContent = $templateContent -replace 'PUT_RSDK_PATH_HERE', $rsdkPathEscaped
Set-Content -Path $psm1DestinationPath -Value $updatedContent

Write-Host "Module installed in $destinationPath"

Remove-Module -Name $ModuleName -ErrorAction SilentlyContinue
Import-Module $ModuleName

Write-Host "Module $ModuleName imported"
