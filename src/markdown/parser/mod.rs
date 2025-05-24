mod node;
#[cfg(test)]
mod tests;

pub use node::{NodeArena, NodeContent, NodeRef};

use super::{Token, Tokens};

pub fn parse(tokens: Tokens) -> NodeArena {
    let arena = NodeArena::new();
    let mut cursor = arena.root();

    let mut last_token = Token::Break {
        hard: true,
        indent: 0,
    };

    for token in tokens {
        match token {
            Token::Text(text) => {
                if !can_contain_text_at(cursor) {
                    cursor = cursor.append_child(NodeContent::Paragraph);
                }
                cursor.append_child(NodeContent::Text(text));
            }
            Token::Raw(text) => {
                cursor.append_child(NodeContent::Raw(text));
            }
            Token::Meta(_) => {}
            Token::Separator(_) => {
                cursor = cursor.root();
                cursor.append_child(NodeContent::Separator);
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
                            while !matches!(cursor.content(), NodeContent::ListItem) {
                                cursor = cursor.up();
                            }
                        } else {
                            // Escape list until we reach same or no indent. Will later merge as needed.
                            while !matches!(cursor.content(), NodeContent::List { .. }) {
                                cursor = cursor.up();
                            }
                            cursor = cursor.up();
                        }
                        if indent >= last_indent {
                            break;
                        }
                    }
                }
                cursor = cursor.append_child(NodeContent::List { ordered, indent });
                cursor = cursor.append_child(NodeContent::ListItem);
                cursor = cursor.append_child(NodeContent::Paragraph);
            }
            Token::Emphasis(strength) => {
                if !can_contain_text_at(cursor) {
                    cursor = cursor.append_child(NodeContent::Paragraph);
                }

                let emphasis_level = emphasis_level_at(cursor);

                if strength == emphasis_level || emphasis_level + strength > 3 {
                    cursor = cursor.up();
                } else {
                    cursor = cursor.append_child(NodeContent::Emphasis(strength));
                }
            }
            Token::BeginReference { .. } => {
                // todo!()
                cursor = cursor.append_child(NodeContent::Reference);
            }
            Token::EndReference { .. } => {
                // todo!()
                cursor = cursor.up();
            }
            Token::Heading(level) => {
                // Deliberately not supporting titles in lists for simplicity.
                cursor = cursor.root();
                cursor = cursor.append_child(NodeContent::Heading(level));
            }
            Token::Fence { lang, text } => {
                let pre = cursor.append_child(NodeContent::Pre(lang));
                pre.append_child(NodeContent::Text(text));
            }
            Token::Code(text) => {
                let code = cursor.append_child(NodeContent::Code);
                code.append_child(NodeContent::Text(text));
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
                        while !matches!(cursor.content(), NodeContent::ListItem) {
                            cursor = cursor.up();
                        }
                        cursor = cursor.append_child(NodeContent::Paragraph);
                    } else {
                        //
                        cursor = cursor.root();
                    }
                } else {
                    cursor.append_child(NodeContent::Joiner {
                        inline: indent == 0,
                    });
                }
            }
        }
        last_token = token;
    }

    let root = cursor.root();
    remove_empty_paragraphs(root);
    trim_joiners(root);
    merge_lists_with_same_indent(root);
    remove_paragraphs_from_simple_lists(root);
    arena
}

fn remove_empty_paragraphs(node: NodeRef) {
    if matches!(node.content(), NodeContent::Paragraph) && node.child_count() == 0 {
        node.remove_reparent(false);
    } else {
        for child in node.children() {
            remove_empty_paragraphs(child);
        }
    }
}

fn trim_joiners(node: NodeRef) {
    while let Some(child) = node.last_child() {
        match child.content() {
            NodeContent::Joiner { .. } => child.remove_reparent(false),
            _ => break,
        }
    }
    while let Some(child) = node.child(0) {
        match child.content() {
            NodeContent::Joiner { .. } => child.remove_reparent(false),
            _ => break,
        }
    }
    for child in node.children() {
        trim_joiners(child);
    }
}

fn merge_lists_with_same_indent(node: NodeRef) {
    fn find_list_pair(node: NodeRef) -> Option<usize> {
        for i in 1..node.child_count() {
            match node.child(i - 1).zip(node.child(i)) {
                Some((a, b))
                    if matches!(a.content(), NodeContent::List { .. })
                        && b.content() == a.content() =>
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

fn remove_paragraphs_from_simple_lists(node: NodeRef) {
    if matches!(node.content(), NodeContent::List { .. }) {
        let all_have_single_p = node.children().all(|li| {
            li.children()
                .filter(|p| matches!(p.content(), NodeContent::Paragraph))
                .count()
                == 1
        });
        if all_have_single_p {
            for li in node.children() {
                for p in li.children() {
                    if matches!(p.content(), NodeContent::Paragraph) {
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

fn can_contain_text_at(node: NodeRef) -> bool {
    fn can_contain_text(node: NodeRef) -> bool {
        match node.content() {
            NodeContent::Empty
            | NodeContent::Raw(_)
            | NodeContent::Text(_)
            | NodeContent::Joiner { .. }
            | NodeContent::Separator
            | NodeContent::List { .. }
            | NodeContent::ListItem => false,
            NodeContent::Paragraph
            | NodeContent::Emphasis(_)
            | NodeContent::Reference
            | NodeContent::Heading(_)
            | NodeContent::Pre(_)
            | NodeContent::Code
            | NodeContent::Quote => true,
        }
    }

    can_contain_text(node) || node.ancestors().any(|node| can_contain_text(node))
}

fn emphasis_level_at(node: NodeRef) -> u8 {
    fn emphasis_level(node: NodeRef) -> u8 {
        match node.content() {
            NodeContent::Emphasis(strength) => strength,
            _ => 0,
        }
    }

    emphasis_level(node)
        + node
            .ancestors()
            .map(|node| emphasis_level(node))
            .sum::<u8>()
}

fn list_indent_at(node: NodeRef) -> Option<usize> {
    match node.content() {
        NodeContent::List { ordered: _, indent } => Some(indent),
        _ => node.parent().and_then(|parent| list_indent_at(parent)),
    }
}
