mod base64;
use base64::{decode_to_vec, encode_to_vec, Base64Config};

fn main() {
    let config = Base64Config::new(
        vec![
            b'A'..=b'Z',
            b'a'..=b'z',
            b'0'..=b'9',
            b'+'..=b'+',
            b'/'..=b'/',
        ],
        base64::Padding::NoPadding,
    ).unwrap();
    let standard_config = Base64Config::standard();
    let _url_config = Base64Config::url();
    let bytes = decode_to_vec(&config, "SGVsbG8sIFdvcmxkIQ".as_bytes()).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    println!("Output: {output}");
    let encoded = encode_to_vec(&standard_config, "Hello, World!".as_bytes());
    let base64str = String::from_utf8(encoded).unwrap();
    println!("Base64: {base64str}");
    // let bytes = base64decode("SGVsbG8sIFdvcmxkIQ==").unwrap();
    // let output = String::from_utf8(bytes).unwrap();
    // println!("Output: {output}");
    // let line = "Hello, World!";
    // let encoded = base64encode(line);
    // let encoded_str = String::from_utf8(encoded).unwrap();
    // println!("Encoded string: {encoded_str}");
}