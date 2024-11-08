# rsdk - Native JVM tools manager

`rsdk` is a native command-line JVM tool manager.

It is an alternative front-end to the great [SDKMAN](https://sdkman.io/).
It does not require external tools (curl, zip) to be installed.

Rsdk can be installed on Windows, Mac and Linux systems.
It integrates with bash, powershell, zsh and fish shells.

Rsdk has limited functionality (no offline mode, etc.)
See [issues](https://github.com/fralalonde/rsdk/issues) for a list of planned features.

## Disclaimer
**Rsdk is beta quality and may spuriously eat your dog even if you didn't have one.**

## Installation

Installing from source is the only way for now (TODO [package managers](https://github.com/fralalonde/rsdk/issues/6)).

Rsdk is based on a compiled program. Installing from source requires [Rust to be installed](https://www.rust-lang.org/tools/install)

### Clone the repo

``git clone https://github.com/fralalonde/rsdk.git``

or [download the source](https://github.com/fralalonde/rsdk/archive/refs/heads/main.zip)

Then from the new `rsdk` directory, run the appropriate install script:

| Shell      | Command                    |
|------------|----------------------------|
| Powershell | `.\dev\Install-Module.ps1` |
| Bash       | `. dev/install-bash`       |
| Bash       | `. dev/install-zsh`        |
| Fish       | `. dev/install-fish`       |

### Debug

Append ``--debug`` to any install script for a debug build - faster compile, better stack traces, slower archive extraction.

## Usage
Rsdk deals in `tools` and `versions`.

Usage is mostly similar to `sdkman`.

| Shell                        | Command Format                    | Examples                                                  |
|------------------------------|-----------------------------------|-----------------------------------------------------------|
| List available tools         | `rsdk list`                       |                                                           |
| List available tool versions | `rsdk list <tool>`                | `rsdk list java`                                          |
| Install default version      | `rsdk install <tool>`             | `rsdk install maven`                                      |
| Install specific version     | `rsdk install <tool> <version>`   | `rsdk install maven 3.9.9`<br/>`rsdk install java 23-tem` |
| Remove version               | `rsdk uninstall <tool> <version>` | `rsdk uninstall maven 3.9.9`                              |
| Set default version          | `rsdk default <tool> <version>`   | `rsdk default maven 3.9.9`                                |
| Set active version           | `rsdk use <tool> <version>`       | `rsdk use maven 3.9.9`                                    |
| Flush entire cache           | `rsdk flush`                      |                                                           |
| Show help                    | `rsdk --help`                     |                                                           |

Running with ``rsdk --debug``  will enable verbose output and stack traces (equivalent of `RUST_BACKTRACE=1` and `RUST_LOG=debug`).  

## Network settings

If proxying is required, ``rsdk`` honors the `http_proxy` and `https_proxy` environment variables (same as curl).

If required, ``--insecure`` disables certificate validation allowing use of self-signed certificates.

### Other platforms

I do not have a collection of exotic machines to test on. If you use an architecture that isn't supported,
please add it to the defined `PLATFORM`s in `api.rs` and submit a [pull request](https://github.com/fralalonde/rsdk/pulls) for it.

Alternative shells may require a bit more work to support but are welcome too. 
`nushell` support in particular would be [nice](https://github.com/fralalonde/rsdk/issues/1).

## Thanks
