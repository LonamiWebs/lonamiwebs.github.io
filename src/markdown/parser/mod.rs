mod node;
#[cfg(test)]
mod tests;

pub use node::Node;

use super::{Token, Tokens, Tokens3Window};
use crate::collections::{Graph, GraphNodeRef as Ref};

pub struct ParseResult<'t> {
    pub additional_style: Vec<u8>,
    pub ast: Graph<Node<'t>>,
}

pub fn parse(tokens: Tokens) -> ParseResult {
    let mut additional_style = Vec::new();
    let arena = Graph::new(Node::Empty);
    let mut cursor = arena.root();

    let mut nodes_with_references_to_resolve = Vec::new();

    for (prev, token, next) in Tokens3Window::new(tokens) {
        match token {
            Token::Text(text) => {
                if let Some(Token::Indent(indent)) = prev {
                    while let Some(last_indent) = list_indent_at(cursor) {
                        if indent > last_indent {
                            break;
                        }
                        cursor = cursor.up();
                    }
                }
                if matches!(cursor.value(), Node::Image(_)) {
                    cursor.append_child(Node::AltText(text));
                } else if text == b"\n" {
                    cursor.append_child(Node::Joiner { inline: false });
                } else {
                    if !is_in_text_container_at(cursor) {
                        cursor = cursor.append_child(Node::Paragraph);
                    }
                    cursor.append_child(Node::Text(text));
                }
            }
            Token::Raw(text) => {
                if text.starts_with(b"<style>") {
                    additional_style.extend_from_slice(text);
                } else {
                    let standalone_line =
                        matches!(prev, None | Some(Token::Break { .. } | Token::Indent(_)))
                            && matches!(next, None | Some(Token::Break { .. }));

                    if !standalone_line && !is_in_text_container_at(cursor) {
                        cursor = cursor.append_child(Node::Paragraph);
                    }
                    cursor.append_child(Node::Raw(text));
                }
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
                let indent = match prev {
                    Some(Token::Indent(i)) => i,
                    _ => 0,
                };
                if list_indent_at(cursor).is_none() {
                    while is_in_text_container_at(cursor) {
                        cursor = cursor.up();
                    }
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
                cursor = cursor
                    .append_child(Node::List { ordered, indent })
                    .append_child(Node::ListItem)
                    .append_child(Node::Paragraph);
            }
            Token::Indent(_) => {}
            Token::Emphasis(strength) => {
                if !is_in_text_container_at(cursor) {
                    cursor = cursor.append_child(Node::Paragraph);
                }

                let emphasis_level = emphasis_level_at(cursor);

                if strength == emphasis_level || emphasis_level + strength > 3 {
                    cursor = cursor.up();
                } else {
                    cursor = cursor.append_child(Node::Emphasis(strength));
                }
            }
            Token::Deleted => {
                if !is_in_text_container_at(cursor) {
                    cursor = cursor.append_child(Node::Paragraph);
                }

                let mut open = true;
                while matches!(cursor.value(), Node::Deleted)
                    || cursor
                        .ancestors()
                        .any(|node| matches!(node.value(), Node::Deleted))
                {
                    cursor = cursor.up();
                    open = false;
                }

                if open {
                    cursor = cursor.append_child(Node::Deleted);
                }
            }
            Token::BeginReference { bang } => {
                if !is_in_text_container_at(cursor) {
                    cursor = cursor.append_child(Node::Paragraph);
                }
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
                        Node::Reference(_) => {
                            if lazy && uri.starts_with(b"^") {
                                cursor.set_value(Node::FootnoteReference(&uri[1..]));
                                while let Some(child) = cursor.child(0) {
                                    child.remove_reparent(false);
                                }
                            } else {
                                cursor.set_value(Node::Reference(uri))
                            }
                        }
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
                cursor = cursor.root().append_child(Node::Heading(level));
            }
            Token::Fence { lang, text } => {
                cursor = cursor.root();
                cursor
                    .append_child(Node::Pre(lang))
                    .append_child(Node::Text(text));
            }
            Token::Code(text) => {
                if !is_in_text_container_at(cursor) {
                    cursor = cursor.append_child(Node::Paragraph);
                }
                cursor
                    .append_child(Node::Code)
                    .append_child(Node::Text(text));
            }
            Token::Quote => {
                if !is_in_quote_at(cursor) {
                    cursor = cursor.append_child(Node::Quote);
                }
            }
            Token::Break { hard } => {
                let indent = match next {
                    Some(Token::Indent(i)) => i,
                    _ => 0,
                };
                if hard {
                    if is_in_quote_at(cursor) {
                        while is_in_quote_at(cursor) {
                            cursor = cursor.up();
                        }
                    } else if list_indent_at(cursor).is_some() && indent > 0 {
                        while !matches!(cursor.value(), Node::ListItem) {
                            cursor = cursor.up();
                        }
                        cursor = cursor.append_child(Node::Paragraph);
                    } else {
                        cursor = cursor.root();
                    }
                } else {
                    match cursor.last_child() {
                        Some(child) if matches!(child.value(), Node::Joiner { .. }) => {
                            child.remove_reparent(false);
                            while is_in_text_container_at(cursor) {
                                cursor = cursor.up();
                            }
                        }
                        _ => {
                            if is_in_text_container_at(cursor) {
                                cursor.append_child(Node::Joiner {
                                    inline: indent == 0 || list_indent_at(cursor).is_some(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let root = cursor.root();
    resolve_references(root, nodes_with_references_to_resolve);
    remove_empty_paragraphs(root);
    trim_joiners(root);
    merge_lists_with_same_indent(root);
    remove_paragraphs_from_simple_lists(root);

    ParseResult {
        additional_style,
        ast: arena,
    }
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
                } else {
                    // On missing definition, restore assumed original formatting.
                    // There's no need to flatten the nested text tags either.
                    node.set_value(Node::Text(b"!["));
                    node.append_child(Node::Text(b"]"));
                }
            }
            Node::Reference(identifier) => {
                if let Some(value) = find_definition(root, identifier) {
                    node.set_value(Node::Reference(value));
                } else {
                    node.set_value(Node::Text(b"["));
                    node.append_child(Node::Text(b"]"));
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
    'paragraph_joiners: loop {
        for i in 2..node.child_count() {
            match node.child(i - 2).zip(node.child(i - 1)).zip(node.child(i)) {
                Some(((a, b), c))
                    if is_text_container(a)
                        && matches!(b.value(), Node::Joiner { .. })
                        && is_text_container(c) =>
                {
                    b.remove_reparent(false);
                    continue 'paragraph_joiners;
                }
                _ => {}
            }
        }
        break;
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

fn is_text_container(node: Ref<Node>) -> bool {
    match node.value() {
        Node::Empty
        | Node::Raw(_)
        | Node::Text(_)
        | Node::AltText(_)
        | Node::Image(_)
        | Node::Joiner { .. }
        | Node::Separator
        | Node::List { .. }
        | Node::ListItem
        | Node::Emphasis(_)
        | Node::Deleted
        | Node::Reference(_)
        | Node::Code
        | Node::Quote
        | Node::FootnoteReference(_) => false,
        Node::Paragraph | Node::Heading(_) | Node::Pre(_) | Node::DefinitionItem(_) => true,
    }
}

fn is_in_text_container_at(node: Ref<Node>) -> bool {
    is_text_container(node) || node.ancestors().any(|node| is_text_container(node))
}

fn is_in_quote_at(node: Ref<Node>) -> bool {
    matches!(node.value(), Node::Quote)
        || node
            .ancestors()
            .any(|node| matches!(node.value(), Node::Quote))
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
