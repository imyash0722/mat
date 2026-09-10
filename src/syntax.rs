use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

struct SyntaxState {
    ps: SyntaxSet,
    theme: Theme,
}

static STATE: OnceLock<SyntaxState> = OnceLock::new();

fn get_state() -> &'static SyntaxState {
    STATE.get_or_init(|| {
        let mut ts = ThemeSet::load_defaults();
        let theme = ts
            .themes
            .remove("base16-ocean.dark")
            .or_else(|| ts.themes.into_values().next())
            .expect("At least one default theme available");

        SyntaxState {
            ps: SyntaxSet::load_defaults_newlines(),
            theme,
        }
    })
}

/// Highlights source code using 24-bit ANSI color sequences.
pub fn highlight_code(code: &str, lang: &str) -> Vec<String> {
    let state = get_state();
    let syntax = if !lang.trim().is_empty() {
        state.ps.find_syntax_by_token(lang.trim())
            .or_else(|| state.ps.find_syntax_by_extension(lang.trim()))
            .unwrap_or_else(|| state.ps.find_syntax_plain_text())
    } else {
        state.ps.find_syntax_plain_text()
    };

    let mut h = HighlightLines::new(syntax, &state.theme);
    let mut lines = Vec::new();

    for line in LinesWithEndings::from(code) {
        let trimmed_end = line.trim_end_matches(['\r', '\n']);
        let ranges = h.highlight_line(line, &state.ps).unwrap_or_default();
        if ranges.is_empty() {
            lines.push(trimmed_end.to_string());
        } else {
            let mut escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            // syntect includes newline in highlighted output if line had newline; remove it
            while escaped.ends_with('\n') || escaped.ends_with('\r') {
                escaped.pop();
            }
            escaped.push_str("\x1b[0m");
            lines.push(escaped);
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_code() {
        let code = "fn main() {\n    let x = 42;\n}";
        let lines = highlight_code(code, "rs");
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("fn"));
    }
}
