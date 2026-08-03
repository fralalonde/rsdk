# Source this file to install the persistent rsdk shell function.
rsdk() {
    local temp_file status
    temp_file="$(mktemp)" || return 1
    if [ "$#" -eq 0 ]; then
        "$(cd "$(dirname "${BASH_SOURCE[0]}")/../../bin" && pwd)/rsdk" --shell bash --envout "$temp_file" --help
    else
        "$(cd "$(dirname "${BASH_SOURCE[0]}")/../../bin" && pwd)/rsdk" --shell bash --envout "$temp_file" "$@"
    fi
    status=$?
    [ -s "$temp_file" ] && . "$temp_file"
    rm -f "$temp_file"
    return "$status"
}
