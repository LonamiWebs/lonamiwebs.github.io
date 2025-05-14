mod generator;
mod minifier;

pub use generator::generate;
pub use minifier::minify;

pub fn escape(text: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(text.len());
    text.iter().for_each(|&c| match c {
        b'&' => result.extend_from_slice(b"&amp;"),
        b'<' => result.extend_from_slice(b"&lt;"),
        b'>' => result.extend_from_slice(b"&gt;"),
        c => result.push(c),
    });
    result
}

pub fn escape_attribute(text: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(text.len());
    text.iter().for_each(|&c| match c {
        b'&' => result.extend_from_slice(b"&amp;"),
        b'"' => result.extend_from_slice(b"&quot;"),
        c => result.push(c),
    });
    result
}
