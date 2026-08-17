# `rsdk` - Universal JVM tools manager

`rsdk` makes it easy to install and juggle with multiple versions of JDK, Maven, Gradle, etc...

`rsdk` works on Windows, Linux, MacOS and integrates with bash, zsh, **powershell**, **fish**, **nushell**

`rsdk` is a self-contained binary executable, it works the same everywhere and does not require additional packages to be installed.

`rsdk` provides a convenient TUI in addition to the classic command-line interface.

![TUI demo](docs/demo.gif)

## Why not SDKMAN?

I'm a fan of [SDKMAN](https://sdkman.io/)! But I mainly use fish shell on Linux and Powershell on Windows,
neither of which are natively supported by SDKMAN (since it is written mostly in bash).

I wrote `rsdk` from scratch and made my own shell-agnostic SDKMAN client.
Although it is completely independent of SDKMAN _locally_, `rsdk` still relies on SDKMAN servers, indexes and downloads.

**PLEASE - DO NOT BOTHER THE SDKMAN MAINTAINERS IF YOU'RE HAVING TROUBLE WITH RSDK.** 

Both projects are _completely separate_ and rsdk's existence should not be a burden to sdkman in _any_ way. 
Instead, do not hesitate to open an [issue](https://github.com/fralalonde/rsdk/issues).

`rsdk` does not try to replicate all of SDKMAN:

- there's no offline mode
- some commands are different
- tools are installed in the `~/.rsdk/tools` folder (so you can have both `rsdk` and SDKMAN installed at once)

## Installation

Linux / macOS
```bash
curl -fsSL https://github.com/fralalonde/rsdk/releases/latest/download/install.sh | sh
```

Windows
```powershell
irm https://github.com/fralalonde/rsdk/releases/latest/download/install.ps1 | iex
```

## Command Line

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
offer to install it first, then make it current.

Running with `--debug` enables verbose output and stack traces (equivalent of `RUST_BACKTRACE=1` and `RUST_LOG=debug`).  

## TUI

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



## Network options

If proxying is required, ``rsdk`` honors the `http_proxy` and `https_proxy` environment variables (same as curl).

If required, ``--insecure`` disables certificate validation allowing use of self-signed certificates.

## Disclaimer
`rsdk` may spuriously eat your dog even if you didn't have one. 

Although not vibe-coded, AI was used for TUI and install scripts.

## Future

See [issues](https://github.com/fralalonde/rsdk/issues) for a list of planned features.
