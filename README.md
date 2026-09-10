# mdview 📄

[![GitHub Release](https://img.shields.io/github/v/release/imyash0722/mdview?color=38d39f&logo=github)](https://github.com/imyash0722/mdview/releases)
[![CI](https://github.com/imyash0722/mdview/actions/workflows/ci.yml/badge.svg)](https://github.com/imyash0722/mdview/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

> **A vibrant, high-fidelity CLI Markdown reader written in Rust.**  
> Blazingly fast, standalone single-binary, bat-style framed elegance, GitHub Flavored Markdown (GFM) support, TrueColor syntax highlighting, and native shell integration for Zsh and Fish.


## ✨ Features

- **🚀 Sub-5ms Startup Latency:** Built in Rust with zero Python/Node runtime overhead. Cold starts render in ~4ms.
- **🖼️ `bat`-Style Framed View:** File header (`File: <name>`) and footer borders inspired by `bat`'s authentic grid style, without numbering every line of prose.
- **💻 Contained Code Blocks with Line Numbers:** Code fences display in a contained grid with language tags, 24-bit TrueColor syntax highlighting (powered by `syntect`), and dimmed, dynamic line numbers.
- **✨ GitHub Flavored Markdown (GFM):**
  - **Alerts / Callouts:** Beautiful rounded panels for `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, and `[!CAUTION]`.
  - **Task Lists:** Renders `[ ]` as `☐` and `[x]` as bright green `✔`.
  - **Tables:** Full Unicode borders (`┌`, `┬`, `┐`, `│`, `├`, `┼`, `┤`, `└`, `┴`, `┘`) with responsive column auto-scaling.
  - **Headings:** Distinctive color hierarchy (plum banner for H1, magenta for H2, cyan for H3, emerald for H4).
- **🔗 True OSC 8 Clickable Links:** Native terminal hyperlinks with cyan underline.
- **📐 Smart Hanging Indentation:** Continuation lines automatically detect list bullets (`- `, `* `, `• `, `1. `) and checkboxes, aligning continuation text neatly under the bullet.
- **📖 Comfortable Reading Gutter:** Universal 2-space left margin and default ~100-column reading width cap to prevent eye fatigue on wide tiling monitors.
- **🐚 Zsh & Fish Shell Support:** Pre-built completions and alias integration for both shells, plus built-in `--completions <shell>` generation.


## 📦 Installation

### Quick Install (Linux & macOS)

Install the latest pre-compiled release and shell completions with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/imyash0722/mdview/main/install.sh | bash
```

### Pre-built Binaries

Download standalone archives for Linux (`x86_64`, `aarch64`, `musl`), macOS (`Intel`, `Apple Silicon`), and Windows directly from [GitHub Releases](https://github.com/imyash0722/mdview/releases/latest).

Each archive includes:
- `mdview` standalone executable
- Shell completions (`zsh`, `fish`, `bash`)
- `README.md` & `LICENSE`
- SHA256 checksums

### From Source

```bash
git clone https://github.com/imyash0722/mdview.git
cd mdview
cargo build --release

# Install binary to ~/.local/bin
install -m 755 target/release/mdview ~/.local/bin/mdview
```

Ensure `~/.local/bin` is in your `$PATH`.


## 🐚 Shell Integration & Setup

### Zsh Setup

1. Add the alias to your `~/.zshrc`:
   ```zsh
   alias md="mdview"
   ```

2. Generate or install completions:
   ```zsh
   # If using ~/.config/zsh/completions (in your $FPATH):
   mdview --completions zsh > ~/.config/zsh/completions/_mdview
   cp ~/.config/zsh/completions/_mdview ~/.config/zsh/completions/_md
   ```

### Fish Setup

1. Add the alias to `~/.config/fish/conf.d/mdview.fish`:
   ```fish
   if type -q mdview
       alias md="mdview"
   end
   ```

2. Generate completions:
   ```fish
   mdview --completions fish > ~/.config/fish/completions/mdview.fish
   cp ~/.config/fish/completions/mdview.fish ~/.config/fish/completions/md.fish
   ```

### Bash Setup

```bash
alias md="mdview"
mdview --completions bash > /etc/bash_completion.d/mdview
```


## 🛠️ Usage

```bash
# View a markdown file with automatic pager (less):
md README.md
mdview tasks.md

# Pipe from standard input:
curl -sL https://raw.githubusercontent.com/.../README.md | md -

# Disable pager:
md --no-pager notes.md
md -p notes.md

# Override terminal width:
md -w 120 architecture.md

# Generate shell completions:
mdview --completions zsh
mdview --completions fish
```

### Pager Navigation (via `less`)

| Key | Action |
| :--- | :--- |
| `j` / `↓` | Scroll down one line |
| `k` / `↑` | Scroll up one line |
| `d` / `u` | Scroll half-page down / up |
| `g` / `G` | Jump to top / bottom of document |
| `/pattern` | Search forward |
| `n` / `N` | Next / previous search match |
| `q` | Exit pager |


## 📋 Architecture

```text
mdview/
├── .github/
│   └── workflows/
│       ├── ci.yml        # CI test matrix across Linux, macOS, Windows
│       └── release.yml   # Multi-platform release builder & publisher
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI argument parsing, shell completions, pager spawning
│   ├── render.rs         # Pulldown-cmark AST event loop, GFM blocks & formatting
│   ├── syntax.rs         # Lazy Syntect TrueColor highlighting (base16-ocean.dark)
│   ├── table.rs          # GFM Unicode table layout, column auto-sizing & alignment
│   └── terminal.rs       # ANSI-aware width calculation, wrapping & hanging indents
├── completions/          # Pre-generated shell completion scripts
│   ├── zsh/              # _mdview, _md
│   ├── fish/             # mdview.fish, md.fish
│   └── bash/             # mdview.bash
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
