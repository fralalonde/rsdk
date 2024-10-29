# used in dev for local module install (full path to rsdk.exe) and at release time (assume rsdk.exe on path)

param (
    [Parameter(Mandatory = $true)]
    [string]$SourceDir,

    [Parameter(Mandatory = $true)]
    [string]$DestinationDir,

    [Parameter(Mandatory = $true)]
    [string]$ExePath            # Path to the rsdk.exe binary
)

# Copy all module template files from the source to the destination, excluding Rsdk.psm1
# Ensure the destination path and any parent directories exist
if (-not (Test-Path -Path $DestinationDir)) {
    Write-Host "Creating directory $DestinationDir and its parent directories"
    New-Item -ItemType Directory -Path $DestinationDir -Force
}
Write-Host "Copying module template files from $SourceDir to $DestinationDir"
Copy-Item -Path "$SourceDir\*" -Destination $DestinationDir -Recurse -Exclude "Rsdk.psm1"

# Get the absolute path of the rsdk executable
$rsdkBinarySource = [System.IO.Path]::GetFullPath($ExePath)

# Generate rsdk.psm1 from template with the correct path to rsdk.exe
$psm1TemplatePath = Join-Path -Path $SourceDir -ChildPath "Rsdk.psm1"
$psm1DestinationPath = Join-Path -Path $DestinationDir -ChildPath "Rsdk.psm1"
$rsdkPathEscaped = $rsdkBinarySource

Write-Host "Generating rsdk.psm1 with rsdk.exe path $rsdkPathEscaped"
$templateContent = Get-Content -Path $psm1TemplatePath -Raw
$updatedContent = $templateContent -replace 'PUT_RSDK_PATH_HERE', $rsdkPathEscaped
Set-Content -Path $psm1DestinationPath -Value $updatedContent

Write-Host "Templates successfully copied and configured in $DestinationDir"
