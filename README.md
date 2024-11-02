# rsdk - Native JVM tools manager

`rsdk` is a native command-line JVM tool manager.

It is an alternative front-end to the well-known SDKMAN.
It does not require external tools (curl, zip) to be installed.

Rsdk can be installed on Windows, Mac and Linux systems.
It integrates with bash, powershell, zsh and fish shells.

Rsdk has limited functionality (no offline mode, etc.)
It is not pretty but works quite well.

## Installation

### From source
To install from source, see [BUILD.md](BUILD.md)

### Windows
Using [Scoop](https://scoop.sh/)

```
scoop bucket add my-bucket https://github.com/fralalonde/scoop-bucket
scoop install rsdk
```

### Bash
Using [asdf](https://asdf-vm.com/)

```
asdf plugin-add rsdk https://github.com/fralalonde/asdf-rsdk.git
asdf install rsdk latest
```

## Zsh
Using [Zinit](https://github.com/zdharma-continuum/zinit)

```
zinit load fralalonde/rsdk
```

Using [Antigen](https://github.com/zsh-users/antigen)

```
antigen bundle fralalonde/rsdk
```

### Fish
Using [fisher](https://github.com/jorgebucaran/fisher)

```
fisher install fralalonde/rsdk@1
```

### Brew
Using [Homebrew](https://brew.sh/)

```
brew tap fralalonde/rsdk
brew install rsdk
```

## Usage
Rsdk deals in ``tools`` and `versions`. 

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
