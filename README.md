Huffman Compression
===================

Simple file compression and decompression tool using Huffman trees.

Example snippet from [`examples/encode_and_decode.rs`](./examples/encode_and_decode.rs):

```rust
use huffman::{encode_file_contents, decode_file_contents};

fn encode_and_decode() {
    let contents = read_file_contents("examples/test.txt");

    let encoded = encode_file_contents(&contents);
    write_to_file("examples/test.txt.encoded", &encoded);

    let decoded = decode_file_contents(&encoded).expect("error decoding file");
    write_to_file("examples/output.txt", &decoded);
}
```

Run `cargo run --example encode_and_decode` to see the example.  Be sure to run it from the root of the project.

The `encode_file_contents` function serializes the Huffman tree in pre-order format, directly into the file header.  The format for a compressed file is:

1.  The length of the header as a plain text integer
2.  A single null byte marking the end of the header length
3.  The header itself, which is the serialized Huffman tree used to encode the file
4.  The compressed file contents

The `decode_file_contents` deserializes the header back into a Huffman tree in order to decode the compressed file contents.

--------------------------------------------------------------------------------

Breakdown of Huffman Trees
--------------------------

Look at [`src/main.rs`](./src/main.rs) for a more detailed breakdown of Huffman compression.  The following code blocks are the direct output of main if you do not have the Rust Toolchain available:

```text
// main.rs output
Original String: "this is a test"

Original String as a Byte String:
01110100 01101000 01101001 01110011 00100000 01101001 01110011 00100000 
01100001 00100000 01110100 01100101 01110011 01110100 
```

A frequency table is created by counting the occurrence of every possible byte (256 possibilities) and dropping any bytes that do not show up.  For the string **"This is a test"**, only the chars *a, e, h, i, s, t,* and *"space"* show up.

```text
// main.rs output
Byte Frequency Table:
	  [00100000] = 3
	a [01100001] = 1
	e [01100101] = 1
	h [01101000] = 1
	i [01101001] = 2
	s [01110011] = 3
	t [01110100] = 3
```

Next, a forest of trees is created from the frequency table.  Each tree has a single node in it, the character from the frequency table.  Every tree also has a weight associated with it, which starts out as the character's frequency.

The two trees with the lowest weight are combined into one and put back into the forest.  The new weight for this tree is the sum of the previous weights.  This process is repeated until the forest is reduced down to one tree.

```text
// main.rs output
Huffman Tree (left=0, right=1):

                               ⅄
               ⅄                               ⅄
       ⅄               t               ␣               ⅄
   h       ⅄       ∅       ∅       ∅       ∅       i       s
 ∅   ∅   a   ⅄   ∅   ∅   ∅   ∅   ∅   ∅   ∅   ∅   ∅   ∅   ∅   ∅
∅ ∅ ∅ ∅ ∅ ∅ † e ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅ ∅
```

The character `⅄` represents a branch in the tree.  Branches do not have characters, only leaves do.  The character `†` represents the End Of File marker.

To encode the original string, start at the root of the tree and follow the branches down to the character you wish to encode.  Going left appends a 0 to the bit string, going right appends a 1.

| Char | Path from Root      | Bit String |
|------|---------------------|------------|
| t    | left, right         | 01         |
| h    | left, left, left    | 000        |
| i    | right, right, left  | 110        |
| s    | right, right, right | 111        |

The encoding for **"this"** is 01000110111.

Here is the encoded byte string for the full original phrase:

```text
// main.rs output
Encoded Byte String:
01000110 11110110 11110001 01001001 11111010 01100000 
```

An encoded byte string can be decoded by walking the tree from the root, going left on 0s and right on 1s, until a leaf is encountered, which will contain the character.

Here is the decoded byte string, which matches the original from above:

```text
// main.rs output
Decoded Byte String:
01110100 01101000 01101001 01110011 00100000 01101001 01110011 00100000 
01100001 00100000 01110100 01100101 01110011 01110100 

Decoded Byte String as a Char String: "this is a test"
```

