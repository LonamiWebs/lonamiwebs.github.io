mod generator;
mod minifier;

use std::array;
use std::iter;

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

pub fn escape_attribute(text: impl Iterator<Item = u8>) -> Vec<u8> {
    enum Item {
        Single(iter::Once<u8>),
        Five(array::IntoIter<u8, 5>),
        Six(array::IntoIter<u8, 6>),
    }

    impl Iterator for Item {
        type Item = u8;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Item::Single(i) => i.next(),
                Item::Five(i) => i.next(),
                Item::Six(i) => i.next(),
            }
        }
    }

    text.flat_map(|c| match c {
        b'&' => Item::Five((*b"&amp;").into_iter()),
        b'"' => Item::Six((*b"&quot;").into_iter()),
        c => Item::Single(iter::once(c)),
    })
    .collect::<Vec<_>>()
}

pub fn text_content(html: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.iter().copied() {
        if in_tag {
            if c == b'>' {
                in_tag = false;
            }
        } else if c == b'<' {
            in_tag = true;
        } else {
            result.push(c);
        }
    }
    result
}
