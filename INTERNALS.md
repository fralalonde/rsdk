# Internals

Rsdk uses shell-specific wrapper scripts to call the native executable.
When required, the executable outputs environment changes that are applied by the wrapper script upon exit.

## Build the executable

The rsdk app by itself cannot alter the current shell environment and requires a shell wrapper to do so. 

It can still be useful to build and call the executable itself.  

``cargo build --release``

``cargo build`` (debug version)


