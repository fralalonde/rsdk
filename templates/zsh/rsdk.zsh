# Source this file to install the persistent rsdk shell function.
rsdk() {
    local temp_file status rsdk_dir
    temp_file="$(mktemp)" || return 1
    rsdk_dir="${${(%):-%N}:A:h}/../../bin"
    if [ "$#" -eq 0 ]; then
        "$rsdk_dir/rsdk" --shell zsh --envout "$temp_file" --help
    else
        "$rsdk_dir/rsdk" --shell zsh --envout "$temp_file" "$@"
    fi
    status=$?
    [ -s "$temp_file" ] && source "$temp_file"
    rm -f "$temp_file"
    return "$status"
}
