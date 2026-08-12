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
Usage: install.sh [--version VERSION] [--yes] [--no-modify-shell] [--shell bash|zsh|fish|nushell]

Installs rsdk below ~/.rsdk.  By default the installer proposes configuring
every shell that has a config file in your home (~/.bashrc, ~/.zshrc,
~/.config/fish, ~/.config/nushell/config.nu); --shell restricts it to one.
--yes accepts the prompt; --no-modify-shell leaves profiles untouched.
RSDK_DOWNLOAD_BASE_URL is supported for offline/testing mirrors.
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
for required in bin/rsdk shell/bash/rsdk.bash shell/zsh/rsdk.zsh shell/fish/rsdk.fish shell/nushell/rsdk.nu VERSION checksums.txt; do
  [ -f "$tmp/unpack/rsdk/$required" ] || { echo "Release archive is missing rsdk/$required" >&2; exit 1; }
done
mkdir -p "$RSDK_HOME"
# Keep installed tools and the download cache on upgrades; replace only the
# release-owned executable, shell adapters, manifest, and archive checksum.
rm -rf "$RSDK_HOME/bin" "$RSDK_HOME/shell"
rm -f "$RSDK_HOME/VERSION" "$RSDK_HOME/checksums.txt"
mv "$tmp/unpack/rsdk/bin" "$tmp/unpack/rsdk/shell" "$RSDK_HOME/"
mv "$tmp/unpack/rsdk/VERSION" "$tmp/unpack/rsdk/checksums.txt" "$RSDK_HOME/"
# Blank line separates the fetch/verify output (curl progress, checksum) from
# the install summary below.
printf '\nInstalled rsdk %s to %s\n' "$VERSION" "$RSDK_HOME"

# v0.5.x keeps adapters under $RSDK_HOME/shell/; older layouts also copied them
# into ~/.config/fish. Remove leftover copies so a stale fish autoload cannot
# shadow the new adapter. Content markers ensure we only delete files this
# installer previously owned. The fish completions file is NOT removed: it is
# the natural install location for `rsdk completions fish` output.
remove_stale_files() {
  stale=0
  for stale_file in \
    "$HOME/.config/fish/functions/rsdk.fish" \
    "$HOME/.config/fish/functions/rsdk_plugin.fish"; do
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

# Blank line separates the install phase from the shell-configuration phase.
printf '\n'

# Detect every shell the user actually uses, based on its config file being
# present in the home: bash → ~/.bashrc, zsh → ~/.zshrc, fish →
# ~/.config/fish. A shell without a config file cannot load an adapter, and
# $SHELL alone is unreliable (a toolbox/container inherits the host's login
# shell). Fall back to the running shell when nothing is configured yet.
SHELLS=""
for shell_config in "bash:$HOME/.bashrc" "zsh:$HOME/.zshrc" "fish:$HOME/.config/fish" "nushell:$HOME/.config/nushell/config.nu"; do
  shell_name=${shell_config%%:*}
  config_path=${shell_config#*:}
  [ -e "$config_path" ] && SHELLS="$SHELLS $shell_name"
done
if [ -z "$SHELLS" ]; then
  parent_shell="$(ps -o comm= -p "$PPID" 2>/dev/null | sed -e 's/^-//' -e 's#.*/##')"
  case "$parent_shell" in
    bash|zsh|fish) SHELLS=" $parent_shell" ;;
    nu|nushell) SHELLS=" nushell" ;;
    *) shell_fallback="${SHELL:-bash}"; SHELLS=" ${shell_fallback##*/}" ;;
  esac
fi
if [ -n "$TARGET_SHELL" ]; then
  # --shell overrides detection: configure only that one.
  case "$TARGET_SHELL" in
    bash|zsh|fish|nushell) SHELLS=" $TARGET_SHELL" ;;
    *) printf 'Unknown --shell: %s\n' "$TARGET_SHELL" >&2; usage >&2; exit 2 ;;
  esac
fi
if [ "$MODIFY_SHELL" = ask ]; then
  # The script itself occupies stdin when invoked as `curl ... | sh`. Only
  # prompt when stdout is a terminal, and read the response from /dev/tty.
  if [ -t 1 ]; then
    shell_label=$(printf '%s' "$SHELLS" | sed -e 's/^ //' -e 's/ /, /g')
    printf 'Configure rsdk for %s now? [y/N] ' "$shell_label" >&2
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
  case "$shell_name" in
    bash)
      install_completions bash "${XDG_DATA_HOME:-$HOME/.local/share}/bash-completion/completions/rsdk"
      ;;
    zsh)
      install_completions zsh "$HOME/.zsh/completions/_rsdk"
      # zsh only loads the file when its directory is on fpath before compinit.
      if ! grep -Eq 'fpath=.*\.zsh/completions|\.zsh/completions.*fpath' "$rc_file" 2>/dev/null; then
        printf '→ Add ~/.zsh/completions to fpath before compinit to load zsh completions.\n'
      fi
      ;;
  esac
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
  install_completions fish "$HOME/.config/fish/completions/rsdk.fish"
}
# Generate shell completions from the freshly installed binary. Best-effort:
# a failure must never fail the install.
install_completions() { # shell_name, completion_file
  mkdir -p "$(dirname "$2")"
  if "$RSDK_HOME/bin/rsdk" completions "$1" > "$2" 2>/dev/null; then
    printf '✓ Installed %s completions: %s\n' "$1" "$2"
  else
    printf '⚠ Could not generate %s completions\n' "$1" >&2
  fi
}
configure_nushell() {
  nu_config="$HOME/.config/nushell/config.nu"
  mkdir -p "$(dirname "$nu_config")"
  if [ -f "$nu_config" ] && { grep -Fq '# >>> rsdk initialize >>>' "$nu_config" || grep -Eiq 'source .*rsdk|rsdk.*\.nu' "$nu_config"; }; then
    printf '✓ rsdk initialization already present in %s; not modified.\n' "$nu_config"
  else
    touch "$nu_config"
    cat >> "$nu_config" <<EOF

# >>> rsdk initialize >>>
source "$RSDK_HOME/shell/nushell/rsdk.nu"
# <<< rsdk initialize <<<
EOF
    printf '✓ Configured nushell via %s.\n' "$nu_config"
  fi
  printf 'Activate in the current session: source "%s"\n' "$RSDK_HOME/shell/nushell/rsdk.nu"
}
if [ "$MODIFY_SHELL" = yes ]; then
  for shell_name in $SHELLS; do
    case "$shell_name" in
      fish) configure_fish ;;
      nushell) configure_nushell ;;
      *) configure_posix "$shell_name" ;;
    esac
  done
else
  printf 'Shell configuration not modified. Activate manually:\n'
  for shell_name in $SHELLS; do
    case "$shell_name" in
      fish) printf '  source "%s"\n' "$RSDK_HOME/shell/fish/rsdk.fish" ;;
      nushell) printf '  source "%s"\n' "$RSDK_HOME/shell/nushell/rsdk.nu" ;;
      *) printf '  source "%s/shell/%s/rsdk.%s"\n' "$RSDK_HOME" "$shell_name" "$shell_name" ;;
    esac
  done
fi
