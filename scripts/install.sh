#!/usr/bin/env sh
# rsdk installer for Unix shells (bash / zsh / fish).
# Usage: curl -fsSL https://github.com/fralalonde/rsdk/releases/latest/download/install.sh | sh
#
# Detects the current shell, downloads the matching release tarball, extracts
# the binary to ~/.rsdk/, writes a per-shell integration snippet, and sources
# it in the current shell. Re-running the script reuses an already-installed
# binary (only updates the shell snippet).

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

detect_shell() {
    if [ -n "${BASH_VERSION:-}" ]; then echo bash; return; fi
    if [ -n "${ZSH_VERSION:-}"  ]; then echo zsh;  return; fi
    if [ -n "${FISH_VERSION:-}" ]; then echo fish; return; fi
    case "${SHELL##*/}" in
        bash|zsh|fish) echo "${SHELL##*/}" ;;
        *) fail "could not detect shell; supported: bash, zsh, fish" ;;
    esac
}

rcfile_for() {
    case "$1" in
        bash) [ -f "$HOME/.bashrc" ] && echo "$HOME/.bashrc" || echo "$HOME/.bash_profile" ;;
        zsh)  echo "$HOME/.zshrc" ;;
        fish) echo "$HOME/.config/fish/config.fish" ;;
    esac
}

latest_tarball_url() {
    # Resolve the "latest" redirect to fetch the assets list, then pick by target.
    local api_url
    api_url=$(curl -sIL -o /dev/null -w '%{url_effective}' \
              "https://github.com/$REPO/releases/latest")
    local tag="${api_url##*/}"
    echo "https://github.com/$REPO/releases/download/$tag/rsdk-$tag-$RUST_TARGET.tar.gz"
}

ensure_binary() {
    mkdir -p "$RSDK_HOME"
    local url
    url=$(latest_tarball_url)
    local tmp
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT

    info "downloading $url"
    if ! curl -fL "$url" -o "$tmp/rsdk.tar.gz"; then
        fail "download failed (no release for $RUST_TARGET?)"
    fi
    tar -xzf "$tmp/rsdk.tar.gz" -C "$tmp"
    [ -f "$tmp/rsdk" ] || fail "tarball is missing the rsdk binary"
    cp "$tmp/rsdk" "$RSDK_HOME/rsdk"
    chmod +x "$RSDK_HOME/rsdk"
    info "binary installed at $RSDK_HOME/rsdk"
}

write_shell_snippet() {
    shell=$1
    # The tarball ships a pre-generated wrapper at <rsdk_home>/<shell>/rsdk.<shell>
    # with PUT_RSDK_PATH_HERE already replaced by "rsdk". We source it and let
    # `rsdk init` configure PATH for the current shell.
    case "$shell" in
        bash|zsh)
            snippet="
# >>> rsdk >>>
export PATH=\"$RSDK_HOME:\$PATH\"
[ -f \"$RSDK_HOME/$shell/rsdk.$shell\" ] && . \"$RSDK_HOME/$shell/rsdk.$shell\"
eval \"\$(rsdk init 2>/dev/null || true)\"
# <<< rsdk <<<"
            ;;
        fish)
            snippet="
# >>> rsdk >>>
fish_add_path -mP \"$RSDK_HOME\" 2>/dev/null || set -gx PATH \"$RSDK_HOME\" \$PATH
[ -f \"$RSDK_HOME/fish/rsdk.fish\" ] && source \"$RSDK_HOME/fish/rsdk.fish\"
rsdk init 2>/dev/null | source
# <<< rsdk <<<"
            ;;
    esac
    echo "$snippet"
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
    info "shell integration written to $rc"

    # Apply to current shell by eval'ing the snippet (only effective in the
    # spawned subshell here; the parent shell will pick it up from rc on restart
    # or manual source).
    info "restart your shell or run:"
    info "  source $rc"
}

main() {
    detect_platform
    shell=$(detect_shell)
    info "platform: ${OS}-${ARCH} ($RUST_TARGET)  shell: $shell  home: $RSDK_HOME"

    if [ -f "$RSDK_HOME/rsdk" ]; then
        info "reusing existing binary at $RSDK_HOME/rsdk"
    else
        ensure_binary
    fi

    install_shell_integration "$shell"
}

main "$@"
