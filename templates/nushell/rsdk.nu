# Source this file to install the persistent rsdk shell function.
# The binary is resolved relative to this file (shell/nushell/ -> ../../bin)
# at load time, so the function keeps working after a reinstall.
#
# `--wrapped` passes unknown flags (e.g. `rsdk --version`, subcommand flags)
# through to the binary instead of failing at parse time. Environment changes
# are applied with `load-env`, since nushell cannot evaluate shell statements
# at runtime. Shell completions are available separately via
# `rsdk completions nushell` (they conflict with this def, so use one or the
# other).
let rsdk_binary = ($env.FILE_PWD | path join ".." ".." "bin" "rsdk")

def --env --wrapped rsdk [...args: string] {
    let temp_file = (mktemp)
    ^$rsdk_binary --shell nushell --envout $temp_file ...$args
    let command_status = $env.LAST_EXIT_CODE
    if ($temp_file | path exists) {
        # The binary writes one nuon record per variable; merge and apply.
        let envs = (open --raw $temp_file
            | lines
            | each {|line| $line | from nuon }
            | reduce --fold {} {|rec, acc| $acc | merge $rec })
        load-env $envs
        if ("PATH" in $envs) {
            # nushell keeps PATH as a list, not a colon-joined string.
            $env.PATH = ($env.PATH | split row ":")
        }
    }
    rm -f $temp_file
    $command_status
}
