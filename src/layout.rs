use std::env;
use std::io::{self, IsTerminal};
use terminal_size::terminal_size;

pub const DEFAULT_MAX_WIDTH: usize = 100;

pub struct Layout {
    pub content_width: usize,
    pub left_pad: usize,
}

pub fn compute_layout(
    custom_width: Option<usize>,
    term_cols_override: Option<usize>,
    force_is_term: bool,
) -> Layout {
    let actual_term_width = if let Some(cols) = term_cols_override {
        cols
    } else if let Some((terminal_size::Width(w), _)) = terminal_size()
        && w > 0
    {
        w as usize
    } else if let Ok(cols) = env::var("COLUMNS")
        && let Ok(w) = cols.parse::<usize>()
        && w > 0
    {
        w
    } else {
        80
    };

    let is_term = force_is_term || io::stdout().is_terminal();

    if let Some(custom) = custom_width {
        if custom == 0 {
            // Explicitly requested full uncapped terminal width
            return Layout {
                content_width: actual_term_width.max(20),
                left_pad: 0,
            };
        }
        let content_width = custom.min(actual_term_width).max(20);
        let left_pad = if is_term {
            (actual_term_width.saturating_sub(content_width)) / 2
        } else {
            0
        };
        return Layout {
            content_width,
            left_pad,
        };
    }

    // Default: cap content width at DEFAULT_MAX_WIDTH (100 cols) and center it
    let content_width = actual_term_width.clamp(40, DEFAULT_MAX_WIDTH);
    let left_pad = if is_term {
        (actual_term_width.saturating_sub(content_width)) / 2
    } else {
        0
    };

    Layout {
        content_width,
        left_pad,
    }
}

pub fn apply_centering(text: &str, left_pad: usize) -> String {
    if left_pad == 0 {
        return text.to_string();
    }
    let pad = " ".repeat(left_pad);
    let mut result = String::with_capacity(text.len() + left_pad * 30);
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            result.push('\n');
        }
        if !line.is_empty() {
            result.push_str(&pad);
            result.push_str(line);
        }
    }
    result
}

pub fn format_bat_header(path: &str, width: usize) -> String {
    let path_obj = std::path::Path::new(path);
    let display_name = if path_obj.is_absolute() {
        env::current_dir()
            .ok()
            .and_then(|cwd| path_obj.strip_prefix(&cwd).ok())
            .and_then(|rel| rel.to_str())
            .or_else(|| path_obj.file_name().and_then(|s| s.to_str()))
            .unwrap_or(path)
    } else {
        path
    };

    let border_color = "\x1b[38;2;90;90;125m";
    let reset = "\x1b[0m";
    let margin = "  ";
    let bar_len = width.saturating_sub(4);
    let bar = "─".repeat(bar_len);

    format!(
        "{margin}{border_color}{bar}{reset}\n\
         {margin}\x1b[38;2;130;130;160mFile: \x1b[1;38;2;84;160;255m{display_name}{reset}\n\
         {margin}{border_color}{bar}{reset}\n\n"
    )
}

pub fn format_bat_footer(width: usize) -> String {
    let border_color = "\x1b[38;2;90;90;125m";
    let reset = "\x1b[0m";
    let margin = "  ";
    let bar_len = width.saturating_sub(4);
    let bar = "─".repeat(bar_len);

    format!("\n\n{margin}{border_color}{bar}{reset}\n")
}
