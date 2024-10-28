# used in dev for local module install (full path to rsdk.exe) and at release time (assume rsdk.exe on path)

param (
    [Parameter(Mandatory = $true)]
    [string]$SourceDirectory,   # Path to the directory containing template files (e.g., .psd1, .psm1)

    [Parameter(Mandatory = $true)]
    [string]$DestinationDirectory, # Path to the directory where templates will be copied

    [Parameter(Mandatory = $true)]
    [string]$ExePath            # Path to the rsdk.exe binary
)

# Copy all module template files from the source to the destination, excluding Rsdk.psm1
Write-Host "Copying module template files from $SourceDirectory to $destinationPath"
Copy-Item -Path "$SourceDirectory\*" -Destination $destinationPath -Recurse -Exclude "Rsdk.psm1"

# Get the absolute path of the rsdk executable
$rsdkBinarySource = [System.IO.Path]::GetFullPath($ExePath)

# Generate rsdk.psm1 from template with the correct path to rsdk.exe
$psm1TemplatePath = Join-Path -Path $SourceDirectory -ChildPath "Rsdk.psm1"
$psm1DestinationPath = Join-Path -Path $destinationPath -ChildPath "Rsdk.psm1"
$rsdkPathEscaped = $rsdkBinarySource

Write-Host "Generating rsdk.psm1 with rsdk.exe path $rsdkPathEscaped"
$templateContent = Get-Content -Path $psm1TemplatePath -Raw
$updatedContent = $templateContent -replace 'PUT_RSDK_PATH_HERE', $rsdkPathEscaped
Set-Content -Path $psm1DestinationPath -Value $updatedContent

Write-Host "Templates successfully copied and configured in $destinationPath"
