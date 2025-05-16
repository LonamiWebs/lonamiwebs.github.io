use std::cell::RefCell;
use std::rc::Rc;
use std::{fmt, mem};

pub struct NodeArena<'t> {
    nodes: RefCell<Vec<Node<'t>>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NodeContent<'t> {
    Empty,
    Raw(&'t [u8]),
    Text(&'t [u8]),
    Paragraph,
    Joiner { inline: bool },
    Separator,
    List { ordered: bool, indent: usize },
    ListItem,
    Emphasis(u8),
    Reference,
    Heading(u8),
    Pre(&'t [u8]),
    Code,
    Quote,
}

pub struct Node<'t> {
    parent: usize,
    children: Vec<usize>,
    content: NodeContent<'t>,
}

pub struct NodeRef<'t> {
    arena: Rc<NodeArena<'t>>,
    index: usize,
}

pub struct NodeChildren<'t> {
    parent: NodeRef<'t>,
    current: usize,
}

pub struct NodeAncestors<'t> {
    current: NodeRef<'t>,
}

impl<'t> NodeArena<'t> {
    pub fn new_root() -> NodeRef<'t> {
        let this = Self {
            nodes: RefCell::new(vec![Node {
                parent: 0,
                children: Vec::new(),
                content: NodeContent::Empty,
            }]),
        };
        NodeRef {
            arena: Rc::new(this),
            index: 0,
        }
    }
}

impl<'t> NodeRef<'t> {
    pub fn append_child(&self, content: NodeContent<'t>) -> Self {
        let mut nodes = self.arena.nodes.borrow_mut();
        let index = nodes.len();
        nodes[self.index].children.push(index);
        nodes.push(Node {
            parent: self.index,
            children: Vec::new(),
            content,
        });
        Self {
            arena: Rc::clone(&self.arena),
            index,
        }
    }

    pub fn children(&self) -> NodeChildren<'t> {
        NodeChildren {
            parent: Self::clone(self),
            current: 0,
        }
    }

    pub fn child_count(&self) -> usize {
        self.arena.nodes.borrow()[self.index].children.len()
    }

    pub fn ancestors(&self) -> NodeAncestors<'t> {
        NodeAncestors {
            current: Self::clone(self),
        }
    }

    pub fn parent(&self) -> Option<Self> {
        let parent = self.arena.nodes.borrow()[self.index].parent;
        if parent == self.index {
            None
        } else {
            Some(Self {
                arena: Rc::clone(&self.arena),
                index: parent,
            })
        }
    }

    pub fn up(&self) -> Self {
        let parent = self.arena.nodes.borrow()[self.index].parent;
        Self {
            arena: Rc::clone(&self.arena),
            index: parent,
        }
    }

    pub fn child(&self, i: usize) -> Option<Self> {
        if let Some(&j) = self.arena.nodes.borrow()[self.index].children.get(i) {
            Some(Self {
                arena: Rc::clone(&self.arena),
                index: j,
            })
        } else {
            None
        }
    }

    pub fn last_child(&self) -> Option<Self> {
        if let Some(&j) = self.arena.nodes.borrow()[self.index].children.last() {
            Some(Self {
                arena: Rc::clone(&self.arena),
                index: j,
            })
        } else {
            None
        }
    }

    pub fn root(&self) -> Self {
        Self {
            arena: Rc::clone(&self.arena),
            index: 0,
        }
    }

    pub fn reparent_to(&self, new_parent: &NodeRef) {
        let mut arena = self.arena.nodes.borrow_mut();
        let old_parent = arena[self.index].parent;
        if let Some(i) = arena[old_parent]
            .children
            .iter()
            .position(|&n| n == self.index)
        {
            arena[old_parent].children.remove(i);
        }
        arena[self.index].parent = new_parent.index;
        arena[new_parent.index].children.push(self.index);
    }

    pub fn remove_reparent(&self, reparent: bool) {
        let mut arena = self.arena.nodes.borrow_mut();
        let parent = arena[self.index].parent;
        if let Some(i) = arena[parent].children.iter().position(|&n| n == self.index) {
            arena[parent].children.remove(i);
        }
        let children = mem::take(&mut arena[self.index].children);
        if reparent {
            arena[parent].children.extend_from_slice(&children);
        }
        arena[self.index].parent = self.index;
        arena[self.index].content = NodeContent::Empty;
    }

    pub fn clone(this: &Self) -> Self {
        Self {
            arena: Rc::clone(&this.arena),
            index: this.index,
        }
    }

    pub fn content(&self) -> NodeContent<'t> {
        self.arena.nodes.borrow()[self.index].content
    }

    fn debug_at_nesting(&self, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blank = "";
        let has_children = self.child(0).is_some();
        if has_children {
            write!(f, "{blank: >i$}{:?} {{\n", self.content(), i = indent)?;
            for child in self.children() {
                child.debug_at_nesting(indent + 2, f)?;
            }
            write!(f, "{blank: >i$}}}\n", i = indent)
        } else {
            write!(f, "{blank: >i$}{:?} {{ }}\n", self.content(), i = indent)
        }
    }
}

impl PartialEq for NodeRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.content() == other.content()
            && self.children().zip(other.children()).all(|(a, b)| a == b)
    }
}

impl<'t> Iterator for NodeChildren<'t> {
    type Item = NodeRef<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(child) = self.parent.child(self.current) {
            self.current += 1;
            Some(child)
        } else {
            None
        }
    }
}

impl<'t> Iterator for NodeAncestors<'t> {
    type Item = NodeRef<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(parent) = self.current.parent() {
            self.current.index = parent.index;
            Some(parent)
        } else {
            None
        }
    }
}

impl fmt::Debug for NodeRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_at_nesting(0, f)
    }
}

impl fmt::Debug for NodeContent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "."),
            Self::Raw(text) => write!(f, "Raw({})", String::from_utf8_lossy(text)),
            Self::Text(text) => write!(f, "Text({})", String::from_utf8_lossy(text)),
            Self::Paragraph => write!(f, "Paragraph"),
            Self::Joiner { inline } => write!(f, "Joiner(inline={inline})"),
            Self::Separator => write!(f, "Separator"),
            Self::List { ordered, indent } => write!(f, "List(ordered={ordered}, indent={indent})"),
            Self::ListItem => write!(f, "ListItem"),
            Self::Emphasis(strength) => write!(f, "Emphasis({strength})"),
            Self::Reference => write!(f, "Reference"),
            Self::Heading(level) => write!(f, "Heading({level})"),
            Self::Pre(text) => write!(f, "Pre({})", String::from_utf8_lossy(text)),
            Self::Code => write!(f, "Code"),
            Self::Quote => write!(f, "Quote"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove() {
        let actual = NodeArena::new_root();
        let a = actual.append_child(NodeContent::Text(b"a"));
        let b = a.append_child(NodeContent::Text(b"b"));
        let _ = b.append_child(NodeContent::Text(b"c"));

        b.remove_reparent(false);

        let expected = NodeArena::new_root();
        let _ = expected.append_child(NodeContent::Text(b"a"));

        assert_eq!(actual, expected);

        let actual = NodeArena::new_root();
        let a = actual.append_child(NodeContent::Text(b"a"));
        let b = a.append_child(NodeContent::Text(b"b"));
        let _ = b.append_child(NodeContent::Text(b"c"));

        b.remove_reparent(true);

        let expected = NodeArena::new_root();
        let a = expected.append_child(NodeContent::Text(b"a"));
        let _ = a.append_child(NodeContent::Text(b"c"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_reparent() {
        let actual = NodeArena::new_root();
        let a = actual.append_child(NodeContent::Text(b"a"));
        let ab1 = a.append_child(NodeContent::Text(b"ab1"));
        let _ = ab1.append_child(NodeContent::Text(b"ab1c1"));
        let _ = ab1.append_child(NodeContent::Text(b"ab1c2"));
        let ab2 = a.append_child(NodeContent::Text(b"ab2"));
        let _ = ab2.append_child(NodeContent::Text(b"ab2c1"));
        let _ = ab2.append_child(NodeContent::Text(b"ab2c2"));

        for child in ab2.children() {
            child.reparent_to(&ab1);
        }
        ab2.remove_reparent(false);

        let expected = NodeArena::new_root();
        let a = expected.append_child(NodeContent::Text(b"a"));
        let ab1 = a.append_child(NodeContent::Text(b"ab1"));
        let _ = ab1.append_child(NodeContent::Text(b"ab1c1"));
        let _ = ab1.append_child(NodeContent::Text(b"ab1c2"));
        let _ = ab1.append_child(NodeContent::Text(b"ab2c1"));
        let _ = ab1.append_child(NodeContent::Text(b"ab2c2"));

        assert_eq!(actual, expected);
    }
}
