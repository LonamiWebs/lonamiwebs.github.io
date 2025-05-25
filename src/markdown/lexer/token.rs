use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum Token<'t> {
    Text(&'t [u8]),
    Raw(&'t [u8]),
    Meta(&'t [u8]),
    Separator(u8),
    BeginDefinition(&'t [u8]),
    BeginItem {
        ordered: bool,
    },
    Indent(usize),
    Emphasis(u8),
    Deleted,
    BeginReference {
        bang: bool,
    },
    EndReference {
        uri: &'t [u8],
        alt: &'t [u8],
        lazy: bool,
    },
    Heading(u8),
    Fence {
        lang: &'t [u8],
        text: &'t [u8],
    },
    Code(&'t [u8]),
    Quote,
    Break {
        hard: bool,
    },
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
            Self::BeginDefinition(x) => f
                .debug_tuple("BeginDefinition")
                .field(&String::from_utf8_lossy(x))
                .finish(),
            Self::BeginItem { ordered } => f
                .debug_struct("BeginItem")
                .field("ordered", ordered)
                .finish(),
            Self::Indent(x) => f.debug_tuple("Indent").field(x).finish(),
            Self::Emphasis(x) => f.debug_tuple("Emphasis").field(x).finish(),
            Self::Deleted => f.write_str("Deleted"),
            Self::BeginReference { bang } => f
                .debug_struct("BeginReference")
                .field("bang", bang)
                .finish(),
            Self::EndReference { uri, alt, lazy } => f
                .debug_struct("EndReference")
                .field("uri", &String::from_utf8_lossy(uri))
                .field("alt", &String::from_utf8_lossy(alt))
                .field("lazy", lazy)
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
            Self::Quote => f.write_str("Quote"),
            Self::Break { hard } => f.debug_struct("Break").field("hard", hard).finish(),
        }
    }
}

pub struct Tokens3Window<'t, I: Iterator<Item = Token<'t>>> {
    iter: I,
    buffer: [Option<Token<'t>>; 3],
}

impl<'t, I: Iterator<Item = Token<'t>>> Tokens3Window<'t, I> {
    pub fn new(mut iter: I) -> Self {
        let buffer = [None, None, iter.next()];
        Self { iter, buffer }
    }
}

impl<'t, I: Iterator<Item = Token<'t>>> Iterator for Tokens3Window<'t, I> {
    type Item = (Option<Token<'t>>, Token<'t>, Option<Token<'t>>);

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer[0] = self.buffer[1];
        self.buffer[1] = self.buffer[2];
        self.buffer[2] = self.iter.next();
        self.buffer[1].map(|current| (self.buffer[0], current, self.buffer[2]))
    }
}
