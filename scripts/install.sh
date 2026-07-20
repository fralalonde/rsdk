#!/usr/bin/env sh
# rsdk installer for Unix shells (bash / zsh / fish).
# Usage: curl -fsSL https://github.com/fralalonde/rsdk/releases/latest/download/install.sh | sh
#
# Downloads the matching release tarball, extracts it to ~/.rsdk/, and
# configures shell integration for every supported shell detected on the
# system (current shell + any with an existing config file). Re-running
# the script reuses an already-installed binary (only updates snippets).

set -e

RSDK_HOME="${RSDK_HOME:-$HOME/.rsdk}"
REPO="fralalonde/rsdk"

info()  { printf '\033[1;34mrsdk:\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33mrsdk:\033[0m %s\n' "$*" >&2; }
fail()  { printf '\033[1;31mrsdk:\033[0m %s\n' "$*" >&2; exit 1; }

detect_platform() {
    case "$(uname -s)" in
        Linux)  OS=linux ;;
        Darwin) OS=mac  ;;
        *)      fail "unsupported OS: $(uname -s)" ;;
    esac
    case "$(uname -m)" in
        x86_64)                   ARCH=x86_64    ;;
        aarch64|arm64)            ARCH=arm64     ;;
        *)                        fail "unsupported arch: $(uname -m)" ;;
    esac
    case "$OS-$ARCH" in
        linux-x86_64) RUST_TARGET=x86_64-unknown-linux-gnu ;;
        linux-arm64)  RUST_TARGET=aarch64-unknown-linux-gnu ;;  # future-proof
        mac-arm64)    RUST_TARGET=aarch64-apple-darwin ;;
        *)            fail "no release for $OS-$ARCH" ;;
    esac
}

rcfile_for() {
    case "$1" in
        bash) [ -f "$HOME/.bashrc" ] && echo "$HOME/.bashrc" || echo "$HOME/.bash_profile" ;;
        zsh)  echo "$HOME/.zshrc" ;;
        fish) echo "$HOME/.config/fish/config.fish" ;;
    esac
}

# Detect every supported shell present on the system: the current shell
# (so the user gets immediate integration) plus any other with an existing
# config file or available on PATH. Prints shell names, one per line.
detect_shells() {
    found=""

    # Current shell — check version vars first (works under `| sh`).
    if [ -n "${BASH_VERSION:-}" ];  then found="$found bash"; fi
    if [ -n "${ZSH_VERSION:-}" ];   then found="$found zsh";  fi
    if [ -n "${FISH_VERSION:-}" ];  then found="$found fish"; fi

    # Fall back to $SHELL if nothing matched.
    if [ -z "$found" ]; then
        case "${SHELL##*/}" in
            bash) found="bash" ;; zsh) found="zsh" ;; fish) found="fish" ;;
        esac
    fi

    # Scan for config files of every supported shell so users with multiple
    # shells get all of them wired up in one pass.
    for s in bash zsh fish; do
        rc=$(rcfile_for "$s")
        [ -n "$rc" ] && [ -f "$rc" ] || continue
        case " $found " in
            *" $s "*) ;;
            *) found="$found $s" ;;
        esac
    done

    # Detect shells on PATH even without a config yet (so the snippet gets
    # written to a fresh rc file on first run).
    for s in bash zsh fish; do
        command -v "$s" >/dev/null 2>&1 || continue
        case " $found " in
            *" $s "*) ;;
            *) found="$found $s" ;;
        esac
    done

    [ -n "$found" ] || fail "no supported shells found (bash, zsh, fish)"
    for s in $found; do echo "$s"; done
}

latest_tarball_url() {
    # Resolve the "latest" redirect to fetch the assets list, then pick by target.
    api_url=$(curl -sIL -o /dev/null -w '%{url_effective}' \
              "https://github.com/$REPO/releases/latest")
    tag="${api_url##*/}"
    echo "https://github.com/$REPO/releases/download/$tag/rsdk-$tag-$RUST_TARGET.tar.gz"
}

ensure_binary() {
    mkdir -p "$RSDK_HOME"
    url=$(latest_tarball_url)
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT

    info "downloading $url"
    if ! curl -fL "$url" -o "$tmp/rsdk.tar.gz"; then
        fail "download failed (no release for $RUST_TARGET?)"
    fi

    # Extract entire tarball (binary + shell wrappers) to $RSDK_HOME
    info "installing to $RSDK_HOME"
    tar -xzf "$tmp/rsdk.tar.gz" -C "$RSDK_HOME"
    chmod +x "$RSDK_HOME/rsdk"
}

write_shell_snippet() {
    shell=$1
    # The tarball ships pre-generated wrappers at <rsdk_home>/<shell>/rsdk.<shell>
    # with PUT_RSDK_PATH_HERE already replaced by "rsdk". We source them and let
    # `rsdk init` configure PATH for the current shell.
    case "$shell" in
        bash|zsh)
            echo "
# >>> rsdk >>>
export PATH=\"$RSDK_HOME:\$PATH\"
[ -f \"$RSDK_HOME/$shell/rsdk.$shell\" ] && . \"$RSDK_HOME/$shell/rsdk.$shell\"
eval \"\$(rsdk init 2>/dev/null || true)\"
# <<< rsdk <<<"
            ;;
        fish)
            echo "
# >>> rsdk >>>
fish_add_path -mP \"$RSDK_HOME\" 2>/dev/null || set -gx PATH \"$RSDK_HOME\" \$PATH
[ -f \"$RSDK_HOME/fish/rsdk.fish\" ] && source \"$RSDK_HOME/fish/rsdk.fish\"
rsdk init 2>/dev/null | source
# <<< rsdk <<<"
            ;;
    esac
}

install_shell_integration() {
    shell=$1
    rc=$(rcfile_for "$shell")
    [ -n "$rc" ] || fail "no rc file known for $shell"
    mkdir -p "$(dirname "$rc")"

    # Strip any prior snippet we wrote so re-running is idempotent.
    if [ -f "$rc" ]; then
        tmp_rc=$(mktemp)
        awk '/# >>> rsdk >>>/{skip=1} !skip{print} /# <<< rsdk <<</{skip=0}' "$rc" > "$tmp_rc"
        mv "$tmp_rc" "$rc"
    fi

    write_shell_snippet "$shell" >> "$rc"
    info "  $shell: $rc"
}

main() {
    detect_platform
    info "platform: ${OS}-${ARCH} ($RUST_TARGET)  home: $RSDK_HOME"

    if [ -f "$RSDK_HOME/rsdk" ]; then
        info "reusing existing binary at $RSDK_HOME/rsdk"
    else
        ensure_binary
    fi

    info "configuring shell integration:"
    detect_shells | while read -r shell; do
        install_shell_integration "$shell"
    done

    info "done! restart your shell(s) or source the rc file(s)."
}

main "$@"
