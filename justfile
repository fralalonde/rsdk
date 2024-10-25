# justfile

# Build the project in release mode
build:
    cargo build --release

# Install the PowerShell module, referring to the debug exe by default
install-module release="false":
    if [[ "{{release}}" == "true" ]]; then \
        rsdk_path="target/release/rsdk.exe"; \
    else \
        rsdk_path="target/debug/rsdk.exe"; \
    fi; \
    pwsh ./build/Module-Install.ps1 -SourceDirectory "./templates/powershell" -ExePath "$rsdk_path"
