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

## The "Dirty" Trick

`rsdk` uses shell-specific wrappers that delegate operations to the binary.

This is because `rsdk` can not directly set the environment of the underlying shell session, 
it prints out `set` commands to a temp file that is executed by the shell-specific wrapper scripts after `rsdk` exits.

(If you know a better way to change the parent environement, _please let me know how!_.)

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

Usage is mostly similar to `sdkman`.

| Shell                        | Command Format                      | Examples                                                  |
|------------------------------|-------------------------------------|-----------------------------------------------------------|
| List available tools         | ``rsdk` list`                       |                                                           |
| List available tool versions | ``rsdk` list <tool>`                | ``rsdk` list java`                                        |
| Install default version      | ``rsdk` install <tool>`             | ``rsdk` install maven`                                    |
| Install specific version     | ``rsdk` install <tool> <version>`   | ``rsdk` install maven 3.9.9`<br/>``rsdk` install java 23-tem` |
| Remove version               | ``rsdk` uninstall <tool> <version>` | ``rsdk` uninstall maven 3.9.9`                              |
| Set default version          | ``rsdk` default <tool> <version>`   | ``rsdk` default maven 3.9.9`                                |
| Set active version           | ``rsdk` use <tool> <version>`       | ``rsdk` use maven 3.9.9`                                    |
| Flush entire cache           | ``rsdk` flush`                      |                                                           |
| Show help                    | ``rsdk` --help`                     |                                                           |

Running with ```rsdk` --debug`` enables verbose output and stack traces (equivalent of `RUST_BACKTRACE=1` and `RUST_LOG=debug`).  

## Network options

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
