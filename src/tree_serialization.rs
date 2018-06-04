
use indextree::{Arena, NodeId};

use super::{Byte, Node};


// TODO: rewrite
pub fn serialize_tree(root: NodeId, arena: &Arena<Node>) -> Vec<u8> {
    let mut ret = Vec::new();
    let mut stack = vec![root];

    while let Some(node) = stack.pop() {
        if let Node::Leaf(byte) = arena[node].data {
            if let Byte::Normal(b) = byte {
                ret.push(2);
                ret.push(b);
            }
            else {
                ret.push(3);
            }
        }
        else {
            ret.push(1);
            let mut children = node.children(&arena);
            let left = children.next().unwrap();
            let right = children.next().unwrap();
            stack.push(right);
            stack.push(left);
        }
    }

    ret
}


// TODO: rewrite
pub fn deserialize_tree(bytes: &[u8]) -> (NodeId, Arena<Node>) {
    let mut arena = Arena::new();
    let root = arena.new_node(Node::Branch);
    let mut node = root;

    let mut i = 1;
    while i < bytes.len() {
        while node.children(&arena).count() == 2 {
            let mut ancestors = node.ancestors(&arena);
            ancestors.next(); // indextree stores the node itself in it's list of ancestors so we need to skip it
            if let Some(parent) = ancestors.next() {
                node = parent;
            }
            else {
                unreachable!();
            }
        }
        if bytes[i] == 1 {
            let branch = arena.new_node(Node::Branch);
            node.append(branch, &mut arena);
            node = branch;
        }
        else if bytes[i] == 2 {
            i += 1;
            let leaf = arena.new_node(Node::Leaf(Byte::Normal(bytes[i])));
            node.append(leaf, &mut arena);
        }
        else if bytes[i] == 3 {
            let leaf = arena.new_node(Node::Leaf(Byte::EndOfFile));
            node.append(leaf, &mut arena);
        }
        i += 1;
    }

    (root, arena)
}


#[allow(unused)]
pub fn deserialize_tree_alternative(bytes: &[u8]) -> (NodeId, Arena<Node>) {
    let mut arena = Arena::new();
    let mut root: Option<NodeId> = None;
    let mut node = root;

    let mut i = 0;
    while i < bytes.len() {
        if node.is_some() {
            while node.unwrap().children(&arena).count() == 2 {
                let mut ancestors = node.unwrap().ancestors(&arena);
                ancestors.next(); // indextree stores the node itself in it's list of ancestors so we need to skip it
                node = ancestors.next();
            }
        }
        if bytes[i] == 1 {
            let branch = arena.new_node(Node::Branch);
            if node.is_some() {
                node.unwrap().append(branch, &mut arena);
                node = Some(branch);
            }
            else {
                root = Some(branch);
                node = root;
            }
        }
        else if bytes[i] == 2 {
            i += 1;
            let leaf = arena.new_node(Node::Leaf(Byte::Normal(bytes[i])));
            if node.is_some() {
                node.unwrap().append(leaf, &mut arena);
            }
            else {
                root = Some(leaf);
                node = root;
            }
        }
        else if bytes[i] == 3 {
            let leaf = arena.new_node(Node::Leaf(Byte::EndOfFile));
            if node.is_some() {
                node.unwrap().append(leaf, &mut arena);
            }
            else {
                root = Some(leaf);
                node = root;
            }
        }
        i += 1;
    }

    (root.unwrap(), arena)
}


