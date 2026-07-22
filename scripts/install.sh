#!/usr/bin/env sh
# rsdk installer for Unix shells (bash / zsh / fish) and Linux pwsh.
# Usage: curl -fsSL https://github.com/fralalonde/rsdk/releases/latest/download/install.sh | sh
#
# Downloads the matching release tarball, extracts it to ~/.rsdk/, and:
#   - Installs the shell adapter for every detected shell
#   - For bash/zsh: auto-adds the `source` line to .bashrc/.zshrc
#     (skips with a warning if "rsdk" already appears)
#   - For fish: copies the adapter to ~/.config/fish/functions/ (autoload)
#   - For pwsh (Linux): copies the module to ~/.local/share/powershell/Modules/

set -e

RSDK_HOME="${RSDK_HOME:-$HOME/.rsdk}"
RSDK_BIN="$RSDK_HOME/bin"
REPO="fralalonde/rsdk"

info()  { printf '\033[1;34mrsdk:\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33mrsdk:\033[0m %s\n' "$*" >&2; }
rsuccess() { printf '\033[1;32mrsdk:\033[0m %s\n' "$*"; }
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

# Detect every supported shell present on the system.
detect_shells() {
    found=""
    [ -n "${BASH_VERSION:-}" ] && found="$found bash"
    [ -n "${ZSH_VERSION:-}" ]  && found="$found zsh"
    [ -n "${FISH_VERSION:-}" ] && found="$found fish"

    [ -z "$found" ] && case "${SHELL##*/}" in
        bash|zsh|fish) found="${SHELL##*/}" ;;
    esac

    for s in bash zsh fish; do
        command -v "$s" >/dev/null 2>&1 || continue
        case " $found " in *" $s "*) ;; *) found="$found $s" ;; esac
    done

    # Detect pwsh (PowerShell Core on Linux/macOS)
    command -v pwsh >/dev/null 2>&1 && case " $found " in
        *" powershell "*) ;; *) found="$found powershell" ;;
    esac

    [ -n "$found" ] || fail "no supported shells found (bash, zsh, fish, pwsh)"
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

# Patch a template's exe path to the absolute binary.
patch_exe() {
    src=$1
    dst=$2
    sed -e "s|PUT_RSDK_PATH_HERE|$RSDK_HOME/rsdk|g" \
        -e "s|\"rsdk\" |\"$RSDK_HOME/rsdk\" |g" \
        -e "s|eval \"rsdk \\\$argument_list\"|\"$RSDK_HOME/rsdk\" \$argument_list|g" \
        "$src" > "$dst"
}

# Install the adapter script for a shell.
#   bash/zsh -> ~/.rsdk/bin/rsdk.<shell>
#   fish     -> ~/.config/fish/functions/ + conf.d plugin (autoload)
#   powershell -> ~/.local/share/powershell/Modules/Rsdk/Rsdk.psm1 (autoload)
install_adapter() {
    shell=$1

    case "$shell" in
        bash|zsh)
            src="$RSDK_HOME/$shell/rsdk.$shell"
            [ -f "$src" ] || { warn "  $shell: no template in tarball, skipping"; return; }
            dst="$RSDK_BIN/rsdk.$shell"
            patch_exe "$src" "$dst"
            chmod +x "$dst"
            info "  $shell: $dst"
            ;;
        fish)
            src="$RSDK_HOME/fish/rsdk.fish"
            [ -f "$src" ] || { warn "  fish: no template in tarball, skipping"; return; }
            func_dir="$HOME/.config/fish/functions"
            conf_dir="$HOME/.config/fish/conf.d"
            mkdir -p "$func_dir" "$conf_dir"
            patch_exe "$src" "$func_dir/rsdk.fish"
            cp "$RSDK_HOME/fish/rsdk_plugin.fish" "$conf_dir/rsdk_plugin.fish" 2>/dev/null \
                || warn "  fish: plugin template missing in tarball"
            info "  fish: $func_dir/rsdk.fish (+ conf.d plugin)"
            ;;
        powershell)
            src="$RSDK_HOME/powershell/Rsdk.psm1"
            [ -f "$src" ] || { warn "  powershell: no template in tarball, skipping"; return; }
            ps_mod_dir="$HOME/.local/share/powershell/Modules/Rsdk"
            mkdir -p "$ps_mod_dir"
            # Powershell template uses PUT_RSDK_PATH_HERE; patch to the Unix binary path
            sed "s|PUT_RSDK_PATH_HERE|$RSDK_HOME/rsdk|g" "$src" > "$ps_mod_dir/Rsdk.psm1"
            info "  powershell: $ps_mod_dir/Rsdk.psm1"
            ;;
    esac
}

# Add the rsdk source line to bash/zsh rc files, skipping if "rsdk" already present.
install_rc() {
    shell=$1
    case "$shell" in
        bash)
            rc="$HOME/.bashrc"
            line="source \"$RSDK_BIN/rsdk.bash\" init"
            ;;
        zsh)
            rc="$HOME/.zshrc"
            line="source \"$RSDK_BIN/rsdk.zsh\" init"
            ;;
        *) return ;;
    esac

    [ -f "$rc" ] || return  # no rc file → nothing to do

    if grep -q 'rsdk' "$rc" 2>/dev/null; then
        warn "  $shell: 'rsdk' already appears in $rc — not modifying"
        return
    fi

    printf '\n# rsdk shell integration\n%s\n' "$line" >> "$rc"
    rsuccess "  $shell: added source line to $rc"
}

activation_hint() {
    shell=$1
    case "$shell" in
        bash|zsh)
            info "    source $RSDK_BIN/rsdk.$shell init  # activate now"
            info "    # already added to ~/.${shell}rc for future shells"
            ;;
        fish)
            info "    fish reloads automatically (functions/ + conf.d/)"
            ;;
        powershell)
            info "    restart pwsh or run: Import-Module Rsdk"
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

    mkdir -p "$RSDK_BIN"

    info "installing shell adapters:"
    detect_shells | while read -r shell; do
        install_adapter "$shell"
        install_rc "$shell"
        activation_hint "$shell"
    done

    info "done! restart your shell or source an adapter."
}

main "$@"
