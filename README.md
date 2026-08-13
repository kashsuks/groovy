<div align="center">

# groovy

TUI music player made using rust and ratatui.

[![Release](https://img.shields.io/github/v/release/kashsuks/groovy?style=flat-square)](https://github.com/kashsuks/groovy/releases)
[![License](https://img.shields.io/github/license/kashsuks/groovy?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white)](Cargo.toml)
[![Downloads](https://img.shields.io/github/downloads/kashsuks/groovy/total?style=flat-square)](https://github.com/kashsuks/groovy/releases)
[![Stars](https://img.shields.io/github/stars/kashsuks/groovy?style=flat-square)](https://github.com/kashsuks/groovy/stargazers)

<img width="1499" height="843" alt="SCR-20260812-rbte" src="https://github.com/user-attachments/assets/43710613-fd90-42de-a10e-c2bcf335cb28" />

</div>

## Installation

Prebuilt binaries for Linux, Windows, and macOS are attached to each [GitHub release](https://github.com/kashsuks/groovy/releases).

- **Windows**: download `groovy-windows-x86_64.zip`, extract, and run `groovy.exe`. No extra dependencies are required (audio plays via WASAPI, built into Windows).
- **Linux**: download `groovy-linux-x86_64.tar.gz`, extract, and run `./install.sh`. It checks for the ALSA runtime library (`libasound2`/`alsa-lib`) rodio needs and installs it via your distro's package manager if missing, then installs `groovy` to `~/.local/bin`.
- **macOS**: download the `groovy-macos-*.tar.gz` matching your architecture, extract, and run the `groovy` binary.

## Setup

Once done installing the project and running it, you can create a new playlist by pressing `n` and searching for the directory through the file explorer or use `/` to type in a path. Once done, save the playlist and open it. Boom!

You're good to go :)

Theres options like shuffle, replay, play/pause, skip forward/backward, and a cinema mode planned for the future!

# AI Usage

Claude Code agents were used for the creation of audio channels in this project. They were also used for debugging issues related to CD files

</content>
</invoke>
