#!/usr/bin/env sh
# rsdk installer for Unix shells (bash / zsh / fish).
# Usage: curl -fsSL https://github.com/fralalonde/rsdk/releases/latest/download/install.sh | sh
#
# Downloads the matching release tarball, extracts it to ~/.rsdk/, and
# installs the shell adapter scripts (one per detected shell) into
# ~/.rsdk/bin/. Does NOT touch global rc files — the user sources the
# adapter themselves, like the dev/install-* scripts.

set -e

RSDK_HOME="${RSDK_HOME:-$HOME/.rsdk}"
RSDK_BIN="$RSDK_HOME/bin"
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
        linux-arm64)  RUST_TARGET=aarch64-unknown-linux-gnu ;;
        mac-arm64)    RUST_TARGET=aarch64-apple-darwin ;;
        *)            fail "no release for $OS-$ARCH" ;;
    esac
}

# Detect every supported shell present on the system (current shell,
# shells with rc files, or shells on PATH). Prints names, one per line.
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

    # Detect shells on PATH so users with multiple shells get all adapters.
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
    api_url=$(curl -sIL -o /dev/null -w '%{url_effective}' \
              "https://github.com/$REPO/releases/latest")
    tag="${api_url##*/}"
    echo "https://github.com/$REPO/releases/download/$tag/rsdk-$tag-$RUST_TARGET.tar.gz"
}

ensure_binary() {
    mkdir -p "$RSDK_BIN"
    url=$(latest_tarball_url)
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT

    info "downloading $url"
    if ! curl -fL "$url" -o "$tmp/rsdk.tar.gz"; then
        fail "download failed (no release for $RUST_TARGET?)"
    fi

    info "installing to $RSDK_HOME"
    tar -xzf "$tmp/rsdk.tar.gz" -C "$RSDK_HOME"
    chmod +x "$RSDK_HOME/rsdk"
}

# Patch a template's exe path to the absolute binary, handling every form
# the templates use: PUT_RSDK_PATH_HERE, the relative "rsdk", and the fish
# `eval "rsdk $argument_list"` form (which the release tarballs bake in).
patch_exe() {
    src=$1
    dst=$2
    sed -e "s|PUT_RSDK_PATH_HERE|$RSDK_HOME/rsdk|g" \
        -e "s|\"rsdk\" |\"$RSDK_HOME/rsdk\" |g" \
        -e "s|eval \"rsdk \$argument_list\"|\"$RSDK_HOME/rsdk\" \$argument_list|g" \
        "$src" > "$dst"
}

# Install the adapter script for a shell, mirroring dev/install-*.
#   bash/zsh -> ~/.rsdk/bin/rsdk.<shell>  (user sources it / adds to rc)
#   fish     -> ~/.config/fish/functions/rsdk.fish + conf.d plugin (autoload)
install_adapter() {
    shell=$1
    src="$RSDK_HOME/$shell/rsdk.$shell"

    [ -f "$src" ] || { warn "  $shell: no template in tarball, skipping"; return; }

    case "$shell" in
        bash|zsh)
            dst="$RSDK_BIN/rsdk.$shell"
            patch_exe "$src" "$dst"
            chmod +x "$dst"
            info "  $shell: $dst"
            ;;
        fish)
            func_dir="$HOME/.config/fish/functions"
            conf_dir="$HOME/.config/fish/conf.d"
            mkdir -p "$func_dir" "$conf_dir"
            patch_exe "$src" "$func_dir/rsdk.fish"
            cp "$RSDK_HOME/fish/rsdk_plugin.fish" "$conf_dir/rsdk_plugin.fish" 2>/dev/null \
                || warn "  fish: plugin template missing in tarball"
            info "  fish: $func_dir/rsdk.fish (+ conf.d plugin)"
            ;;
    esac
}

# Print activation instructions for a shell.
activation_hint() {
    shell=$1
    case "$shell" in
        bash|zsh)
            dst="$RSDK_BIN/rsdk.$shell"
            info "    source $dst init  # activate now"
            info "    # add to ~/.${shell}rc to make permanent"
            ;;
        fish)
            info "    fish reloads automatically (functions/ + conf.d/)"
            info "    # or run: source ~/.config/fish/functions/rsdk.fish init"
            ;;
    esac
}

main() {
    detect_platform
    info "platform: ${OS}-${ARCH} ($RUST_TARGET)  home: $RSDK_HOME"

    if [ -f "$RSDK_HOME/rsdk" ]; then
        info "reusing existing binary at $RSDK_HOME/rsdk"
    else
        ensure_binary
    fi

    # Ensure adapter dir exists (ensure_binary only makes it on fresh install).
    mkdir -p "$RSDK_BIN"

    info "installing shell adapters:"
    detect_shells | while read -r shell; do
        install_adapter "$shell"
        activation_hint "$shell"
    done

    info "done! source an adapter to activate rsdk."
}

main "$@"
