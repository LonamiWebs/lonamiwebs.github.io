pub fn minify(html: &[u8]) -> Vec<u8> {
    let mut minified = Vec::new();

    let mut in_other = Option::<&[u8]>::None;
    let mut in_comment = false;
    let mut in_pre = false;
    let mut in_tag = false;
    let mut maybe_space = false;

    html.iter().copied().enumerate().for_each(|(i, c)| {
        if c == b'\r' {
            return;
        }

        if let Some(other) = in_other {
            if html.get(i.saturating_sub(other.len() - 1)..i + 1) == Some(other) {
                in_other = None;
            }
            minified.push(c);
        } else if matches!(html.get(i..i + 6), Some(b"<style")) {
            in_other = Some(b"</style>");
            minified.push(c)
        } else if matches!(html.get(i..i + 7), Some(b"<script")) {
            in_other = Some(b"</script>");
            minified.push(c);
        } else if in_comment {
            if matches!(html.get(i - 2..i + 1), Some(b"-->")) {
                in_comment = false
            }
        } else if matches!(html.get(i..i + 4), Some(b"<!--")) {
            in_comment = true
        } else if in_pre {
            if matches!(html.get(i - 5..i + 1), Some(b"</pre>")) {
                in_pre = false
            }
            minified.push(c);
        } else if matches!(html.get(i..i + 4), Some(b"<pre")) {
            in_pre = true;
            minified.push(c)
        } else if in_tag {
            if c == b'>' {
                in_tag = false;
                minified.push(c);
            } else if !c.is_ascii_whitespace()
                || !matches!(minified.last(), Some(b' ' | b'\t' | b'\n' | b'<'))
            {
                minified.push(c);
            }
        } else if c == b'<' {
            in_tag = true;
            maybe_space = false;
            minified.push(c)
        } else if c.is_ascii_whitespace() && minified.last() == Some(&b'>') {
            maybe_space = true;
        } else if !c.is_ascii_whitespace() && maybe_space {
            maybe_space = false;
            minified.push(b' ');
            minified.push(c)
        } else if !c.is_ascii_whitespace() || !matches!(minified.last(), Some(b' ' | b'\t' | b'\n'))
        {
            minified.push(c)
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
            br#"< ul  class="left  top  right" id=nav>
    <li><a href="/">my&nbsp;site</a></li>
<!-- ignore this -->
    <li><a href="/blog">some   blog</a></li>
    <li><a href="/golb"> other <b>words</b>  too </a></li>
</ul><pre>
keep
<!-- not this -->
all</pre>  <script>
this

too</script>
"#,
        );
        let expected = r#"<ul class="left top right" id=nav><li><a href="/">my&nbsp;site</a></li><li><a href="/blog">some blog</a></li><li><a href="/golb"> other <b>words</b> too </a></li></ul><pre>
keep

all</pre><script>
this

too</script>"#;

        assert_eq!(String::from_utf8_lossy(&result), expected);
    }
}
