# mat 📄

[![GitHub Release](https://img.shields.io/github/v/release/imyash0722/mat?color=38d39f&logo=github)](https://github.com/imyash0722/mat/releases)
[![CI](https://github.com/imyash0722/mat/actions/workflows/ci.yml/badge.svg)](https://github.com/imyash0722/mat/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

> **A vibrant, high-fidelity CLI Markdown reader written in Rust.**  
> Blazingly fast, standalone single-binary, bat-style framed elegance, GitHub Flavored Markdown (GFM) support, TrueColor syntax highlighting, full Neovim-style alternate screen viewer, and native shell completions for Zsh, Fish, and Bash.


## ✨ Features

- **🚀 Sub-5ms Startup Latency:** Built in Rust with zero Python/Node runtime overhead. Cold starts render in ~4ms.
- **🪟 Neovim-Style Full Terminal Viewer:** Takes over the alternate terminal screen buffer, restoring the terminal cleanly upon exit without polluting terminal history.
- **🎯 Dynamic Real-Time Centering:** Caps reading width at ~100 columns and dynamically re-centers content in real time across terminal resize events.
- **🖱️ Smooth Touchpad & Mouse Wheel Scrolling:** Native vertical scrolling on trackpads and mouse wheels.
- **⌨️ Neovim Keybindings & Search:** Full Vim movement (`j`/`k`, `Ctrl+d`/`u`, `gg`/`G`, `<num>G`), forward/backward search (`/` & `?`), match jumping (`n`/`N`), ex commands (`:q`, `:help`), and `ZZ`.
- **📊 Terminal-Synced Statusline:** Status bar rendered in reverse video (`\x1b[7m`) that automatically synchronizes with any terminal theme (dark, light, Catppuccin, Gruvbox, Tokyo Night, etc.), displaying active mode (`[NORMAL]`, `[COMMAND]`, `[SEARCH]`), file name, reading progress percentage, and line position (`[Line X/Y]`).
- **🖼️ `bat`-Style Framed View:** File header (`File: <name>`) and footer borders inspired by `bat`'s authentic grid style, without numbering every line of prose.
- **💻 Contained Code Blocks with Line Numbers:** Code fences display in a contained grid with language tags, 24-bit TrueColor syntax highlighting (powered by `syntect`), and dimmed, dynamic line numbers.
- **✨ GitHub Flavored Markdown (GFM):**
  - **Alerts / Callouts:** Beautiful rounded panels for `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, and `[!CAUTION]`.
  - **Task Lists:** Renders `[ ]` as `☐` and `[x]` as bright green `✔`.
  - **Tables:** Full Unicode borders (`┌`, `┬`, `┐`, `│`, `├`, `┼`, `┤`, `└`, `┴`, `┘`) with responsive column auto-scaling.
  - **Headings:** Distinctive color hierarchy (plum banner for H1, magenta for H2, cyan for H3, emerald for H4).
- **🔗 True OSC 8 Clickable Links:** Native terminal hyperlinks with cyan underline.
- **📐 Smart Hanging Indentation:** Continuation lines automatically detect list bullets (`- `, `* `, `• `, `1. `) and checkboxes, aligning continuation text neatly under the bullet.
- **🐚 Zsh, Fish & Bash Shell Support:** Built-in `--completions <shell>` generation and pre-generated completion scripts.


## 📦 Installation

### Quick Install (Linux & macOS)

Install the latest pre-compiled release and shell completions with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/imyash0722/mat/main/install.sh | bash
```

### Pre-built Binaries

Download standalone archives for Linux (`x86_64`, `aarch64`, `musl`), macOS (`Intel`, `Apple Silicon`), and Windows directly from [GitHub Releases](https://github.com/imyash0722/mat/releases/latest).

Each archive includes:
- `mat` standalone executable
- Shell completions (`zsh`, `fish`, `bash`)
- `README.md` & `LICENSE`
- SHA256 checksums

### From Source

```bash
git clone https://github.com/imyash0722/mat.git
cd mat
cargo build --release

# Install binary to ~/.local/bin
install -m 755 target/release/mat ~/.local/bin/mat
```

Ensure `~/.local/bin` is in your `$PATH`.


## 🐚 Shell Completion Setup

### Zsh

```zsh
# Generate completions directly to your fpath (e.g. ~/.config/zsh/completions):
mat --completions zsh > ~/.config/zsh/completions/_mat
```

### Fish

```fish
# Generate completions directly to fish completion directory:
mat --completions fish > ~/.config/fish/completions/mat.fish
```

### Bash

```bash
mat --completions bash > /etc/bash_completion.d/mat
```


## 🛠️ Usage

```bash
# View a markdown file in full-screen Neovim-style viewer:
mat README.md
mat tasks.md

# Pipe to another command or output directly (auto-detects non-TTY):
mat README.md | grep "Features"
curl -sL https://raw.githubusercontent.com/.../README.md | mat -

# Print directly to stdout without alternate screen / interactive viewer:
mat -p notes.md
mat --no-pager notes.md

# Override terminal width (default: centered 100 columns):
mat -w 120 architecture.md

# Generate shell completions:
mat --completions zsh
mat --completions fish
```

### Interactive Viewer Navigation

| Key / Gesture | Action |
| :--- | :--- |
| `j` / `↓` / `Enter` | Scroll down one line |
| `k` / `↑` | Scroll up one line |
| `Ctrl+e` / `Ctrl+y` | Scroll down / up one line |
| `d` / `Ctrl+d` | Scroll half-page down |
| `u` / `Ctrl+u` | Scroll half-page up |
| `f` / `PageDown` / `Space` | Scroll full-page down |
| `b` / `PageUp` | Scroll full-page up |
| `gg` / `Home` | Jump to top of document |
| `G` / `End` | Jump to bottom of document |
| `<number>G` (e.g. `50G`) | Jump to specific line number |
| `/pattern` | Search forward in document |
| `?pattern` | Search backward in document |
| `n` / `N` | Jump to next / previous search match |
| `:q` / `q` / `ZZ` | Quit viewer and restore terminal |
| `:help` / `F1` | Toggle interactive keyboard shortcut cheat sheet |
| **Touchpad / Mouse Wheel** | Smooth vertical scrolling |


## 📋 Architecture

```text
mat/
├── .github/
│   └── workflows/
│       ├── ci.yml        # CI test matrix across Linux, macOS, Windows
│       └── release.yml   # Multi-platform release builder & publisher
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI argument parsing, shell completions & mode dispatch
│   ├── layout.rs         # Layout calculations, dynamic centering & bat-style frames
│   ├── viewer.rs         # Neovim-style full-terminal TUI, event loop & keybindings
│   ├── render.rs         # Pulldown-cmark AST event loop, GFM blocks & formatting
│   ├── syntax.rs         # Lazy Syntect TrueColor highlighting (base16-ocean.dark)
│   ├── table.rs          # GFM Unicode table layout, column auto-sizing & alignment
│   └── terminal.rs       # ANSI-aware width calculation, wrapping & hanging indents
├── completions/          # Pre-generated shell completion scripts
│   ├── zsh/              # _mat
│   ├── fish/             # mat.fish
│   └── bash/             # mat.bash
├── scripts/
│   └── release.sh        # Release automation helper
├── install.sh            # Universal one-line installer
└── CHANGELOG.md          # Keep a Changelog specification
```


## 🚀 Release Management

Releases are fully automated via GitHub Actions on git tag pushes:

```bash
# Release a new version using the helper script:
./scripts/release.sh 1.2.0
```

This will run all quality gates (`cargo test`, `cargo clippy`, `cargo fmt`), update versions, tag the commit, and trigger the GitHub release workflow. See [`CHANGELOG.md`](CHANGELOG.md) for version history.


## 📄 License

MIT License © 2026 [Yashwanth](https://github.com/imyash0722). See [LICENSE](LICENSE) for details.
