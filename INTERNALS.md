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

## Build the executable

The rsdk app by itself cannot alter the current shell environment and requires a shell wrapper to do so.

It can still be useful to build and call the executable itself.

``cargo build --release``

``cargo build`` (debug version)

## Tests

Integration tests live in `tests/` and exercise the install/use/uninstall/env
lifecycle against a temporary `~/.rsdk` (no network, no touching the real one):

``cargo test``
