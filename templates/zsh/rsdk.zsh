#!/bin/zsh

# Path to rsdk (replace with actual path)
rsdkPath="PUT_RSDK_PATH_HERE"

# Function to invoke rsdk and capture environment changes
function invoke_rsdk() {
    local command="$1"
    shift
    local args=("$@")

    # Create a temporary file to capture environment variable changes
    local temp_file
    temp_file=$(mktemp)

    # Build the argument list with --shell and --envout
    local argument_list=( "--shell" "zsh" "--envout" "$temp_file" "$command" "${args[@]}" )

    # Run rsdk and capture live output
    echo "$rsdkPath ${argument_list[*]}"
    "$rsdkPath" "${argument_list[@]}"

    # Apply environment changes if any were output
    if [[ -s "$temp_file" ]]; then
        echo "envout contains:"
        cat "$temp_file"
        # Source the temp file to apply any environment variable changes
        source "$temp_file"
    else
        echo "envout is empty"
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
    echo "rsdk initialized and alias set for global use."
else
    # Otherwise, just call invoke_rsdk with the provided command and arguments
    invoke_rsdk "$@"
fi
