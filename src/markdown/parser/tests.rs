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

    let expected = NodeArena::new();
    let expected = expected.root();
    expected
        .append_child(NodeContent::Heading(1))
        .append_child(NodeContent::Text(b"heading"));
    expected
        .append_child(NodeContent::Paragraph)
        .append_child(NodeContent::Text(b"first paragraph"));

    let ul = expected.append_child(NodeContent::List {
        ordered: false,
        indent: 0,
    });
    let li = ul.append_child(NodeContent::ListItem);
    let p = li.append_child(NodeContent::Paragraph);
    p.append_child(NodeContent::Text(b"list"));
    p.append_child(NodeContent::Joiner { inline: false });
    p.append_child(NodeContent::Text(b"item"));
    li.append_child(NodeContent::Paragraph)
        .append_child(NodeContent::Text(b"with paragraphs"));
    let ul2 = li.append_child(NodeContent::List {
        ordered: false,
        indent: 2,
    });
    ul2.append_child(NodeContent::ListItem)
        .append_child(NodeContent::Text(b"nested"));
    ul2.append_child(NodeContent::ListItem)
        .append_child(NodeContent::Text(b"list"));
    let li = ul.append_child(NodeContent::ListItem);
    li.append_child(NodeContent::Paragraph)
        .append_child(NodeContent::Text(b"resuming paragraph list"));
    li.append_child(NodeContent::List {
        ordered: false,
        indent: 2,
    })
    .append_child(NodeContent::ListItem)
    .append_child(NodeContent::Text(b"second nested list"));
    expected
        .append_child(NodeContent::Paragraph)
        .append_child(NodeContent::Text(b"closing paragraph"));

    assert_eq!(parse(tokens).root(), expected);
}
