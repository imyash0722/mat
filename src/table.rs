use pulldown_cmark::Alignment;
use crate::terminal::visible_width;

#[derive(Debug)]
pub struct TableData {
    pub alignments: Vec<Alignment>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl TableData {
    pub fn new(alignments: Vec<Alignment>) -> Self {
        Self {
            alignments,
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub fn render(&self, max_term_width: usize) -> Vec<String> {
        let num_cols = self.alignments.len()
            .max(self.headers.len())
            .max(self.rows.iter().map(|r| r.len()).max().unwrap_or(0));

        if num_cols == 0 {
            return Vec::new();
        }

        // Calculate max visible width per column
        let mut col_widths = vec![3usize; num_cols];

        for (i, h) in self.headers.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(visible_width(h));
            }
        }

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < num_cols {
                    col_widths[i] = col_widths[i].max(visible_width(cell));
                }
            }
        }

        // Adjust column widths if total width exceeds max_term_width
        // Available width for columns = max_term_width - (3 * num_cols + 1)
        let overhead = 3 * num_cols + 1;
        if overhead < max_term_width {
            let available_content = max_term_width - overhead;
            let total_content: usize = col_widths.iter().sum();
            if total_content > available_content && available_content >= num_cols * 3 {
                // Scale columns proportionally
                for w in &mut col_widths {
                    let scaled = (*w * available_content) / total_content;
                    *w = scaled.max(3);
                }
            }
        }

        let border_color = "\x1b[38;2;95;39;205m";
        let reset = "\x1b[0m";
        let header_bg = "\x1b[1;37;48;2;52;31;151m";

        let mut output = Vec::new();

        // Top border: ┌───┬───┐
        let mut top = String::from(border_color);
        top.push('┌');
        for (i, &w) in col_widths.iter().enumerate() {
            top.push_str(&"─".repeat(w + 2));
            if i + 1 < num_cols {
                top.push('┬');
            } else {
                top.push('┐');
            }
        }
        top.push_str(reset);
        output.push(top);

        // Header row
        if !self.headers.is_empty() {
            let mut h_row = String::new();
            h_row.push_str(border_color);
            h_row.push('│');
            h_row.push_str(reset);

            for (i, &w) in col_widths.iter().enumerate() {
                let text = self.headers.get(i).map(|s| s.as_str()).unwrap_or("");
                let align = self.alignments.get(i).copied().unwrap_or(Alignment::None);
                let formatted_cell = format_cell(text, w, align, header_bg);
                h_row.push_str(&formatted_cell);
                h_row.push_str(border_color);
                h_row.push('│');
                h_row.push_str(reset);
            }
            output.push(h_row);

            // Divider: ├───┼───┤
            let mut mid = String::from(border_color);
            mid.push('├');
            for (i, &w) in col_widths.iter().enumerate() {
                mid.push_str(&"─".repeat(w + 2));
                if i + 1 < num_cols {
                    mid.push('┼');
                } else {
                    mid.push('┤');
                }
            }
            mid.push_str(reset);
            output.push(mid);
        }

        // Data rows
        for row in &self.rows {
            let mut r_line = String::new();
            r_line.push_str(border_color);
            r_line.push('│');
            r_line.push_str(reset);

            for (i, &w) in col_widths.iter().enumerate() {
                let text = row.get(i).map(|s| s.as_str()).unwrap_or("");
                let align = self.alignments.get(i).copied().unwrap_or(Alignment::None);
                let formatted_cell = format_cell(text, w, align, "");
                r_line.push_str(&formatted_cell);
                r_line.push_str(border_color);
                r_line.push('│');
                r_line.push_str(reset);
            }
            output.push(r_line);
        }

        // Bottom border: └───┴───┘
        let mut bot = String::from(border_color);
        bot.push('└');
        for (i, &w) in col_widths.iter().enumerate() {
            bot.push_str(&"─".repeat(w + 2));
            if i + 1 < num_cols {
                bot.push('┴');
            } else {
                bot.push('┘');
            }
        }
        bot.push_str(reset);
        output.push(bot);

        output
    }
}

fn format_cell(text: &str, width: usize, align: Alignment, bg_code: &str) -> String {
    let text_w = visible_width(text);
    let reset = "\x1b[0m";

    if text_w >= width {
        // If exact or slightly truncated
        format!(" {}{}{} ", bg_code, text, reset)
    } else {
        let diff = width - text_w;
        match align {
            Alignment::Right => {
                let pad_left = " ".repeat(diff);
                format!(" {}{}{}{} ", bg_code, pad_left, text, reset)
            }
            Alignment::Center => {
                let pad_left = " ".repeat(diff / 2);
                let pad_right = " ".repeat(diff - diff / 2);
                format!(" {}{}{}{}{} ", bg_code, pad_left, text, pad_right, reset)
            }
            Alignment::None | Alignment::Left => {
                let pad_right = " ".repeat(diff);
                format!(" {}{}{}{} ", bg_code, text, pad_right, reset)
            }
        }
    }
}
