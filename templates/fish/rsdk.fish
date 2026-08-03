# Source this file to install the persistent rsdk shell function.
function rsdk
    set -l temp_file (mktemp)
    or return 1
    set -l rsdk_binary (dirname (status --current-filename))/../../bin/rsdk
    if test (count $argv) -eq 0
        command $rsdk_binary --shell fish --envout $temp_file --help
    else
        command $rsdk_binary --shell fish --envout $temp_file $argv
    end
    set -l command_status $status
    if test -s $temp_file
        source $temp_file
    end
    rm -f $temp_file
    return $command_status
end
