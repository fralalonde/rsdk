#!/bin/bash

# Path to rsdk (replace with actual path)
rsdkPath="PUT_RSDK_PATH_HERE"

# Function to invoke rsdk and capture environment changes
function invoke_rsdk() {
    local command="$1"
    shift
    local args=("$@")

    local temp_file
    temp_file=$(mktemp)

    # Build the argument list with --shell and --envout
    local argument_list=( "--shell" "bash" "--envout" "$temp_file" "$command" "${args[@]}" )

    # Run rsdk and capture live output
    "$rsdkPath" "${argument_list[@]}"

    # Apply environment changes if any were output
    if [[ -s "$temp_file" ]]; then
        # Source the temp file to apply any environment variable changes
        source "$temp_file"
    fi

    # Clean up
    rm -f "$temp_file"
}

# Check if the script is called with `init`
if [[ "$1" == "init" ]]; then
    # Initialize the module
    invoke_rsdk init

    # Alias `rsdk` to `invoke_rsdk` for global use
    alias rsdk="invoke_rsdk"
elif [[ $# -eq 0 ]]; then
    # If no parameters are provided, call invoke_rsdk with --help
    invoke_rsdk --help
else
    # Otherwise, just call invoke_rsdk with the provided command and arguments
    invoke_rsdk "$@"
fi
