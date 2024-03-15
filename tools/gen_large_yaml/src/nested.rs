use std::{cell::RefCell, rc::Rc};

use rand::{rngs::ThreadRng, Rng};

/// Create a deep object with the given amount of nodes.
pub fn create_deep_object<W: std::io::Write>(
    writer: &mut W,
    n_nodes: usize,
) -> std::io::Result<()> {
    let mut tree = Tree::new();
    for _ in 0..n_nodes {
        tree.push_node();
    }
    tree.write_to(writer)
}

/// An n-tree.
///
/// The algorithm used to generate a potentially deep object is to create a tree, one node at a
/// time, where each node is put as a child of a random existing node in the tree.
struct Tree {
    /// The tree-view of the tree.
    root: Rc<RefCell<Node>>,
    /// Array of all the nodes in the tree, including the root node.
    nodes: Vec<Rc<RefCell<Node>>>,
    /// The RNG state.
    rng: ThreadRng,
}

/// A node in a tree.
struct Node {
    /// All the children of the node.
    children: Vec<Rc<RefCell<Node>>>,
}

impl Tree {
    /// Create a new tree.
    fn new() -> Self {
        let root = Node::new_rc_refcell();
        Tree {
            root: root.clone(),
            nodes: vec![root],
            rng: rand::thread_rng(),
        }
    }

    /// Add a new node as a child of a random node in the tree.
    fn push_node(&mut self) {
        let new_node = Node::new_rc_refcell();
        let n_nodes = self.nodes.len();
        // Bias the nodes towards the end so that there is more nesting.
        let parent = &mut self.nodes[self.rng.gen_range((3 * n_nodes / 4)..n_nodes)];
        (**parent).borrow_mut().push_child(new_node.clone());
        self.nodes.push(new_node);
    }

    /// Write the YAML representation of the tree to `writer`.
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        (*self.root).borrow().write_to(writer, 0)
    }
}

impl Node {
    /// Create a new node.
    fn new() -> Self {
        Node { children: vec![] }
    }

    fn new_rc_refcell() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new()))
    }

    /// Append a child to the node.
    fn push_child(&mut self, child: Rc<RefCell<Self>>) {
        self.children.push(child);
    }

    /// Write the YAML representation of the node to `writer`.
    fn write_to<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        if self.children.is_empty() {
            write_n(writer, ' ', indent)?;
            writer.write_all(b"a: 1\n")?;
        } else {
            for (n, child) in self.children.iter().enumerate() {
                write_n(writer, ' ', indent)?;
                write_id_for_number(writer, n)?;
                writer.write_all(b":\n")?;
                (**child).borrow().write_to(writer, indent + 2)?;
            }
        }
        Ok(())
    }
}

/// Write `n` times `c` to `out`.
fn write_n<W: std::io::Write>(out: &mut W, c: char, n: usize) -> std::io::Result<()> {
    for _ in 0..n {
        write!(out, "{c}")?;
    }
    Ok(())
}

/// Create a valid identifier for the given number.
fn write_id_for_number<W: std::io::Write>(out: &mut W, mut n: usize) -> std::io::Result<()> {
    const DIGITS: &[u8] = b"_abcdefghijklmnopqrstuvwxyz";
    n += 1;
    while n > 0 {
        write!(out, "{}", DIGITS[n % DIGITS.len()] as char)?;
        n /= DIGITS.len();
    }
    Ok(())
}
