#!/usr/bin/env sh
set -eu

# This installer is intentionally POSIX-sh compatible because the documented
# invocation is `curl ... | sh`. Do not use Bash-only features here.

REPOSITORY="${RSDK_REPOSITORY:-fralalonde/rsdk}"
RSDK_HOME="${RSDK_HOME:-$HOME/.rsdk}"
VERSION="${RSDK_VERSION:-}"
MODIFY_SHELL=ask
TARGET_SHELL=""

usage() { cat <<'EOF'
Usage: install.sh [--version VERSION] [--yes] [--no-modify-shell] [--shell bash|zsh|fish]

Installs rsdk below ~/.rsdk.  --yes accepts the shell-profile prompt; --no-modify-shell
leaves profiles untouched.  RSDK_DOWNLOAD_BASE_URL is supported for offline/testing mirrors.
EOF
}
while [ "$#" -gt 0 ]; do
  case "$1" in
    --version) VERSION="${2:?--version needs a value}"; shift 2 ;;
    --yes) MODIFY_SHELL=yes; shift ;;
    --no-modify-shell) MODIFY_SHELL=no; shift ;;
    --shell) TARGET_SHELL="${2:?--shell needs a value}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'Unknown option: %s\n' "$1" >&2; usage >&2; exit 2 ;;
  esac
done

if [ -z "$VERSION" ]; then
  VERSION="$(curl --fail --silent --show-error --location "https://api.github.com/repos/$REPOSITORY/releases/latest" | tr -d '\r' | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v\([^"]*\)".*/\1/p' | head -n1)"
fi
[ -n "$VERSION" ] || { echo 'Unable to determine rsdk version; pass --version VERSION.' >&2; exit 1; }
case "$(uname -s)" in
  Linux) platform=linux ;;
  Darwin) platform=mac ;;
  *) echo "Unsupported platform: $(uname -s). Use install.ps1 on Windows." >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) arch=x86_64 ;;
  arm64|aarch64) arch=aarch64 ;;
  *) echo "Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac
asset="rsdk-${VERSION}-${platform}-${arch}.tar.gz"
base="${RSDK_DOWNLOAD_BASE_URL:-https://github.com/$REPOSITORY/releases/download/v$VERSION}"
tmp="$(mktemp -d)"
cleanup() { rm -rf "$tmp"; }
trap cleanup EXIT
curl --fail --location --retry 3 --output "$tmp/$asset" "$base/$asset"
if curl --fail --silent --show-error --location --output "$tmp/checksums.txt" "$base/checksums.txt"; then
  (cd "$tmp" && expected=$(grep -F " $asset" checksums.txt | cut -d' ' -f1) && \
    if command -v sha256sum >/dev/null 2>&1; then
      printf '%s  %s\n' "$expected" "$asset" | sha256sum -c -
    else
      actual=$(shasum -a 256 "$asset" | cut -d' ' -f1)
      [ "$actual" = "$expected" ] || { echo "checksum mismatch for $asset" >&2; exit 1; }
    fi)
fi
mkdir -p "$tmp/unpack"
tar -xzf "$tmp/$asset" -C "$tmp/unpack"
for required in bin/rsdk shell/bash/rsdk.bash shell/zsh/rsdk.zsh shell/fish/rsdk.fish VERSION checksums.txt; do
  [ -f "$tmp/unpack/rsdk/$required" ] || { echo "Release archive is missing rsdk/$required" >&2; exit 1; }
done
mkdir -p "$RSDK_HOME"
# Keep installed tools and the download cache on upgrades; replace only the
# release-owned executable, shell adapters, manifest, and archive checksum.
rm -rf "$RSDK_HOME/bin" "$RSDK_HOME/shell"
rm -f "$RSDK_HOME/VERSION" "$RSDK_HOME/checksums.txt"
mv "$tmp/unpack/rsdk/bin" "$tmp/unpack/rsdk/shell" "$RSDK_HOME/"
mv "$tmp/unpack/rsdk/VERSION" "$tmp/unpack/rsdk/checksums.txt" "$RSDK_HOME/"
printf 'Installed rsdk %s to %s\n' "$VERSION" "$RSDK_HOME"

# v0.5.x keeps adapters under $RSDK_HOME/shell/; older layouts also copied them
# into ~/.config/fish. Remove leftover copies so a stale fish autoload or
# completion set cannot shadow the new adapter. Content markers ensure we only
# delete files this installer previously owned.
remove_stale_files() {
  stale=0
  for stale_file in \
    "$HOME/.config/fish/functions/rsdk.fish" \
    "$HOME/.config/fish/functions/rsdk_plugin.fish" \
    "$HOME/.config/fish/completions/rsdk.fish"; do
    if [ -f "$stale_file" ] && grep -Eq 'envout|rsdk_plugin|__fish_rsdk' "$stale_file" 2>/dev/null; then
      rm -f "$stale_file"
      stale=$((stale + 1))
    fi
  done
  # The very first layout also kept a binary copy at $RSDK_HOME/rsdk.
  if [ -f "$RSDK_HOME/rsdk" ]; then
    rm -f "$RSDK_HOME/rsdk"
    stale=$((stale + 1))
  fi
  [ "$stale" -eq 0 ] || printf '✓ Removed %d stale file(s) from a previous rsdk install.\n' "$stale"
}
remove_stale_files

if [ -z "$TARGET_SHELL" ]; then TARGET_SHELL="${SHELL:-bash}"; TARGET_SHELL=${TARGET_SHELL##*/}; fi
case "$TARGET_SHELL" in bash|zsh|fish) ;; *) TARGET_SHELL=bash ;; esac
if [ "$MODIFY_SHELL" = ask ]; then
  # The script itself occupies stdin when invoked as `curl ... | sh`. Only
  # prompt when stdout is a terminal, and read the response from /dev/tty.
  if [ -t 1 ]; then
    printf 'Configure rsdk for %s now? [y/N] ' "$TARGET_SHELL" >&2
    read -r reply < /dev/tty || reply=n
  else
    reply=n
    printf 'No terminal available; shell configuration was not modified.\n' >&2
  fi
  case "$reply" in y|Y|yes|YES) MODIFY_SHELL=yes ;; *) MODIFY_SHELL=no ;; esac
fi
configure_posix() {
  shell_name="$1"
  if [ "$shell_name" = bash ]; then rc_file="$HOME/.bashrc"; else rc_file="$HOME/.zshrc"; fi
  loader="$RSDK_HOME/shell/$shell_name/rsdk.$shell_name"
  touch "$rc_file"
  if grep -Fq '# >>> rsdk initialize >>>' "$rc_file" || grep -Eiq '(^|[[:space:]])(source|\.)[[:space:]].*rsdk|rsdk.*(init|\.bash|\.zsh)' "$rc_file"; then
    printf '✓ rsdk initialization already present in %s; not modified.\n' "$rc_file"
  else
    cat >> "$rc_file" <<EOF
# >>> rsdk initialize >>>
source "$loader"
# <<< rsdk initialize <<<
EOF
    printf '✓ Configured %s via %s.\n' "$shell_name" "$rc_file"
  fi
  printf 'Activate in the current session: source "%s"\n' "$loader"
}
configure_fish() {
  fish_loader="$HOME/.config/fish/conf.d/rsdk.fish"
  mkdir -p "$(dirname "$fish_loader")"
  if [ -f "$fish_loader" ] && { grep -Fq '# >>> rsdk initialize >>>' "$fish_loader" || grep -Eiq 'source .*rsdk|rsdk.*init' "$fish_loader"; }; then
    printf '✓ rsdk initialization already present in %s; not modified.\n' "$fish_loader"
  else
    cat > "$fish_loader" <<EOF
# >>> rsdk initialize >>>
source "$RSDK_HOME/shell/fish/rsdk.fish"
# <<< rsdk initialize <<<
EOF
    printf '✓ Configured fish via %s.\n' "$fish_loader"
  fi
  printf 'Activate in the current session: functions -e rsdk; source "%s"\n' "$fish_loader"
}
if [ "$MODIFY_SHELL" = yes ]; then
  if [ "$TARGET_SHELL" = fish ]; then configure_fish; else configure_posix "$TARGET_SHELL"; fi
else
  printf 'Shell configuration not modified. Activate manually:\n'
  case "$TARGET_SHELL" in
    fish) printf '  source "%s"\n' "$RSDK_HOME/shell/fish/rsdk.fish" ;;
    *) printf '  source "%s/shell/%s/rsdk.%s"\n' "$RSDK_HOME" "$TARGET_SHELL" "$TARGET_SHELL" ;;
  esac
fi
