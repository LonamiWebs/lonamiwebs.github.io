pub fn escape_text(text: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(text.len() + text.len() / 5);
    text.iter().for_each(|&c| match c {
        b'&' => result.extend_from_slice(b"&amp;"),
        b'<' => result.extend_from_slice(b"&lt;"),
        c => result.push(c),
    });
    result
}
