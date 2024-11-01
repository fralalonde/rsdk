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

## Build the executable

The rsdk app by itself cannot alter the current shell environment and requires a shell wrapper to do so. 

It can still be useful to build and call the executable itself.  

``cargo build --release``

``cargo build`` (debug version)

## Package
[GitHub Actions](https://github.com/fralalonde/rsdk/actions)
``git tag v0.0.3; git push --tags``

## Release 
_TBD_


# WARNING: TEMP CRAP BELOW

## Prepare scoop release

### SHA-256 Hash
Replace "PUT_YOUR_ZIP_HASH_HERE" with the correct SHA-256 hash of your rsdk-windows.zip. Generate it with the following command:

```powershell
Get-FileHash "rsdk-windows.zip" -Algorithm SHA256
```

Or using sha256sum in Linux/macOS:
```bash
sha256sum rsdk-windows.zip
```
