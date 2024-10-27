#!/bin/bash
# should also work for zsh

# Ensure `rsdk` binary is in PATH or provide its absolute path here.
rsdk_binary="rsdk"  # or use `/path/to/rsdk` if needed

# Check for required commands
command -v "$rsdk_binary" >/dev/null 2>&1 || {
    echo >&2 "Error: $rsdk_binary not found in PATH. Please ensure it's installed and available."
    exit 1
}

# Create a temporary file for capturing environment changes
tempfile=$(mktemp)
trap 'rm -f "$tempfile"' EXIT  # Ensure tempfile is removed on script exit

# Check if the script is run in Bash or Zsh
if [ -n "$BASH_VERSION" ]; then
    shell="bash"
elif [ -n "$ZSH_VERSION" ]; then
    shell="zsh"
else
    echo "This script is for Bash or Zsh"
    exit 1
fi

"$rsdk_binary" --shell "$shell" --envout "$tempfile" "${args[@]}"

# Source and evaluate the tempfile if rsdk wrote any environment changes
if [ -s "$tempfile" ]; then
    echo "Applying environment changes from $tempfile"
    source "$tempfile"
fi
