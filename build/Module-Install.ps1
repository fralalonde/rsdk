param (
    [Parameter(Mandatory = $true)]
    [string]$SourceDirectory,   # Path to the local directory containing the module files (e.g., .psd1, .psm1 files)

    [string]$TargetDir = "target/debug",  # Default to `target/debug` for rsdk.exe debug version

    [string]$ModuleName         # Optional: Module name if it differs from the source directory name
)

# Get the PowerShell module path for the current user
$targetModulePath = Join-Path -Path $HOME -ChildPath "Documents\PowerShell\Modules"

# Determine the final module installation path
if (-not $ModuleName) {
    # If ModuleName is not specified, assume the directory name is the module name
    $ModuleName = (Get-Item -Path $SourceDirectory).BaseName
}
$destinationPath = Join-Path -Path $targetModulePath -ChildPath $ModuleName

Write-Host "Installing module from $SourceDirectory and rsdk.exe from $TargetDir to $destinationPath..."

# Check if the destination path already exists and prompt for overwrite if necessary
if (Test-Path -Path $destinationPath) {
    $overwrite = Read-Host "Module already exists. Do you want to overwrite? (y/n)"
    if ($overwrite -ne 'y') {
        Write-Host "Installation aborted."
        exit
    }

    # Remove existing module files
    Remove-Item -Recurse -Force -Path $destinationPath
}

# Copy the PowerShell module files except rsdk.psm1 from the source to the destination
Write-Host "Copying module files from $SourceDirectory..."
Copy-Item -Path "$SourceDirectory\*" -Destination $destinationPath -Recurse -Exclude "rsdk.psm1"

# Copy rsdk.exe binary to the module directory
$rsdkBinarySource = Join-Path -Path $TargetDir -ChildPath "rsdk.exe"
$rsdkBinaryDestination = Join-Path -Path $destinationPath -ChildPath "rsdk.exe"

if (Test-Path -Path $rsdkBinarySource) {
    Write-Host "Copying rsdk.exe from $TargetDir to module directory..."
    Copy-Item -Path $rsdkBinarySource -Destination $rsdkBinaryDestination
} else {
    Write-Error "rsdk.exe not found in target directory: $TargetDir. Make sure it has been built in debug mode."
    exit 1
}

# Generate rsdk.psm1 with the correct path to rsdk.exe
$psm1TemplatePath = Join-Path -Path $SourceDirectory -ChildPath "rsdk.psm1"
$psm1DestinationPath = Join-Path -Path $destinationPath -ChildPath "rsdk.psm1"
$rsdkPath = Join-Path -Path $destinationPath -ChildPath "rsdk.exe" -replace '\\', '\\\\'  # Escape backslashes for PowerShell

Write-Host "Generating rsdk.psm1 with rsdk.exe path $rsdkPath..."
$templateContent = Get-Content -Path $psm1TemplatePath -Raw
$updatedContent = $templateContent -replace 'PUT_RSDK_PATH_HERE', $rsdkPath
Set-Content -Path $psm1DestinationPath -Value $updatedContent

Write-Host "Module installed successfully in $destinationPath."
Write-Host "You can now import the module using 'Import-Module $ModuleName'."
