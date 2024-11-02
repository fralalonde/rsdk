# templates/fish/rsdk_plugin.fish

# Ensure `rsdk` is attached to the current shell session
if type -q rsdk
    rsdk init
else
    echo "Warning: rsdk command not found; make sure rsdk is installed and in your PATH."
end
