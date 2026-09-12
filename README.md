# mat.nvim 📄

[![GitHub Release](https://img.shields.io/github/v/release/imyash0722/mat?color=38d39f&logo=github)](https://github.com/imyash0722/mat/releases)
[![CI](https://github.com/imyash0722/mat/actions/workflows/ci.yml/badge.svg)](https://github.com/imyash0722/mat/actions/workflows/ci.yml)
[![Neovim](https://img.shields.io/badge/Neovim-%3E%3D%200.8.0-blueviolet?logo=neovim)](https://neovim.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

> **A blazingly fast terminal Markdown preview plugin for Neovim powered by a sub-5ms Rust engine.**  
> Zero browser overhead, zero Node.js/Python runtimes. Experience high-fidelity Markdown preview—with TrueColor syntax highlighting, GitHub Flavored Markdown (GFM) callouts and tables, bat-style framing, and interactive navigation—directly inside your terminal Neovim.

---

## ✨ Features

- **🚀 Sub-5ms Startup Latency:** Built with a standalone Rust core (`pulldown-cmark` SIMD & `syntect`). Renders instantly without browser spinning or Node/Python bloat.
- **🪟 Interactive Floating Preview Modal (`:Mat`):** Centered floating popup with full Vim navigation (`j`/`k`, `Ctrl+d`/`u`, `gg`/`G`, `/` search), smooth trackpad/mouse wheel scrolling, and instant dismiss with `q`.
- **🔄 Live Side-by-Side Split (`:MatPreview`):** Vertical split preview (`:vsplit`) that automatically updates whenever you save (`:w`) using zero-flicker atomic buffer swapping.
- **✂️ Visual Snippet Preview (`:'<,'>Mat`):** Highlight any Markdown lines or docstrings in visual mode and preview just that selection in a floating card.
- **🎨 Native Terminal Theme Sync:** Reverse video statusline (`\x1b[7m`) and adaptive ANSI attributes that automatically sync with whatever colorscheme or terminal palette you use.
- **🖼️ `bat`-Style Framed Views:** Clean header borders and contained code fence grids with language badges, TrueColor syntax highlighting, and dimmed line numbers.
- **✨ Full GitHub Flavored Markdown (GFM):**
  - **Alerts / Callouts:** Rounded Unicode panels for `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, and `[!CAUTION]`.
  - **Tables:** Full Unicode box-drawing borders (`┌`, `┬`, `┐`, `│`, `├`, `┼`, `┤`, `└`, `┴`, `┘`) with proportional column auto-scaling to fit split widths.
  - **Task Lists:** Renders `[ ]` as `☐` and `[x]` as bright green `✔`.
  - **Clickable Links:** Native terminal OSC 8 hyperlinks.
- **📐 Responsive Column Wrapping:** Automatically detects the split or modal column width and reformats text, tables, and borders cleanly.
- **🩺 First-Class Health Checks:** Run `:checkhealth mat` to inspect toolchains and binary status at any time.

---

## 📦 Installation

### [lazy.nvim](https://github.com/folke/lazy.nvim) (Recommended)

```lua
{
  "imyash0722/mat",
  build = "cargo build --release",
  ft = { "markdown", "md" },
  cmd = { "Mat", "MatPreview", "MatPreviewToggle", "MatClose", "MatBuild" },
  keys = {
    { "<leader>mv", "<cmd>Mat<cr>", desc = "Markdown Preview (Float)" },
    { "<leader>mp", "<cmd>MatPreviewToggle<cr>", desc = "Markdown Live Preview (Split)" },
  },
  opts = {
    float = {
      width = 0.85,       -- float width ratio (85% of editor)
      height = 0.85,      -- float height ratio
      border = "rounded", -- "rounded" | "single" | "double" | "shadow"
    },
    preview = {
      width = 0.45,       -- split width ratio (45% of editor)
      auto_update = true, -- live re-render on buffer save
    },
  },
}
```

### [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use({
  "imyash0722/mat",
  run = "cargo build --release",
  ft = { "markdown", "md" },
  config = function()
    require("mat").setup()
  end,
})
```

### [vim-plug](https://github.com/junegunn/vim-plug)

```vim
Plug 'imyash0722/mat', { 'do': 'cargo build --release', 'for': ['markdown', 'md'] }
```

---

## 🛠️ Commands

| Command | Mode | Description |
| :--- | :--- | :--- |
| `:Mat` | Normal | Opens interactive floating preview modal for current buffer |
| `:Mat <file>` | Normal | Opens interactive floating preview modal for specified file |
| `:'<,'>Mat` | Visual | Renders the selected Markdown lines in a floating preview card |
| `:MatPreview` | Normal | Opens live auto-updating side split preview (`vsplit`) |
| `:MatPreviewToggle` | Normal | Toggles the live side split preview open or closed |
| `:MatClose` | Normal | Closes the active live side split preview |
| `:MatBuild` | Any | Compiles the Rust backend binary via `cargo build --release` |

---

## ⌨️ Floating Modal Navigation

When viewing Markdown in the floating modal (`:Mat`):

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
| `:q` / `q` / `ZZ` / `<Esc><Esc>` | Dismiss floating modal and return to editor |
| `:help` / `F1` | Toggle in-app keyboard shortcut cheat sheet |
| **Touchpad / Mouse Wheel** | Smooth vertical scrolling |

---

## ⚙️ Configuration

Pass your custom options table to `require("mat").setup(opts)`:

```lua
require("mat").setup({
  -- Custom path to mat backend binary (auto-detected if nil)
  binary = nil,

  -- Floating modal settings (:Mat)
  float = {
    width = 0.85,         -- Ratio of editor columns (0.1 to 1.0)
    height = 0.85,        -- Ratio of editor lines (0.1 to 1.0)
    border = "rounded",   -- "none" | "single" | "double" | "rounded" | "solid" | "shadow"
  },

  -- Live split preview settings (:MatPreview)
  preview = {
    width = 0.45,         -- Split width ratio (0.1 to 1.0)
    auto_update = true,   -- Automatically re-render preview when saving buffer (:w)
    debounce_ms = 150,    -- Debounce delay for updates
  },

  -- Default keybindings (set to nil or "" to disable)
  keymaps = {
    float = "<leader>mv",   -- Open floating preview
    preview = "<leader>mp", -- Toggle live preview split
  },
})
```

---

## 🩺 Health Check

Run Neovim's health check command to verify your environment:

```vim
:checkhealth mat
```

Output:
```text
==============================================================================
mat:                                                                        ✅
mat.nvim ~
- ✅ OK Neovim >= 0.8.0 (0.12.5)
- ✅ OK Rust backend binary found: .../mat/target/release/mat
- ✅ OK Rust toolchain available: cargo 1.98.1
```

---

## 📋 Architecture

```text
mat/
├── lua/
│   └── mat/
│       ├── init.lua       # Main plugin API, float manager & snippet renderer
│       ├── config.lua     # User configuration & backend binary detection
│       ├── preview.lua    # Live side split manager, atomic swap & autocmd hooks
│       └── health.lua     # Neovim :checkhealth provider
├── plugin/
│   └── mat.lua            # Global Neovim user commands (:Mat, :MatPreview, etc.)
├── Cargo.toml             # Rust package configuration
└── src/
    ├── main.rs            # CLI argument parsing, streaming & TUI dispatch
    ├── layout.rs          # Layout calculations, dynamic centering & bat-style frames
    ├── viewer.rs          # Neovim-style alternate screen TUI, event loop & keybindings
    ├── render.rs          # Pulldown-cmark AST event loop, GFM blocks & formatting
    ├── syntax.rs          # Syntect TrueColor highlighting (base16-ocean.dark)
    ├── table.rs           # GFM Unicode table layout, auto-sizing & alignment
    └── terminal.rs        # ANSI-aware width calculation, wrapping & hanging indents
```

---

## 📄 License

[MIT License](LICENSE) © 2026 Yashwanth
