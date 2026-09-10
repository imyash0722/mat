use pulldown_cmark::{
    BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use crate::syntax::highlight_code;
use crate::table::TableData;
use crate::terminal::{collapse_blank_lines, compute_smart_indent, visible_width, wrap_ansi};

struct ListContext {
    is_ordered: bool,
    current_index: u64,
}

enum BlockQuoteContext {
    Standard,
    Alert(BlockQuoteKind),
}

pub struct MarkdownRenderer {
    term_width: usize,
    output: Vec<String>,
    current_text: String,
    heading_level: Option<HeadingLevel>,
    code_block: Option<(String, String)>, // (lang, code)
    table_data: Option<TableData>,
    in_table_head: bool,
    current_table_row: Vec<String>,
    current_cell: String,
    list_stack: Vec<ListContext>,
    item_marker_pending: Option<String>,
    blockquote_stack: Vec<BlockQuoteContext>,
    blockquote_buffer: Vec<String>,
    active_link_url: Option<String>,
}

impl MarkdownRenderer {
    pub fn new(term_width: usize) -> Self {
        Self {
            term_width: term_width.max(40),
            output: Vec::new(),
            current_text: String::new(),
            heading_level: None,
            code_block: None,
            table_data: None,
            in_table_head: false,
            current_table_row: Vec::new(),
            current_cell: String::new(),
            list_stack: Vec::new(),
            item_marker_pending: None,
            blockquote_stack: Vec::new(),
            blockquote_buffer: Vec::new(),
            active_link_url: None,
        }
    }

    pub fn render(mut self, markdown: &str) -> String {
        let parser = Parser::new_ext(markdown, Options::all());

        for event in parser {
            self.handle_event(event);
        }

        self.flush_text_to_paragraph();

        let cleaned = collapse_blank_lines(self.output);
        cleaned.join("\n")
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            // Headings
            Event::Start(Tag::Heading { level, .. }) => {
                self.flush_text_to_paragraph();
                self.heading_level = Some(level);
                self.current_text.clear();
            }
            Event::End(TagEnd::Heading(level)) => {
                let h_text = std::mem::take(&mut self.current_text);
                self.heading_level = None;
                self.render_heading(level, &h_text);
            }

            // Horizontal Rule / Full-width Divider
            Event::Rule => {
                self.flush_text_to_paragraph();
                let divider = "─".repeat(self.term_width.saturating_sub(2));
                self.output.push(String::new());
                self.output.push(format!("\x1b[1;38;2;165;94;234m{}\x1b[0m", divider));
                self.output.push(String::new());
            }

            // Code Blocks
            Event::Start(Tag::CodeBlock(kind)) => {
                self.flush_text_to_paragraph();
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                self.code_block = Some((lang, String::new()));
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((lang, code)) = self.code_block.take() {
                    self.render_code_block(&lang, &code);
                }
            }

            // BlockQuotes / Alerts
            Event::Start(Tag::BlockQuote(kind_opt)) => {
                self.flush_text_to_paragraph();
                match kind_opt {
                    Some(kind) => self.blockquote_stack.push(BlockQuoteContext::Alert(kind)),
                    None => self.blockquote_stack.push(BlockQuoteContext::Standard),
                }
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                self.flush_text_to_paragraph();
                if let Some(ctx) = self.blockquote_stack.pop() {
                    self.render_blockquote(ctx);
                }
            }

            // Tables
            Event::Start(Tag::Table(alignments)) => {
                self.flush_text_to_paragraph();
                self.table_data = Some(TableData::new(alignments));
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = self.table_data.take() {
                    self.output.push(String::new());
                    for line in table.render(self.term_width) {
                        self.output.push(line);
                    }
                    self.output.push(String::new());
                }
            }
            Event::Start(Tag::TableHead) => {
                self.in_table_head = true;
                self.current_table_row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                self.in_table_head = false;
                if let Some(table) = &mut self.table_data {
                    table.headers = std::mem::take(&mut self.current_table_row);
                }
            }
            Event::Start(Tag::TableRow) => {
                self.current_table_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                if !self.in_table_head {
                    if let Some(table) = &mut self.table_data {
                        table.rows.push(std::mem::take(&mut self.current_table_row));
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                self.current_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                let cell_content = std::mem::take(&mut self.current_cell);
                self.current_table_row.push(cell_content);
            }

            // Lists
            Event::Start(Tag::List(start_num)) => {
                self.flush_text_to_paragraph();
                match start_num {
                    Some(num) => self.list_stack.push(ListContext { is_ordered: true, current_index: num }),
                    None => self.list_stack.push(ListContext { is_ordered: false, current_index: 1 }),
                }
            }
            Event::End(TagEnd::List(_)) => {
                self.flush_text_to_paragraph();
                self.list_stack.pop();
                if self.list_stack.is_empty() {
                    self.output.push(String::new());
                }
            }
            Event::Start(Tag::Item) => {
                self.flush_text_to_paragraph();
                let indent_level = self.list_stack.len().saturating_sub(1);
                let indent = "  ".repeat(indent_level);

                if let Some(ctx) = self.list_stack.last_mut() {
                    if ctx.is_ordered {
                        let marker = format!("{}\x1b[38;2;72;219;251m{}.\x1b[0m ", indent, ctx.current_index);
                        ctx.current_index += 1;
                        self.item_marker_pending = Some(marker);
                    } else {
                        let marker = format!("{}\x1b[38;2;254;202;87m•\x1b[0m ", indent);
                        self.item_marker_pending = Some(marker);
                    }
                }
            }
            Event::End(TagEnd::Item) => {
                self.flush_text_to_paragraph();
                self.item_marker_pending = None;
            }

            // Task List Marker
            Event::TaskListMarker(checked) => {
                let indent_level = self.list_stack.len().saturating_sub(1);
                let indent = "  ".repeat(indent_level);
                let check_str = if checked {
                    format!("{}\x1b[1;32m✔\x1b[0m ", indent)
                } else {
                    format!("{}\x1b[38;2;130;130;130m☐\x1b[0m ", indent)
                };
                self.item_marker_pending = Some(check_str);
            }

            // Links (OSC 8 Clickable)
            Event::Start(Tag::Link { dest_url, .. }) => {
                self.active_link_url = Some(dest_url.to_string());
                let link_open = format!("\x1b]8;;{}\x1b\\\x1b[4;38;2;0;210;211m", dest_url);
                self.append_text(&link_open);
            }
            Event::End(TagEnd::Link) => {
                let link_close = "\x1b[0m\x1b]8;;\x1b\\";
                self.append_text(link_close);
                self.active_link_url = None;
            }

            // Paragraphs
            Event::Start(Tag::Paragraph) => {
                self.flush_text_to_paragraph();
            }
            Event::End(TagEnd::Paragraph) => {
                self.flush_text_to_paragraph();
            }

            // Inline formatting
            Event::Start(Tag::Emphasis) => self.append_text("\x1b[3m"),
            Event::End(TagEnd::Emphasis) => self.append_text("\x1b[23m"),
            Event::Start(Tag::Strong) => self.append_text("\x1b[1m"),
            Event::End(TagEnd::Strong) => self.append_text("\x1b[22m"),
            Event::Start(Tag::Strikethrough) => self.append_text("\x1b[9m"),
            Event::End(TagEnd::Strikethrough) => self.append_text("\x1b[29m"),

            // Inline code
            Event::Code(code) => {
                let mut formatted = format!("\x1b[38;2;255;107;107;48;2;30;30;46m{}\x1b[0m", code);
                // If inside active link, restore link styling after code reset
                if self.active_link_url.is_some() {
                    formatted.push_str("\x1b[4;38;2;0;210;211m");
                }
                self.append_text(&formatted);
            }

            // Text
            Event::Text(text) => {
                if let Some((_, ref mut code_buf)) = self.code_block {
                    code_buf.push_str(&text);
                } else {
                    self.append_text(&text);
                }
            }

            Event::SoftBreak => {
                if self.code_block.is_some() {
                    if let Some((_, ref mut code_buf)) = self.code_block {
                        code_buf.push('\n');
                    }
                } else {
                    self.append_text(" ");
                }
            }
            Event::HardBreak => {
                if self.code_block.is_some() {
                    if let Some((_, ref mut code_buf)) = self.code_block {
                        code_buf.push('\n');
                    }
                } else {
                    self.append_text("\n");
                }
            }

            _ => {}
        }
    }

    fn append_text(&mut self, s: &str) {
        if self.table_data.is_some() {
            self.current_cell.push_str(s);
        } else {
            self.current_text.push_str(s);
        }
    }

    fn flush_text_to_paragraph(&mut self) {
        let raw_text = std::mem::take(&mut self.current_text);
        let text = format_inline_task_markers(&raw_text);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }

        if !self.blockquote_stack.is_empty() {
            self.blockquote_buffer.push(trimmed.to_string());
            return;
        }

        let safe_width = self.term_width.saturating_sub(2);
        if let Some(marker) = self.item_marker_pending.take() {
            let marker_w = visible_width(&marker);
            let rest_indent = " ".repeat(marker_w);
            let wrapped = wrap_ansi(trimmed, safe_width, &marker, &rest_indent);
            for line in wrapped {
                self.output.push(line);
            }
        } else {
            let (first_indent, rest_indent) = compute_smart_indent(trimmed);
            let wrapped = wrap_ansi(trimmed, safe_width, &first_indent, &rest_indent);
            for line in wrapped {
                self.output.push(line);
            }
            self.output.push(String::new());
        }
    }

    fn render_heading(&mut self, level: HeadingLevel, text: &str) {
        self.output.push(String::new());
        let clean_text = text.trim();

        match level {
            HeadingLevel::H1 => {
                // Plum banner with bold bright yellow text
                let banner = format!(" ▊ {} ", clean_text);
                self.output.push(format!("\x1b[1;93;48;2;36;20;50m{}\x1b[0m", banner));
                self.output.push(String::new());
            }
            HeadingLevel::H2 => {
                // Bold bright magenta with accent bar
                self.output.push(format!("\x1b[1;95m▌ {}\x1b[0m", clean_text));
                self.output.push(String::new());
            }
            HeadingLevel::H3 => {
                // Bold bright cyan with accent bar
                self.output.push(format!("\x1b[1;96m▎ {}\x1b[0m", clean_text));
            }
            HeadingLevel::H4 => {
                // Bold emerald green
                self.output.push(format!("\x1b[1;92m{}\x1b[0m", clean_text));
            }
            HeadingLevel::H5 => {
                // Bold warm amber/orange
                self.output.push(format!("\x1b[1;38;2;255;159;67m{}\x1b[0m", clean_text));
            }
            HeadingLevel::H6 => {
                // Bold cornflower blue
                self.output.push(format!("\x1b[1;38;2;84;160;255m{}\x1b[0m", clean_text));
            }
        }
    }

    fn render_code_block(&mut self, lang: &str, code: &str) {
        let highlighted = highlight_code(code, lang);
        let border_color = "\x1b[38;2;90;90;120m";
        let num_color = "\x1b[38;2;110;110;130m";
        let reset = "\x1b[0m";

        let max_width = self.term_width.saturating_sub(2);
        let lang_trimmed = lang.trim();
        let lang_tag = if !lang_trimmed.is_empty() {
            format!(" \x1b[1;38;2;165;94;234m{}\x1b[0m{} ", lang_trimmed, border_color)
        } else {
            String::new()
        };
        let lang_vis = if !lang_trimmed.is_empty() { lang_trimmed.len() + 2 } else { 0 };

        // Header like bat: ─────┬── <lang> ──────────────────────────────
        let top_fill = max_width.saturating_sub(8 + lang_vis);
        self.output.push(String::new());
        self.output.push(format!("{}──────┬──{}{}{}", border_color, lang_tag, "─".repeat(top_fill), reset));

        // Line number gutter width: " 1234 │ " = 8 chars
        let gutter_width = 8;
        let content_width = max_width.saturating_sub(gutter_width).max(20);

        for (idx, line) in highlighted.iter().enumerate() {
            let line_num = idx + 1;
            let line_clean = line.trim_end_matches(['\r', '\n']);

            if line_clean.is_empty() {
                self.output.push(format!("{}{:>5} │{}", num_color, line_num, reset));
                continue;
            }

            let (_first, rest_indent) = compute_smart_indent(line_clean);
            let wrapped = wrap_ansi(line_clean, content_width, "", &rest_indent);
            for (w_idx, w_line) in wrapped.iter().enumerate() {
                if w_idx == 0 {
                    self.output.push(format!("{}{:>5} {}│{} {}", num_color, line_num, border_color, reset, w_line));
                } else {
                    self.output.push(format!("      {}│{} {}", border_color, reset, w_line));
                }
            }
        }

        // Footer like bat: ──────┴─────────────────────────────────────────
        let bot_fill = max_width.saturating_sub(7);
        self.output.push(format!("{}──────┴{}{}", border_color, "─".repeat(bot_fill), reset));
        self.output.push(String::new());
    }

    fn render_blockquote(&mut self, ctx: BlockQuoteContext) {
        let lines = std::mem::take(&mut self.blockquote_buffer);
        if lines.is_empty() {
            return;
        }

        match ctx {
            BlockQuoteContext::Alert(kind) => {
                let (color, icon_title) = match kind {
                    BlockQuoteKind::Note => ("\x1b[1;38;2;47;129;247m", "ℹ NOTE"),
                    BlockQuoteKind::Tip => ("\x1b[1;38;2;46;213;115m", "💡 TIP"),
                    BlockQuoteKind::Important => ("\x1b[1;38;2;165;94;234m", "🟣 IMPORTANT"),
                    BlockQuoteKind::Warning => ("\x1b[1;38;2;255;165;2m", "⚠️ WARNING"),
                    BlockQuoteKind::Caution => ("\x1b[1;38;2;255;71;87m", "🛑 CAUTION"),
                };
                let reset = "\x1b[0m";

                let box_width = (self.term_width.saturating_sub(2)).min(88);
                let title_tag = format!(" {} ", icon_title);
                let title_vis = visible_width(&title_tag);
                let top_fill = box_width.saturating_sub(3 + title_vis);

                self.output.push(String::new());
                self.output.push(format!("{}╭─{}{}{}─╮{}", color, title_tag, color, "─".repeat(top_fill), reset));

                let content_width = box_width.saturating_sub(6);

                for block in lines {
                    let wrapped = wrap_ansi(&block, content_width, "", "");
                    for wline in wrapped {
                        let wline_vis = visible_width(&wline);
                        let pad_right = content_width.saturating_sub(wline_vis);
                        self.output.push(format!("{}│{}  {}{}{}  {}│{}", color, reset, wline, " ".repeat(pad_right), color, color, reset));
                    }
                }

                self.output.push(format!("{}╰{}╯{}", color, "─".repeat(box_width.saturating_sub(2)), reset));
                self.output.push(String::new());
            }
            BlockQuoteContext::Standard => {
                self.output.push(String::new());
                for block in lines {
                    let wrapped = wrap_ansi(&block, self.term_width.saturating_sub(4), "\x1b[38;2;253;203;110m│ \x1b[0m\x1b[3m", "\x1b[38;2;253;203;110m│ \x1b[0m\x1b[3m");
                    for wline in wrapped {
                        self.output.push(format!("{}\x1b[0m", wline));
                    }
                }
                self.output.push(String::new());
            }
        }
    }
}

fn format_inline_task_markers(s: &str) -> String {
    s.replace("[ ] ", "\x1b[38;2;130;130;130m☐\x1b[0m ")
        .replace("[x] ", "\x1b[1;32m✔\x1b[0m ")
        .replace("[X] ", "\x1b[1;32m✔\x1b[0m ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_alert_and_tasks() {
        let md = "# Heading 1\n\n> [!NOTE]\n> This is a note.\n\n- **Status:** [ ] Pending\n- [x] Done";
        let renderer = MarkdownRenderer::new(80);
        let output = renderer.render(md);
        assert!(output.contains("NOTE"));
        assert!(output.contains("☐"));
        assert!(output.contains("✔"));
    }
}
