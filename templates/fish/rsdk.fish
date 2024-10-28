#!/usr/bin/env fish

# Path to rsdk (replace with actual path)
set rsdkPath "PUT_RSDK_PATH_HERE"

# Function to invoke rsdk and capture environment changes
function invoke_rsdk
    set command $argv[1]
    set args $argv[2..-1]

    # Create a temporary file to capture environment variable changes
    set temp_file (mktemp)

    # Build the argument list with --shell and --envout
    set argument_list "--shell" "fish" "--envout" $temp_file $command $args

    # Run rsdk and capture live output
    echo "$rsdkPath $argument_list"
    eval "$rsdkPath $argument_list"

    # Apply environment changes if any were output
    if test -s $temp_file
        echo "envout contains:"
        cat $temp_file
        # Source the temp file to apply any environment variable changes
        source $temp_file
    else
        echo "envout is empty"
    end

    # Clean up
    rm -f $temp_file
end

# Check if the script is called with `init`
if test "$argv[1]" = "init"
    # Initialize the module
    invoke_rsdk attach

    # Alias `rsdk` to `invoke_rsdk` for global use
    function rsdk
        invoke_rsdk $argv
    end
    echo "rsdk initialized and alias set for global use."
else
    # Otherwise, just call invoke_rsdk with the provided command and arguments
    invoke_rsdk $argv
end
