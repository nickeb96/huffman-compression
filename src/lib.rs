
#![feature(optin_builtin_traits)]
#![feature(box_syntax)]
#![feature(negative_impls)]

extern crate indextree;
extern crate bit_vec;


use std::collections::{BinaryHeap, HashMap, VecDeque};

use indextree::{Arena, NodeId};
use bit_vec::BitVec;


mod tree_serialization;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Byte {
    Normal(u8),
    EndOfFile,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Node {
    Branch,
    Leaf(Byte),
}

#[derive(Debug)]
struct Tree {
    root: NodeId,
    weight: usize,
}

impl Eq for Tree {}

impl PartialEq for Tree {
    fn eq(&self, other: &Tree) -> bool {
        self.weight == other.weight
    }
}

impl PartialOrd for Tree {
    fn partial_cmp(&self, other: &Tree) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Implement Ord for tree so that it is reversed to make MaxBinaryHeap a MinBinaryHeap
impl Ord for Tree {
    fn cmp(&self, other: &Tree) -> std::cmp::Ordering {
        other.weight.cmp(&self.weight)
    }
}


// Returning large objects is okay because of RVO
pub fn make_byte_weights(bytes: &[u8]) -> [usize; 256] {
    let mut ret = [0; 256];
    for &byte in bytes {
        ret[byte as usize] += 1;
    }
    ret
}


pub fn make_huffman_tree(weights: &[usize; 256]) -> (NodeId, Arena<Node>) {
    let mut arena = Arena::new();
    let mut queue = BinaryHeap::new();

    // put bytes that show up in the file into the priority queue, ignore bytes
    // that don't exist in the file
    for (byte, &weight) in weights.iter().enumerate() {
        if weight > 0 {
            queue.push(Tree {
                root: arena.new_node(Node::Leaf(Byte::Normal(byte as u8))),
                weight: weight,
            });
        }
    }

    queue.push(Tree {
        root: arena.new_node(Node::Leaf(Byte::EndOfFile)),
        weight: 0,
    });

    // reduce forest into a singular tree
    while queue.len() > 1 {
        let first = queue.pop().unwrap();
        let second = queue.pop().unwrap();
        let root = arena.new_node(Node::Branch);
        root.append(first.root, &mut arena);
        root.append(second.root, &mut arena);
        queue.push(Tree {
            root: root,
            weight: first.weight + second.weight,
        });
    }

    let tree = queue.pop().unwrap();

    (tree.root, arena)
}


fn get_depth(leaf: NodeId, arena: &Arena<Node>) -> usize {
    let mut depth = 0;
    let mut ancestor_iter = leaf.ancestors(&arena);
    // skip the node itself because indextree includes it in its list of ancestors
    ancestor_iter.next();

    while let Some(_) = ancestor_iter.next() {
        depth += 1;
    }

    depth
}


/// Turn the huffman tree into a map for faster lookup of leaves
pub fn build_encoding_map(root: NodeId, arena: &Arena<Node>) -> HashMap<Byte, BitVec<u8>> {
    let mut ret = HashMap::new();
    let mut node_stack = vec![root];
    let mut bitvec_stack = vec![BitVec::<u8>::default()];

    // depth first traversal of huffman tree
    while let Some(node) = node_stack.pop() {
        match arena[node].data {
            Node::Leaf(byte) => {
                let encoding = bitvec_stack.pop().unwrap();
                ret.insert(byte, encoding);
            }
            Node::Branch => {
                let mut child_iter = node.children(arena);
                let left_child = child_iter.next().unwrap();
                let right_child = child_iter.next().unwrap();
                let mut left_bitvec = bitvec_stack.pop().unwrap();
                let mut right_bitvec = left_bitvec.clone();
                // left path is 0 and right path is 1
                left_bitvec.push(false);
                right_bitvec.push(true);
                node_stack.push(left_child);
                bitvec_stack.push(left_bitvec);
                node_stack.push(right_child);
                bitvec_stack.push(right_bitvec);
            }
        }
    }

    ret
}


pub fn decode_bitvec(root: NodeId, arena: &Arena<Node>, bits: &BitVec) -> Vec<u8> {
    let mut ret = Vec::new();
    let mut iter = bits.iter();
    let mut node = root;

    while let Some(flag) = iter.next() {
        if arena[node].data == Node::Branch {
            let mut children = node.children(&arena);
            let left = children.next().unwrap();
            let right = children.next().unwrap();
            if flag {
                node = right;
            }
            else {
                node = left;
            }
        }
        if let Node::Leaf(byte) = arena[node].data {
            match byte {
                Byte::Normal(b) => {
                    ret.push(b);
                    node = root;
                }
                Byte::EndOfFile => {
                    break;
                }
            }
        }
    }

    ret
}


pub fn print_tree(root: NodeId, arena: &Arena<Node>) {
    // awful workaround to get max depth of tree because indextree does not
    // support simple iteration across all nodes
    let max_depth = {
        let mut depth = 0isize;
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            stack.extend(node.children(arena));
            let temp = get_depth(node, arena) as isize;
            if temp > depth {
                depth = temp;
            }
        }
        depth
    };

    let branch_char = '\u{2144}';
    let empty_char = '\u{2205}';
    let eof_char = '\u{2020}';
    let new_line_char = '\u{2424}';
    let visible_space_char = '\u{2423}';
    let spacing = |depth| (2isize.pow((max_depth - depth + 1) as u32) - 1) as usize;
    let indentation = |depth| (2isize.pow((max_depth - depth) as u32) - 1) as usize;
    let mut last_depth = -1isize;
    let mut queue = VecDeque::from(vec![(Some(root), 0)]);

    while let Some((maybe_node, depth)) = queue.pop_front() {
        if depth > max_depth {
            break;
        } else if depth > last_depth {
            print!("\n{:width$}", "", width=indentation(depth));
            last_depth = depth;
        }
        if let Some(node) = maybe_node {
            let symbol = match arena[node].data {
                Node::Branch => branch_char,
                // special char for space
                Node::Leaf(Byte::Normal(0x20)) => visible_space_char,
                // special char for newline
                Node::Leaf(Byte::Normal(0x0a)) => new_line_char,
                Node::Leaf(Byte::Normal(byte)) => byte as char,
                Node::Leaf(Byte::EndOfFile) => eof_char,
            };
            print!("{}{:width$}", symbol, "", width=spacing(depth));
            // Huffman tree nodes are guaranteed to have either 2 children or 0
            // childern, never 1.
            let mut child_iter = node.children(arena);
            // Push children nodes, even if a node does not have any in order
            // to maintain spacing.
            if let (Some(left), Some(right)) = (child_iter.next(), child_iter.next()) {
                queue.push_back((Some(left), depth + 1));
                queue.push_back((Some(right), depth + 1));
            } else {
                queue.push_back((None, depth + 1));
                queue.push_back((None, depth + 1));
            }
        } else {
            print!("{}{:width$}", empty_char, "", width=spacing(depth));
            queue.push_back((None, depth + 1));
            queue.push_back((None, depth + 1));
        }
    }
    print!("\n");
}


pub fn encode_bitvec(bytes: &[u8], encoding_map: HashMap<Byte, BitVec<u8>>) -> BitVec {
    let mut ret = BitVec::new();
    for byte in bytes.iter() {
        ret.extend(encoding_map[&Byte::Normal(*byte)].iter());
    }
    // write EOF special marker as last bit string to account for not all
    // compressed files being a multiple of 8 bits
    ret.extend(encoding_map[&Byte::EndOfFile].iter());
    ret
}

// Encoded format is header length as a string, then a null byte, then the
// header contents, then encoded file contents.
pub fn encode_file_contents(bytes: &[u8]) -> Vec<u8> {
    let byte_weights = make_byte_weights(bytes);
    let (root, arena) = make_huffman_tree(&byte_weights);
    let encoding_map = build_encoding_map(root, &arena);
    let mut serialized_tree = tree_serialization::serialize_tree(root, &arena);
    let header_length = serialized_tree.len();
    let mut ret = Vec::new();
    ret.extend(header_length.to_string().as_bytes());
    ret.push(b'\0');
    ret.append(&mut serialized_tree);

    let encoded_bitvec = encode_bitvec(bytes, encoding_map);
    ret.extend(encoded_bitvec.to_bytes());

    ret
}


#[derive(Debug)]
pub struct DecodingError(Box<dyn std::error::Error>);

impl std::error::Error for DecodingError {}

impl std::fmt::Display for DecodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


// Workaround for implementing `From<Error> for DecodingError` without collision
// with core blanket implementation of `impl<T> From<T> for T`
pub auto trait NotDecodingError {}  // auto implement for all types
impl !NotDecodingError for DecodingError {}  // "unimplement" for DecodingError

// only impl From<E> for DecodingError where E is not DecodingError
impl<E: 'static + NotDecodingError + std::error::Error> From<E> for DecodingError {
    fn from(error: E) -> DecodingError {
        DecodingError(Box::new(error))
    }
}


// File format is header length, then null byte, then header contents,
// then compressed file contents.  Header contains the serialized tree.
pub fn decode_file_contents(bytes: &[u8]) -> Result<Vec<u8>, DecodingError> {
    let size_of_header_length = bytes.iter().take_while(|&&byte| byte != 0).count();
    let start_of_header: usize = size_of_header_length + 1;

    let header_length = std::str::from_utf8(&bytes[..size_of_header_length])?;
    let header_length: usize = header_length.parse()?;
    let end_of_header = start_of_header + header_length;

    let serialized_tree = &bytes[start_of_header..end_of_header];
    let (root, arena) = tree_serialization::deserialize_tree(serialized_tree);
    let compressed_contents = &bytes[end_of_header..];
    let decompressed_contents = decode_bitvec(root, &arena, &BitVec::from_bytes(compressed_contents));

    Ok(decompressed_contents)
}

