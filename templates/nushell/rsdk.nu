# Source this file to install the persistent rsdk shell function.
# The binary is resolved from $env.RSDK_HOME (set by the installer in
# config.nu) because nushell does not reliably expose the path of a file
# sourced from config.nu on Windows ($env.FILE_PWD is empty there). When
# sourced directly (no config.nu), fall back to $env.FILE_PWD.
#
# `--wrapped` passes unknown flags (e.g. `rsdk --version`, subcommand flags)
# through to the binary instead of failing at parse time. Environment changes
# are applied with `load-env`, since nushell cannot evaluate shell statements
# at runtime. Shell completions are available separately via
# `rsdk completions nushell` (they conflict with this def, so use one or the
# other).
let rsdk_binary = if 'RSDK_HOME' in $env {
    ($env.RSDK_HOME | path join 'bin' 'rsdk')
} else {
    ($env.FILE_PWD | path join '..' '..' 'bin' 'rsdk')
}

def --env --wrapped rsdk [...args: string] {
    let temp_file = (mktemp)
    ^$rsdk_binary --shell nushell --envout $temp_file ...$args
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
    # No trailing `$command_status`: a `def`'s return value is emitted to the
    # pipeline and printed, unlike fish/bash/zsh `return N` (exit status). The
    # binary's exit code is already reflected in `$env.LAST_EXIT_CODE`.
}
