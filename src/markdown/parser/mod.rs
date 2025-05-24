mod node;
#[cfg(test)]
mod tests;

pub use node::Node;

use super::{Token, Tokens};
use crate::collections::{Graph, GraphNodeRef as Ref};

pub fn parse(tokens: Tokens) -> Graph<Node> {
    let arena = Graph::new(Node::Empty);
    let mut cursor = arena.root();

    let mut last_token = Token::Break {
        hard: true,
        indent: 0,
    };

    let mut nodes_with_references_to_resolve = Vec::new();

    for token in tokens {
        match token {
            Token::Text(text) => {
                if !can_contain_text_at(cursor) {
                    cursor = cursor.append_child(Node::Paragraph);
                }
                cursor.append_child(Node::Text(text));
            }
            Token::Raw(text) => {
                cursor.append_child(Node::Raw(text));
            }
            Token::Meta(_) => {}
            Token::Separator(_) => {
                cursor = cursor.root();
                cursor.append_child(Node::Separator);
            }
            Token::BeginDefinition(identifier) => {
                while matches!(cursor.value(), Node::DefinitionItem(_))
                    || cursor
                        .ancestors()
                        .any(|node| matches!(node.value(), Node::DefinitionItem(_)))
                {
                    cursor = cursor.up();
                }
                cursor = cursor.append_child(Node::DefinitionItem(identifier));
            }
            Token::BeginItem { ordered } => {
                let indent = match last_token {
                    Token::Break { hard: _, indent } => indent,
                    _ => 0,
                };
                if list_indent_at(cursor).is_none() {
                    // Exit any text content.
                    cursor = cursor.root();
                } else {
                    while let Some(last_indent) = list_indent_at(cursor) {
                        if indent > last_indent {
                            // Will push nested list.
                            while !matches!(cursor.value(), Node::ListItem) {
                                cursor = cursor.up();
                            }
                        } else {
                            // Escape list until we reach same or no indent. Will later merge as needed.
                            while !matches!(cursor.value(), Node::List { .. }) {
                                cursor = cursor.up();
                            }
                            cursor = cursor.up();
                        }
                        if indent >= last_indent {
                            break;
                        }
                    }
                }
                cursor = cursor.append_child(Node::List { ordered, indent });
                cursor = cursor.append_child(Node::ListItem);
                cursor = cursor.append_child(Node::Paragraph);
            }
            Token::Emphasis(strength) => {
                if !can_contain_text_at(cursor) {
                    cursor = cursor.append_child(Node::Paragraph);
                }

                let emphasis_level = emphasis_level_at(cursor);

                if strength == emphasis_level || emphasis_level + strength > 3 {
                    cursor = cursor.up();
                } else {
                    cursor = cursor.append_child(Node::Emphasis(strength));
                }
            }
            Token::FootnoteReference(identifier) => {
                cursor.append_child(Node::FootnoteReference(identifier));
            }
            Token::BeginReference { bang } => {
                cursor = cursor.append_child(if bang {
                    Node::Image(b"")
                } else {
                    Node::Reference(b"")
                });
            }
            Token::EndReference { uri, alt, lazy } => {
                if !alt.is_empty() {
                    cursor.append_child(Node::AltText(alt));
                }
                loop {
                    match cursor.value() {
                        Node::Empty => {}
                        Node::Image(_) => cursor.set_value(Node::Image(uri)),
                        Node::Reference(_) => cursor.set_value(Node::Reference(uri)),
                        _ => {
                            cursor = cursor.up();
                            continue;
                        }
                    }
                    break;
                }
                if lazy {
                    nodes_with_references_to_resolve.push(cursor);
                }
                cursor = cursor.up();
            }
            Token::Heading(level) => {
                // Deliberately not supporting titles in lists for simplicity.
                cursor = cursor.root();
                cursor = cursor.append_child(Node::Heading(level));
            }
            Token::Fence { lang, text } => {
                let pre = cursor.append_child(Node::Pre(lang));
                pre.append_child(Node::Text(text));
            }
            Token::Code(text) => {
                let code = cursor.append_child(Node::Code);
                code.append_child(Node::Text(text));
            }
            Token::Quote(_) => {
                // todo!()
            }
            Token::TableRow(_) => {
                // todo!()
            }
            Token::Break { hard, indent } => {
                if hard {
                    if list_indent_at(cursor).is_some() && indent > 0 {
                        while !matches!(cursor.value(), Node::ListItem) {
                            cursor = cursor.up();
                        }
                        cursor = cursor.append_child(Node::Paragraph);
                    } else {
                        //
                        cursor = cursor.root();
                    }
                } else {
                    cursor.append_child(Node::Joiner {
                        inline: indent == 0,
                    });
                }
            }
        }
        last_token = token;
    }

    let root = cursor.root();
    resolve_references(root, nodes_with_references_to_resolve);
    remove_empty_paragraphs(root);
    trim_joiners(root);
    merge_lists_with_same_indent(root);
    remove_paragraphs_from_simple_lists(root);
    arena
}

fn resolve_references<'t>(root: Ref<Node<'t>>, pending: Vec<Ref<Node<'t>>>) {
    fn find_definition<'t>(node: Ref<Node<'t>>, identifier: &[u8]) -> Option<&'t [u8]> {
        match node.value() {
            Node::DefinitionItem(id) if id == identifier => {
                match node.child(0).map(|text| text.value()) {
                    Some(Node::Text(value)) => Some(value),
                    _ => None,
                }
            }
            _ => node
                .children()
                .find_map(|child| find_definition(child, identifier)),
        }
    }

    for node in pending {
        match node.value() {
            Node::Image(identifier) => {
                if let Some(value) = find_definition(root, identifier) {
                    node.set_value(Node::Image(value));
                }
            }
            Node::Reference(identifier) => {
                if let Some(value) = find_definition(root, identifier) {
                    node.set_value(Node::Reference(value));
                }
            }
            _ => {}
        }
    }
}

fn remove_empty_paragraphs(node: Ref<Node>) {
    if matches!(node.value(), Node::Paragraph) && node.is_leaf() {
        node.remove_reparent(false);
    } else {
        for child in node.children() {
            remove_empty_paragraphs(child);
        }
    }
}

fn trim_joiners(node: Ref<Node>) {
    while let Some(child) = node.last_child() {
        match child.value() {
            Node::Joiner { .. } => child.remove_reparent(false),
            _ => break,
        }
    }
    while let Some(child) = node.child(0) {
        match child.value() {
            Node::Joiner { .. } => child.remove_reparent(false),
            _ => break,
        }
    }
    for child in node.children() {
        trim_joiners(child);
    }
}

fn merge_lists_with_same_indent(node: Ref<Node>) {
    fn find_list_pair(node: Ref<Node>) -> Option<usize> {
        for i in 1..node.child_count() {
            match node.child(i - 1).zip(node.child(i)) {
                Some((a, b))
                    if matches!(a.value(), Node::List { .. }) && b.value() == a.value() =>
                {
                    return Some(i - 1);
                }
                _ => {}
            }
        }
        None
    }

    while let Some(i) = find_list_pair(node) {
        let first = node.child(i).unwrap();
        let second = node.child(i + 1).unwrap();
        for child in second.children() {
            child.reparent_to(first);
        }
        second.remove_reparent(false);
    }

    for child in node.children() {
        merge_lists_with_same_indent(child);
    }
}

fn remove_paragraphs_from_simple_lists(node: Ref<Node>) {
    if matches!(node.value(), Node::List { .. }) {
        let all_have_single_p = node.children().all(|li| {
            li.children()
                .filter(|p| matches!(p.value(), Node::Paragraph))
                .count()
                == 1
        });
        if all_have_single_p {
            for li in node.children() {
                for p in li.children() {
                    if matches!(p.value(), Node::Paragraph) {
                        p.remove_reparent(true);
                        break;
                    }
                }
            }
        }
    }
    for child in node.children() {
        remove_paragraphs_from_simple_lists(child);
    }
}

fn can_contain_text_at(node: Ref<Node>) -> bool {
    fn can_contain_text(node: Ref<Node>) -> bool {
        match node.value() {
            Node::Empty
            | Node::Raw(_)
            | Node::Text(_)
            | Node::AltText(_)
            | Node::Joiner { .. }
            | Node::Separator
            | Node::List { .. }
            | Node::ListItem
            | Node::FootnoteReference(_) => false,
            Node::Paragraph
            | Node::Emphasis(_)
            | Node::Reference(_)
            | Node::Image(_)
            | Node::Heading(_)
            | Node::Pre(_)
            | Node::Code
            | Node::Quote
            | Node::DefinitionItem(_) => true,
        }
    }

    can_contain_text(node) || node.ancestors().any(|node| can_contain_text(node))
}

fn emphasis_level_at(node: Ref<Node>) -> u8 {
    fn emphasis_level(node: Ref<Node>) -> u8 {
        match node.value() {
            Node::Emphasis(strength) => strength,
            _ => 0,
        }
    }

    emphasis_level(node)
        + node
            .ancestors()
            .map(|node| emphasis_level(node))
            .sum::<u8>()
}

fn list_indent_at(node: Ref<Node>) -> Option<usize> {
    match node.value() {
        Node::List { ordered: _, indent } => Some(indent),
        _ => node.parent().and_then(|parent| list_indent_at(parent)),
    }
}
