# Antigen (zsh) manifest
# Directory to store the binary
BIN_DIR="${HOME}/.local/bin"
mkdir -p "$BIN_DIR"

# Function to download the latest release of the binary
download_latest_release() {
    local repo="your-username/rsdk-plugin"
    local binary="rsdk"

    # Get the latest release info from GitHub API
    local latest_release_url=$(curl -s "https://api.github.com/repos/${repo}/releases/latest" \
        | jq -r '.assets[] | select(.name == "'"$binary"'") | .browser_download_url')

    if [[ -z "$latest_release_url" ]]; then
        echo "Error: Unable to find release for $binary"
        return 1
    fi

    # Download the binary
    echo "Downloading $binary from $latest_release_url..."
    curl -L -o "$BIN_DIR/$binary" "$latest_release_url"
    chmod +x "$BIN_DIR/$binary"
}

# Check if the binary exists, and download if not
if [[ ! -x "$BIN_DIR/rsdk" ]]; then
    download_latest_release
fi

# Add the binary directory to PATH
export PATH="$BIN_DIR:$PATH"
