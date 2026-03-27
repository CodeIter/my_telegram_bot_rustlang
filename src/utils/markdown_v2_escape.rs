/// Escapes **any** text for safe use with Telegram `MarkdownV2`.
pub fn markdown_v2_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '[' => escaped.push_str("\\["),
            ']' => escaped.push_str("\\]"),
            '(' => escaped.push_str("\\("),
            ')' => escaped.push_str("\\)"),
            '~' => escaped.push_str("\\~"),
            '>' => escaped.push_str("\\>"),
            '#' => escaped.push_str("\\#"),
            '+' => escaped.push_str("\\+"),
            '-' => escaped.push_str("\\-"),
            '=' => escaped.push_str("\\="),
            '|' => escaped.push_str("\\|"),
            '{' => escaped.push_str("\\{"),
            '}' => escaped.push_str("\\}"),
            '.' => escaped.push_str("\\."),
            '!' => escaped.push_str("\\!"),
            '`' => escaped.push_str("\\`"),
            '*' => escaped.push_str("\\*"),
            '_' => escaped.push_str("\\_"),
            _ => escaped.push(c),
        }
    }
    escaped
}
