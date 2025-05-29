use std::cmp::Reverse;
use std::fmt;

use crate::entry::Entry;
use crate::xml;

pub enum Node {
    Feed {
        lang: &'static str,
        children: Vec<Node>,
    },
    Link {
        href: String,
        rel: Option<&'static str>,
        r#type: Option<&'static str>,
    },
    Id(String),
    Entry {
        lang: &'static str,
        children: Vec<Node>,
    },
    Title(String),
    Published(String),
    Updated(String),
    Content {
        r#type: &'static str,
        contents: String,
    },
}

fn datetime_from_date(date: &str) -> String {
    format!("{}T00:00:00+00:00", date)
}

pub fn from_markdown_entries<'e>(entries: impl Iterator<Item = &'e Entry>) -> Node {
    let mut entries = entries
        .map(|entry| Node::Entry {
            lang: "en",
            children: vec![
                Node::Title(entry.title.clone()),
                Node::Published(datetime_from_date(&entry.date)),
                Node::Updated(datetime_from_date(
                    entry.updated.as_deref().unwrap_or(entry.date.as_ref()),
                )),
                Node::Link {
                    href: format!("https://lonami.dev{}", entry.permalink),
                    rel: Some("alternate"),
                    r#type: Some("text/html"),
                },
                Node::Id(format!("https://lonami.dev{}", entry.permalink)),
                Node::Content {
                    r#type: "html",
                    contents: String::from_utf8_lossy(&xml::escape_text(&entry.processed_contents))
                        .into_owned(),
                },
            ],
        })
        .collect::<Vec<_>>();

    fn entry_published(entry: &Node) -> String {
        match entry {
            Node::Entry { children, .. } => match &children[1] {
                Node::Published(published) => published.clone(),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    entries.sort_by_key(|e| Reverse(entry_published(e)));

    Node::Feed {
        lang: "en",
        children: [
            Node::Title(String::from("Lonami's Site - My Blog")),
            Node::Link {
                href: String::from("https://lonami.dev/blog/atom.xml"),
                rel: Some("self"),
                r#type: Some("application/atom+xml"),
            },
            Node::Link {
                href: String::from("https://lonami.dev/blog/"),
                rel: None,
                r#type: None,
            },
            Node::Updated(entry_published(&entries[0])),
            Node::Id(String::from("https://lonami.dev/blog/atom.xml")),
        ]
        .into_iter()
        .chain(entries)
        .collect(),
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Feed { lang, children } => {
                write!(
                    f,
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                    <feed xmlns=\"http://www.w3.org/2005/Atom\" xml:lang=\"{lang}\">"
                )?;
                for child in children {
                    child.fmt(f)?;
                }
                f.write_str("</feed>")
            }
            Node::Link {
                href,
                rel,
                r#type: ty,
            } => {
                write!(f, "<link href=\"{href}\"")?;
                if let Some(rel) = rel {
                    write!(f, " rel=\"{rel}\"")?;
                }
                if let Some(ty) = ty {
                    write!(f, " type=\"{ty}\"")?;
                }
                f.write_str("/>")
            }
            Node::Id(id) => write!(f, "<id>{id}</id>"),
            Node::Entry { lang, children } => {
                write!(f, "<entry xml:lang=\"{lang}\">")?;
                for child in children {
                    child.fmt(f)?;
                }
                f.write_str("</entry>")
            }
            Node::Title(title) => write!(f, "<title>{title}</title>"),
            Node::Published(published) => write!(f, "<published>{published}</published>"),
            Node::Updated(updated) => write!(f, "<updated>{updated}</updated>"),
            Node::Content {
                r#type: ty,
                contents,
            } => write!(f, "<content type=\"{ty}\">{contents}</content>"),
        }
    }
}
