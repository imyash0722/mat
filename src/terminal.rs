use unicode_width::UnicodeWidthChar;

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
            width += UnicodeWidthChar::width(c).unwrap_or(0);
        }
    }
    width
}

/// Strips ANSI CSI and OSC escape codes from a string.
#[allow(dead_code)]
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
        current_width += UnicodeWidthChar::width(c).unwrap_or(0);
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

/// Trims trailing spaces from the end of an ANSI string while preserving any trailing ANSI sequences.
fn trim_trailing_ansi_spaces(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut i = chars.len();

    while i > 0 {
        let prev = chars[i - 1];
        if prev == ' ' || prev == '\t' {
            i -= 1;
        } else {
            break;
        }
    }

    chars.into_iter().take(i).collect()
}

/// Wraps text cleanly at word boundaries respecting terminal width,
/// with support for leading line indent and continuation indent.
pub fn wrap_ansi(text: &str, max_width: usize, first_indent: &str, rest_indent: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let tokens = tokenize_ansi(text);

    let first_indent_width = visible_width(first_indent);
    let rest_indent_width = visible_width(rest_indent);

    let mut current_line = String::from(first_indent);
    let mut current_line_width = first_indent_width;
    let mut is_first_line = true;
    let mut has_content_on_line = false;

    for token in tokens {
        if token.is_whitespace {
            if !has_content_on_line {
                continue;
            }
            current_line.push_str(&token.raw);
            current_line_width += token.visible_width;
        } else {
            let limit = max_width;
            if has_content_on_line && current_line_width + token.visible_width > limit {
                lines.push(trim_trailing_ansi_spaces(&current_line));
                current_line = String::from(rest_indent);
                current_line_width = rest_indent_width;
                is_first_line = false;
            }

            current_line.push_str(&token.raw);
            current_line_width += token.visible_width;
            has_content_on_line = true;
        }
    }

    if has_content_on_line || is_first_line && !current_line.is_empty() {
        lines.push(trim_trailing_ansi_spaces(&current_line));
    }

    if lines.is_empty() {
        lines.push(String::from(first_indent));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_width() {
        assert_eq!(visible_width("hello"), 5);
        assert_eq!(visible_width("\x1b[1;31mhello\x1b[0m"), 5);
        assert_eq!(visible_width("\x1b]8;;https://example.com\x1b\\Click\x1b]8;;\x1b\\"), 5);
    }

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[1;31mhello\x1b[0m world"), "hello world");
        assert_eq!(strip_ansi("\x1b]8;;https://example.com\x1b\\link\x1b]8;;\x1b\\"), "link");
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
    fn test_wrap_with_ansi_links() {
        let text = "Check out \x1b]8;;https://rust-lang.org\x1b\\Rust Language Website\x1b]8;;\x1b\\ for details.";
        let wrapped = wrap_ansi(text, 35, "", "");
        assert_eq!(wrapped.len(), 2);
        assert!(wrapped[0].contains("Rust Language"));
    }
}
