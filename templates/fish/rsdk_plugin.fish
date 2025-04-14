# templates/fish/rsdk_plugin.fish

# Ensure `rsdk` function is in session scope
if type -q rsdk
    rsdk init
else
    echo "Warning: rsdk function not found"
end
