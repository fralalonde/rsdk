param (
    [Parameter(Mandatory = $true)]
    [string]$Version                   # New version for the release, e.g., "1.1.0"
)

# Define file paths
$PsModuleTemplatePath = "./templates/build/rsdk.psm1"
$PsManifestTemplatePath = "./templates/build/rsdk.psd1"
$PsModulePath = "./powershell/rsdk.psm1"
$PsManifestPath = "./powershell/rsdk.psd1"

# Ensure the version tag is unique
Write-Host "Checking if version tag v$Version already exists..."
if (git tag --list "v$Version" | Select-String -Pattern "^v$Version$") {
    Write-Error "Version tag v$Version already exists. Please choose a unique version."
    exit 1
}

# Step 1: Update PowerShell files from templates
Write-Host "Updating PowerShell module manifest (.psd1) and module script (.psm1) with new version $Version..."

# Update rsdk.psd1
$psManifestContent = Get-Content -Path $PsManifestTemplatePath -Raw
$updatedPsManifestContent = $psManifestContent -replace 'PUT_YOUR_VERSION_HERE', $Version
Set-Content -Path $PsManifestPath -Value $updatedPsManifestContent

# Update rsdk.psm1 to use `rsdk.exe` directly, assuming it's in the PATH (for Scoop usage)
$psModuleContent = Get-Content -Path $PsModuleTemplatePath -Raw
$updatedPsModuleContent = $psModuleContent -replace 'PUT_RSDK_PATH_HERE', 'rsdk.exe'
Set-Content -Path $PsModulePath -Value $updatedPsModuleContent

Write-Host "PowerShell files updated."

# Step 2: Build `rsdk.exe` and run tests
Write-Host "Building rsdk.exe..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed. Exiting release process."
    exit 1
}

Write-Host "Running tests..."
cargo test
if ($LASTEXITCODE -ne 0) {
    Write-Error "Tests failed. Exiting release process."
    exit 1
}

# Step 3: Commit changes and tag the release
Write-Host "Committing changes and tagging the release..."
git add $PsModulePath $PsManifestPath
git commit -m "Prepare release $Version"
git tag "v$Version"

Write-Host "Release preparation completed. Please manually push changes with 'git push origin main --tags' to trigger GitHub Actions build."
