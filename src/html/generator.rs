use super::{escape, escape_attribute};
use crate::markdown::Token;

enum State {
    CanBeginBlock,
    InParagraph,
    InListItem,
    InAttribute,
}

impl State {
    fn opening_tag(&self) -> &'static [u8] {
        match self {
            Self::CanBeginBlock | Self::InAttribute => b"",
            Self::InParagraph => b"<p>",
            Self::InListItem => b"<li>",
        }
    }

    fn closing_tag(&self) -> &'static [u8] {
        match self {
            Self::CanBeginBlock | Self::InAttribute => b"",
            Self::InParagraph => b"</p>",
            Self::InListItem => b"</li>",
        }
    }
}

fn open_emphasis(strength: u8) -> &'static [u8] {
    match strength {
        1 => b"<em>",
        2 => b"<strong>",
        3 => b"<em><strong>",
        _ => panic!("invalid emphasis strength"),
    }
}

fn close_emphasis(strength: u8) -> &'static [u8] {
    match strength {
        1 => b"</em>",
        2 => b"</strong>",
        3 => b"</strong></em>",
        _ => panic!("invalid emphasis strength"),
    }
}

pub fn generate(tokens: &[Token]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut state = State::CanBeginBlock;
    let mut emphasis = 0;

    for (i, token) in tokens.iter().enumerate() {
        match *token {
            Token::Text(text) => match state {
                State::InParagraph | State::InListItem => {
                    result.extend_from_slice(&escape(text));
                }
                State::InAttribute => {
                    result.extend_from_slice(&escape_attribute(text));
                }
                _ => {
                    result.extend_from_slice(state.closing_tag());
                    state = State::InParagraph;
                    result.extend_from_slice(state.opening_tag());
                    result.extend_from_slice(&escape(text));
                }
            },
            Token::Raw(text) => {
                result.extend_from_slice(text);
            }
            Token::Meta(_) => {}
            Token::Separator(_) => {
                result.extend_from_slice(state.closing_tag());
                result.extend_from_slice(b"<hr>");
                state = State::CanBeginBlock;
            }
            Token::BeginItem { ordered } => match state {
                State::CanBeginBlock => {
                    if ordered {
                        result.extend_from_slice(b"<ol>");
                    } else {
                        result.extend_from_slice(b"<ul>");
                    }
                    state = State::InListItem;
                    result.extend_from_slice(state.opening_tag());
                }
                State::InListItem => {
                    result.extend_from_slice(state.opening_tag());
                }
                _ => {}
            },
            Token::EndItem => {
                result.extend_from_slice(state.closing_tag());
            }
            Token::Emphasis(strength) => {
                if strength == emphasis || emphasis + strength > 3 {
                    result.extend_from_slice(close_emphasis(strength));
                    emphasis = emphasis.saturating_sub(strength)
                } else {
                    result.extend_from_slice(open_emphasis(strength));
                    emphasis += strength
                }
            }
            Token::BeginReference { bang } => {
                tokens[i + 1..].iter().find(|&end| match end {
                    &Token::EndReference { uri, alt } => {
                        if bang {
                            result.extend_from_slice(b"<img src=\"");
                        } else {
                            result.extend_from_slice(b"<a href=\"");
                        }
                        result.extend_from_slice(uri);
                        result.extend_from_slice(b"\"");
                        if !alt.is_empty() {
                            result.extend_from_slice(b" title=\"");
                            result.extend_from_slice(alt);
                            result.extend_from_slice(b"\"");
                        }
                        if bang {
                            result.extend_from_slice(b" alt=\"");
                            state = State::InAttribute;
                        } else {
                            result.extend_from_slice(b">");
                        }
                        true
                    }
                    _ => false,
                });
            }
            Token::EndReference { uri: _, alt: _ } => {
                tokens[..i].iter().rfind(|&end| match end {
                    &Token::BeginReference { bang } => {
                        if bang {
                            result.extend_from_slice(b"\">");
                        } else {
                            result.extend_from_slice(b"</a>");
                        }
                        true
                    }
                    _ => false,
                });
            }
            Token::Heading { level, text } => {
                result.extend_from_slice(state.closing_tag());
                result.extend_from_slice(b"<h");
                result.push(b'0' + level);
                result.extend_from_slice(b">");
                result.extend_from_slice(text);
                result.extend_from_slice(b"</h");
                result.push(b'0' + level);
                result.extend_from_slice(b">");
            }
            Token::Fence { lang: _, text } => {
                result.extend_from_slice(b"<pre><code>");
                result.extend_from_slice(text);
                result.extend_from_slice(b"</code></pre>");
            }
            Token::Code(text) => {
                result.extend_from_slice(b"<code>");
                result.extend_from_slice(text);
                result.extend_from_slice(b"</code>");
            }
            Token::Quote => {}
            Token::TableRow(_text) => {}
            Token::Break => {
                result.extend_from_slice(state.closing_tag());
                state = State::CanBeginBlock;
            }
        }
    }

    result
}
