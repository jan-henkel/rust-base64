use std::error;
use std::ops::RangeInclusive;

#[allow(dead_code)]
pub enum Padding {
    Required(u8),
    Optional(u8),
    NoPadding,
}

pub struct Base64Config {
    ranges: Vec<RangeInclusive<u8>>,
    padding: Padding,
}

#[derive(Debug)]
pub enum Base64ConfigError {
    OverlappingRanges(RangeInclusive<u8>, RangeInclusive<u8>),
    PaddingCharInRange(u8, RangeInclusive<u8>),
    RangeLengthsDoNotSumTo64(usize),
}

#[derive(Debug)]
pub enum Base64Error {
    InvalidCharacter(u8),
    InvalidLength(usize, u8),
    HasPaddingAndLengthNotMultipleOf4(usize),
    TooManyPaddingCharacters(usize),
}

impl error::Error for Base64ConfigError {}
impl error::Error for Base64Error {}

impl std::fmt::Display for Base64ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Base64ConfigError::OverlappingRanges(range1, range2) => {
                write!(
                    f,
                    "Overlapping ranges {}..={} and {}..={}",
                    *range1.start() as char,
                    *range1.end() as char,
                    *range2.start() as char,
                    *range2.end() as char
                )
            }
            Base64ConfigError::PaddingCharInRange(c, range) => {
                write!(
                    f,
                    "Padding character \'{}\' found in range {}..={}",
                    *c as char,
                    *range.start() as char,
                    *range.end() as char
                )
            }
            Base64ConfigError::RangeLengthsDoNotSumTo64(length) => {
                write!(f, "Range lengths sum to {}, not 64", *length)
            }
        }
    }
}

impl std::fmt::Display for Base64Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Base64Error::InvalidCharacter(c) => {
                write!(f, "Invalid character \'{}\'", *c as char)
            }
            Base64Error::InvalidLength(length, padding_char) => {
                write!(
                    f,
                    "Length {} not a multiple of 4. Padding with character \'{}\' required",
                    *length, *padding_char as char
                )
            }
            Base64Error::HasPaddingAndLengthNotMultipleOf4(length) => {
                write!(
                    f,
                    "Padding characters detected and length {} not a multiple of 4",
                    *length
                )
            }
            Base64Error::TooManyPaddingCharacters(count) => {
                write!(f, "Too many padding characters: {}", *count)
            }
        }
    }
}

impl Base64Config {
    #[allow(dead_code)]
    pub fn new(
        ranges: Vec<RangeInclusive<u8>>,
        padding: Padding,
    ) -> Result<Self, Base64ConfigError> {
        let config = Self { ranges, padding };
        validate_config(&config)?;
        Ok(config)
    }
    #[allow(dead_code)]
    pub fn standard() -> Self {
        Base64Config::new(
            vec![
                b'A'..=b'Z',
                b'a'..=b'z',
                b'0'..=b'9',
                b'+'..=b'+',
                b'/'..=b'/',
            ],
            Padding::Optional(b'='),
        )
        .unwrap()
    }
    #[allow(dead_code)]
    pub fn url() -> Self {
        Base64Config::new(
            vec![
                b'A'..=b'Z',
                b'a'..=b'z',
                b'0'..=b'9',
                b'-'..=b'-',
                b'_'..=b'_',
            ],
            Padding::Optional(b'='),
        )
        .unwrap()
    }
    #[allow(dead_code)]
    pub fn mime() -> Self {
        Base64Config::new(
            vec![
                b'A'..=b'Z',
                b'a'..=b'z',
                b'0'..=b'9',
                b'+'..=b'+',
                b'/'..=b'/',
            ],
            Padding::Required(b'='),
        )
        .unwrap()
    }
}

fn choose2<'a, Type: 'a>(
    iter: impl Iterator<Item = &'a Type> + Clone,
) -> impl Iterator<Item = (&'a Type, &'a Type)> {
    let mut n: usize = 1;
    let mut outer = iter.clone().peekable();
    let mut inner = iter.clone().skip(1);
    std::iter::from_fn(move || {
        let outer_el;
        if let Some(inner_el) = inner.next() {
            outer_el = *outer.peek()?;
            return Some((outer_el, inner_el));
        } else {
            _ = outer.next()?;
            outer_el = *outer.peek()?;
            inner = iter.clone().skip({
                n += 1;
                n
            });
            let inner_el = inner.next()?;
            return Some((outer_el, inner_el));
        }
    })
}

fn ranges_overlap<T: std::cmp::PartialOrd>(r1: &RangeInclusive<T>, r2: &RangeInclusive<T>) -> bool {
    !((r1.end() < r2.start()) || (r1.start() > r2.end()))
}

fn validate_config(config: &Base64Config) -> Result<(), Base64ConfigError> {
    if let Some((r1, r2)) = choose2(config.ranges.iter())
        .filter(|(r1, r2)| ranges_overlap(r1, r2))
        .next()
    {
        return Err(Base64ConfigError::OverlappingRanges(r1.clone(), r2.clone()));
    }
    if let Padding::Required(c) | Padding::Optional(c) = config.padding {
        for r in &config.ranges {
            if r.contains(&c) {
                return Err(Base64ConfigError::PaddingCharInRange(c, r.clone()));
            }
        }
    }
    let len_sum = config.ranges.iter().map(|r| r.len()).sum::<usize>();
    if len_sum != 64usize {
        return Err(Base64ConfigError::RangeLengthsDoNotSumTo64(len_sum));
    }
    return Ok(());
}

fn count_trailing_pad_characters(config: &Base64Config, base64_encoded_bytes: &[u8]) -> usize {
    fn count_from_back(bytes: &[u8], p: u8) -> usize {
        match bytes
            .iter()
            .rev()
            .enumerate()
            .map_while(|(i, c)| if i <= 2 && *c == p { Some(i) } else { None })
            .last()
        {
            Some(i) => i + 1,
            None => 0,
        }
    }
    if let Padding::Required(p) | Padding::Optional(p) = config.padding {
        return count_from_back(base64_encoded_bytes, p);
    } else {
        return 0;
    }
}

fn validate_base64(
    config: &Base64Config,
    base64_encoded_bytes: &[u8],
) -> Result<usize, Base64Error> {
    let trailing_pad_count = count_trailing_pad_characters(config, base64_encoded_bytes);
    if trailing_pad_count >= 3 {
        return Err(Base64Error::TooManyPaddingCharacters(trailing_pad_count));
    }
    for c in base64_encoded_bytes.iter().rev().skip(trailing_pad_count) {
        if !config.ranges.iter().any(|r| r.contains(&c)) {
            return Err(Base64Error::InvalidCharacter(*c));
        }
    }
    let length = base64_encoded_bytes.len();
    if let Padding::Required(c) = config.padding {
        if length % 4 != 0 {
            return Err(Base64Error::InvalidLength(length, c));
        }
    }
    if let Padding::Optional(_) = config.padding {
        if trailing_pad_count != 0 && length % 4 != 0 {
            return Err(Base64Error::HasPaddingAndLengthNotMultipleOf4(length));
        }
    }
    Ok(length - trailing_pad_count)
}

fn decode_byte(config: &Base64Config, b: u8) -> u8 {
    let mut offset = 0;
    for r in &config.ranges {
        if r.contains(&b) {
            return b - r.start() + offset;
        }
        offset += r.len() as u8;
    }
    return 0u8;
}

fn encode_byte(config: &Base64Config, b: u8) -> u8 {
    let mut offset = 0;
    for r in &config.ranges {
        let b_in_range = b + r.start() - offset;
        if r.contains(&b_in_range) {
            return b_in_range;
        }
        offset += r.len() as u8;
    }
    return 0u8;
}

fn unpack_triplet(raw_triplet: &[u8]) -> [u8; 4] {
    let bits_per_element: usize = 6;
    let bits_per_byte: usize = 8;
    let number = (raw_triplet[0] as u32) << bits_per_byte * 2
        | (raw_triplet[1] as u32) << bits_per_byte * 1
        | (raw_triplet[2] as u32);
    let element_mask = (1u32 << bits_per_element) - 1;
    [
        ((number >> bits_per_element * 3) & element_mask) as u8,
        ((number >> bits_per_element * 2) & element_mask) as u8,
        ((number >> bits_per_element * 1) & element_mask) as u8,
        (number & element_mask) as u8,
    ]
}

fn pack_triplet(encoded_triplet: &[u8]) -> [u8; 3] {
    let bits_per_element: usize = 6;
    let bits_per_byte: usize = 8;
    let number = (encoded_triplet[0] as u32) << bits_per_element * 3
        | (encoded_triplet[1] as u32) << bits_per_element * 2
        | (encoded_triplet[2] as u32) << bits_per_element * 1
        | (encoded_triplet[3] as u32);
    let byte_mask = (1u32 << bits_per_byte) - 1;
    [
        ((number >> bits_per_byte * 2) & byte_mask) as u8,
        ((number >> bits_per_byte * 1) & byte_mask) as u8,
        (number & byte_mask) as u8,
    ]
}

fn chunk_iter<T: Default + Copy, const CHUNK_SIZE: usize, U: Iterator<Item = T> + Clone>(
    iter: &U,
) -> impl Iterator<Item = [T; CHUNK_SIZE]> + use<T, CHUNK_SIZE, U> {
    let mut iter = iter.clone();
    std::iter::from_fn(move || {
        let mut array_chunk: [T; CHUNK_SIZE] = [T::default(); CHUNK_SIZE];
        for i in 0..CHUNK_SIZE {
            array_chunk[i] = iter.next()?;
        }
        Some(array_chunk)
    })
}

pub fn decode<'a>(
    config: &'a Base64Config,
    base64_encoded_bytes: &'a [u8],
) -> Result<impl Iterator<Item = u8> + use<'a>, Base64Error> {
    let unpadded_length = validate_base64(config, &base64_encoded_bytes)?;
    let pad_length = (4 - (unpadded_length % 4)) % 4;
    let zeroes = [0u8].repeat(pad_length);
    let padded_base64_encoded_bytes = base64_encoded_bytes.iter().map(|c| *c).chain(zeroes);
    let padded_segments = padded_base64_encoded_bytes.map(|b| decode_byte(config, b));
    let chunked_segments = chunk_iter::<u8, 4, _>(&padded_segments);
    let chunked_bytes = chunked_segments.map(|chunk| pack_triplet(chunk.as_slice()));
    let decoded_bytes = chunked_bytes.flatten();
    let bits_per_segment = 6usize;
    let bits_per_byte = 8usize;
    let num_bytes_unpadded = (unpadded_length * bits_per_segment) / bits_per_byte;
    Ok(decoded_bytes.take(num_bytes_unpadded))
}

pub fn decode_to_vec(
    config: &Base64Config,
    base64_encoded_bytes: &[u8],
) -> Result<Vec<u8>, Base64Error> {
    let decoded_iter = decode(config, base64_encoded_bytes)?;
    Ok(Vec::from_iter(decoded_iter))
}

pub fn encode(config: &Base64Config, bytes: &[u8]) -> impl Iterator<Item = u8> {
    let pad_length = (3 - bytes.len() % 3) % 3;
    let zeroes = [0u8].repeat(pad_length);
    let padded_bytes = bytes.iter().map(|c| *c).chain(zeroes);
    let chunked_bytes = chunk_iter::<u8, 3, _>(&padded_bytes);
    let chunked_segments = chunked_bytes.map(|chunk| unpack_triplet(chunk.as_slice()));
    let segments = chunked_segments.flatten();
    let base64_encoded_segments = segments.map(|b| encode_byte(config, b));
    let bits_per_segment = 6usize;
    let bits_per_byte = 8usize;
    let num_segments_unpadded =
        (bytes.len() * bits_per_byte + bits_per_segment - 1) / bits_per_segment;
    let num_pad_segments = pad_length * bits_per_byte / bits_per_segment;
    match config.padding {
        Padding::Required(c) | Padding::Optional(c) => base64_encoded_segments
            .take(num_segments_unpadded)
            .chain([c].repeat(num_pad_segments)),
        Padding::NoPadding => base64_encoded_segments
            .take(num_segments_unpadded)
            .chain([0u8].repeat(0)), // to obtain the same type
    }
}

pub fn encode_to_vec(config: &Base64Config, bytes: &[u8]) -> Vec<u8> {
    let encoded_iter = encode(config, bytes);
    Vec::from_iter(encoded_iter)
}

#[cfg(test)]
mod tests;
