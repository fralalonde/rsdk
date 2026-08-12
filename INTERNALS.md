# Internals

Rsdk uses shell-specific wrapper scripts to call the native executable.
When required, the executable outputs environment changes that are applied by the wrapper script upon exit.

## Layout

`~/.rsdk` holds everything:

- `tools/<tool>/<version>/` — an installed tool version.
- `tools/<tool>/current` — symlink to the **active** version (set by `use` / `env`).
- `tools/<tool>/default` — symlink to the **default** version (set by `default`, used by `init`).
- `cache/` — HTTP cache for API responses and downloaded archives.
- `temp/` — scratch space for extraction.

## Environment model (SDKMAN-style)

The active version is defined by the on-disk `current` symlink, **not** by the
process environment. This is what makes `current` / `use` / `env` work in any
shell, including freshly opened ones.

`rsdk init` puts each default tool's stable `…/<tool>/current/bin` on `PATH`
once and sets `*_HOME` to the default's resolved path. Afterwards:

- `rsdk use <tool> <version>` flips the `current` symlink and emits the updated
  `*_HOME`. `PATH` already points at `current/bin`, so it is left untouched.
- `rsdk env` / `rsdk env clear` do the same, driven by `.sdkmanrc` / defaults.

### Shell completions

Completions are generated from the installed binary:

- **bash, zsh, fish** — the installer generates and wires them automatically
  when you accept shell configuration (bash → `bash-completion/completions`,
  zsh → `~/.zsh/completions/_rsdk`, fish → `~/.config/fish/completions/`).
  For zsh, make sure `~/.zsh/completions` is on your `fpath` **before**
  `compinit` runs.
  - **powershell** — the module registers tab-completions automatically on
    `Import-Module` (generated from the installed binary, so always in sync).
  - **nushell** — manual only: `rsdk completions nushell` generates a completion
    module; `use` it from `config.nu` **instead of** the adapter's `source` line
    (the function def and the completion extern cannot both define `rsdk`).

Regenerate manually at any time with `rsdk completions <shell>`.

### Notes

- Re-running the installer reuses an already-installed binary (only updates
  the shell adapters). If you use multiple shells, the script wires up all of
  them it detects in one pass. nushell is detected via the `nu` binary even
  when `config.nu` doesn't exist yet (the installer creates it).
- If you just want the binary on PATH without shell aliases, clone the repo
  and build it yourself — see [BUILD.md](BUILD.md).

## How version switching works

Like SDKMAN, `rsdk` tracks the **active** version of each tool with a `current`
symlink at `~/.rsdk/tools/<tool>/current` pointing at the selected version, and
the **default** version with a `default` symlink.

`rsdk init` adds each default tool's stable `…/<tool>/current/bin` directory to
`PATH` (once) and sets the tool's `*_HOME` variable. Because `PATH` points at the
`current` symlink, `rsdk use`, `rsdk env`, and `rsdk env clear` only need to flip
that symlink and update `*_HOME` — `PATH` is never rewritten after `init`.

This means the active version survives across shells and new terminal sessions
(the symlink is on disk, not in one shell's environment).

## Build the executable

The rsdk app by itself cannot alter the current shell environment and requires a shell wrapper to do so.

It can still be useful to build and call the executable itself.

``cargo build --release``

``cargo build`` (debug version)

## Tests

Integration tests live in `tests/` and exercise the install/use/uninstall/env
lifecycle against a temporary `~/.rsdk` (no network, no touching the real one):

``cargo test``
