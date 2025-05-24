use super::{escape, escape_attribute};
use crate::collections::{Graph, GraphNodeRef as Ref};
use crate::markdown::Node;

pub fn generate(arena: Graph<Node>) -> Vec<u8> {
    let mut result = Vec::new();
    visit(arena.root(), &mut result);
    result
}

fn visit(cursor: Ref<Node>, buffer: &mut Vec<u8>) {
    match cursor.value() {
        Node::Empty => {}
        Node::Raw(text) => buffer.extend_from_slice(text),
        Node::Text(text) => buffer.extend_from_slice(&escape(text)),
        Node::AltText(_) => {
            return; // processed earlier
        }
        Node::Paragraph => buffer.extend_from_slice(b"<p>"),
        Node::Joiner { inline } => buffer.extend_from_slice(if inline { b" " } else { b"<br>" }),
        Node::Separator => buffer.extend_from_slice(b"<hr>"),
        Node::List { ordered, indent: _ } => {
            buffer.extend_from_slice(if ordered { b"<ol>" } else { b"<ul>" })
        }
        Node::ListItem => {
            buffer.extend_from_slice(b"<li>");
        }
        Node::DefinitionItem(identifier) => {
            if !identifier.starts_with(b"^") {
                return; // only footnote references should be visible
            }
            buffer.extend_from_slice(b"<p id=\"fn:");
            buffer.extend_from_slice(&identifier[1..]);
            buffer.extend_from_slice(b"\">");
        }
        Node::Emphasis(strength) => {
            buffer.extend_from_slice(match strength {
                1 => b"<em>",
                2 => b"<strong>",
                3 => b"<em><strong>",
                _ => panic!("bad emphasis strength"),
            });
        }
        Node::FootnoteReference(identifier) => {
            buffer.extend_from_slice(b"<a href=\"#fn:");
            buffer.extend_from_slice(identifier);
            buffer.extend_from_slice(b"\"><sup id=\"fnref:");
            buffer.extend_from_slice(identifier);
            buffer.extend_from_slice("\">↪".as_bytes());
            buffer.extend_from_slice(identifier);
        }
        Node::Reference(url) => {
            buffer.extend_from_slice(b"<a href=\"");
            buffer.extend_from_slice(url);
            buffer.extend_from_slice(b"\"");
            if let Some(Node::AltText(alt)) = cursor.last_child().map(|child| child.value()) {
                buffer.extend_from_slice(b" title=\"");
                buffer.extend_from_slice(&escape_attribute(alt));
                buffer.extend_from_slice(b"\"");
            }
            buffer.extend_from_slice(b">");
        }
        Node::Image(url) => {
            buffer.extend_from_slice(b"<img src=\"");
            buffer.extend_from_slice(url);
            buffer.extend_from_slice(b"\"");
            if let Some(Node::AltText(alt)) = cursor.last_child().map(|child| child.value()) {
                buffer.extend_from_slice(b" alt=\"");
                buffer.extend_from_slice(&escape_attribute(alt));
                buffer.extend_from_slice(b"\"");
            }
            buffer.extend_from_slice(b">");
        }
        Node::Heading(level) => {
            buffer.extend_from_slice(match level {
                1 => b"<h1>",
                2 => b"<h2>",
                3 => b"<h3>",
                4 => b"<h4>",
                5 => b"<h5>",
                6 => b"<h6>",
                _ => panic!("bad heading level"),
            });
        }
        Node::Pre(lang) => {
            if lang.is_empty() {
                buffer.extend_from_slice(b"<pre>");
            } else {
                buffer.extend_from_slice(b"<pre><code class=\"language-");
                buffer.extend_from_slice(lang);
                buffer.extend_from_slice(b"\">");
            }
        }
        Node::Code => {
            buffer.extend_from_slice(b"<code>");
        }
        Node::Quote => {
            buffer.extend_from_slice(b"<blockquote><p>");
        }
    }
    for child in cursor.children() {
        visit(child, buffer);
    }
    match cursor.value() {
        Node::Empty => {}
        Node::Raw(_) => {}
        Node::Text(_) => {}
        Node::AltText(_) => unreachable!(),
        Node::Paragraph => buffer.extend_from_slice(b"</p>"),
        Node::Joiner { .. } => {}
        Node::Separator => {}
        Node::List { ordered, indent: _ } => {
            buffer.extend_from_slice(if ordered { b"</ol>" } else { b"</ul>" })
        }
        Node::ListItem => {
            buffer.extend_from_slice(b"</li>");
        }
        Node::DefinitionItem(identifier) => {
            buffer.extend_from_slice(b"&nbsp;<a href=\"#fnref:");
            buffer.extend_from_slice(&identifier[1..]);
            buffer.extend_from_slice("\">↩</a></p>".as_bytes());
        }
        Node::Emphasis(strength) => {
            buffer.extend_from_slice(match strength {
                1 => b"</em>",
                2 => b"</strong>",
                3 => b"</em></strong>",
                _ => unreachable!(),
            });
        }
        Node::FootnoteReference(_) => {
            buffer.extend_from_slice(b"</sup></a>");
        }
        Node::Reference(_) => {
            buffer.extend_from_slice(b"</a>");
        }
        Node::Image(_) => {}
        Node::Heading(level) => {
            buffer.extend_from_slice(match level {
                1 => b"</h1>",
                2 => b"</h2>",
                3 => b"</h3>",
                4 => b"</h4>",
                5 => b"</h5>",
                6 => b"</h6>",
                _ => unreachable!(),
            });
        }
        Node::Pre(lang) => {
            buffer.extend_from_slice(if lang.is_empty() {
                b"</pre>"
            } else {
                b"</code></pre>"
            });
        }
        Node::Code => {
            buffer.extend_from_slice(b"</code>");
        }
        Node::Quote => {
            buffer.extend_from_slice(b"</p></blockquote>");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_paragraph() {
        let arena = Graph::new(Node::Empty);
        let mut cursor = arena.root();
        cursor = cursor.append_child(Node::Paragraph);
        cursor = cursor.append_child(Node::Emphasis(1));
        cursor.append_child(Node::Text(b"text"));

        assert_eq!(
            String::from_utf8_lossy(&generate(arena)),
            "<p><em>text</em></p>"
        );
    }

    #[test]
    fn test_lists() {
        let arena = Graph::new(Node::Empty);
        let mut cursor = arena.root();
        cursor = cursor.append_child(Node::List {
            ordered: false,
            indent: 0,
        });
        let li = cursor.append_child(Node::ListItem);
        li.append_child(Node::Text(b"first"));
        cursor = li.append_child(Node::List {
            ordered: false,
            indent: 0,
        });
        let li = cursor.append_child(Node::ListItem);
        li.append_child(Node::Text(b"second"));

        assert_eq!(
            String::from_utf8_lossy(&generate(arena)),
            "<ul><li>first<ul><li>second</li></ul></li></ul>"
        );
    }

    #[test]
    fn test_escaping() {
        let arena = Graph::new(Node::Empty);
        let mut cursor = arena.root();
        cursor = cursor.append_child(Node::Paragraph);
        cursor = cursor.append_child(Node::Code);
        cursor.append_child(Node::Text(b"<tag>"));
        assert_eq!(
            String::from_utf8_lossy(&generate(arena)),
            "<p><code>&lt;tag&gt;</code></p>"
        );
    }
}
