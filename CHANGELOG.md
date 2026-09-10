# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
