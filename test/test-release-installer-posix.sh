#!/usr/bin/env sh
# Static checks for the POSIX release installer and its shell adapters
# (counterpart of test-release-installer-powershell.ps1). Run from anywhere:
#   sh test/test-release-installer-posix.sh
#
# Linux-only: Windows is covered by the PowerShell tests and macOS is assumed
# to behave like Linux. Skips (exit 0) on other platforms so it can be wired
# into a per-platform CI matrix unchanged.
set -eu

case "$(uname -s)" in
  Linux) ;;
  *) echo "skip: POSIX installer tests are Linux-only (host: $(uname -s))"; exit 0 ;;
esac

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
failures=0

check_contains() { # haystack, needle, label
  case "$1" in
    *"$2"*) printf 'ok   %s\n' "$3" ;;
    *) printf 'FAIL %s (missing: %s)\n' "$3" "$2"; failures=$((failures + 1)) ;;
  esac
}

for path in \
  scripts/install.sh \
  templates/bash/rsdk.bash \
  templates/zsh/rsdk.zsh \
  templates/fish/rsdk.fish \
  templates/nushell/rsdk.nu; do
  [ -f "$ROOT/$path" ] || { printf 'FAIL missing %s\n' "$path"; failures=$((failures + 1)); }
done

installer="$(cat "$ROOT/scripts/install.sh")"
bash_adapter="$(cat "$ROOT/templates/bash/rsdk.bash")"
zsh_adapter="$(cat "$ROOT/templates/zsh/rsdk.zsh")"
fish_adapter="$(cat "$ROOT/templates/fish/rsdk.fish")"
nu_adapter="$(cat "$ROOT/templates/nushell/rsdk.nu")"

check_contains "$installer" 'shell/nushell/rsdk.nu' 'installer requires the nushell adapter in the release archive'
check_contains "$installer" 'nushell:$HOME/.config/nushell/config.nu' 'installer detects nushell via config.nu'
check_contains "$installer" 'configure_nushell' 'installer has a nushell configuration function'
check_contains "$installer" '# >>> rsdk initialize >>>' 'installer writes the standard init block'
check_contains "$installer" 'remove_stale_files' 'installer removes legacy adapter files'
check_contains "$installer" 'install_completions' 'installer generates shell completions'
check_contains "$installer" 'completions fish' 'installer wires fish completions'
check_contains "$installer" 'completions zsh' 'installer wires zsh completions'
check_contains "$installer" 'bash|zsh|fish|nushell' 'installer accepts --shell nushell'
check_contains "$bash_adapter" 'envout' 'bash adapter uses the env-capture wrapper'
check_contains "$zsh_adapter" 'envout' 'zsh adapter uses the env-capture wrapper'
check_contains "$fish_adapter" 'envout' 'fish adapter uses the env-capture wrapper'
check_contains "$nu_adapter" '--wrapped' 'nushell adapter passes unknown flags through'
check_contains "$nu_adapter" 'load-env' 'nushell adapter applies env via load-env'
check_contains "$nu_adapter" 'FILE_PWD' 'nushell adapter resolves the binary from its own path'

if [ "$failures" -gt 0 ]; then
  printf '%s static check(s) failed\n' "$failures" >&2
  exit 1
fi
echo 'POSIX release installer static checks passed'
