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
    p.append_child(Node::Joiner { inline: true });
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

    assert_eq!(parse(tokens).ast.root(), expected);
}

#[test]
fn test_references() {
    let tokens = lex(br#"
[text] [reusable][r] footnote[^1] [inline](https://example.com/inline "title") ![image](https://example.com/image "alt")

[^1]: footnote text

[r]: https://example.com/reusable
"#.trim_ascii());

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let p = expected.append_child(Node::Paragraph);

    let unresolved_reference = p.append_child(Node::Text(b"["));
    unresolved_reference.append_child(Node::Text(b"text"));
    unresolved_reference.append_child(Node::Text(b"]"));
    p.append_child(Node::Text(b" "));

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
    inline.append_child(Node::AltText(b"image"));
    inline.append_child(Node::AltText(b"alt"));

    expected
        .append_child(Node::DefinitionItem(b"^1"))
        .append_child(Node::Text(b"footnote text"));

    expected
        .append_child(Node::DefinitionItem(b"r"))
        .append_child(Node::Text(b"https://example.com/reusable"));

    assert_eq!(parse(tokens).ast.root(), expected);
}

#[test]
fn test_paragraph_gets_added() {
    let tokens = lex(b"*emphasis* text");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let p = expected.append_child(Node::Paragraph);
    p.append_child(Node::Emphasis(1))
        .append_child(Node::Text(b"emphasis"));
    p.append_child(Node::Text(b" text"));
    assert_eq!(parse(tokens).ast.root(), expected);

    let tokens = lex(b"`code` text");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let p = expected.append_child(Node::Paragraph);
    p.append_child(Node::Code).append_child(Node::Text(b"code"));
    p.append_child(Node::Text(b" text"));
    assert_eq!(parse(tokens).ast.root(), expected);

    let tokens = lex(b"[ref](url) text");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let p = expected.append_child(Node::Paragraph);
    p.append_child(Node::Reference(b"url"))
        .append_child(Node::Text(b"ref"));
    p.append_child(Node::Text(b" text"));
    assert_eq!(parse(tokens).ast.root(), expected);

    let tokens = lex(b"[^ref] text");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let p = expected.append_child(Node::Paragraph);
    p.append_child(Node::FootnoteReference(b"ref"));
    p.append_child(Node::Text(b" text"));
    assert_eq!(parse(tokens).ast.root(), expected);

    let tokens = lex(b"text\n```lang\npre\n```\n\nremaining `code` word");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    expected
        .append_child(Node::Paragraph)
        .append_child(Node::Text(b"text"));
    expected
        .append_child(Node::Pre(b"lang"))
        .append_child(Node::Text(b"pre\n"));
    let p = expected.append_child(Node::Paragraph);
    p.append_child(Node::Text(b"remaining "));
    p.append_child(Node::Code).append_child(Node::Text(b"code"));
    p.append_child(Node::Text(b" word"));
    assert_eq!(parse(tokens).ast.root(), expected);
}

#[test]
fn test_hard_breaks_inside_quotes() {
    let tokens = lex(b"> start\n> middle\n>\n> end");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let quote = expected.append_child(Node::Quote);
    let p = quote.append_child(Node::Paragraph);
    p.append_child(Node::Text(b"start"));
    p.append_child(Node::Joiner { inline: true });
    p.append_child(Node::Text(b"middle"));
    quote
        .append_child(Node::Paragraph)
        .append_child(Node::Text(b"end"));
    assert_eq!(parse(tokens).ast.root(), expected);
}

#[test]
fn test_nested_list_does_not_reorder_text() {
    let tokens = lex(b"1. 1\n2. 2\n  * a\n  * b\n3. 3");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let ol = expected.append_child(Node::List {
        ordered: true,
        indent: 0,
    });
    ol.append_child(Node::ListItem)
        .append_child(Node::Text(b"1"));
    let li = ol.append_child(Node::ListItem);
    li.append_child(Node::Text(b"2"));
    let ul = li.append_child(Node::List {
        ordered: false,
        indent: 2,
    });
    ul.append_child(Node::ListItem)
        .append_child(Node::Text(b"a"));
    ul.append_child(Node::ListItem)
        .append_child(Node::Text(b"b"));
    ol.append_child(Node::ListItem)
        .append_child(Node::Text(b"3"));

    assert_eq!(parse(tokens).ast.root(), expected);
}

#[test]
fn test_list_inside_quote() {
    let tokens = lex(b"> start\n> * list\n> end");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let quote = expected.append_child(Node::Quote);
    quote
        .append_child(Node::Paragraph)
        .append_child(Node::Text(b"start"));
    quote
        .append_child(Node::List {
            ordered: false,
            indent: 1,
        })
        .append_child(Node::ListItem)
        .append_child(Node::Text(b"list"));
    quote
        .append_child(Node::Paragraph)
        .append_child(Node::Text(b"end"));
    assert_eq!(parse(tokens).ast.root(), expected);

    let tokens = lex(b"> * list\n>   * nested");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let quote = expected.append_child(Node::Quote);
    let li = quote
        .append_child(Node::List {
            ordered: false,
            indent: 1,
        })
        .append_child(Node::ListItem);
    li.append_child(Node::Text(b"list"));
    li.append_child(Node::List {
        ordered: false,
        indent: 3,
    })
    .append_child(Node::ListItem)
    .append_child(Node::Text(b"nested"));
    assert_eq!(parse(tokens).ast.root(), expected);
}

#[test]
fn test_soft_breaks_inside_quote() {
    let tokens = lex(br"
> **strong**
>
> soft\
> break
>
> <kbd>a</kbd> <kbd>b</kbd>
"
    .trim_ascii());

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let quote = expected.append_child(Node::Quote);
    quote
        .append_child(Node::Paragraph)
        .append_child(Node::Emphasis(2))
        .append_child(Node::Text(b"strong"));
    let p = quote.append_child(Node::Paragraph);
    p.append_child(Node::Text(b"soft"));
    p.append_child(Node::Joiner { inline: false });
    p.append_child(Node::Text(b"break"));
    let p = quote.append_child(Node::Paragraph);
    p.append_child(Node::Raw(b"<kbd>a</kbd>"));
    p.append_child(Node::Text(b" "));
    p.append_child(Node::Raw(b"<kbd>b</kbd>"));

    assert_eq!(parse(tokens).ast.root(), expected);
}

#[test]
fn test_lazy_reference_with_formatting() {
    let tokens = lex(b"[`lazy`]\n\n[`lazy`]: url");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    expected
        .append_child(Node::Paragraph)
        .append_child(Node::Reference(b"url"))
        .append_child(Node::Code)
        .append_child(Node::Text(b"lazy"));
    expected
        .append_child(Node::DefinitionItem(b"`lazy`"))
        .append_child(Node::Text(b"url"));

    assert_eq!(parse(tokens).ast.root(), expected);
}

#[test]
fn html_inside_headings() {
    let tokens = lex(b"# heading <abbr>abbr</abbr>\n\ncontinued");

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();
    let heading = expected.append_child(Node::Heading(1));
    heading.append_child(Node::Text(b"heading "));
    heading.append_child(Node::Raw(b"<abbr>abbr</abbr>"));
    expected
        .append_child(Node::Paragraph)
        .append_child(Node::Text(b"continued"));

    assert_eq!(parse(tokens).ast.root(), expected);
}

#[test]
fn no_stray_paragraphs() {
    let tokens = lex(br"
<span>no p</span>

yes <span>p</span>

<span>no p</span>
  <span>no p</span>
"
    .trim_ascii());

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();

    expected.append_child(Node::Raw(b"<span>no p</span>"));
    let p = expected.append_child(Node::Paragraph);
    p.append_child(Node::Text(b"yes "));
    p.append_child(Node::Raw(b"<span>p</span>"));
    expected.append_child(Node::Raw(b"<span>no p</span>"));
    expected.append_child(Node::Raw(b"<span>no p</span>"));

    assert_eq!(parse(tokens).ast.root(), expected);

    let tokens = lex(br"
text

<details><summary>details</summary>

> text

</details>
"
    .trim_ascii());

    let expected = Graph::new(Node::Empty);
    let expected = expected.root();

    expected
        .append_child(Node::Paragraph)
        .append_child(Node::Text(b"text"));
    expected.append_child(Node::Raw(b"<details><summary>details</summary>"));
    expected
        .append_child(Node::Quote)
        .append_child(Node::Paragraph)
        .append_child(Node::Text(b"text"));
    expected.append_child(Node::Raw(b"</details>"));

    assert_eq!(parse(tokens).ast.root(), expected);
}
