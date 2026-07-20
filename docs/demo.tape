# docs/demo.tape — generates docs/demo.gif
# Requires VHS: https://github.com/charmbracelet/vhs
#   brew install vhs        (macOS)
#   yay -S vhs              (Arch)
#   go install github.com/charmbracelet/vhs@latest
# Then:  vhs docs/demo.tape
#
# The gif is referenced from README.md as ![TUI demo](docs/demo.gif).
# Regenerate after significant TUI changes.

Output docs/demo.gif
Set FontSize 14
Set Padding 8
Set Width 1100
Set Height 680
Set Theme "Dracula"

# Build a debug binary first so the tape doesn't depend on an installed rsdk.
Type "cargo build --quiet"
Enter
Sleep 2s

# Launch the TUI.
Type "./target/debug/rsdk tui"
Enter
Sleep 3s

# Search for "java".
Type "java"
Sleep 1s

# Drill into java's versions.
Enter
Sleep 2s

# Clear the filter and browse.
Backspace@4
Down
Down
Sleep 1s

# Open the action modal on a version.
Enter
Sleep 2s

# Dismiss it.
Escape
Sleep 1s

# Back out to the tools list.
Left
Sleep 1s

# Browse a couple more tools.
Down
Down
Sleep 1s

# Quit.
Escape
