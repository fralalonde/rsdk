#!/usr/bin/env zsh

# Function to invoke rsdk and capture environment changes
function invoke_rsdk() {
    local command="$1"
    shift
    local args=("$@")

    local temp_file
    temp_file=$(mktemp)

    local argument_list=( "--shell" "zsh" "--envout" "$temp_file" "$command" "${args[@]}" )

    if "PUT_RSDK_PATH_HERE" "${argument_list[@]}"; then
        # Apply environment changes if any were output
        if [[ -s "$temp_file" ]]; then
            # Source the temp file to apply any environment variable changes
            source "$temp_file"
        fi
    fi

    rm -f "$temp_file"
}

if [[ $# -eq 0 ]]; then
    # If no parameters are provided, call invoke_rsdk with --help
    invoke_rsdk --help
else
    # Otherwise, just call invoke_rsdk with the provided command and arguments
    invoke_rsdk "$@"
fi
