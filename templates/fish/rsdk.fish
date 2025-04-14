#!/usr/bin/env fish

# Function to invoke rsdk and capture environment changes
function rsdk
    set command $argv[1]
    set args $argv[2..-1]

    # Create a temporary file to capture environment variable changes
    set temp_file (mktemp)

    # Build the argument list with --shell and --envout
    set argument_list "--shell" "fish" "--envout" $temp_file $command $args

    # Run rsdk and capture live output
    eval "PUT_RSDK_PATH_HERE $argument_list"

    # Dump temp file if debugging
    if contains -- --debug $argv
        echo "---[ debug env changes ]---"
        cat $temp_file
        echo "---------------------------"
    end

    # Apply environment changes if any were output
    if test -s $temp_file
        # Source the temp file to apply any environment variable changes
        source $temp_file
    end

    # Clean up (should not be needed?)
    rm -f $temp_file
end
