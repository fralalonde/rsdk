# `rsdk` - Native JVM tools manager

`rsdk` is a native command-line JVM tool manager.

It is an _alternative_ front-end to the excellent [SDKMAN](https://sdkman.io/).

## Differences from SDKMAN!

`rsdk` is self contained and does not require curl or zip to be installed.

`rsdk` integrates natively with bash, zsh, **powershell**, and **fish** shells (without plugins required)

`rsdk` may have limited functionality (no offline mode, etc.) and minor variations in features or behavior.

## Motivation

My main shells are PowerShell and fish:

- The PowerShell version of sdkman kept tripping up on file operations, maybe because of Defender
- Fish integration requires installing a third-party plugin which I find suboptimal.

`rsdk` is a 100% original Rust re-implementation of the sdkman CLI.
I only discovered that sdkman CLI actually uses Rust apps for some operations after I was done writing RSDK.

## Design

`rsdk` is a single binary application implementation of the sdkman CLI functionality.

`rsdk` still totally relies on sdkman server infrastructure, packages, list and indexes. 

`rsdk` uses minimal shell wrappers that delegate _all_ operations to the binary.

`rsdk` can not directly set the environment of the current shell session. 
it prints out `set` commands to a temp file that is sourced by the shell wrapper after `rsdk` exits.

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

## Disclaimer
**`rsdk` is beta quality and may spuriously eat your dog even if you didn't have one.**

## Installation

Installing from source is the only way for now (TODO [package managers](https://github.com/fralalonde/rsdk/issues/6)).

`rsdk` is based on a compiled program. Installing from source requires [Rust to be installed](https://www.rust-lang.org/tools/install)

### Clone the repo

``git clone https://github.com/fralalonde/rsdk.git``

or [download the source](https://github.com/fralalonde/rsdk/archive/refs/heads/main.zip)

Then from the new `rsdk` directory, run the appropriate install script:

| Shell      | Command                    |
|------------|----------------------------|
| Powershell | `.\dev\Install-Module.ps1` |
| Bash       | `. dev/install-bash`       |
| Zsh        | `. dev/install-zsh`        |
| Fish       | `. dev/install-fish`       |

### Debug

Append ``--debug`` to any install script for a debug build - faster compile, better stack traces, slower archive extraction.

## Usage
`rsdk` deals in `tools` and `versions`.

Usage is mostly similar to `sdkman`.

| Shell                        | Command Format                    | Examples                                                  |
|------------------------------|-----------------------------------|-----------------------------------------------------------|
| List available tools         | ``rsdk` list`                       |                                                           |
| List available tool versions | ``rsdk` list <tool>`                | ``rsdk` list java`                                          |
| Install default version      | ``rsdk` install <tool>`             | ``rsdk` install maven`                                      |
| Install specific version     | ``rsdk` install <tool> <version>`   | ``rsdk` install maven 3.9.9`<br/>``rsdk` install java 23-tem` |
| Remove version               | ``rsdk` uninstall <tool> <version>` | ``rsdk` uninstall maven 3.9.9`                              |
| Set default version          | ``rsdk` default <tool> <version>`   | ``rsdk` default maven 3.9.9`                                |
| Set active version           | ``rsdk` use <tool> <version>`       | ``rsdk` use maven 3.9.9`                                    |
| Save env to `.sdkmanrc`      | ``rsdk` env init`                   |                                                             |
| Apply `.sdkmanrc` env        | ``rsdk` env`                        |                                                             |
| Install `.sdkmanrc` tools    | ``rsdk` env install`                |                                                             |
| Revert env to defaults       | ``rsdk` env clear`                  |                                                             |
| Flush entire cache           | ``rsdk` flush`                      |                                                           |
| Show help                    | ``rsdk` --help`                     |                                                           |

Running ``rsdk` use <tool> <version>`` for a version that isn't installed will
offer to install it first (like SDKMAN), then make it current.

Running with ```rsdk` --debug``  will enable verbose output and stack traces (equivalent of `RUST_BACKTRACE=1` and `RUST_LOG=debug`).  

## Releasing

`./release.sh` bumps the semantic version and tags the repo, mirroring `release.ps1`.

```bash
./release.sh <major|minor|patch> [--push]
```

It reads the latest `vX.Y.Z` tag (default `v0.0.0`), bumps the requested part,
syncs `Cargo.toml`/`Cargo.lock` to the new version in a `Release <tag>` commit,
and creates an annotated `v<new>` tag. With no `--push` it prompts before
pushing the branch and tag (default no); on a dirty tree it offers to commit
everything first, and it refuses to run in detached HEAD state.

## Network settings

If proxying is required, ``rsdk`` honors the `http_proxy` and `https_proxy` environment variables (same as curl).

If required, ``--insecure`` disables certificate validation allowing use of self-signed certificates.

### Other platforms

I do not have a collection of exotic machines to test on. If you use an architecture that isn't supported,
please add it to the defined `PLATFORM`s in `api.rs` and submit a [pull request](https://github.com/fralalonde/rsdk/pulls) for it.

Alternative shells may require a bit more work to support but are welcome too. 
`nushell` support in particular would be [nice](https://github.com/fralalonde/rsdk/issues/1).

## Future

See [issues](https://github.com/fralalonde/rsdk/issues) for a list of planned features.

This is a fun-only project.

## Thanks
