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
        ranges: vec![b'='..=b'='],
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

#[test]
fn test_error_display_and_config_variants() {
    // Base64ConfigError display variants
    let ovr = Base64ConfigError::OverlappingRanges(b'A'..=b'Z', b'Z'..=b'z');
    let s = format!("{ovr}");
    assert!(s.contains("Overlapping ranges"));

    let pad_in_range = Base64ConfigError::PaddingCharInRange(b'=', b'='..=b'=');
    let s = format!("{pad_in_range}");
    assert!(s.contains("Padding character"));

    let len_err = Base64ConfigError::RangeLengthsDoNotSumTo64(10);
    let s = format!("{len_err}");
    assert!(s.contains("not 64"));

    // Base64Error display variants
    let e1 = Base64Error::InvalidCharacter(b'!');
    assert!(format!("{e1}").contains("Invalid character"));
    let e2 = Base64Error::InvalidLength(3, b'=');
    assert!(format!("{e2}").contains("Length 3"));
    let e3 = Base64Error::HasPaddingAndLengthNotMultipleOf4(7);
    assert!(format!("{e3}").contains("Padding characters detected"));
    let e4 = Base64Error::TooManyPaddingCharacters(3);
    assert!(format!("{e4}").contains("Too many padding"));
}

#[test]
fn test_overlapping_ranges_validation() {
    let cfg = Base64Config {
        ranges: vec![b'A'..=b'Z', b'Z'..=b'z'],
        padding: Padding::NoPadding,
    };
    match validate_config(&cfg) {
        Err(Base64ConfigError::OverlappingRanges(_, _)) => {}
        other => panic!("expected overlapping ranges error, got {:?}", other),
    }
}

#[test]
fn test_count_trailing_pad_characters_and_validate_errors() {
    let std = Base64Config::standard();
    // Too many '=' padding (3) should error
    let res = decode_to_vec(&std, b"AAAA===");
    match res {
        Err(Base64Error::TooManyPaddingCharacters(3)) => {}
        Err(e) => panic!("expected TooManyPaddingCharacters, got {:?}", e),
        Ok(_) => panic!("expected Err, got Ok"),
    }

    // Required padding and invalid length
    let mime = Base64Config::mime();
    let res = decode_to_vec(&mime, b"AA");
    match res {
        Err(Base64Error::InvalidLength(_, _)) => {}
        other => panic!("expected InvalidLength for mime, got {:?}", other),
    }

    // Optional padding present but length not multiple of 4
    let res = decode_to_vec(&std, b"AA=");
    match res {
        Err(Base64Error::HasPaddingAndLengthNotMultipleOf4(_)) => {}
        other => panic!(
            "expected HasPaddingAndLengthNotMultipleOf4, got {:?}",
            other
        ),
    }
}

#[test]
fn test_pack_unpack_roundtrip_and_byte_mappings() {
    // pack/unpack roundtrip
    let raw = [0x01u8, 0x02u8, 0x03u8];
    let segments = unpack_triplet(&raw);
    let out = pack_triplet(&segments);
    assert_eq!(out, raw);

    // encode_byte/decode_byte mapping for standard config
    let std = Base64Config::standard();
    assert_eq!(encode_byte(&std, 0), b'A');
    assert_eq!(decode_byte(&std, b'A'), 0);
    // last value 63 -> '/'
    assert_eq!(encode_byte(&std, 63), b'/');
    assert_eq!(decode_byte(&std, b'/'), 63);
}

#[test]
fn test_chunk_iter_partial() {
    let v = vec![1u8, 2u8, 3u8, 4u8, 5u8];
    let iter = v.into_iter();
    let mut chunks = chunk_iter::<u8, 3, _>(&iter);
    let first = chunks.next().expect("first chunk");
    assert_eq!(first, [1u8, 2u8, 3u8]);
    // there should be no second full chunk (only 2 elements left)
    assert!(chunks.next().is_none());
}

#[test]
fn test_config_constructors() {
    // call url and mime constructors to increase coverage
    let _ = Base64Config::url();
    let _ = Base64Config::mime();
}

#[test]
fn test_no_padding_count_and_encode_branch() {
    // construct a standard-like config but with NoPadding
    let cfg = Base64Config::new(
        vec![
            b'A'..=b'Z',
            b'a'..=b'z',
            b'0'..=b'9',
            b'+'..=b'+',
            b'/'..=b'/',
        ],
        Padding::NoPadding,
    )
    .expect("valid config");

    // count_trailing_pad_characters should return 0 for NoPadding
    let trailing = count_trailing_pad_characters(&cfg, b"AA=");
    assert_eq!(trailing, 0);

    // encode should take the NoPadding branch (chain empty) and succeed
    let enc = encode_to_vec(&cfg, b"Hi");
    // should not contain '=' as padding
    assert!(!enc.contains(&b'='));
    assert!(!enc.is_empty());
}

#[test]
fn test_validate_direct_branches() {
    // Required padding invalid length
    let mime = Base64Config::mime();
    match validate_base64(&mime, b"AA") {
        Err(Base64Error::InvalidLength(len, _)) if len == 2 => {}
        other => panic!("expected InvalidLength, got {:?}", other),
    }

    // Optional padding with padding present but length not multiple of 4
    let std = Base64Config::standard();
    match validate_base64(&std, b"AA=") {
        Err(Base64Error::HasPaddingAndLengthNotMultipleOf4(len)) if len == 3 => {}
        other => panic!(
            "expected HasPaddingAndLengthNotMultipleOf4, got {:?}",
            other
        ),
    }
}

#[test]
fn test_encode_byte_no_match_returns_zero() {
    let std = Base64Config::standard();
    // pick a byte that is too high to fit into 6 bits (e.g. 64)
    // encode_byte should return 0 for values outside 0..=63
    assert_eq!(encode_byte(&std, 64u8), 0u8);
}

#[test]
fn test_decode_byte_no_match_returns_zero() {
    let std = Base64Config::standard();
    // pick a byte that is not part of the Base64 alphabet (e.g. '?')
    // decode_byte should return 0 for characters not in any range
    assert_eq!(decode_byte(&std, b'?'), 0u8);
}

#[test]
fn test_valid_optional_padding_decodes() {
    let cfg = Base64Config::standard(); // Padding::Optional
    // "Hello, World!" with standard base64 including optional padding
    let b64 = b"SGVsbG8sIFdvcmxkIQ==";
    let decoded = decode_to_vec(&cfg, b64).expect("should decode with optional padding");
    assert_eq!(decoded, b"Hello, World!");
}

#[test]
fn test_valid_required_padding_decodes() {
    let cfg = Base64Config::mime(); // Padding::Required
    // "Ma" encoded with required padding is "TWE="
    let b64 = b"TWE=";
    let decoded = decode_to_vec(&cfg, b64).expect("should decode with required padding");
    assert_eq!(decoded, b"Ma");
}
