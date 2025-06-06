#[cfg(test)]
mod tests;
mod token;

pub use token::{Token, Tokens3Window};

pub struct Tokens<'t> {
    text: &'t [u8],
    pos: usize,
    possible_text_start: usize,
    next_is_start_of_line: bool,
    reference_text_start: Option<usize>,
}

impl<'t> Iterator for Tokens<'t> {
    type Item = Token<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! flush_text {
            () => {
                if self.possible_text_start < self.pos {
                    let range = self.possible_text_start..self.pos;
                    self.possible_text_start = self.pos;
                    return Some(Token::Text(&self.text[range]));
                }
            };
        }

        macro_rules! emit {
            ($token:expr => $j:expr) => {
                self.possible_text_start = $j;
                self.pos = $j;
                return Some($token);
            };
        }

        while let Some(&c) = self.text.get(self.pos) {
            let i = self.pos;
            let start_of_line = self.next_is_start_of_line;
            self.next_is_start_of_line = false;

            match c {
                // Escape sequences '\X'
                b'\\' => {
                    flush_text!();
                    self.possible_text_start = i + 1;
                    self.pos = self.text.len().min(i + 2);
                    if self.char_at(i + 1) == b'\n' {
                        self.next_is_start_of_line = true;
                        flush_text!();
                    }
                    continue;
                }

                // HTML tags that do not contain markdown to be parsed
                b'<' if self.text_at(i + 1).starts_with(b"pre")
                    || self.text_at(i + 1).starts_with(b"script")
                    || self.text_at(i + 1).starts_with(b"style") =>
                {
                    flush_text!();
                    let closing_tag = match self.char_at(i + 2) {
                        b'r' => b"</pre>".as_ref(),
                        b'c' => b"</script>".as_ref(),
                        b't' => b"</style>".as_ref(),
                        _ => unreachable!(),
                    };
                    let j = self.substring_end(closing_tag, i + closing_tag.len() - 1);
                    let k = if self.char_at(j) == b'\n' {
                        self.next_is_start_of_line = self.char_at(j - 1) == b'\n';
                        j + 1
                    } else {
                        j
                    };

                    emit!(Token::Raw(&self.text[i..k]) => k);
                }

                // HTML tags that may be separated from upcoming markdown
                b'<' if matches!(self.char_at(i + 1), b'/' | b'A'..=b'Z' | b'a'..=b'z') => {
                    flush_text!();
                    let j = if self.char_at(i + 1) == b'/' {
                        self.substring_end(b">", i + 2)
                    } else {
                        let k = i
                            + 1
                            + self
                                .text_at(i + 1)
                                .iter()
                                .take_while(
                                    |c| matches!(c,  b'A'..=b'Z' | b'a'..=b'z'| b'0'..=b'9'| b'-'),
                                )
                                .count();
                        self.tag_end(self.text_in(i + 1, k), k + 1)
                    };

                    emit!(Token::Raw(&self.text[i..j]) => j);
                }

                b'&' if self.entity_end(i + 1).is_some() => {
                    flush_text!();
                    let j = self.entity_end(i + 1).unwrap(); // won't panic due to match guard
                    emit!(Token::Raw(&self.text[i..j]) => j);
                }

                // Metadata
                d @ (b'-' | b'+')
                    if i == 0
                        && self.line_at(i + 1).iter().take_while(|&&e| e == d).count() >= 2 =>
                {
                    let j = self.substring_end(b"\n", i + 3); // 3 = minimum starting length
                    let separator = &self.text_in(i, j - 1);

                    let k = self.substring_end(separator, j);
                    if self.char_at(k - separator.len() - 1) == b'\n'
                        && matches!(self.char_at(k), 0 | b'\n')
                    {
                        self.next_is_start_of_line = true;
                        emit!(Token::Meta(&self.text[j..k - separator.len() - 1]) => k + 1);
                    }
                }

                // Decorative separator
                d @ (b'*' | b'=' | b'_' | b'-')
                    if start_of_line && self.line_at(i).iter().all(|&e| e == d) =>
                {
                    flush_text!();
                    let j = self.char_start(b'\n', i + 1);
                    emit!(Token::Separator(d) => j);
                }

                // Unordered-list item
                b'*' | b'-' if start_of_line && self.char_at(i + 1) == b' ' => {
                    flush_text!();
                    emit!(Token::BeginItem { ordered: false } => i + 2);
                }

                // Ordered-list item
                b'0'..=b'9' if start_of_line && self.text_in(i + 1, i + 3) == b". " => {
                    flush_text!();
                    emit!(Token::BeginItem { ordered: true } => i + 3);
                }

                // Emphasis
                b'*' if (i == 0 || self.char_at(i - 1) != b'*')
                    && self
                        .text_at(i + 1)
                        .iter()
                        .take_while(|&&d| d == b'*')
                        .count()
                        <= 2 =>
                {
                    flush_text!();
                    let strength = 1 + self
                        .text_at(i + 1)
                        .iter()
                        .take_while(|&&d| d == b'*')
                        .count();
                    emit!(Token::Emphasis(strength as u8) => i + strength);
                }

                // Deleted
                b'~' if self.char_at(i + 1) == b'~' => {
                    flush_text!();
                    emit!(Token::Deleted => i + 2);
                }

                // Definition
                b'[' if start_of_line
                    && self
                        .unescaped_reference_end(i + 1)
                        .is_some_and(|j| self.char_at(j + 1) == b':') =>
                {
                    flush_text!();
                    let j = self.unescaped_reference_end(i + 1).unwrap();
                    let mut k = j + 2;
                    while self.char_at(k) == b' ' {
                        k += 1
                    }
                    emit!(Token::BeginDefinition(&self.text[i + 1..j]) => k);
                }

                // Reference
                b'!' if self.char_at(i + 1) == b'['
                    && self.char_at(i + 2) != b'^'
                    && self.unescaped_reference_end(i + 2).is_some() =>
                {
                    flush_text!();
                    self.reference_text_start = Some(i + 2);
                    emit!(Token::BeginReference { bang: true } => i + 2);
                }

                b'[' if self.unescaped_reference_end(i + 1).is_some() => {
                    flush_text!();
                    self.reference_text_start = Some(i + 1);
                    emit!(Token::BeginReference { bang: false } => i + 1);
                }

                b']' if self.reference_text_start.is_some() => {
                    flush_text!();
                    let rts = self.reference_text_start.take().unwrap(); // won't panic due to match guard
                    let d = self.char_at(i + 1);
                    if d == b'[' || d == b'(' {
                        let e = if d == b'[' { b']' } else { b')' };
                        let j = self.char_start(e, i + 2);
                        let k = self.char_start_till(b' ', i + 2, j);

                        // Unlike Markdown, quoting alt text is optional
                        let mut alt = &self.text[j.min(k + 1)..j];
                        if matches!(alt.first(), Some(b'"')) {
                            alt = &alt[1..];
                        }
                        if matches!(alt.last(), Some(b'"')) {
                            alt = &alt[..alt.len() - 1];
                        }

                        emit!(Token::EndReference {
                            uri: &self.text[i + 2..k],
                            alt,
                            lazy: d == b'[',
                        } => j + 1);
                    } else {
                        emit!(Token::EndReference { uri: self.text_in(rts, i), alt: b"", lazy: true } => i + 1);
                    }
                }

                // Heading
                b'#' if start_of_line
                    && self
                        .text_at(i + 1)
                        .iter()
                        .take_while(|&&d| d == b'#')
                        .count()
                        <= 5 =>
                {
                    flush_text!();
                    let level = 1 + self
                        .text_at(i + 1)
                        .iter()
                        .take_while(|&&d| d == b'#')
                        .count();

                    let mut valid = false;
                    for (j, d) in self.text_at(i + level).iter().enumerate() {
                        match d {
                            b' ' => {
                                valid = true;
                            }
                            &d => {
                                if valid || d == b'\n' {
                                    emit!(Token::Heading(level as u8) => i + level + j);
                                }
                                break;
                            }
                        }
                    }
                }

                // Fenced block
                b'`' if start_of_line
                    && self
                        .text_at(i + 1)
                        .iter()
                        .take_while(|&&d| d == b'`')
                        .count()
                        >= 2 =>
                {
                    flush_text!();
                    let j = i
                        + 1
                        + self
                            .text_at(i + 1)
                            .iter()
                            .take_while(|&&d| d == b'`')
                            .count();

                    let separator = &self.text_in(i, j);
                    let k = self.char_start(b'\n', j);
                    let m = self.substring_end(separator, k + 1);

                    emit!(Token::Fence {
                        lang: &self.text[j..k],
                        text: &self.text[self.text.len().min(k + 1)..m - separator.len()],
                    } => m + 1);
                }

                // Inline code
                b'`' => {
                    flush_text!();
                    let j = self.char_start(b'`', i + 1);
                    emit!(Token::Code(&self.text[i + 1..j]) => j + 1);
                }

                // Blockquotes
                b'>' if start_of_line => {
                    flush_text!();
                    self.next_is_start_of_line = true;
                    emit!(Token::Quote => i + 1);
                }

                // Paragraph break
                b'\n' => {
                    flush_text!();
                    let mut j = i + 1;
                    let mut k = j;
                    while match self.char_at(j) {
                        b'\n' => {
                            k = j + 1;
                            true
                        }
                        b' ' | b'\t' => true,
                        _ => false,
                    } {
                        j += 1;
                    }

                    self.next_is_start_of_line = true;
                    emit!(Token::Break { hard: k != i + 1 } => k);
                }

                // Indent after break
                b' ' if start_of_line => {
                    let mut j = i + 1;
                    while matches!(self.char_at(j), b' ') {
                        j += 1;
                    }
                    self.next_is_start_of_line = true;
                    emit!(Token::Indent(j - i) => j);
                }

                _ => {}
            }
            self.pos += 1;
        }

        flush_text!();
        None
    }
}

impl<'t> Tokens<'t> {
    #[inline]
    fn text_at(&self, i: usize) -> &'t [u8] {
        self.text.get(i..).unwrap_or(b"")
    }

    #[inline]
    fn text_in(&self, i: usize, j: usize) -> &'t [u8] {
        self.text.get(i..j).unwrap_or(b"")
    }

    #[inline]
    fn line_at(&self, i: usize) -> &'t [u8] {
        self.text_in(i, self.char_start(b'\n', i))
    }

    #[inline]
    fn char_at(&self, i: usize) -> u8 {
        self.text.get(i).copied().unwrap_or(0)
    }

    #[inline]
    fn char_start(&self, needle: u8, search_start: usize) -> usize {
        self.char_start_till(needle, search_start, self.text.len())
    }

    #[inline]
    fn char_start_till(&self, needle: u8, search_start: usize, search_end: usize) -> usize {
        self.text_in(search_start, search_end)
            .iter()
            .position(|&t| t == needle)
            .map(|j| search_start + j)
            .unwrap_or(search_end)
    }

    #[inline]
    fn substring_end(&self, needle: &[u8], search_start: usize) -> usize {
        self.text_at(search_start)
            .windows(needle.len())
            .position(|t| t == needle)
            .map(|j| search_start + j + needle.len())
            .unwrap_or(self.text.len())
    }

    fn tag_end(&self, tag: &[u8], search_start: usize) -> usize {
        self.text_at(search_start)
            .windows(tag.len() + 3)
            .position(|t| {
                let e = t.len() - 1;
                t[0] == b'\n' && t[1] == b'\n'
                    || (t[0] == b'<' && t[1] == b'/' && &t[2..e] == tag && t[e] == b'>')
            })
            .map(|j| {
                if self.text[search_start + j] == b'\n' {
                    search_start + j
                } else {
                    search_start + j + tag.len() + 3
                }
            })
            .unwrap_or(self.text.len())
    }

    fn entity_end(&self, search_start: usize) -> Option<usize> {
        let j = search_start
            + self
                .text_at(search_start)
                .iter()
                .take_while(|c| c.is_ascii_alphabetic())
                .count();

        if self.char_at(j) == b';' {
            Some(j + 1)
        } else {
            None
        }
    }

    fn unescaped_reference_end(&self, i: usize) -> Option<usize> {
        self.line_at(i)
            .windows(2)
            .position(|window| window[0] != b'\\' && window[1] == b']')
            .map(|j| i + j + 1)
    }
}

pub fn lex(text: &[u8]) -> Tokens {
    Tokens {
        text,
        pos: 0,
        possible_text_start: 0,
        next_is_start_of_line: true,
        reference_text_start: None,
    }
}
