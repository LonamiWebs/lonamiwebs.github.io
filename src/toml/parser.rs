use std::collections::HashMap;

pub type ParseResult<'t> = HashMap<&'t [u8], Vec<&'t [u8]>>;

fn strip<'s>(mut string: &'s [u8], chars: &[u8]) -> &'s [u8] {
    let matches = |c: Option<&u8>| c.is_some_and(|c| chars.contains(c));
    while matches(string.first()) {
        string = &string[1..];
    }
    while matches(string.last()) {
        string = &string[..string.len() - 1];
    }
    string
}

pub fn parse(text: &[u8]) -> ParseResult<'_> {
    let mut result = ParseResult::new();

    for mut line in text.split(|c| matches!(c, b'\r' | b'\n')) {
        line = line.trim_ascii();
        if line.is_empty() || line.starts_with(b"[") {
            continue;
        }
        let equals_index = match line.iter().position(|&c| c == b'=') {
            Some(i) => i,
            None => continue,
        };
        let name = strip(&line[..equals_index], b"\" ");
        let value = strip(&line[equals_index + 1..], b"\" ");
        result.insert(
            name,
            if value.starts_with(b"[") {
                value
                    .split(|&c| c == b',')
                    .map(|v| strip(v, b"\" []"))
                    .collect()
            } else {
                vec![value]
            },
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse(
            br#"title = "Some, title"
date = 1234-56-78
[taxonomies]
category = ["cat"]
tags = ["t", "a", "g"]
"#,
        );
        let mut expected = ParseResult::new();
        expected.insert(b"title", vec![b"Some, title"]);
        expected.insert(b"date", vec![b"1234-56-78"]);
        expected.insert(b"category", vec![b"cat"]);
        expected.insert(b"tags", vec![b"t", b"a", b"g"]);

        assert_eq!(result, expected);
    }
}
