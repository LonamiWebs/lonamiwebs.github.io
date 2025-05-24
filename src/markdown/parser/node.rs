use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum Node<'t> {
    Empty,
    Raw(&'t [u8]),
    Text(&'t [u8]),
    Paragraph,
    Joiner { inline: bool },
    Separator,
    List { ordered: bool, indent: usize },
    ListItem,
    Emphasis(u8),
    Reference,
    Heading(u8),
    Pre(&'t [u8]),
    Code,
    Quote,
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "."),
            Self::Raw(text) => write!(f, "Raw({})", String::from_utf8_lossy(text)),
            Self::Text(text) => write!(f, "Text({})", String::from_utf8_lossy(text)),
            Self::Paragraph => write!(f, "Paragraph"),
            Self::Joiner { inline } => write!(f, "Joiner(inline={inline})"),
            Self::Separator => write!(f, "Separator"),
            Self::List { ordered, indent } => write!(f, "List(ordered={ordered}, indent={indent})"),
            Self::ListItem => write!(f, "ListItem"),
            Self::Emphasis(strength) => write!(f, "Emphasis({strength})"),
            Self::Reference => write!(f, "Reference"),
            Self::Heading(level) => write!(f, "Heading({level})"),
            Self::Pre(text) => write!(f, "Pre({})", String::from_utf8_lossy(text)),
            Self::Code => write!(f, "Code"),
            Self::Quote => write!(f, "Quote"),
        }
    }
}
