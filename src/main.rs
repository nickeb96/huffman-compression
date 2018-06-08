
extern crate huffman_compression as huffman;

use huffman::*;

fn pretty_print_bytes(bytes: &[u8]) {
    for chunk in bytes.chunks(8) {
        for &byte in chunk {
            print!("{:08b} ", byte);
        }
        print!("\n");
    }
}

fn main() {
    let bytes = b"this is a test";
    println!("Original String: \"{}\"", std::str::from_utf8(&bytes[..]).unwrap());
    println!("\nOriginal String as a Byte String:");
    pretty_print_bytes(&bytes[..]);

    let byte_weights = make_byte_weights(&bytes[..]);
    println!("\nByte Frequency Table:");
    for (byte, &frequency) in byte_weights.iter().enumerate() {
        let byte = byte as u8;
        if frequency > 0 {
            println!("\t{} [{:08b}] = {}", byte as char, byte, frequency);
        }
    }

    let (root, arena) = make_huffman_tree(&byte_weights);
    println!("\nHuffman Tree (left=0, right=1):");
    print_tree(root, &arena);

    let encoding_map = build_encoding_map(root, &arena);

    let encoded = encode_bitvec(&bytes[..], encoding_map);
    println!("\nEncoded Byte String:");
    pretty_print_bytes(&encoded.to_bytes());

    let decoded = decode_bitvec(root, &arena, &encoded);

    println!("\nDecoded Byte String:");
    pretty_print_bytes(&decoded);

    println!("\nDecoded Byte String as a Char String: \"{}\"",
        std::str::from_utf8(&decoded).unwrap());
}
