param (
    [Parameter(Mandatory = $true)]
    [string]$Version                   # New version for the release, e.g., "1.1.0"
)

# Define file paths
$PsModuleTemplatePath = "./templates/build/rsdk.psm1"
$PsManifestTemplatePath = "./templates/build/rsdk.psd1"

# Ensure the version tag is unique
Write-Host "Checking if version tag v$Version already exists..."
if (git tag --list "v$Version" | Select-String -Pattern "^v$Version$") {
    Write-Error "Version tag v$Version already exists. Please choose a unique version."
    exit 1
}

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
