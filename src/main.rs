
extern crate huffman;

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
    println!("original string: \"{}\"", std::str::from_utf8(&bytes[..]).unwrap());
    println!("\noriginal string as byte string:");
    pretty_print_bytes(&bytes[..]);

    let byte_weights = make_byte_weights(&bytes[..]);
    println!("\nbyte frequencies:");
    for (byte, &frequency) in byte_weights.iter().enumerate() {
        let byte = byte as u8;
        if frequency > 0 {
            println!("\t{} [{:08b}] = {}", byte as char, byte, frequency);
        }
    }

    let (root, arena) = make_huffman_tree(&byte_weights);
    println!("\nhuffman tree (left=0, right=1):");
    print_tree(root, &arena);

    let encoding_map = build_encoding_map(root, &arena);

    let encoded = encode_bitvec(&bytes[..], encoding_map);
    println!("\nencoded byte string:");
    pretty_print_bytes(&encoded.to_bytes());

    let decoded = decode_bitvec(root, &arena, &encoded);

    println!("\ndecoded byte string:");
    pretty_print_bytes(&decoded);

    println!("\ndecoded byte string as char string: \"{}\"",
        std::str::from_utf8(&decoded).unwrap());
}
