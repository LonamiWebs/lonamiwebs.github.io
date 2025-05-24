use crate::markdown::lex;

use super::*;

#[test]
fn test_lex_and_parse_nested_lists_with_paragraphs() {
    let tokens = lex(br"
# heading

first paragraph
* list
  item

  with paragraphs
  * nested
  * list

* resuming paragraph list

  * second nested list

closing paragraph
"
    .trim_ascii());

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    expected
        .append_child(Node::Heading(1))
        .append_child(Node::Text(b"heading"));
    expected
        .append_child(Node::Paragraph)
        .append_child(Node::Text(b"first paragraph"));

    let ul = expected.append_child(Node::List {
        ordered: false,
        indent: 0,
    });
    let li = ul.append_child(Node::ListItem);
    let p = li.append_child(Node::Paragraph);
    p.append_child(Node::Text(b"list"));
    p.append_child(Node::Joiner { inline: false });
    p.append_child(Node::Text(b"item"));
    li.append_child(Node::Paragraph)
        .append_child(Node::Text(b"with paragraphs"));
    let ul2 = li.append_child(Node::List {
        ordered: false,
        indent: 2,
    });
    ul2.append_child(Node::ListItem)
        .append_child(Node::Text(b"nested"));
    ul2.append_child(Node::ListItem)
        .append_child(Node::Text(b"list"));
    let li = ul.append_child(Node::ListItem);
    li.append_child(Node::Paragraph)
        .append_child(Node::Text(b"resuming paragraph list"));
    li.append_child(Node::List {
        ordered: false,
        indent: 2,
    })
    .append_child(Node::ListItem)
    .append_child(Node::Text(b"second nested list"));
    expected
        .append_child(Node::Paragraph)
        .append_child(Node::Text(b"closing paragraph"));

    assert_eq!(parse(tokens).root(), expected);
}

#[test]
fn test_references() {
    let tokens = lex(br#"
[text] [reusable][r] footnote[^1] [inline](https://example.com/inline "title") ![image](https://example.com/image "alt")

[^1]: footnote text

[r]: https://example.com/reusable
"#.trim_ascii());

    dbg!(lex(br#"
[text] [reusable][r] footnote[^1] [inline](https://example.com/inline "title") ![image](https://example.com/image "alt")

[^1]: footnote text

[r]: https://example.com/reusable
"#.trim_ascii()).collect::<Vec<_>>());

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let p = expected.append_child(Node::Paragraph);

    p.append_child(Node::Text(b"[text] "));

    let reusable = p.append_child(Node::Reference(b"https://example.com/reusable"));
    reusable.append_child(Node::Text(b"reusable"));

    p.append_child(Node::Text(b" footnote"));

    p.append_child(Node::FootnoteReference(b"1"));

    p.append_child(Node::Text(b" "));

    let inline = p.append_child(Node::Reference(b"https://example.com/inline"));
    inline.append_child(Node::Text(b"inline"));
    inline.append_child(Node::AltText(b"title"));

    p.append_child(Node::Text(b" "));

    let inline = p.append_child(Node::Image(b"https://example.com/image"));
    inline.append_child(Node::Text(b"image"));
    inline.append_child(Node::AltText(b"alt"));

    expected
        .append_child(Node::DefinitionItem(b"^1"))
        .append_child(Node::Text(b"footnote text"));

    expected
        .append_child(Node::DefinitionItem(b"r"))
        .append_child(Node::Text(b"https://example.com/reusable"));

    assert_eq!(parse(tokens).root(), expected);
}
