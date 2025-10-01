use super::*;

#[test]
fn test_range_lengths_do_not_sum_to_64() {
    // only one range (26) -> should error with length sum != 64
    let res = Base64Config::new(vec![b'A'..=b'Z'], Padding::NoPadding);
    match res {
        Err(Base64ConfigError::RangeLengthsDoNotSumTo64(26)) => {}
        Err(e) => panic!("expected RangeLengthsDoNotSumTo64(26), got {:?}", e),
        Ok(_) => panic!("expected Err, got Ok"),
    }
}

#[test]
fn test_padding_char_in_range() {
    // Create a config where the padding character '=' is inside a range.
    let cfg = Base64Config {
        ranges: vec![b'='..=b'=' ],
        padding: Padding::Optional(b'='),
    };
    // validate_config should detect the padding char in a range
    match validate_config(&cfg) {
        Err(Base64ConfigError::PaddingCharInRange(c, _)) if c == b'=' => {}
        other => panic!("expected PaddingCharInRange('='), got {:?}", other),
    }
}

#[test]
fn test_invalid_character_detection() {
    let cfg = Base64Config::standard();
    // '!' is not part of the standard Base64 alphabet
    let res = decode_to_vec(&cfg, b"SGVsbG8sIS!");
    match res {
        Err(Base64Error::InvalidCharacter(c)) if c == b'!' => {}
        Err(e) => panic!("expected InvalidCharacter('!'), got {:?}", e),
        Ok(_) => panic!("expected Err, got Ok"),
    }
}

#[test]
fn test_encode_decode_roundtrip_standard() {
    let cfg = Base64Config::standard();
    let input = b"Hello, World!";
    let encoded = encode_to_vec(&cfg, input);
    let decoded = decode_to_vec(&cfg, &encoded).expect("decode should succeed");
    assert_eq!(decoded, input);
}

#[test]
fn test_decode_without_padding_optional() {
    let cfg = Base64Config::standard();
    // Base64 for "Hello, World!" without trailing padding (two '=' omitted)
    let input_b64 = b"SGVsbG8sIFdvcmxkIQ";
    let decoded = decode_to_vec(&cfg, input_b64).expect("decode should succeed");
    assert_eq!(decoded, b"Hello, World!");
}
