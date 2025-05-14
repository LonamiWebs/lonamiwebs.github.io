pub fn minify(css: &[u8]) -> Vec<u8> {
    let mut minified = Vec::new();

    let mut in_comment = false;

    css.iter().copied().enumerate().for_each(|(i, c)| {
        if c == b'\r' {
            return;
        }

        if in_comment {
            if matches!(css.get(i - 1..i + 1), Some(b"*/")) {
                in_comment = false;
            }
        } else if matches!(css.get(i..i + 2), Some(b"/*")) {
            in_comment = true;
        } else if !c.is_ascii_whitespace()
            || !matches!(
                minified.last(),
                Some(b' ' | b'\t' | b'\n' | b',' | b';' | b':' | b'{' | b'}' | b'(' | b')'),
            )
        {
            minified.push(c);
        }
    });

    minified
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify() {
        let result = minify(
            b"@media (prefers-color-scheme: dark) {
    .foo:target {/*comment*/
        background-color: rgba(255, 127, 0, 0.1);
        transition: color 300ms, border-bottom 300ms;
    }
}
",
        );
        let expected = "@media (prefers-color-scheme:dark){.foo:target {background-color:rgba(255,127,0,0.1);transition:color 300ms,border-bottom 300ms;}}";

        assert_eq!(String::from_utf8_lossy(&result), expected);
    }
}
