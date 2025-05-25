use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum Node<'t> {
    Empty,
    Raw(&'t [u8]),
    Text(&'t [u8]),
    AltText(&'t [u8]),
    Paragraph,
    Joiner { inline: bool },
    Separator,
    List { ordered: bool, indent: usize },
    ListItem,
    DefinitionItem(&'t [u8]),
    Emphasis(u8),
    Deleted,
    FootnoteReference(&'t [u8]),
    Reference(&'t [u8]),
    Image(&'t [u8]),
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
            Self::AltText(text) => write!(f, "AltText({})", String::from_utf8_lossy(text)),
            Self::Paragraph => write!(f, "Paragraph"),
            Self::Joiner { inline } => write!(f, "Joiner(inline={inline})"),
            Self::Separator => write!(f, "Separator"),
            Self::List { ordered, indent } => write!(f, "List(ordered={ordered}, indent={indent})"),
            Self::ListItem => write!(f, "ListItem"),
            Self::DefinitionItem(identifier) => {
                write!(f, "DefinitionItem({})", String::from_utf8_lossy(identifier))
            }
            Self::Emphasis(strength) => write!(f, "Emphasis({strength})"),
            Self::Deleted => write!(f, "Deleted"),
            Self::FootnoteReference(url) => {
                write!(f, "FootnoteReference({})", String::from_utf8_lossy(url))
            }
            Self::Reference(url) => write!(f, "Reference({})", String::from_utf8_lossy(url)),
            Self::Image(url) => write!(f, "Image({})", String::from_utf8_lossy(url)),
            Self::Heading(level) => write!(f, "Heading({level})"),
            Self::Pre(text) => write!(f, "Pre({})", String::from_utf8_lossy(text)),
            Self::Code => write!(f, "Code"),
            Self::Quote => write!(f, "Quote"),
        }
    }
}
