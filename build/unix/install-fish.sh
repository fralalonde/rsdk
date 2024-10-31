#!/usr/bin/env fish

# Set default paths for Fish plugin installation
set fish_plugin_dir ~/.config/fish/functions
set fish_conf_dir ~/.config/fish/conf.d

# Ensure required directories exist
mkdir -p $fish_plugin_dir
mkdir -p $fish_conf_dir

# Path to the rsdk binary passed from justfile
set rsdk_path $argv[2]

# Read the template file, replace placeholder, and save to the plugin directory
set template_file $argv[1]
set output_file $fish_plugin_dir/rsdk_plugin.fish

# Process the template, replace placeholder with actual rsdk path, and write to the functions directory
cat $template_file | sed "s|PUT_RSDK_PATH_HERE|$rsdk_path|g" > $output_file

# Create a Fish plugin descriptor in the conf.d directory
echo "set -gx PATH $fish_plugin_dir \$PATH" > $fish_conf_dir/rsdk_plugin.fish
echo "echo 'RSDK Fish plugin installed successfully'" >> $fish_conf_dir/rsdk_plugin.fish

# Output completion message
echo "Installed RSDK Fish plugin to $fish_plugin_dir and generated descriptor at $fish_conf_dir/rsdk_plugin.fish"
