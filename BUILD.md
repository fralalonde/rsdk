# Building `rsdk` from source

## Prerequisites

Because `rsdk` is compiled, installing it requires [Rust to be installed](https://www.rust-lang.org/tools/install).

(I know I said that `rsdk` didn't need other stuff to be installed first. I lied.)

## Clone the repo

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

Git tags are the source of truth for versioning; the build files (`Cargo.toml`,
`Cargo.lock`) are synced as a side effect of the release commit.

## CI

GitHub Actions (`.github/workflows/release.yml`) builds for three targets on
every `v*` tag push:

| Target                    | Runner      |
|---------------------------|-------------|
| `x86_64-pc-windows-msvc`  | `windows-2022` |
| `x86_64-unknown-linux-gnu`| `ubuntu-22.04`|
| `aarch64-apple-darwin`    | `macos-14` (native ARM) |

Each build produces a versioned archive (`.zip` for Windows, `.tar.gz` for
Unix) containing the binary plus shell-wrapper scripts. A release job attaches
all archives plus `install.sh` / `install.ps1` to a GitHub Release.

`rustls` (pure-Rust TLS) is used instead of native-tls/OpenSSL so the binary
has no external native dependencies.

## Other platforms

I do not have a collection of exotic machines to test on. If you use an architecture that isn't supported,
please add it to the defined `PLATFORM`s in `api.rs` and submit a [pull request](https://github.com/fralalonde/rsdk/pulls) for it.

Alternative shells may require a bit more work to support but are welcome too. 
`nushell` support in particular would be [nice](https://github.com/fralalonde/rsdk/issues/1).

## Disclaimer and policy

Original CLI functionality was hand coded. Later features may have been generated. I reviewed and will support all of it.

I expect contributors to follow the same guidelines. Stand by your work; automation does not absolve one of responsibility. 

This is a fun-only project. I reserve the right to refuse anything that doesn't make me happy.
