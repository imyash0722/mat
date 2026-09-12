# mat.nvim 📄

[![GitHub Release](https://img.shields.io/github/v/release/imyash0722/mat?color=38d39f&logo=github)](https://github.com/imyash0722/mat/releases)
[![CI](https://github.com/imyash0722/mat/actions/workflows/ci.yml/badge.svg)](https://github.com/imyash0722/mat/actions/workflows/ci.yml)
[![Neovim](https://img.shields.io/badge/Neovim-%3E%3D%200.8.0-blueviolet?logo=neovim)](https://neovim.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

> **A blazingly fast in-editor Markdown preview plugin for Neovim powered by a sub-5ms Rust engine.**  
> Seamlessly flip between **Preview Mode** and **Edit Mode** with a single keystroke. Zero browser overhead, zero Node.js/Python runtimes. Experience high-fidelity Markdown preview—with TrueColor syntax highlighting, GitHub Flavored Markdown (GFM) callouts and tables, bat-style framing, and interactive navigation—directly inside your terminal Neovim.


## ⚡ The Killer Feature: Instant Preview ⇄ Edit Toggle

When editing any `.md` file in Neovim, toggle between **rendered preview** and your **normal editor** with a single shortcut:

```
                  ┌──────────────────────────────┐
                  │   Normal Neovim Edit Mode    │
                  │   (Raw Markdown Buffer)      │
                  └──────────────┬───────────────┘
                                 │
                   <leader>mp    │    <leader>mp
                   or :Mat       │    or 'i' to edit
                                 ▼
                  ┌──────────────────────────────┐
                  │    Rendered Preview Mode     │
                  │ (mat TrueColor ANSI Engine)  │
                  └──────────────────────────────┘
```

1. **In Edit Mode:** Hit `<leader>mp` (or run `:Mat`). If the file is a `.md`, `mat` instantly renders the document in-place in your current window with full TrueColor formatting, tables, code blocks, and alerts.
2. **In Preview Mode:** Hit `<leader>mp` again—or simply press **`i`** to start inserting or **`e`** to edit—and you are immediately back in the normal Neovim editor with your cursor placed right where you were reading!
3. **Unsaved Edits Supported:** Previews live in-memory buffer changes instantly without forcing you to write (`:w`) first.
4. **Filetype Guarded:** Automatically protects non-markdown files from unintended toggles.

## ✨ Features

- **🚀 Sub-5ms Startup Latency:** Built with a standalone Rust core (`pulldown-cmark` SIMD & `syntect`). Renders in ~4ms with zero browser or Node/Python bloat.
- **🔄 In-Place Preview ⇄ Edit Switch (`:Mat` / `:MatToggle`):** Flip your current window between rendered reading mode and normal editing mode. Press `i` to jump straight into editing.
- **🪟 Interactive Floating Modal (`:MatFloat`):** Centered floating popup with full Vim navigation (`j`/`k`, `Ctrl+d`/`u`, `gg`/`G`, `/` search), smooth trackpad/mouse wheel scrolling, and instant dismiss with `q`.
- **📑 Side-by-Side Live Split (`:MatSplit`):** Vertical split preview (`:vsplit`) that automatically updates whenever you save (`:w`) using zero-flicker atomic buffer swapping.
- **✂️ Visual Snippet Preview (`:'<,'>Mat`):** Highlight any Markdown lines or docstrings in visual mode and preview just that selection in a popup card.
- **🎨 Native Terminal Theme Sync:** Reverse video statusline (`\x1b[7m`) and adaptive ANSI attributes that automatically sync with whatever colorscheme or terminal palette you use.
- **🖼️ `bat`-Style Framed Views:** Clean header borders and contained code fence grids with language badges, TrueColor syntax highlighting, and dimmed line numbers.
- **✨ Full GitHub Flavored Markdown (GFM):**
  - **Alerts / Callouts:** Rounded Unicode panels for `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, and `[!CAUTION]`.
  - **Tables:** Full Unicode box-drawing borders (`┌`, `┬`, `┐`, `│`, `├`, `┼`, `┤`, `└`, `┴`, `┘`) with proportional column auto-scaling.
  - **Task Lists:** Renders `[ ]` as `☐` and `[x]` as bright green `✔`.
  - **Clickable Links:** Native terminal OSC 8 hyperlinks.
- **📐 Responsive Column Wrapping:** Automatically detects the split or modal column width and reformats text, tables, and borders cleanly.
- **🩺 First-Class Health Checks:** Run `:checkhealth mat` to inspect toolchains and binary status at any time.

## 📦 Installation

### [lazy.nvim](https://github.com/folke/lazy.nvim) (Recommended)

```lua
{
  "imyash0722/mat",
  build = "cargo build --release",
  ft = { "markdown", "md" },
  cmd = { "Mat", "MatToggle", "MatPreview", "MatEdit", "MatSplit", "MatFloat", "MatClose", "MatBuild" },
  keys = {
    { "<leader>mp", "<cmd>MatToggle<cr>", desc = "Toggle Markdown Preview / Edit Mode" },
    { "<leader>mv", "<cmd>MatFloat<cr>", desc = "Markdown Preview (Float Modal)" },
    { "<leader>ms", "<cmd>MatSplit<cr>", desc = "Markdown Live Preview (Side Split)" },
  },
  opts = {
    auto_preview = false, -- Set to true to automatically open .md files in Preview Mode
    keymaps = {
      toggle = "<leader>mp", -- In-place preview/edit toggle shortcut
      float = "<leader>mv",  -- Floating modal shortcut
      split = "<leader>ms",  -- Side-by-side split shortcut
    },
    float = {
      width = 0.85,          -- Float width ratio (85% of editor)
      height = 0.85,         -- Float height ratio
      border = "rounded",    -- "rounded" | "single" | "double" | "shadow"
    },
    preview = {
      width = 0.45,          -- Side split width ratio (45% of editor)
      auto_update = true,    -- Live re-render on buffer save
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

## ⌨️ Mode Controls & Navigation

### While in Preview Mode:

When viewing rendered Markdown in the current window:

| Key | Action |
| :--- | :--- |
| **`i`** | **Switch to Edit Mode and enter Insert Mode** immediately |
| **`a`** | **Switch to Edit Mode and enter Append Mode** immediately |
| **`e`** / **`q`** / **`<Esc>`** | **Switch to Edit Mode in Normal Mode** |
| **`<leader>mp`** | **Toggle back to Edit Mode** |
| `j` / `k` / `↓` / `↑` | Scroll down / up by line |
| `Ctrl+d` / `Ctrl+u` | Scroll half page down / up |
| `Ctrl+f` / `Ctrl+b` | Scroll full page down / up |
| `gg` / `G` | Jump to top / bottom of document |
| `/pattern` / `?pattern` | Search forward / backward |
| `n` / `N` | Jump to next / previous match |
| **Touchpad / Mouse** | Smooth vertical scrolling |

## 🛠️ Commands

| Command | Mode | Description |
| :--- | :--- | :--- |
| `:Mat` / `:MatToggle` | Normal | Toggles current `.md` window between Preview Mode and Edit Mode |
| `:'<,'>Mat` | Visual | Renders highlighted Markdown snippet in a floating card |
| `:MatEdit` | Normal | Switches current window back to normal Markdown edit mode |
| `:MatPreview` | Normal | Switches current window into rendered Markdown preview mode |
| `:MatSplit` | Normal | Opens live auto-updating side split preview (`vsplit`) |
| `:MatFloat [file]` | Normal | Opens interactive floating preview modal |
| `:MatClose` | Normal | Closes active preview (in-place or split) |
| `:MatBuild` | Any | Compiles the Rust backend engine via `cargo build --release` |

## ⚙️ Configuration Options

```lua
require("mat").setup({
  -- Custom path to mat backend binary (auto-detected if nil)
  binary = nil,

  -- Automatically enter preview mode when opening a .md file
  auto_preview = false,

  -- Keybindings (set to nil or "" to disable)
  keymaps = {
    toggle = "<leader>mp",  -- Toggle Preview/Edit mode in current window
    float = "<leader>mv",   -- Open floating preview modal
    split = "<leader>ms",   -- Open side-by-side live split
  },

  -- Floating modal settings (:MatFloat)
  float = {
    width = 0.85,           -- Ratio of editor columns (0.1 to 1.0)
    height = 0.85,          -- Ratio of editor lines (0.1 to 1.0)
    border = "rounded",     -- "none" | "single" | "double" | "rounded" | "solid" | "shadow"
  },

  -- Live split preview settings (:MatSplit)
  preview = {
    width = 0.45,           -- Split width ratio (0.1 to 1.0)
    auto_update = true,     -- Automatically re-render when saving buffer (:w)
    debounce_ms = 150,      -- Debounce delay for updates
  },
})
```

## 🩺 Health Check

Run Neovim's health check command at any time to verify your environment:

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

## 📋 Architecture

```text
mat/
├── lua/
│   └── mat/
│       ├── init.lua       # Main plugin API & keybinding orchestrator
│       ├── toggle.lua     # In-place Preview ⇄ Edit mode switcher & auto-preview
│       ├── config.lua     # User configuration & backend binary detection
│       ├── preview.lua    # Side split live preview manager & atomic buffer swap
│       └── health.lua     # Neovim :checkhealth provider
├── plugin/
│   └── mat.lua            # Global Neovim user commands (:Mat, :MatToggle, etc.)
├── Cargo.toml             # Rust package configuration
└── src/
    ├── main.rs            # CLI argument parsing, streaming & TUI dispatch
    ├── layout.rs          # Layout calculations, dynamic centering & bat-style frames
    ├── viewer.rs          # Alternate screen TUI, event loop & keybindings
    ├── render.rs          # Pulldown-cmark AST event loop, GFM blocks & formatting
    ├── syntax.rs          # Syntect TrueColor highlighting (base16-ocean.dark)
    ├── table.rs           # GFM Unicode table layout, auto-sizing & alignment
    └── terminal.rs        # ANSI-aware width calculation, wrapping & hanging indents
```

## 📄 License

[MIT License](LICENSE) © 2026 Yashwanth
