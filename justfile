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
    pwsh ./build/windows/Module-Install.ps1 -SourceDirectory "./templates/powershell" -ExePath "$rsdk_path"

install-fish release="false":
    if [[ "{{release}}" == "true" ]]; then \
        rsdk_path="target/release/rsdk"; \
    else \
        rsdk_path="target/debug/rsdk"; \
    fi; \
    fish ./build/fish/install.fish templates/fish/rsdk.fish $rsdk_path