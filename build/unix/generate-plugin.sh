#!/bin/bash

# Usage: ./script.sh <source_dir> <dest_dir> <exe_path>
# Ensure correct usage
if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <source_dir> <dest_dir> <exe_path>"
  exit 1
fi

# Input parameters
source_dir="$1"
dest_dir="$2"
exe_path="$3"  # Path to the rsdk executable

# Ensure the destination directory exists
if [ ! -d "$dest_dir" ]; then
  echo "Creating directory $dest_dir and its parent directories"
  mkdir -p "$dest_dir"
fi

# Copy module template files from the source to the destination
echo "Copying module template files from $source_dir to $dest_dir"
rsync -av "$source_dir/" "$dest_dir/"

# Replace 'PUT_RSDK_PATH_HERE' with the exe_path in all files in the destination directory
echo "Replacing 'PUT_RSDK_PATH_HERE' with '$exe_path' in all files in $dest_dir"
find "$dest_dir" -type f -exec sed -i "s|PUT_RSDK_PATH_HERE|$exe_path|g" {} +

echo "Unix shell templates generated to $dest_dir with executable $exe_path"
