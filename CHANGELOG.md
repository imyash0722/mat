# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **In-Place Preview ⇄ Edit Mode Switching (`mat.nvim`)**: Instant in-window toggle between raw Markdown editing and rendered reading mode with `<leader>mp` or `:MatToggle`, returning straight to editing on `i`/`e`/`q` with relative cursor position preserved.
- **Neovim Plugin Integration (`mat.nvim`)**: Native Lua plugin for Neovim providing `:Mat` / `:MatToggle` (in-place preview switch), `:MatSplit` (live auto-updating side split preview), `:MatFloat` (floating preview modal), `:'<,'>Mat` (visual selection preview), and `:checkhealth mat`.
- **Neovim-Style Alternate Screen Viewer**: Interactive full-window TUI on alternate buffer that restores the terminal cleanly upon exit without leaving scrollback debris.
- **Dynamic Real-Time Centering**: Reading width capped at ~100 columns and dynamically re-centered in real time upon terminal window resize events.
- **Vim / Neovim Keybindings**: Full movement suite (`j`/`k`, `Enter`, `Ctrl+e`/`y`, `d`/`u`, `Ctrl+d`/`u`, `f`/`b`, `PageDown`/`Up`, `Space`, `gg`/`G`, `<count>G`, `:q`, `q`, `ZZ`).
- **Interactive Forward & Backward Search**: Regex-free text search (`/pattern`, `?pattern`) with match counter and navigation (`n`/`N`).
- **Smooth Touchpad & Mouse Wheel Scrolling**: Native vertical scrolling support for mouse wheels and precision trackpads.
- **Neovim Statusline**: Bottom status bar displaying active mode (`[NORMAL]`, `[COMMAND]`, `[SEARCH]`), file name, reading progress percentage, and line position (`[Line X/Y]`).
### Changed
- **Rebranded to `mat`**: Renamed package, binary, and repository from `mdview` to `mat` with direct standalone command execution.
- **Terminal-Synced Statusline**: Replaced hardcoded RGB colors in status bar with universal Reverse Video (`\x1b[7m`) and standard ANSI attributes, syncing seamlessly with any terminal palette (dark, light, Catppuccin, Gruvbox, etc.).
- **Interactive Cursor**: Position and show cursor dynamically during `:` command and `/` search input, while keeping it hidden in normal mode.

### Fixed
- **BCE Background Color Bleed**: Solved horizontal color bleed stripes caused by unclosed ANSI background sequences when erasing to end-of-line (`\x1b[K`).
- **Style Tracking Across Line Wrapping**: Fixed `wrap_ansi` and `tokenize_ansi` so wrapped lines cleanly close active ANSI styles with `\x1b[0m` and restore them at the beginning of continuation lines.
- **Inline Code Background Compatibility**: Removed hardcoded dark slate background from inline code, displaying clean syntax-colored text that remains readable across all terminal themes.

## [1.1.0] - 2026-09-10

### Added
- **bat-Style Framed Layout**: File header (`File: <path>`) and footer borders inspired by `bat`, without line-numbering ordinary prose.
- **Contained Code Blocks**: Language badges, 24-bit TrueColor syntax highlighting powered by `syntect` (`base16-ocean.dark`), and dimmed gutter line numbers.
- **GFM Callouts & Alerts**: Rounded Unicode panels for `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, and `[!CAUTION]`.
- **Unicode Table Engine**: Full box-drawing borders (`┌`, `┬`, `┐`, `│`, `├`, `┼`, `┤`, `└`, `┴`, `┘`), cell alignment (left, center, right), and proportional column shrinking when exceeding terminal width.
- **Task Lists**: Renders `[ ]` as `☐` and `[x]` as bright green `✔`.
- **Clickable Hyperlinks**: Real terminal hyperlinks via OSC 8 escape sequences.
- **Shell Completions**: Built-in completion generator (`-c`, `--completions <zsh|fish|bash>`) and bundled completion scripts for Zsh, Fish, and Bash.
- **Smart Pager Integration**: Paged output via `$PAGER` or `less` with ANSI colors (`-R`), screen preservation (`-X`), and `LESSUTFCHARDEF` for Nerd Font glyphs.
- **Automated Release Pipeline**: Multi-platform cross-compilation matrix for Linux (glibc & musl), macOS (x86_64 & ARM64), and Windows.

### Changed
- Refactored event parser from DOM trees to streaming pull-parsing (`pulldown-cmark` SIMD).
- Clamped default terminal display width to 40..100 columns for enhanced reading comfort.
- Upgraded to Rust 2024 Edition with link-time optimization (LTO) and binary stripping.

### Fixed
- Proper column measurement for Nerd Font / Private Use Area (PUA) glyphs and zero-width codes.
- Smart hanging indent for wrapped list items and code continuations.
- Eliminated redundant heap allocations across link tags and line trimming routines.

## [1.0.0] - 2026-09-09

### Added
- Initial standalone Rust implementation of high-performance CLI Markdown reader.
- Sub-5ms startup latency.
- Terminal width autodetection via IOCTL.
