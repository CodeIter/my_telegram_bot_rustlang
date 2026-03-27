pub const MAX_MESSAGE_LENGTH: usize = 4096;

pub fn split_long_message(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < text.len() {
        let mut end = (start + MAX_MESSAGE_LENGTH).min(text.len());
        if end < text.len() {
            if let Some(pos) = text[start..end].rfind("\n\n") {
                end = start + pos + 2;
            } else if let Some(pos) = text[start..end].rfind('\n') {
                end = start + pos + 1;
            } else if let Some(pos) = text[start..end].rfind(". ") {
                end = start + pos + 2;
            } else if let Some(pos) = text[start..end].rfind(' ') {
                end = start + pos + 1;
            }
        }
        let chunk = text[start..end].to_string();
        if !chunk.trim().is_empty() {
            chunks.push(chunk);
        }
        start = end;
    }
    chunks
}
