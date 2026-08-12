#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
version=9.9.9
mirror="$work/mirror"; mkdir -p "$mirror"
stage="$work/stage/rsdk"
mkdir -p "$stage/bin" "$stage/shell"/{bash,zsh,fish,nushell,powershell}
cat > "$stage/bin/rsdk" <<'EOF'
#!/usr/bin/env sh
while [ "$#" -gt 0 ]; do
    if [ "$1" = --envout ]; then printf 'export RSDK_WRAPPER_TEST=loaded\n' > "$2"; break; fi
    shift
done
EOF
chmod +x "$stage/bin/rsdk"
for s in bash zsh fish; do cp "$repo_root/templates/$s/rsdk.$s" "$stage/shell/$s/rsdk.$s"; done
cp "$repo_root/templates/fish/rsdk_plugin.fish" "$stage/shell/fish/rsdk_plugin.fish"
cp "$repo_root/templates/nushell/rsdk.nu" "$stage/shell/nushell/rsdk.nu"
cp "$repo_root/templates/powershell/Rsdk.psd1" "$repo_root/templates/powershell/Rsdk.psm1" "$stage/shell/powershell/"
printf '%s\n' "$version" > "$stage/VERSION"
(cd "$stage" && sha256sum bin/* shell/bash/* shell/zsh/* shell/fish/* shell/nushell/* shell/powershell/* VERSION > checksums.txt)
tar -C "$work/stage" -czf "$mirror/rsdk-$version-linux-x86_64.tar.gz" rsdk
tar -tzf "$mirror/rsdk-$version-linux-x86_64.tar.gz" | grep -Fx 'rsdk/shell/powershell/Rsdk.psm1'
tar -tzf "$mirror/rsdk-$version-linux-x86_64.tar.gz" | grep -Fx 'rsdk/shell/fish/rsdk_plugin.fish'
tar -tzf "$mirror/rsdk-$version-linux-x86_64.tar.gz" | grep -Fx 'rsdk/checksums.txt'
(cd "$mirror" && sha256sum rsdk-$version-linux-x86_64.tar.gz > checksums.txt)
HOME="$work/home" RSDK_HOME="$work/home/.rsdk" RSDK_DOWNLOAD_BASE_URL="file://$mirror" bash "$repo_root/scripts/install.sh" --version "$version" --yes --shell bash
[ -x "$work/home/.rsdk/bin/rsdk" ]
grep -Fx '# >>> rsdk initialize >>>' "$work/home/.bashrc"
grep -Fq 'source "'"$work/home"'/.rsdk/shell/bash/rsdk.bash"' "$work/home/.bashrc"
HOME="$work/home" RSDK_HOME="$work/home/.rsdk" RSDK_DOWNLOAD_BASE_URL="file://$mirror" bash "$repo_root/scripts/install.sh" --version "$version" --yes --shell bash
[ "$(grep -Fc '# >>> rsdk initialize >>>' "$work/home/.bashrc")" -eq 1 ]
# Sourcing provides a persistent function and applies the binary's envout file.
source "$work/home/.rsdk/shell/bash/rsdk.bash"
declare -F rsdk >/dev/null
rsdk test
[ "${RSDK_WRAPPER_TEST:-}" = loaded ]
printf 'release installer integration checks passed\n'
