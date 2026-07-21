# `rsdk` - Native JVM tools manager

![TUI demo](docs/demo.gif)

`rsdk` is an alternative _client_ to the excellent [SDKMAN](https://sdkman.io/) JVM tool manager.
It still relies on SDKMAN servers, indexes and downloads.

`rsdk` natively integrates with bash, zsh, **powershell**, and **fish** shells, on Windows, Linux and Mac, no external plugins required.

`rsdk` provides a TUI in a addition to the classic command-line interface.

`rsdk` is a self-contained binary executable, it works the same everywhere and does not require additional packages to be installed.

`rsdk` does not try to replicate all of SDKMAN:

- no offline mode
- some commands are different
- tools are installed in the `~/.rsdk/tools` folder

## Installation

The installer scripts are attached to each GitHub Release. The URLs below
fetch the latest published release.

### Linux / macOS (bash, zsh, fish)

```bash
curl -fsSL https://github.com/fralalonde/rsdk/releases/latest/download/install.sh | sh
```

Follow additional script instructions for desired shell integrations. 

### Windows (PowerShell)

```powershell
irm https://github.com/fralalonde/rsdk/releases/latest/download/install.ps1 | iex
```

This downloads the prebuilt `rsdk.exe` to `$HOME\.rsdk\` and installs the
PowerShell module.

### Notes

- Re-running the installer reuses an already-installed binary (only updates
  the shell adapters). If you use multiple shells, the script wires up all of
  them it detects in one pass.
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

## Usage (CLI)

`rsdk` deals in `tools` and `versions`.

| Shell                        | Command Format                    | Examples                     |
|------------------------------|-----------------------------------|------------------------------|
| List available tools         | `rsdk list`                       |                              |
| List available tool versions | `rsdk list <tool>`                | `rsdk list java`             |
| Install default version      | `rsdk install <tool>`             | `rsdk install maven`         |
| Install specific version     | `rsdk install <tool> <version>`   | `rsdk install maven 3.9.9`   |
| Remove version               | `rsdk uninstall <tool> <version>` | `rsdk uninstall maven 3.9.9` |
| Set default version          | `rsdk default <tool> <version>`   | `rsdk default maven 3.9.9`   |
| Set active version           | `rsdk use <tool> <version>`       | `rsdk use maven 3.9.9`       |
| Flush downloads cache        | `rsdk flush`                      |                              |
| Save env to `.sdkmanrc`      | `rsdk env init`                   |                              |
| Apply `.sdkmanrc` env        | `rsdk env`                        |                              |
| Install `.sdkmanrc` tools    | `rsdk env install`                |                              |
| Revert env to defaults       | `rsdk env clear`                  |                              |
| Show help                    | `rsdk --help`                     |                              |

Running `rsdk use <tool> <version>` for a version that isn't installed will
offer to install it first (like SDKMAN), then make it current.

Running with `--debug` enables verbose output and stack traces (equivalent of `RUST_BACKTRACE=1` and `RUST_LOG=debug`).  

## Usage (TUI)

`rsdk tui` launches an interactive tool browser for
discovering, installing, and managing JVM tools without having to type commands.

**Layout:** two panes. Left lists tools (installed ones starred and ranked
first). Right shows the selected tool's description + installed versions,
or — after drilling in — the list of available versions.

**Navigation:**

| Key            | Action                                    |
|----------------|-------------------------------------------|
| `↑` `↓` / `k` `j` | move selection                         |
| `Enter` / `→`  | drill in (tool → versions / open actions) |
| `Esc` / `←`    | go back (Esc at top level quits)          |
| `Tab`          | switch active pane                        |
| `PgUp` / `PgDn`| jump by 10 rows                           |
| type any text  | filter the active pane                    |
| `Enter` on a version | pick an action: Install, Use, Set default, Remove |

**Action modal:** selecting a version pops a compact modal. Installed versions
offer Use / Set as default / Remove; uninstalled versions offer Install
only. Installing shows a live progress bar with a cancel option. After a
successful install, if other versions are already present, you're asked
whether to make the new one the default.

**Visual cues:** installed tools/versions are starred (`*`) and sorted first;
versions are ranked default → current → others (latest first), then uninstalled
(latest first). The current version is highlighted yellow, the default magenta.

## Network options

If proxying is required, ``rsdk`` honors the `http_proxy` and `https_proxy` environment variables (same as curl).

If required, ``--insecure`` disables certificate validation allowing use of self-signed certificates.

## Disclaimer

**`rsdk` is beta quality and may spuriously eat your dog even if you didn't have one.**

## Future

See [issues](https://github.com/fralalonde/rsdk/issues) for a list of planned features.
