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

Installing from source is the only way for now (TODO package managers).

Rsdk is based on a compiled app. Installing from source requires [Rust to be installed](https://www.rust-lang.org/tools/install)

### Clone the repo

``git clone https://github.com/fralalonde/rsdk.git``

or [download the source](https://github.com/fralalonde/rsdk/archive/refs/heads/main.zip)

Then run the appropriate install script:

### Powershell
``.\dev\Install-Module.ps1``

### Fish

``. dev/install-fish``

### Bash
``. dev/install-bash``

### Zsh
``. dev/install-zsh``

### Debug

Append ``--debug`` to any install script for a debug build - faster compile, better stack traces, slower unzip.

### Other platforms

I do not have a collection of exotic machines to test on. If you use an architecture that isn't supported,
please add it to the defined `PLATFORM`s in `api.rs` and submit a [pull request](https://github.com/fralalonde/rsdk/pulls) for it.

Alternative shells may require a bit more work to support but are welcome too. `nushell` support in particular would be nice.

## Network settings

If proxying is required, ``rsdk`` honors the `http_proxy` and `https_proxy` environment variables (same as curl).

If required, ``--insecure`` disables certificate validation allowing use of self-signed certificates.

## Usage
Rsdk deals in ``tools`` and `versions`.

Usage is mostly similar to ``sdkman`` but not quite as pretty or nice (yet).

### List available tools
``rsdk list``

### Install the default version of a tool 
``rsdk install <tool>``

Example: ``rsdk install maven``

Some tools (actually, just `java` (?)) do not have a default version. You must select a version to be installed.

### List available versions of a tool
``rsdk list <tool>`` 

Example: ``rsdk list java``

### Install specific version of a tool
``rsdk install <tool> <version>``

Example: ``rsdk install java 22.0.2-tem``

Example: ``rsdk install maven 3.3.3``

### Remove a tool 
``rsdk remove <tool> <version>``

Example: ``rsdk uninstall java 17``

### Set the default version of a tool

The default version of each tool is set on the PATH of new shell sessions.

``rsdk default <tool> <version>``

Example: ``rsdk default java 17``

### Temporarily change version of a tool

``rsdk use <tool> <version>``

Example: ``rsdk use java 17``

### Show Help

To display a full list of commands and options:

``rsdk --help``

or just...

```rsdk```

## Thanks
