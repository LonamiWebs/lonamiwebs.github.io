use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Token<'t> {
    Text(&'t [u8]),
    Raw(&'t [u8]),
    Meta(&'t [u8]),
    Separator(u8),
    BeginItem { ordered: bool },
    Emphasis(u8),
    BeginReference { bang: bool },
    EndReference { uri: &'t [u8], alt: &'t [u8] },
    Heading(u8),
    Fence { lang: &'t [u8], text: &'t [u8] },
    Code(&'t [u8]),
    Quote(u8),
    TableRow(&'t [u8]),
    Break { hard: bool, indent: usize },
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
            Self::Heading(x) => f.debug_tuple("Heading").field(x).finish(),
            Self::Fence { lang, text } => f
                .debug_struct("Fence")
                .field("lang", &String::from_utf8_lossy(lang))
                .field("text", &String::from_utf8_lossy(text))
                .finish(),
            Self::Code(x) => f
                .debug_tuple("Code")
                .field(&String::from_utf8_lossy(x))
                .finish(),
            Self::Quote(x) => f.debug_tuple("Quote").field(x).finish(),
            Self::TableRow(x) => f
                .debug_tuple("TableRow")
                .field(&String::from_utf8_lossy(x))
                .finish(),
            Self::Break { hard, indent } => f
                .debug_struct("Break")
                .field("hard", hard)
                .field("indent", indent)
                .finish(),
        }
    }
}
