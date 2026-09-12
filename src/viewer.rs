use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write, stdout};
use std::time::Duration;

use crate::layout::{apply_centering, compute_layout, format_bat_footer, format_bat_header};
use crate::render::MarkdownRenderer;
use crate::terminal::strip_ansi;

/// Guard to ensure raw mode and alternate screen are always restored on drop.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture, Hide)?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), Show, DisableMouseCapture, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

enum InputMode {
    Normal,
    Command { buffer: String },
    Search { buffer: String, reverse: bool },
    Help,
}

pub struct Viewer<'a> {
    content: &'a str,
    file_path: Option<&'a str>,
    custom_width: Option<usize>,
    lines: Vec<String>,
    scroll_offset: usize,
    term_cols: usize,
    term_rows: usize,
    input_mode: InputMode,
    status_message: Option<String>,
    pending_g: bool,
    pending_z: bool,
    number_prefix: usize,
    search_query: Option<String>,
    search_matches: Vec<usize>,
    current_match_idx: Option<usize>,
}

impl<'a> Viewer<'a> {
    pub fn new(content: &'a str, file_path: Option<&'a str>, custom_width: Option<usize>) -> Self {
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let term_cols = cols as usize;
        let term_rows = rows as usize;

        let mut viewer = Self {
            content,
            file_path,
            custom_width,
            lines: Vec::new(),
            scroll_offset: 0,
            term_cols,
            term_rows,
            input_mode: InputMode::Normal,
            status_message: None,
            pending_g: false,
            pending_z: false,
            number_prefix: 0,
            search_query: None,
            search_matches: Vec::new(),
            current_match_idx: None,
        };

        viewer.recompute_lines();
        viewer
    }

    fn recompute_lines(&mut self) {
        let layout = compute_layout(self.custom_width, Some(self.term_cols), true);
        let renderer = MarkdownRenderer::new(layout.content_width);
        let rendered = renderer.render(self.content);

        let mut full_output = String::new();
        if let Some(path) = self.file_path {
            full_output.push_str(&format_bat_header(path, layout.content_width));
        }
        full_output.push_str(&rendered);
        if self.file_path.is_some() {
            full_output.push_str(&format_bat_footer(layout.content_width));
        }

        let centered = apply_centering(&full_output, layout.left_pad);
        self.lines = centered.split('\n').map(|s| s.to_string()).collect();

        // If search was active, re-index matches
        if let Some(q) = self.search_query.clone() {
            self.update_search_matches(&q);
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let _guard = TerminalGuard::new()?;

        self.draw()?;

        loop {
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key(key)? {
                            break;
                        }
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse);
                    }
                    Event::Resize(cols, rows) => {
                        self.term_cols = cols as usize;
                        self.term_rows = rows as usize;
                        self.recompute_lines();
                        self.clamp_scroll();
                    }
                    _ => {}
                }
                self.draw()?;
            }
        }

        Ok(())
    }

    fn viewport_height(&self) -> usize {
        self.term_rows.saturating_sub(1)
    }

    fn max_scroll(&self) -> usize {
        self.lines.len().saturating_sub(self.viewport_height())
    }

    fn clamp_scroll(&mut self) {
        let max_s = self.max_scroll();
        if self.scroll_offset > max_s {
            self.scroll_offset = max_s;
        }
    }

    fn scroll_down(&mut self, n: usize) {
        let max_s = self.max_scroll();
        self.scroll_offset = (self.scroll_offset + n).min(max_s);
    }

    fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                self.scroll_down(3);
            }
            MouseEventKind::ScrollUp => {
                self.scroll_up(3);
            }
            _ => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        match &self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Command { .. } => self.handle_command_key(key),
            InputMode::Search { .. } => self.handle_search_key(key),
            InputMode::Help => {
                // Any key closes help
                self.input_mode = InputMode::Normal;
                Ok(false)
            }
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        let half_page = (self.viewport_height() / 2).max(1);
        let full_page = self.viewport_height().max(1);

        // Clear status message on next keypress
        self.status_message = None;

        // Check number prefix for line jumps (e.g. 50G)
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() && key.modifiers.is_empty() => {
                if c == '0' && self.number_prefix == 0 {
                    // '0' with no prefix jumps to start of line or top
                    self.scroll_offset = 0;
                    return Ok(false);
                }
                self.number_prefix = self.number_prefix * 10 + (c as usize - '0' as usize);
                return Ok(false);
            }
            _ => {}
        }

        let prefix = self.number_prefix;
        let count = if prefix > 0 { prefix } else { 1 };
        self.number_prefix = 0;

        match key.code {
            // Quit
            KeyCode::Char('q') => return Ok(true),

            // Scroll down
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => {
                self.scroll_down(count);
                self.pending_g = false;
                self.pending_z = false;
            }
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_down(count);
            }

            // Scroll up
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up(count);
                self.pending_g = false;
                self.pending_z = false;
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_up(count);
            }

            // Half page down / up
            KeyCode::Char('d') | KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.scroll_down(half_page * count);
                self.pending_g = false;
                self.pending_z = false;
            }
            KeyCode::Char('u') | KeyCode::Char('u')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.scroll_up(half_page * count);
                self.pending_g = false;
                self.pending_z = false;
            }

            // Full page down / up
            KeyCode::Char('f') | KeyCode::PageDown | KeyCode::Char(' ')
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.code == KeyCode::Char('f') =>
            {
                self.scroll_down(full_page * count);
                self.pending_g = false;
                self.pending_z = false;
            }
            KeyCode::Char('b') | KeyCode::PageUp => {
                self.scroll_up(full_page * count);
                self.pending_g = false;
                self.pending_z = false;
            }

            // Jump to Top
            KeyCode::Char('g') => {
                if self.pending_g {
                    // 'gg' jump
                    self.scroll_offset = 0;
                    self.pending_g = false;
                } else {
                    self.pending_g = true;
                }
                self.pending_z = false;
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
                self.pending_g = false;
                self.pending_z = false;
            }

            // Jump to Bottom or specified Line (e.g. 25G)
            KeyCode::Char('G') | KeyCode::End => {
                if prefix > 0 {
                    self.scroll_offset = (prefix.saturating_sub(1)).min(self.max_scroll());
                } else {
                    self.scroll_offset = self.max_scroll();
                }
                self.pending_g = false;
                self.pending_z = false;
            }

            // ZZ quit
            KeyCode::Char('Z') => {
                if self.pending_z {
                    return Ok(true);
                } else {
                    self.pending_z = true;
                }
            }

            // Search
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search {
                    buffer: String::new(),
                    reverse: false,
                };
                self.pending_g = false;
                self.pending_z = false;
            }
            KeyCode::Char('?') => {
                self.input_mode = InputMode::Search {
                    buffer: String::new(),
                    reverse: true,
                };
                self.pending_g = false;
                self.pending_z = false;
            }
            KeyCode::Char('n') => {
                self.jump_next_match(false);
            }
            KeyCode::Char('N') => {
                self.jump_next_match(true);
            }

            // Command mode
            KeyCode::Char(':') => {
                self.input_mode = InputMode::Command {
                    buffer: String::new(),
                };
                self.pending_g = false;
                self.pending_z = false;
            }

            // Help
            KeyCode::F(1) => {
                self.input_mode = InputMode::Help;
            }

            // Clear search highlights or pending state
            KeyCode::Esc => {
                self.pending_g = false;
                self.pending_z = false;
                self.search_query = None;
                self.search_matches.clear();
                self.current_match_idx = None;
            }

            _ => {
                self.pending_g = false;
                self.pending_z = false;
            }
        }

        Ok(false)
    }

    fn handle_command_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        let mut exit = false;
        let mut next_mode = None;
        let mut new_offset = None;
        let mut new_status = None;

        if let InputMode::Command { ref mut buffer } = self.input_mode {
            match key.code {
                KeyCode::Enter => {
                    let cmd = buffer.trim().to_string();
                    match cmd.as_str() {
                        "q" | "q!" | "quit" | "qa" | "qa!" | "exit" => exit = true,
                        "h" | "help" => {
                            next_mode = Some(InputMode::Help);
                        }
                        s if s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty() => {
                            if let Ok(line_num) = s.parse::<usize>() {
                                new_offset =
                                    Some((line_num.saturating_sub(1)).min(self.max_scroll()));
                            }
                            next_mode = Some(InputMode::Normal);
                        }
                        "top" | "0" => {
                            new_offset = Some(0);
                            next_mode = Some(InputMode::Normal);
                        }
                        "bot" | "$" => {
                            new_offset = Some(self.max_scroll());
                            next_mode = Some(InputMode::Normal);
                        }
                        "" => {
                            next_mode = Some(InputMode::Normal);
                        }
                        other => {
                            new_status = Some(format!("Unknown command: :{}", other));
                            next_mode = Some(InputMode::Normal);
                        }
                    }
                }
                KeyCode::Esc => {
                    next_mode = Some(InputMode::Normal);
                }
                KeyCode::Backspace => {
                    buffer.pop();
                    if buffer.is_empty() {
                        next_mode = Some(InputMode::Normal);
                    }
                }
                KeyCode::Char(c) => {
                    buffer.push(c);
                }
                _ => {}
            }
        }

        if exit {
            return Ok(true);
        }
        if let Some(off) = new_offset {
            self.scroll_offset = off;
        }
        if let Some(msg) = new_status {
            self.status_message = Some(msg);
        }
        if let Some(mode) = next_mode {
            self.input_mode = mode;
        }
        Ok(false)
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        let mut enter_query = None;
        let mut reverse_search = false;
        let mut next_mode = None;

        if let InputMode::Search {
            ref mut buffer,
            reverse,
        } = self.input_mode
        {
            reverse_search = reverse;
            match key.code {
                KeyCode::Enter => {
                    let query = buffer.trim().to_string();
                    if !query.is_empty() {
                        enter_query = Some(query);
                    }
                    next_mode = Some(InputMode::Normal);
                }
                KeyCode::Esc => {
                    next_mode = Some(InputMode::Normal);
                }
                KeyCode::Backspace => {
                    buffer.pop();
                    if buffer.is_empty() {
                        next_mode = Some(InputMode::Normal);
                    }
                }
                KeyCode::Char(c) => {
                    buffer.push(c);
                }
                _ => {}
            }
        }

        if let Some(query) = enter_query {
            self.update_search_matches(&query);
            self.search_query = Some(query);
            self.jump_next_match(reverse_search);
        }
        if let Some(mode) = next_mode {
            self.input_mode = mode;
        }
        Ok(false)
    }

    fn update_search_matches(&mut self, query: &str) {
        let q_lower = query.to_lowercase();
        let mut matches = Vec::new();

        for (idx, line) in self.lines.iter().enumerate() {
            let plain = strip_ansi(line).to_lowercase();
            if plain.contains(&q_lower) {
                matches.push(idx);
            }
        }

        self.search_matches = matches;
    }

    fn jump_next_match(&mut self, reverse: bool) {
        if self.search_matches.is_empty() {
            self.status_message = Some("Pattern not found".to_string());
            return;
        }

        let total = self.search_matches.len();

        let next_idx = if reverse {
            // Find last match strictly before current scroll_offset
            let mut found = None;
            for (idx, &line_idx) in self.search_matches.iter().enumerate().rev() {
                if line_idx < self.scroll_offset {
                    found = Some(idx);
                    break;
                }
            }
            found.unwrap_or(total - 1)
        } else {
            // Find first match strictly after current scroll_offset
            let mut found = None;
            for (idx, &line_idx) in self.search_matches.iter().enumerate() {
                if line_idx > self.scroll_offset {
                    found = Some(idx);
                    break;
                }
            }
            found.unwrap_or(0)
        };

        self.current_match_idx = Some(next_idx);
        let target_line = self.search_matches[next_idx];
        self.scroll_offset = target_line.min(self.max_scroll());
        self.status_message = Some(format!(
            "[{}/{}] Match at line {}",
            next_idx + 1,
            total,
            target_line + 1
        ));
    }

    fn draw(&mut self) -> io::Result<()> {
        let mut out = String::with_capacity(self.term_rows * (self.term_cols + 10));

        // Move cursor to top-left
        out.push_str("\x1b[H");

        let viewport_h = self.viewport_height();
        let total_lines = self.lines.len();

        for i in 0..viewport_h {
            let line_idx = self.scroll_offset + i;
            if line_idx < total_lines {
                out.push_str(&self.lines[line_idx]);
            } else {
                // Past EOF: print Neovim/Vim '~' using terminal dim text
                out.push_str("\x1b[2m~\x1b[0m");
            }
            // Clear rest of line and move to next
            // ALWAYS reset SGR styling first so BCE (Background Color Erase) never erases with an active background color!
            out.push_str("\x1b[0m\x1b[K\r\n");
        }

        // Render Neovim statusline on bottom row
        out.push_str(&self.render_statusline());

        // Draw Help overlay if active
        if matches!(self.input_mode, InputMode::Help) {
            out.push_str(&self.render_help_overlay());
        }

        // Cursor visibility: show when typing a command or search, hide in normal mode
        match &self.input_mode {
            InputMode::Command { buffer } => {
                let cursor_x = (1 + buffer.len()).min(self.term_cols.saturating_sub(1));
                out.push_str(&format!(
                    "{}{}",
                    MoveTo(cursor_x as u16, viewport_h as u16),
                    Show
                ));
            }
            InputMode::Search { buffer, .. } => {
                let cursor_x = (1 + buffer.len()).min(self.term_cols.saturating_sub(1));
                out.push_str(&format!(
                    "{}{}",
                    MoveTo(cursor_x as u16, viewport_h as u16),
                    Show
                ));
            }
            _ => {
                out.push_str(&format!("{}", Hide));
            }
        }

        let mut stdout_handle = stdout().lock();
        stdout_handle.write_all(out.as_bytes())?;
        stdout_handle.flush()?;

        Ok(())
    }

    fn render_statusline(&self) -> String {
        let total_lines = self.lines.len();
        let current_line = (self.scroll_offset + 1).min(total_lines);

        let pct_str = if total_lines == 0 || self.scroll_offset == 0 {
            "Top".to_string()
        } else if self.scroll_offset >= self.max_scroll() {
            "Bot".to_string()
        } else {
            format!("{}%", ((self.scroll_offset * 100) / total_lines).max(1))
        };

        let file_name = self.file_path.unwrap_or("stdin");

        match &self.input_mode {
            InputMode::Command { buffer } => {
                format!(":{}\x1b[0m\x1b[K", buffer)
            }
            InputMode::Search { buffer, reverse } => {
                let prefix = if *reverse { "?" } else { "/" };
                format!("{}{}\x1b[0m\x1b[K", prefix, buffer)
            }
            _ => {
                if let Some(ref msg) = self.status_message {
                    return format!("\x1b[7m\x1b[1m {} \x1b[0m\x1b[K", msg);
                }

                let mode_str = " NORMAL ";
                let file_str = format!(" {} ", file_name);
                let pos_str = format!(" {}  [Line {}/{}] ", pct_str, current_line, total_lines);

                let visible_w = mode_str.len() + file_str.len() + pos_str.len();
                let gap = self.term_cols.saturating_sub(visible_w);
                let filler = " ".repeat(gap);

                // Full statusline rendered in reverse video (\x1b[7m), syncing natively with any terminal theme
                format!(
                    "\x1b[7m\x1b[1m{}\x1b[22m{}{}{}\x1b[0m\x1b[K",
                    mode_str, file_str, filler, pos_str
                )
            }
        }
    }

    fn render_help_overlay(&self) -> String {
        let help_lines = [
            "┌──────────────────────────── mat Help ───────────────────────────┐",
            "│                                                                  │",
            "│  Navigation:                                                     │",
            "│    j / k, ↓ / ↑           Scroll down / up by 1 line             │",
            "│    d / u, Ctrl+d / u      Scroll down / up by half page          │",
            "│    f / b, PageDown / Up   Scroll down / up by full page          │",
            "│    gg / G                 Jump to beginning / end of document    │",
            "│    <number>G              Jump to specific line number           │",
            "│    Touchpad / Mouse       Smooth vertical scrolling              │",
            "│                                                                  │",
            "│  Search & Commands:                                              │",
            "│    /pattern               Search forward                         │",
            "│    ?pattern               Search backward                        │",
            "│    n / N                  Next / previous search match           │",
            "│    :q, q, ZZ              Quit mat                               │",
            "│    :help, F1              Toggle this help screen                │",
            "│                                                                  │",
            "│                 Press Esc or q to close help                     │",
            "└──────────────────────────────────────────────────────────────────┘",
        ];

        let box_w = 68;
        let box_h = help_lines.len();

        let start_x = (self.term_cols.saturating_sub(box_w)) / 2;
        let start_y = (self.viewport_height().saturating_sub(box_h)) / 2;

        let mut out = String::new();
        for (idx, line) in help_lines.iter().enumerate() {
            let row = start_y + idx;
            let col = start_x;
            out.push_str(&format!(
                "{}\x1b[7m\x1b[1m{}\x1b[0m",
                MoveTo(col as u16, row as u16),
                line
            ));
        }

        out
    }
}
