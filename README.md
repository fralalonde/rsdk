# `rsdk` - Native JVM tools manager

`rsdk` is a from-scratch rewrite of the otherwise excellent [SDKMAN](https://sdkman.io/) JVM tool manager. 

The problem with SDKMAN is that it's made mostly of bash scripts, limiting its portability to other shells and non-Unix OS.

Because `rsdk` is a self-contained executable, it works the same everywhere and does not require additonal plugins or packages to be installed.

`rsdk` integrates with bash, zsh, **powershell**, and **fish** shells, on Windows, Linux and Mac.

`rsdk` is does not require curl, zip or any other package to run.

`rsdk` is an alternative _client_, it still relies on SDKMAN indexes and downloads.

`rsdk` does not try to replicate all of SDKMAN:

 - no offline mode
 - some commands are different
 - tools are installed in the `~/.rsdk/tools` folder

## Installation

```bash
# Unix shells (bash / zsh / fish)
curl -fsSL https://github.com/fralalonde/rsdk/releases/latest/download/install.sh | sh

# PowerShell
irm https://github.com/fralalonde/rsdk/releases/latest/download/install.ps1 | iex
```

The installer downloads the right prebuilt binary for your platform, places it
at `~/.rsdk/rsdk` (unix) / `$HOME\.rsdk\rsdk.exe`, and sets up shell integration
for your **current** shell only. If you use multiple shells, re-run the
oneliner from each — the script reuses an already-installed binary.

If shell integration (aliases for `rsdk`, `use`, `ls`, `flush`) isn't needed
and you just want to install the binary on PATH, pass `--shell none`:

```bash
curl -fsSL https://github.com/fralalonde/rsdk/releases/latest/download/install.sh | sh -s -- --shell none
```

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

Unfortunately, installing `rsdk` from source is the only way for now (TODO [package managers](https://github.com/fralalonde/rsdk/issues/6)).

Because `rsdk` is compiled, installing it requires [Rust to be installed](https://www.rust-lang.org/tools/install). 

(I know I said that `rsdk` didn't need other stuff to be installed first. I lied.)

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

## Network options

If proxying is required, ``rsdk`` honors the `http_proxy` and `https_proxy` environment variables (same as curl).

If required, ``--insecure`` disables certificate validation allowing use of self-signed certificates.

## Other platforms

I do not have a collection of exotic machines to test on. If you use an architecture that isn't supported,
please add it to the defined `PLATFORM`s in `api.rs` and submit a [pull request](https://github.com/fralalonde/rsdk/pulls) for it.

Alternative shells may require a bit more work to support but are welcome too. 
`nushell` support in particular would be [nice](https://github.com/fralalonde/rsdk/issues/1).

## Future

See [issues](https://github.com/fralalonde/rsdk/issues) for a list of planned features.

## Disclaimer and policy

Original CLI functionality was hand coded. Later features may have been generated. I reviewed and will support all of it.

I expect contributors to follow the same guidelines. Stand by your work; automation does not absolve one of responsibility. 

This is a fun-only project. I reserve the right to refuse anything that doesn't make me happy.
