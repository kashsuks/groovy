# groovy
tui music player

## Installation

Prebuilt binaries for Linux, Windows, and macOS are attached to each [GitHub release](https://github.com/kashsuks/groovy/releases).

- **Windows**: download `groove-windows-x86_64.zip`, extract, and run `groove.exe`. No extra dependencies are required (audio plays via WASAPI, built into Windows).
- **Linux**: download `groove-linux-x86_64.tar.gz`, extract, and run `./install.sh`. It checks for the ALSA runtime library (`libasound2`/`alsa-lib`) rodio needs and installs it via your distro's package manager if missing, then installs `groove` to `~/.local/bin`.
- **macOS**: download the `groove-macos-*.tar.gz` matching your architecture, extract, and run the `groove` binary.
