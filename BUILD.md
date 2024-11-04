# RSDK Local install & development

## Installing from source

Rsdk is based on a native app. Installing from source requires [Rust to be installed](https://www.rust-lang.org/tools/install)

If you do not want to install Rust, use the package managers from [the main page](README.md)

### Clone the repo

``git clone https://github.com/fralalonde/rsdk.git``

or [download the source](https://github.com/fralalonde/rsdk/archive/refs/heads/main.zip)

Then run the appropriate install script. 

(Add ``--debug`` to any script for a debug build)

### Powershell
``.\dev\Install-Module.ps1``

### Fish

``. dev/install-fish``

### Bash
``. dev/install-bash``

### Zsh
``. dev/install-zsh``

### Other platforms

I do not have a collection of exotic machines to test on. If you use an architecture that isn't supported, 
please add it to the defined `PLATFORM`s in `api.rs` and submit a [pull request](https://github.com/fralalonde/rsdk/pulls) for it.

Alternative shells may require a bit more work to support but are welcome too. `nushell` support in particular would be nice.

## Build the executable

The rsdk app by itself cannot alter the current shell environment and requires a shell wrapper to do so. 

It can still be useful to build and call the executable itself.  

``cargo build --release``

``cargo build`` (debug version)

## Debugging

Running ``rsdk`` with the `--debug` flag will enable debug output and stack traces (equivalent of `RUST_BACKTRACE=1` and `RUST_LOG=debug`).

For network debugging, ``--insecure`` disables certificate validation allowing use of self-signed certificates. 