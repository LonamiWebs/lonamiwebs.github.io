use std::cell::RefCell;
use std::{fmt, mem};

pub struct Graph<T> {
    nodes: RefCell<Vec<Node<T>>>,
}

pub struct Node<T> {
    parent: usize,
    children: Vec<usize>,
    value: T,
}

pub struct NodeRef<'a, T> {
    arena: &'a Graph<T>,
    index: usize,
}

pub struct NodeChildren<'a, T> {
    parent: NodeRef<'a, T>,
    current: usize,
}

pub struct NodeAncestors<'a, T> {
    current: NodeRef<'a, T>,
}

impl<T> Graph<T> {
    pub fn new(root_value: T) -> Self {
        Self {
            nodes: RefCell::new(vec![Node {
                parent: 0,
                children: Vec::new(),
                value: root_value,
            }]),
        }
    }

    pub fn root(&self) -> NodeRef<T> {
        NodeRef {
            arena: self,
            index: 0,
        }
    }
}

impl<'a, T> NodeRef<'a, T> {
    pub fn append_child(&self, value: T) -> Self {
        let mut nodes = self.arena.nodes.borrow_mut();
        let index = nodes.len();
        nodes[self.index].children.push(index);
        nodes.push(Node {
            parent: self.index,
            children: Vec::new(),
            value,
        });
        Self {
            arena: self.arena,
            index,
        }
    }

    pub fn children(&self) -> NodeChildren<'a, T> {
        NodeChildren {
            parent: *self,
            current: 0,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.arena.nodes.borrow()[self.index].children.is_empty()
    }

    pub fn child_count(&self) -> usize {
        self.arena.nodes.borrow()[self.index].children.len()
    }

    pub fn ancestors(&self) -> NodeAncestors<'a, T> {
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
                arena: self.arena,
                index: parent,
            })
        }
    }

    pub fn up(&self) -> Self {
        let parent = self.arena.nodes.borrow()[self.index].parent;
        Self {
            arena: self.arena,
            index: parent,
        }
    }

    pub fn child(&self, i: usize) -> Option<Self> {
        if let Some(&j) = self.arena.nodes.borrow()[self.index].children.get(i) {
            Some(Self {
                arena: self.arena,
                index: j,
            })
        } else {
            None
        }
    }

    pub fn last_child(&self) -> Option<Self> {
        if let Some(&j) = self.arena.nodes.borrow()[self.index].children.last() {
            Some(Self {
                arena: self.arena,
                index: j,
            })
        } else {
            None
        }
    }

    pub fn root(&self) -> Self {
        Self {
            arena: self.arena,
            index: 0,
        }
    }

    pub fn set_value(&self, value: T) {
        let mut arena = self.arena.nodes.borrow_mut();
        arena[self.index].value = value;
    }

    pub fn reparent_to(&self, new_parent: NodeRef<'a, T>) {
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
    }
}

impl<T: Clone> NodeRef<'_, T> {
    pub fn value(&self) -> T {
        self.arena.nodes.borrow()[self.index].value.clone()
    }
}

impl<T: Clone + fmt::Debug> NodeRef<'_, T> {
    fn debug_at_nesting(&self, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blank = "";
        let has_children = self.child(0).is_some();
        if has_children {
            writeln!(f, "{blank: >i$}{:?} {{", self.value(), i = indent)?;
            for child in self.children() {
                child.debug_at_nesting(indent + 2, f)?;
            }
            writeln!(f, "{blank: >i$}}}", i = indent)
        } else {
            writeln!(f, "{blank: >i$}{:?} {{ }}", self.value(), i = indent)
        }
    }
}

impl<T> Clone for NodeRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NodeRef<'_, T> {}

impl<T: Clone + PartialEq> PartialEq for NodeRef<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value() && self.children().zip(other.children()).all(|(a, b)| a == b)
    }
}

impl<'a, T> Iterator for NodeChildren<'a, T> {
    type Item = NodeRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(child) = self.parent.child(self.current) {
            self.current += 1;
            Some(child)
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for NodeAncestors<'a, T> {
    type Item = NodeRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(parent) = self.current.parent() {
            self.current.index = parent.index;
            Some(parent)
        } else {
            None
        }
    }
}

impl<T: Clone + fmt::Debug> fmt::Debug for NodeRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_at_nesting(0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove() {
        let actual = Graph::new("");
        let a = actual.root().append_child("a");
        let b = a.append_child("b");
        let _ = b.append_child("c");

        b.remove_reparent(false);

        let expected = Graph::new("");
        let _ = expected.root().append_child("a");

        assert_eq!(actual.root(), expected.root());

        let actual = Graph::new("");
        let a = actual.root().append_child("a");
        let b = a.append_child("b");
        let _ = b.append_child("c");

        b.remove_reparent(true);

        let expected = Graph::new("");
        let a = expected.root().append_child("a");
        let _ = a.append_child("c");

        assert_eq!(actual.root(), expected.root());
    }

    #[test]
    fn test_reparent() {
        let actual = Graph::new("");
        let a = actual.root().append_child("a");
        let ab1 = a.append_child("ab1");
        let _ = ab1.append_child("ab1c1");
        let _ = ab1.append_child("ab1c2");
        let ab2 = a.append_child("ab2");
        let _ = ab2.append_child("ab2c1");
        let _ = ab2.append_child("ab2c2");

        for child in ab2.children() {
            child.reparent_to(ab1);
        }
        ab2.remove_reparent(false);

        let expected = Graph::new("");
        let a = expected.root().append_child("a");
        let ab1 = a.append_child("ab1");
        let _ = ab1.append_child("ab1c1");
        let _ = ab1.append_child("ab1c2");
        let _ = ab1.append_child("ab2c1");
        let _ = ab1.append_child("ab2c2");

        assert_eq!(actual.root(), expected.root());
    }
}
