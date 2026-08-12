# Source this file to install the persistent rsdk shell function.
# Resolve the binary directory once at source time: %N inside a function
# would expand to the function name and resolve against the cwd, so this
# must live at file top level, not inside rsdk().
rsdk_dir="${${(%):-%N}:A:h}/../../bin"
rsdk() {
    local temp_file rsdk_status
    temp_file="$(mktemp)" || return 1
    if [ "$#" -eq 0 ]; then
        "$rsdk_dir/rsdk" --shell zsh --envout "$temp_file" --help
    else
        "$rsdk_dir/rsdk" --shell zsh --envout "$temp_file" "$@"
    fi
    rsdk_status=$?
    [ -s "$temp_file" ] && source "$temp_file"
    rm -f "$temp_file"
    return "$rsdk_status"
}
