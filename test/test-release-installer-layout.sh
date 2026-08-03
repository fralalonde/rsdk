#!/usr/bin/env bash
set -euo pipefail
repo_root=/mnt/d/Code/rsdk

for file in scripts/install.sh scripts/install.ps1; do
    test -f "$repo_root/$file" || {
        printf 'missing release bootstrap installer: %s\n' "$file" >&2
        exit 1
    }
done

grep -Fq 'rsdk/bin/rsdk.exe' "$repo_root/.github/workflows/release.yml"
grep -Fq 'rsdk/shell/bash/rsdk.bash' "$repo_root/.github/workflows/release.yml"
grep -Fq 'rsdk/shell/powershell/Rsdk.psd1' "$repo_root/.github/workflows/release.yml"
grep -Fq 'checksums.txt' "$repo_root/.github/workflows/release.yml"
