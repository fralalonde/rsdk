#!/bin/bash

# Usage: ./script.sh <SourceDir> <DestinationDir> <ExePath>
# Ensure correct usage
if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <SourceDir> <DestinationDir> <ExePath>"
  exit 1
fi

# Input parameters
SourceDir="$1"
DestinationDir="$2"
ExePath="$3"  # Path to the rsdk executable

# Ensure the destination directory exists
if [ ! -d "$DestinationDir" ]; then
  echo "Creating directory $DestinationDir and its parent directories"
  mkdir -p "$DestinationDir"
fi

# Copy module template files from the source to the destination, excluding Rsdk.psm1
echo "Copying module template files from $SourceDir to $DestinationDir"
rsync -av --exclude 'Rsdk.psm1' "$SourceDir/" "$DestinationDir/"

# Get the absolute path of the rsdk executable
rsdkBinarySource=$(realpath "$ExePath")

# Update Rsdk.psm1 template with the correct path to rsdk
psm1TemplatePath="$SourceDir/Rsdk.psm1"
psm1DestinationPath="$DestinationDir/Rsdk.psm1"

# Escape path for sed replacement
rsdkPathEscaped=$(printf '%s\n' "$rsdkBinarySource" | sed 's/[&/\]/\\&/g')

# Replace 'PUT_RSDK_PATH_HERE' in template and write to destination
sed "s/PUT_RSDK_PATH_HERE/$rsdkPathEscaped/g" "$psm1TemplatePath" > "$psm1DestinationPath"

echo "rsdk.psm1 generated with rsdk path $rsdkPathEscaped"
