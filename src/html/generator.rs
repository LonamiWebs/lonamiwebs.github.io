use super::escape;
use crate::markdown::{NodeContent, NodeRef};

pub fn generate(root: NodeRef) -> Vec<u8> {
    let mut result = Vec::new();
    visit(root, &mut result);
    result
}

fn visit(cursor: NodeRef, buffer: &mut Vec<u8>) {
    match cursor.content() {
        NodeContent::Empty => {}
        NodeContent::Raw(text) => buffer.extend_from_slice(text),
        NodeContent::Text(text) => buffer.extend_from_slice(&escape(text)),
        NodeContent::Paragraph => buffer.extend_from_slice(b"<p>"),
        NodeContent::Joiner { inline } => {
            buffer.extend_from_slice(if inline { b" " } else { b"<br>" })
        }
        NodeContent::Separator => buffer.extend_from_slice(b"<hr>"),
        NodeContent::List { ordered, indent: _ } => {
            buffer.extend_from_slice(if ordered { b"<ol>" } else { b"<ul>" })
        }
        NodeContent::ListItem => {
            buffer.extend_from_slice(b"<li>");
        }
        NodeContent::Emphasis(strength) => {
            buffer.extend_from_slice(match strength {
                1 => b"<em>",
                2 => b"<strong>",
                3 => b"<em><strong>",
                _ => panic!("bad emphasis strength"),
            });
        }
        NodeContent::Reference => {}
        NodeContent::Heading(level) => {
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
        NodeContent::Pre(lang) => {
            if lang.is_empty() {
                buffer.extend_from_slice(b"<pre>");
            } else {
                buffer.extend_from_slice(b"<pre><code class=\"language-");
                buffer.extend_from_slice(lang);
                buffer.extend_from_slice(b"\">");
            }
        }
        NodeContent::Code => {
            buffer.extend_from_slice(b"<code>");
        }
        NodeContent::Quote => {}
    }
    for child in cursor.children() {
        visit(child, buffer);
    }
    match cursor.content() {
        NodeContent::Empty => {}
        NodeContent::Raw(_) => {}
        NodeContent::Text(_) => {}
        NodeContent::Paragraph => buffer.extend_from_slice(b"</p>"),
        NodeContent::Joiner { .. } => {}
        NodeContent::Separator => {}
        NodeContent::List { ordered, indent: _ } => {
            buffer.extend_from_slice(if ordered { b"</ol>" } else { b"</ul>" })
        }
        NodeContent::ListItem => {
            buffer.extend_from_slice(b"</li>");
        }
        NodeContent::Emphasis(strength) => {
            buffer.extend_from_slice(match strength {
                1 => b"</em>",
                2 => b"</strong>",
                3 => b"</em></strong>",
                _ => unreachable!(),
            });
        }
        NodeContent::Reference => {
            // todo!()
        }
        NodeContent::Heading(level) => {
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
        NodeContent::Pre(lang) => {
            buffer.extend_from_slice(if lang.is_empty() {
                b"</pre>"
            } else {
                b"</code></pre>"
            });
        }
        NodeContent::Code => {
            buffer.extend_from_slice(b"</code>");
        }
        NodeContent::Quote => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::markdown::NodeArena;

    #[test]
    fn test_begin_paragraph() {
        let mut cursor = NodeArena::new_root();
        cursor = cursor.append_child(NodeContent::Paragraph);
        cursor = cursor.append_child(NodeContent::Emphasis(1));
        cursor.append_child(NodeContent::Text(b"text"));

        assert_eq!(
            String::from_utf8_lossy(&generate(cursor.root())),
            "<p><em>text</em></p>"
        );
    }

    #[test]
    fn test_lists() {
        let mut cursor = NodeArena::new_root();
        cursor = cursor.append_child(NodeContent::List {
            ordered: false,
            indent: 0,
        });
        let li = cursor.append_child(NodeContent::ListItem);
        li.append_child(NodeContent::Text(b"first"));
        cursor = li.append_child(NodeContent::List {
            ordered: false,
            indent: 0,
        });
        let li = cursor.append_child(NodeContent::ListItem);
        li.append_child(NodeContent::Text(b"second"));

        assert_eq!(
            String::from_utf8_lossy(&generate(cursor.root())),
            "<ul><li>first<ul><li>second</li></ul></li></ul>"
        );
    }

    #[test]
    fn test_escaping() {
        let mut cursor = NodeArena::new_root();
        cursor = cursor.append_child(NodeContent::Paragraph);
        cursor = cursor.append_child(NodeContent::Code);
        cursor.append_child(NodeContent::Text(b"<tag>"));
        assert_eq!(
            String::from_utf8_lossy(&generate(cursor.root())),
            "<p><code>&lt;tag&gt;</code></p>"
        );
    }
}
