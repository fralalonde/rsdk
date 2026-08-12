#!/usr/bin/env sh
# Sandboxed integration test for scripts/install.sh: builds a fake release
# tree from the repo's templates + debug binary, then exercises installer
# scenarios in isolated HOME directories (counterpart of
# test-release-installer-powershell-integration.ps1).
#   sh test/test-release-installer-posix-integration.sh
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
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

failures=0
ok() { printf 'ok   %s\n' "$1"; }
fail() { printf 'FAIL %s\n' "$1"; failures=$((failures + 1)); }
assert_contains() { # file, needle, label
  if grep -q -- "$2" "$1" 2>/dev/null; then ok "$3"; else fail "$3"; fi
}
assert_missing() { # path, label
  if [ ! -e "$1" ]; then ok "$2"; else fail "$2"; fi
}

# ── Build the fake release tree (mirrors the release.yml layout) ────────────
[ -x "$ROOT/target/debug/rsdk" ] || (cd "$ROOT" && cargo build)
RELEASE="$WORK/release"
mkdir -p "$RELEASE/rsdk/bin" "$RELEASE/rsdk/shell/bash" "$RELEASE/rsdk/shell/zsh" \
  "$RELEASE/rsdk/shell/fish" "$RELEASE/rsdk/shell/nushell"
cp "$ROOT/target/debug/rsdk" "$RELEASE/rsdk/bin/rsdk"
cp "$ROOT/templates/bash/rsdk.bash" "$RELEASE/rsdk/shell/bash/rsdk.bash"
cp "$ROOT/templates/zsh/rsdk.zsh" "$RELEASE/rsdk/shell/zsh/rsdk.zsh"
cp "$ROOT/templates/fish/rsdk.fish" "$RELEASE/rsdk/shell/fish/rsdk.fish"
cp "$ROOT/templates/nushell/rsdk.nu" "$RELEASE/rsdk/shell/nushell/rsdk.nu"
printf '0.5.7\n' > "$RELEASE/rsdk/VERSION"
printf 'placeholder\n' > "$RELEASE/rsdk/checksums.txt"
chmod +x "$RELEASE/rsdk/bin/rsdk"
tar -C "$RELEASE" -czf "$RELEASE/rsdk-0.5.7-linux-x86_64.tar.gz" rsdk
if command -v sha256sum >/dev/null 2>&1; then
  SHA="sha256sum"
else
  SHA="shasum -a 256"
fi
$SHA "$RELEASE/rsdk-0.5.7-linux-x86_64.tar.gz" \
  | awk '{print $1"  rsdk-0.5.7-linux-x86_64.tar.gz"}' > "$RELEASE/checksums.txt"
BASE_URL="file://$RELEASE"
INSTALLER="$ROOT/scripts/install.sh"

run_install() { # home, extra args...
  home="$1"; shift
  HOME="$home" RSDK_DOWNLOAD_BASE_URL="$BASE_URL" \
    sh "$INSTALLER" --version 0.5.7 --yes "$@" >"$WORK/last.log" 2>&1
}

# ── Scenarios ───────────────────────────────────────────────────────────────
echo '== multi-shell home: bash + zsh + fish + nushell =='
MULTI="$WORK/home-multi"
mkdir -p "$MULTI/.config/fish" "$MULTI/.config/nushell"
: > "$MULTI/.bashrc"; : > "$MULTI/.zshrc"; : > "$MULTI/.config/nushell/config.nu"
run_install "$MULTI"
assert_contains "$MULTI/.bashrc" '# >>> rsdk initialize >>>' 'bashrc configured'
assert_contains "$MULTI/.zshrc" '# >>> rsdk initialize >>>' 'zshrc configured'
assert_contains "$MULTI/.config/fish/conf.d/rsdk.fish" 'rsdk initialize' 'fish configured'
assert_contains "$MULTI/.config/nushell/config.nu" 'rsdk initialize' 'nushell configured'
assert_contains "$MULTI/.config/fish/completions/rsdk.fish" '__fish_rsdk' 'fish completions installed'
assert_contains "$MULTI/.local/share/bash-completion/completions/rsdk" '_rsdk' 'bash completions installed'
assert_contains "$MULTI/.zsh/completions/_rsdk" '#compdef' 'zsh completions installed'

echo '== only ~/.bashrc -> only bash =='
ONLYBASH="$WORK/home-onlybash"
mkdir -p "$ONLYBASH"
: > "$ONLYBASH/.bashrc"
run_install "$ONLYBASH"
assert_contains "$ONLYBASH/.bashrc" 'rsdk initialize' 'bashrc configured'
assert_missing "$ONLYBASH/.config/fish/conf.d/rsdk.fish" 'fish left untouched'

echo '== no config files + bash parent -> fallback bash =='
NONE="$WORK/home-none"
mkdir -p "$NONE"
bash -c "HOME=$NONE RSDK_DOWNLOAD_BASE_URL=$BASE_URL sh $INSTALLER --version 0.5.7 --yes & wait" >"$WORK/last.log" 2>&1
assert_contains "$NONE/.bashrc" 'rsdk initialize' 'fallback detected bash'

echo '== --shell override =='
run_install "$MULTI" --shell zsh
zsh_lines=$(grep -c 'rsdk initialize' "$MULTI/.zshrc" || true)
[ "$zsh_lines" -eq 2 ] && ok '--shell zsh wrote exactly one block' || fail "--shell zsh block count = $zsh_lines"

echo '== --no-modify-shell prints hints =='
HOME="$MULTI" RSDK_DOWNLOAD_BASE_URL="$BASE_URL" sh "$INSTALLER" --version 0.5.7 --no-modify-shell >"$WORK/last.log" 2>&1
hints=$(grep -c 'source' "$WORK/last.log" || true)
[ "$hints" -ge 4 ] && ok 'manual hints for all shells' || fail "manual hints = $hints"

echo '== idempotent re-run =='
run_install "$MULTI"
already=$(grep -c 'already present' "$WORK/last.log" || true)
[ "$already" -eq 4 ] && ok 're-run reports 4 already-present' || fail "re-run reported $already already-present"

echo '== legacy stale-file cleanup (completions file is a live user install) =='
STALE="$WORK/home-stale"
mkdir -p "$STALE/.config/fish/functions" "$STALE/.config/fish/completions" \
  "$STALE/.rsdk/bin" "$STALE/.rsdk/tools/java/current/bin"
printf '# Source this file to install the persistent rsdk shell function.\nfunction rsdk\n    set -l temp_file (mktemp)\n    set -l rsdk_binary (dirname (status --current-filename))/../../bin/rsdk\n    command $rsdk_binary --shell fish --envout $temp_file $argv\nend\n' \
  > "$STALE/.config/fish/functions/rsdk.fish"
printf 'echo user-own-file\n' > "$STALE/.config/fish/functions/rsdk_plugin.fish"
printf 'function __fish_rsdk_global_optspecs\nend\n' > "$STALE/.config/fish/completions/rsdk.fish"
printf '#!/bin/sh\necho orphan\n' > "$STALE/.rsdk/rsdk"
printf '#!/bin/sh\necho java\n' > "$STALE/.rsdk/tools/java/current/bin/java"
run_install "$STALE"
assert_missing "$STALE/.config/fish/functions/rsdk.fish" 'stale functions/rsdk.fish removed'
assert_contains "$STALE/.config/fish/completions/rsdk.fish" '__fish_rsdk' 'completions regenerated by installer'
assert_missing "$STALE/.rsdk/rsdk" 'orphan binary removed'
assert_contains "$STALE/.config/fish/functions/rsdk_plugin.fish" 'user-own-file' 'user decoy preserved'
assert_contains "$STALE/.rsdk/tools/java/current/bin/java" 'java' 'tools dir preserved'

echo '== nushell adapter e2e (skipped if nu missing) =='
if command -v nu >/dev/null 2>&1; then
  out=$(nu --config "$MULTI/.config/nushell/config.nu" -c 'rsdk --version' 2>&1)
  case "$out" in
    *rsdk*) ok "nushell adapter runs the binary ($out)" ;;
    *) fail "nushell adapter: got '$out'" ;;
  esac
else
  echo 'skip nushell e2e (nu not installed)'
fi

if [ "$failures" -gt 0 ]; then
  printf '%s integration check(s) failed\n' "$failures" >&2
  tail -20 "$WORK/last.log" >&2
  exit 1
fi
echo 'POSIX release installer integration checks passed'
