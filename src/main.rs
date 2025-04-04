mod base64;
use base64::{decode_to_vec, encode_to_vec, Base64Config};
use std::ops::Drop;
use std::time;

struct RAIITimer {
    start: time::Instant,
    callback: fn(time::Duration) -> (),
}

impl RAIITimer {
    fn new(callback: fn(time::Duration) -> ()) -> Self {
        RAIITimer {
            start: time::Instant::now(),
            callback,
        }
    }
}

impl Drop for RAIITimer {
    fn drop(&mut self) {
        (self.callback)(self.start.elapsed());
    }
}

fn print_time_elapsed(duration: time::Duration) {
    println!("Time elapsed in scope: {duration:#?}");
}

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
    )
    .unwrap();
    let output;
    let base64str;
    {
        let _timer = RAIITimer::new(print_time_elapsed);
        let _url_config = Base64Config::url();
        let bytes = decode_to_vec(&config, "SGVsbG8sIFdvcmxkIQ".as_bytes()).unwrap();
        output = String::from_utf8(bytes).unwrap();
        let standard_config = Base64Config::standard();
        let encoded = encode_to_vec(&standard_config, "Hello, World!".as_bytes());
        base64str = String::from_utf8(encoded).unwrap();
    }
    println!("Output: {output}");
    println!("Base64: {base64str}");
}
