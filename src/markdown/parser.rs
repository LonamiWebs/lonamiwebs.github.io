use crate::toml;
use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Token<'t> {
    Text(&'t [u8]),
    Raw(&'t [u8]),
    Meta(toml::ParseResult<'t>),
    Separator(u8),
    BeginItem { ordered: bool },
    EndItem,
    Emphasis(u8),
    BeginReference { bang: bool },
    EndReference { uri: &'t [u8], alt: &'t [u8] },
    Heading { level: u8, text: &'t [u8] },
    Fence { lang: &'t [u8], text: &'t [u8] },
    Code(&'t [u8]),
    Quote,
    TableRow(&'t [u8]),
    Break,
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(x) => f
                .debug_tuple("Text")
                .field(&String::from_utf8_lossy(x))
                .finish(),
            Self::Raw(x) => f
                .debug_tuple("Raw")
                .field(&String::from_utf8_lossy(x))
                .finish(),
            Self::Meta(x) => f.debug_tuple("Meta").field(x).finish(),
            Self::Separator(x) => f.debug_tuple("Separator").field(&(*x as char)).finish(),
            Self::BeginItem { ordered } => f
                .debug_struct("BeginItem")
                .field("ordered", ordered)
                .finish(),
            Self::EndItem => f.write_str("EndItem"),
            Self::Emphasis(x) => f.debug_tuple("Emphasis").field(x).finish(),
            Self::BeginReference { bang } => f
                .debug_struct("BeginReference")
                .field("bang", bang)
                .finish(),
            Self::EndReference { uri, alt } => f
                .debug_struct("EndReference")
                .field("uri", &String::from_utf8_lossy(uri))
                .field("alt", &String::from_utf8_lossy(alt))
                .finish(),
            Self::Heading { level, text } => f
                .debug_struct("Heading")
                .field("level", level)
                .field("text", &String::from_utf8_lossy(text))
                .finish(),
            Self::Fence { lang, text } => f
                .debug_struct("Fence")
                .field("lang", &String::from_utf8_lossy(lang))
                .field("text", &String::from_utf8_lossy(text))
                .finish(),
            Self::Code(x) => f
                .debug_tuple("Code")
                .field(&String::from_utf8_lossy(x))
                .finish(),
            Self::Quote => f.write_str("Quote"),
            Self::TableRow(x) => f
                .debug_tuple("TableRow")
                .field(&String::from_utf8_lossy(x))
                .finish(),
            Self::Break => f.write_str("Break"),
        }
    }
}

#[derive(PartialEq)]
pub struct ParseResult<'t> {
    pub text: &'t [u8],
    pub tokens: Vec<Token<'t>>,
}

impl fmt::Debug for ParseResult<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseResult")
            .field("tokens", &self.tokens)
            .finish_non_exhaustive()
    }
}

struct TextState<'t> {
    text: &'t [u8],
    text_start: usize,
}

impl<'t> TextState<'t> {
    fn end_text(&mut self, i: usize) -> Option<Token<'t>> {
        if i > self.text_start {
            let text = &self.text[self.text_start..i];
            return Some(Token::Text(text));
        }
        None
    }

    fn begin_text(&mut self, i: usize) {
        self.text_start = i;
    }
}

#[inline]
fn char_start(text: &[u8], needle: u8, search_start: usize) -> usize {
    text[search_start..]
        .iter()
        .position(|&t| t == needle)
        .map(|j| search_start + j)
        .unwrap_or(text.len())
}

#[inline]
fn substring_end(text: &[u8], needle: &[u8], search_start: usize) -> usize {
    text[search_start..]
        .windows(needle.len())
        .position(|t| t == needle)
        .map(|j| search_start + j + needle.len())
        .unwrap_or(text.len())
}

pub fn parse(text: &[u8]) -> ParseResult {
    let mut tokens = Vec::<Token>::new();

    let mut iter = text.iter().enumerate();

    let mut text_state = TextState {
        text,
        text_start: 0,
    };

    let mut in_list = 0;
    let mut in_reference = false;

    while let Some((i, &c)) = iter.next() {
        let start_of_line = i == 0 || text[i - 1] == b'\n';

        macro_rules! flush_text {
            () => {
                if let Some(token) = text_state.end_text(i) {
                    tokens.push(token)
                }
            };
        }

        macro_rules! maybe_close_list {
            () => {
                if start_of_line && in_list > 0 {
                    flush_text!();
                    while in_list > 0 {
                        tokens.push(Token::EndItem);
                        in_list -= 1;
                    }
                    text_state.begin_text(i);
                }
            };
        }

        macro_rules! continue_to {
            ($j:expr) => {
                // nth is base-0, and next iteration calls next one more time, so subtract an extra 2.
                if let Some(j) = $j.checked_sub(i + 2) {
                    iter.nth(j);
                }
                text_state.begin_text(text.len().min($j));
                continue;
            };
        }

        match c {
            // Fast-path
            b'A'..=b'Z' | b'a'..=b'z' => {
                maybe_close_list!();
            }

            // Escape sequences '\X'
            b'\\' => {
                maybe_close_list!();
                flush_text!();
                // Unlike the rest, no continue_to!() because we want to keep the character but not process it.
                iter.next();
                text_state.begin_text(i + 1);
            }

            // HTML tags that do not contain markdown to be parsed
            b'<' if text.get(i + 1..).is_some_and(|t| t.starts_with(b"pre"))
                || text.get(i + 1..).is_some_and(|t| t.starts_with(b"script"))
                || text.get(i + 1..).is_some_and(|t| t.starts_with(b"style")) =>
            {
                maybe_close_list!();
                flush_text!();
                let closing_tag = match text[i + 2] {
                    b'r' => b"</pre>".as_ref(),
                    b'c' => b"</script>".as_ref(),
                    b't' => b"</style>".as_ref(),
                    _ => unreachable!(),
                };
                let j = substring_end(text, closing_tag, i + closing_tag.len() - 1);

                tokens.push(Token::Raw(&text[i..j]));
                continue_to!(j);
            }

            // HTML tags that may be separated from upcoming markdown
            b'<' if matches!(text.get(i + 1), Some(b'/' | b'A'..=b'Z' | b'a'..=b'z')) => {
                maybe_close_list!();
                flush_text!();
                let separator = b"\n\n";
                let j = substring_end(text, separator, i + 3); // 3 = <X>

                tokens.push(Token::Raw(&text[i..j]));
                continue_to!(j);
            }

            // Metadata
            d @ (b'-' | b'+')
                if i == 0
                    && text[i + 1..char_start(text, b'\n', i + 1)]
                        .iter()
                        .all(|&e| e == d) =>
            {
                let j = substring_end(text, b"\n", i + 3); // 3 = minimum starting length
                let separator = &text[i..j - 1];

                let k = substring_end(text, separator, j);
                if text[k - separator.len() - 1] == b'\n'
                    && matches!(text.get(k), None | Some(b'\n'))
                {
                    tokens.push(Token::Meta(toml::parse(&text[j..k - separator.len() - 1]))); // eat leading newline
                    continue_to!(k + 1); // try to eat trailing newline
                }
            }

            // Decorative separator
            d @ (b'*' | b'=' | b'_' | b'-')
                if start_of_line
                    && text[i..char_start(text, b'\n', i + 1)]
                        .iter()
                        .all(|&e| e == d) =>
            {
                maybe_close_list!();
                flush_text!();
                let j = substring_end(text, b"\n", i + 1);
                tokens.push(Token::Separator(d));
                continue_to!(if j == text.len() { j } else { j - 1 });
            }

            // Unordered-list item
            b'*' | b'-' if start_of_line && text.get(i + 1).is_some_and(|&d| d == b' ') => {
                maybe_close_list!();
                flush_text!();
                tokens.push(Token::BeginItem { ordered: false });
                in_list += 1;
                continue_to!(i + 2);
            }

            // Ordered-list item
            b'0'..=b'9' if start_of_line && text.get(i + 1..i + 3).is_some_and(|d| d == b". ") => {
                maybe_close_list!();
                flush_text!();
                tokens.push(Token::BeginItem { ordered: true });
                in_list += 1;
                continue_to!(i + 3);
            }

            // Emphasis
            b'*' if (i == 0 || text[i - 1] != b'*')
                && text[i + 1..].iter().take_while(|&&d| d == b'*').count() <= 2 =>
            {
                maybe_close_list!();
                flush_text!();
                let strength = 1 + text[i + 1..].iter().take_while(|&&d| d == b'*').count();
                tokens.push(Token::Emphasis(strength as u8));
                continue_to!(i + strength);
            }

            // Reference
            d @ (b'!' | b'[') if d == b'[' || matches!(text.get(i + 1), Some(b'[')) => {
                maybe_close_list!();
                let offset = if d == b'[' { 1 } else { 2 };
                let j = char_start(text, b'\n', i + offset);
                let k = char_start(&text[..j], b']', i + offset);

                if k != j {
                    flush_text!();
                    in_reference = true;
                    tokens.push(Token::BeginReference { bang: d == b'!' });
                    continue_to!(i + offset);
                }
            }

            b']' if in_reference => {
                maybe_close_list!();
                flush_text!();
                in_reference = false;

                if matches!(text.get(i + 1), Some(b'(')) {
                    let j = char_start(text, b')', i + 2);
                    let k = char_start(&text[..j], b' ', i + 2);
                    tokens.push(Token::EndReference {
                        uri: &text[i + 2..k],
                        alt: &text[j.min(k + 1)..j],
                    });
                    continue_to!(j + 1);
                } else {
                    tokens.push(Token::EndReference { uri: b"", alt: b"" });
                    continue_to!(i + 1);
                }
            }

            // Heading
            b'#' if start_of_line
                && text[i + 1..].iter().take_while(|&&d| d == b'#').count() <= 5 =>
            {
                maybe_close_list!();
                flush_text!();
                let level = 1 + text[i + 1..].iter().take_while(|&&d| d == b'#').count();

                let j = char_start(text, b'\n', i + level);

                tokens.push(Token::Heading {
                    level: level as u8,
                    text: text[i + level..j].trim_ascii(),
                });
                continue_to!(j + 1);
            }

            // Fenced block
            b'`' if start_of_line
                && text[i + 1..].iter().take_while(|&&d| d == b'`').count() >= 2 =>
            {
                maybe_close_list!();
                flush_text!();
                let j = i + 1 + text[i + 1..].iter().take_while(|&&d| d == b'`').count();

                let separator = &text[i..j];
                let k = char_start(text, b'\n', j);
                let m = substring_end(text, separator, text.len().min(k + 1));

                tokens.push(Token::Fence {
                    lang: &text[j..k],
                    text: &text
                        [text.len().min(k + 1)..text.len().min(k + 1).max(m - separator.len())],
                });
                continue_to!(m + 1);
            }

            // Inline code
            b'`' => {
                maybe_close_list!();
                flush_text!();
                let j = char_start(text, b'`', i + 1);
                tokens.push(Token::Code(&text[i + 1..j]));
                continue_to!(j + 1);
            }

            // Blockquotes
            b'>' if start_of_line => {
                maybe_close_list!();
                flush_text!();
                tokens.push(Token::Quote);
                continue_to!(i + 1);
            }

            // Table rows
            b'|' if start_of_line => {
                maybe_close_list!();
                flush_text!();
                let j = char_start(text, b'\n', i + 1);
                tokens.push(Token::TableRow(&text[i..j]));
                continue_to!(j + 1);
            }

            // Paragraph break
            b'\n' if text.get(i + 1).is_some_and(|&d| d == b'\n') => {
                flush_text!();
                tokens.push(Token::Break);
                continue_to!(i + 2);
            }

            _ => continue,
        }
    }

    if let Some(token) = text_state.end_text(text.len()) {
        tokens.push(token)
    }

    while in_list > 0 {
        tokens.push(Token::EndItem);
        in_list -= 1;
    }

    ParseResult { text, tokens }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escaping() {
        for c in "\\[<`*+=_-".chars() {
            let c_byte = c.to_string();
            let c_byte = c_byte.as_bytes();

            let text = format!("\\{c}\\text\\n\\{c}\\");
            let text = text.as_bytes();
            assert_eq!(
                parse(text),
                ParseResult {
                    text,
                    tokens: vec![
                        Token::Text(c_byte),
                        Token::Text(b"text"),
                        Token::Text(b"n"),
                        Token::Text(c_byte),
                    ],
                }
            );
        }
    }

    #[test]
    fn test_raw() {
        let text = b"text <pre>keep\nas-is\n\n  </pre>done<style></style>end<script unclosed";
        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![
                    Token::Text(b"text "),
                    Token::Raw(b"<pre>keep\nas-is\n\n  </pre>"),
                    Token::Text(b"done"),
                    Token::Raw(b"<style></style>"),
                    Token::Text(b"end"),
                    Token::Raw(b"<script unclosed"),
                ],
            }
        );

        let text = b"<h1>h1</h1>\n<p>long\nparagraph</p>\n\n<h2 id=\"about\">h2</h2>\n<p>another paragraph</p>";
        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![
                    Token::Raw(b"<h1>h1</h1>\n<p>long\nparagraph</p>\n\n"),
                    Token::Raw(b"<h2 id=\"about\">h2</h2>\n<p>another paragraph</p>")
                ],
            }
        );
    }

    #[test]
    fn test_html() {
        let text = b"<p>p *tag*</p><details>\n\ndetails *tag*\n\n</details>\n\ntext";
        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![
                    Token::Raw(b"<p>p *tag*</p><details>\n\n"),
                    Token::Text(b"details "),
                    Token::Emphasis(1),
                    Token::Text(b"tag"),
                    Token::Emphasis(1),
                    Token::Break,
                    Token::Raw(b"</details>\n\n"),
                    Token::Text(b"text"),
                ],
            }
        );
    }

    #[test]
    fn test_metadata_ok() {
        for separator in ["---", "---------", "+++", "+++++++++"] {
            let text = format!("{separator}\nmeta\n{separator}");
            let text = text.as_bytes();
            assert_eq!(
                parse(text),
                ParseResult {
                    text,
                    tokens: vec![Token::Meta(toml::parse(b""))]
                }
            );

            let text = format!("{separator}\nmeta\n{separator}\ntext");
            let text = text.as_bytes();
            assert_eq!(
                parse(text),
                ParseResult {
                    text,
                    tokens: vec![Token::Meta(toml::parse(b"")), Token::Text(b"text")]
                }
            );

            let text = format!("{separator}\nmeta\n{separator}text");
            let text = text.as_bytes();
            assert_eq!(
                parse(text),
                ParseResult {
                    text,
                    tokens: vec![Token::Text(text)]
                }
            );
        }

        let text = format!("text\n+++\nmeta\n+++\ntext");
        let text = text.as_bytes();
        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![Token::Text(text)]
            }
        );
    }

    #[test]
    fn test_separator() {
        for &c in b"*=_-" {
            for length in [1, 3, 10] {
                let separator = vec![c; length];
                let separator = String::from_utf8_lossy(&separator);
                let text = format!("start\n{separator}");
                let text = text.as_bytes();
                assert_eq!(
                    parse(text),
                    ParseResult {
                        text,
                        tokens: vec![Token::Text(b"start\n"), Token::Separator(c)]
                    }
                );

                let text = format!("start\n{separator}\ntext");
                let text = text.as_bytes();
                assert_eq!(
                    parse(text),
                    ParseResult {
                        text,
                        tokens: vec![
                            Token::Text(b"start\n"),
                            Token::Separator(c),
                            Token::Text(b"\ntext")
                        ]
                    }
                );
            }
        }
    }

    #[test]
    fn test_item() {
        for item in ["*", "-", "0.", "1."] {
            let text = format!("{item} text");
            let text = text.as_bytes();
            assert_eq!(
                parse(text),
                ParseResult {
                    text,
                    tokens: vec![
                        Token::BeginItem {
                            ordered: item.as_bytes()[0].is_ascii_digit()
                        },
                        Token::Text(b"text"),
                        Token::EndItem,
                    ]
                }
            );
        }

        let text = b"* star\n0. zero\n- dash\n1. one";
        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![
                    Token::BeginItem { ordered: false },
                    Token::Text(b"star\n"),
                    Token::EndItem,
                    Token::BeginItem { ordered: true },
                    Token::Text(b"zero\n"),
                    Token::EndItem,
                    Token::BeginItem { ordered: false },
                    Token::Text(b"dash\n"),
                    Token::EndItem,
                    Token::BeginItem { ordered: true },
                    Token::Text(b"one"),
                    Token::EndItem,
                ]
            }
        );

        let text = b"* a\n* [b](u)\n* c\n\n  c\n* d";
        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![
                    Token::BeginItem { ordered: false },
                    Token::Text(b"a\n"),
                    Token::EndItem,
                    Token::BeginItem { ordered: false },
                    Token::BeginReference { bang: false },
                    Token::Text(b"b"),
                    Token::EndReference {
                        uri: b"u",
                        alt: b""
                    },
                    Token::Text(b"\n"),
                    Token::EndItem,
                    Token::BeginItem { ordered: false },
                    Token::Text(b"c"),
                    Token::Break,
                    Token::Text(b"  c\n"),
                    Token::EndItem,
                    Token::BeginItem { ordered: false },
                    Token::Text(b"d"),
                    Token::EndItem,
                ]
            }
        );
    }

    #[test]
    fn test_emphasis() {
        let text = b"*1* **2** ***3***";
        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![
                    Token::Emphasis(1),
                    Token::Text(b"1"),
                    Token::Emphasis(1),
                    Token::Text(b" "),
                    Token::Emphasis(2),
                    Token::Text(b"2"),
                    Token::Emphasis(2),
                    Token::Text(b" "),
                    Token::Emphasis(3),
                    Token::Text(b"3"),
                    Token::Emphasis(3),
                ]
            }
        );
    }

    #[test]
    fn test_reference() {
        fn open<'t>(bang: bool) -> Token<'t> {
            Token::BeginReference { bang }
        }
        fn close<'t>(uri: &'t [u8], alt: &'t [u8]) -> Token<'t> {
            Token::EndReference { uri, alt }
        }

        let text = b"[0] [t](1) ![e](2) [x](3 \"a\") ![t](4 \"b\")\n\n[0]: u";
        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![
                    open(false),
                    Token::Text(b"0"),
                    close(b"", b""),
                    Token::Text(b" "),
                    open(false),
                    Token::Text(b"t"),
                    close(b"1", b""),
                    Token::Text(b" "),
                    open(true),
                    Token::Text(b"e"),
                    close(b"2", b""),
                    Token::Text(b" "),
                    open(false),
                    Token::Text(b"x"),
                    close(b"3", b"\"a\""),
                    Token::Text(b" "),
                    open(true),
                    Token::Text(b"t"),
                    close(b"4", b"\"b\""),
                    Token::Break,
                    open(false),
                    Token::Text(b"0"),
                    close(b"", b""),
                    Token::Text(b": u"),
                ]
            }
        );
    }

    #[test]
    fn test_heading() {
        for length in 1..7 {
            let text = format!("{:#^length$} heading\ntext", "");
            let text = text.as_bytes();

            assert_eq!(
                parse(text),
                ParseResult {
                    text,
                    tokens: vec![
                        Token::Heading {
                            level: length as u8,
                            text: b"heading"
                        },
                        Token::Text(b"text"),
                    ]
                }
            );
        }
    }

    #[test]
    fn test_fence() {
        let text = b"```lang\npre```\ntext";

        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![
                    Token::Fence {
                        lang: b"lang",
                        text: b"pre"
                    },
                    Token::Text(b"text"),
                ]
            }
        );
        let text = b"```````lang spaces\npre\n```\nfalse```````";

        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![Token::Fence {
                    lang: b"lang spaces",
                    text: b"pre\n```\nfalse"
                }]
            }
        );
    }

    #[test]
    fn test_code() {
        let text = b"`co\\de`text";

        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![Token::Code(b"co\\de"), Token::Text(b"text")]
            }
        );
    }

    #[test]
    fn test_quote() {
        let text = b"> quote";

        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![Token::Quote, Token::Text(b" quote")]
            }
        );
    }

    #[test]
    fn test_row() {
        let text = b"|1|2|3|";

        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![Token::TableRow(b"|1|2|3|")]
            }
        );
    }

    #[test]
    fn test_break() {
        let text = b"\n\n";

        assert_eq!(
            parse(text),
            ParseResult {
                text,
                tokens: vec![Token::Break]
            }
        );
    }
}
