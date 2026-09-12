use unicode_width::UnicodeWidthChar;

/// Computes the visual display width of a single unicode character,
/// properly accounting for Nerd Font / Private Use Area glyphs and zero-width codes.
pub fn char_width(c: char) -> usize {
    if let Some(w) = UnicodeWidthChar::width(c) {
        w
    } else if is_zero_width(c) {
        0
    } else {
        // PUA (Private Use Area - Nerd Fonts: U+E000..=U+F8FF, U+F0000..=U+FFFFD, U+100000..=U+10FFFD)
        // or unassigned terminal glyphs that occupy 1 column.
        1
    }
}

fn is_zero_width(c: char) -> bool {
    c.is_control()
        || matches!(c,
            '\u{200B}'..='\u{200F}' |
            '\u{202A}'..='\u{202E}' |
            '\u{2060}'..='\u{206F}' |
            '\u{FE00}'..='\u{FE0F}' |
            '\u{FEFF}'
        )
}

/// Computes the visual column width of a string, ignoring ANSI and OSC escape sequences.
pub fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.next() {
                Some('[') => {
                    // CSI sequence: consumes until 0x40..=0x7E
                    for next_c in chars.by_ref() {
                        if ('\x40'..='\x7e').contains(&next_c) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    // OSC sequence: consumes until BEL (\x07) or ST (\x1b\)
                    while let Some(next_c) = chars.next() {
                        if next_c == '\x07' {
                            break;
                        }
                        if next_c == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
        } else {
            width += char_width(c);
        }
    }
    width
}

/// Strips ANSI CSI and OSC escape codes from a string.
pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.next() {
                Some('[') => {
                    for next_c in chars.by_ref() {
                        if ('\x40'..='\x7e').contains(&next_c) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    while let Some(next_c) = chars.next() {
                        if next_c == '\x07' {
                            break;
                        }
                        if next_c == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// A word-token for ANSI-aware wrapping.
#[derive(Debug, Clone)]
struct Token {
    raw: String,
    visible_width: usize,
    is_whitespace: bool,
}

/// Tokenizes an ANSI string into whitespace and non-whitespace tokens,
/// keeping ANSI sequences attached to the preceding or following word.
fn tokenize_ansi(s: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_raw = String::new();
    let mut current_width = 0;
    let mut in_whitespace: Option<bool> = None;

    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // If we were accumulating whitespace, finish that whitespace token first!
            // The ANSI sequence belongs to the following non-whitespace token or styled text.
            if in_whitespace == Some(true) && !current_raw.is_empty() {
                tokens.push(Token {
                    raw: std::mem::take(&mut current_raw),
                    visible_width: current_width,
                    is_whitespace: true,
                });
                current_width = 0;
                in_whitespace = Some(false);
            }

            // Collect ANSI escape sequence into current token
            let mut seq = String::from("\x1b");
            match chars.next() {
                Some('[') => {
                    seq.push('[');
                    for next_c in chars.by_ref() {
                        seq.push(next_c);
                        if ('\x40'..='\x7e').contains(&next_c) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    seq.push(']');
                    while let Some(next_c) = chars.next() {
                        seq.push(next_c);
                        if next_c == '\x07' {
                            break;
                        }
                        if next_c == '\x1b' && chars.peek() == Some(&'\\') {
                            seq.push(chars.next().unwrap());
                            break;
                        }
                    }
                }
                Some(other) => {
                    seq.push(other);
                }
                None => {}
            }
            current_raw.push_str(&seq);
            continue;
        }

        let is_space = c == ' ' || c == '\t';
        if let Some(ws) = in_whitespace {
            if ws != is_space {
                tokens.push(Token {
                    raw: std::mem::take(&mut current_raw),
                    visible_width: current_width,
                    is_whitespace: ws,
                });
                current_width = 0;
                in_whitespace = Some(is_space);
            }
        } else {
            in_whitespace = Some(is_space);
        }

        current_raw.push(c);
        current_width += char_width(c);
    }

    if !current_raw.is_empty() {
        tokens.push(Token {
            raw: current_raw,
            visible_width: current_width,
            is_whitespace: in_whitespace.unwrap_or(false),
        });
    }

    tokens
}

/// Trims trailing whitespace from the end of a line, preserving trailing ANSI reset sequences.
fn trim_trailing_ansi_spaces(s: &str) -> String {
    let mut s_trimmed = s.to_string();
    let mut trailing_ansi = String::new();
    loop {
        let before_len = s_trimmed.len();
        let trimmed = s_trimmed.trim_end_matches([' ', '\t']);
        if trimmed.len() != s_trimmed.len() {
            s_trimmed = trimmed.to_string();
        }
        if let Some(pos) = s_trimmed.rfind('\x1b') {
            let candidate = &s_trimmed[pos..];
            if (candidate.ends_with('m')
                || candidate.ends_with('\\')
                || candidate.ends_with('\x07'))
                && visible_width(candidate) == 0
            {
                trailing_ansi.insert_str(0, candidate);
                s_trimmed.truncate(pos);
            }
        }
        if s_trimmed.len() == before_len {
            break;
        }
    }
    s_trimmed.push_str(&trailing_ansi);
    s_trimmed
}

fn update_active_styles(raw: &str, active_sgr: &mut String, active_osc: &mut Option<String>) {
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.next() {
                Some('[') => {
                    let mut sgr = String::from("\x1b[");
                    for next_c in chars.by_ref() {
                        sgr.push(next_c);
                        if ('\x40'..='\x7e').contains(&next_c) {
                            break;
                        }
                    }
                    if sgr.ends_with('m') {
                        if sgr == "\x1b[0m" || sgr == "\x1b[m" {
                            active_sgr.clear();
                        } else {
                            active_sgr.push_str(&sgr);
                        }
                    }
                }
                Some(']') => {
                    let mut osc = String::from("\x1b]");
                    while let Some(next_c) = chars.next() {
                        osc.push(next_c);
                        if next_c == '\x07' {
                            break;
                        }
                        if next_c == '\x1b' && chars.peek() == Some(&'\\') {
                            osc.push(chars.next().unwrap());
                            break;
                        }
                    }
                    if osc.starts_with("\x1b]8;;") {
                        if osc == "\x1b]8;;\x1b\\" || osc == "\x1b]8;;\x07" {
                            *active_osc = None;
                        } else {
                            *active_osc = Some(osc);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Analyzes a line's visible structure (leading spaces, list markers, numbers)
/// to determine the appropriate smart hanging indent for continuation lines.
pub fn compute_smart_indent(line: &str) -> (String, String) {
    let plain = strip_ansi(line);
    let trimmed = plain.trim_start();
    let leading_spaces = plain.len() - trimmed.len();
    let indent_spaces = " ".repeat(leading_spaces);

    // List bullets: "- ", "* ", "+ ", "• "
    if trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || trimmed.starts_with("• ")
    {
        let rest_spaces = " ".repeat(leading_spaces + 2);
        return (indent_spaces, rest_spaces);
    }

    // Task list markers: "- [ ] ", "- [x] ", etc.
    if trimmed.starts_with("- [ ] ")
        || trimmed.starts_with("- [x] ")
        || trimmed.starts_with("- [X] ")
    {
        let rest_spaces = " ".repeat(leading_spaces + 6);
        return (indent_spaces, rest_spaces);
    }

    // Numbered lists: "1. ", "10. ", "1) "
    if let Some(dot_idx) = trimmed.find(". ")
        && dot_idx > 0
        && dot_idx <= 4
        && trimmed[..dot_idx].chars().all(|c| c.is_ascii_digit())
    {
        let rest_spaces = " ".repeat(leading_spaces + dot_idx + 2);
        return (indent_spaces, rest_spaces);
    }
    if let Some(paren_idx) = trimmed.find(") ")
        && paren_idx > 0
        && paren_idx <= 4
        && trimmed[..paren_idx].chars().all(|c| c.is_ascii_digit())
    {
        let rest_spaces = " ".repeat(leading_spaces + paren_idx + 2);
        return (indent_spaces, rest_spaces);
    }

    // Indented code or paragraphs
    if leading_spaces > 0 {
        let rest_spaces = " ".repeat(leading_spaces + 2);
        return (indent_spaces, rest_spaces);
    }

    // Default top-level text continuation indent (align with first line)
    (String::new(), String::new())
}

/// Wraps text cleanly at word boundaries respecting terminal width,
/// with support for leading line indent and continuation indent.
pub fn wrap_ansi(
    text: &str,
    max_width: usize,
    first_indent: &str,
    rest_indent: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    let tokens = tokenize_ansi(text);

    let first_indent_width = visible_width(first_indent);
    let rest_indent_width = visible_width(rest_indent);

    let mut current_line = String::from(first_indent);
    let mut current_line_width = first_indent_width;
    let mut is_first_line = true;
    let mut has_content_on_line = false;

    let mut active_sgr = String::new();
    let mut active_osc: Option<String> = None;

    for token in tokens {
        update_active_styles(&token.raw, &mut active_sgr, &mut active_osc);

        if token.is_whitespace {
            if !has_content_on_line {
                continue;
            }
            current_line.push_str(&token.raw);
            current_line_width += token.visible_width;
        } else {
            let limit = max_width;
            if has_content_on_line && current_line_width + token.visible_width > limit {
                let mut finished = trim_trailing_ansi_spaces(&current_line);
                if active_osc.is_some() {
                    finished.push_str("\x1b]8;;\x1b\\");
                }
                if !active_sgr.is_empty() {
                    finished.push_str("\x1b[0m");
                }
                lines.push(finished);

                current_line = String::from(rest_indent);
                if let Some(ref osc) = active_osc {
                    current_line.push_str(osc);
                }
                if !active_sgr.is_empty() {
                    current_line.push_str(&active_sgr);
                }
                current_line_width = rest_indent_width;
                is_first_line = false;
            }

            current_line.push_str(&token.raw);
            current_line_width += token.visible_width;
            has_content_on_line = true;
        }
    }

    if has_content_on_line || (is_first_line && !current_line.is_empty()) {
        let mut finished = trim_trailing_ansi_spaces(&current_line);
        if active_osc.is_some() {
            finished.push_str("\x1b]8;;\x1b\\");
        }
        if !active_sgr.is_empty() {
            finished.push_str("\x1b[0m");
        }
        lines.push(finished);
    }

    if lines.is_empty() {
        lines.push(String::from(first_indent));
    }

    lines
}

/// Collapses consecutive blank lines into at most one blank line,
/// and eliminates leading and trailing empty lines from terminal output.
pub fn collapse_blank_lines(lines: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    let mut prev_was_empty = true;

    for line in lines {
        let is_empty = line.trim().is_empty();
        if is_empty {
            if !prev_was_empty {
                result.push(String::new());
                prev_was_empty = true;
            }
        } else {
            result.push(line);
            prev_was_empty = false;
        }
    }

    // Remove trailing empty line if any
    while let Some(last) = result.last() {
        if last.trim().is_empty() {
            result.pop();
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_width() {
        assert_eq!(visible_width("hello"), 5);
        assert_eq!(visible_width("\x1b[1;31mhello\x1b[0m"), 5);
        assert_eq!(
            visible_width("\x1b]8;;https://example.com\x1b\\Click\x1b]8;;\x1b\\"),
            5
        );
    }

    #[test]
    fn test_nerd_font_width() {
        // Nerd font icon should have width 1
        assert_eq!(char_width('󰌪'), 1);
        assert_eq!(visible_width("Icon: 󰌪"), 7);
    }

    #[test]
    fn test_smart_indent() {
        let (first, rest) = compute_smart_indent("- Standardize repository metadata");
        assert_eq!(first, "");
        assert_eq!(rest, "  ");

        let (first2, rest2) = compute_smart_indent("   - Indented item");
        assert_eq!(first2, "   ");
        assert_eq!(rest2, "     ");

        let (first3, rest3) = compute_smart_indent("1. First numbered item");
        assert_eq!(first3, "");
        assert_eq!(rest3, "   ");

        let (first4, rest4) = compute_smart_indent("Plain paragraph text");
        assert_eq!(first4, "");
        assert_eq!(rest4, "");
    }

    #[test]
    fn test_wrap_ansi() {
        let text = "The quick brown fox jumps over the lazy dog";
        let wrapped = wrap_ansi(text, 20, "", "");
        assert_eq!(wrapped.len(), 3);
        assert_eq!(wrapped[0], "The quick brown fox");
        assert_eq!(wrapped[1], "jumps over the lazy");
        assert_eq!(wrapped[2], "dog");
    }

    #[test]
    fn test_wrap_ansi_styled_split() {
        let text = "Prefix \x1b[31mred text that wraps across lines\x1b[0m suffix";
        let wrapped = wrap_ansi(text, 15, "", "");
        for line in &wrapped {
            let plain = strip_ansi(line);
            assert!(plain.len() <= 15);
        }
        // First line must close with reset
        assert!(wrapped[0].ends_with("\x1b[0m"));
        // Continuation line must restore active red color
        assert!(wrapped[1].starts_with("\x1b[31m"));
        assert!(wrapped[1].ends_with("\x1b[0m"));
    }
}
