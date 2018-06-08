
extern crate huffman_compression as huffman;

use std::fs::File;
use std::io::prelude::*;

use huffman::{encode_file_contents, decode_file_contents};

fn read_file_contents(file_name: &str) -> Vec<u8> {
    let mut file = File::open(file_name).expect("could not open file");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).expect("error reading file");
    contents
}

fn write_to_file(file_name: &str, contents: &[u8]) {
    let mut file = File::create(file_name).expect("could not create file");
    file.write_all(contents).expect("error writing to file");
}

fn main() {
    let contents = read_file_contents("examples/test.txt");
    println!("size of original file: {} bytes", contents.len());

    let encoded = encode_file_contents(&contents);
    write_to_file("examples/test.txt.encoded", &encoded);
    println!("size of encoded file: {} bytes", encoded.len());

    let decoded = decode_file_contents(&encoded).expect("error decoding file");
    write_to_file("examples/output.txt", &decoded);
    println!("size of decoded file: {} bytes", decoded.len());
}
