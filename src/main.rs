mod base64;
use base64::{decode_to_vec, encode_to_vec, Base64Config};
use clap::{Parser, ValueEnum};
use std::io::{self, Read, Write};

#[derive(ValueEnum, Clone, Default)]
enum Alphabet {
    /// A-Z, a-z, 0-9, +, / with optional = padding
    #[default]
    Standard,
    /// A-Z, a-z, 0-9, -, _ with optional = padding (URL/filename safe)
    Url,
    /// A-Z, a-z, 0-9, +, / with required = padding (MIME)
    Mime,
}

#[derive(Parser)]
#[command(
    name = "base64",
    about = "Encode or decode data using base64",
    long_about = None
)]
struct Cli {
    /// Decode input instead of encoding it
    #[arg(short, long)]
    decode: bool,

    /// Alphabet / encoding variant to use
    #[arg(short, long, value_enum, default_value_t = Alphabet::Standard)]
    alphabet: Alphabet,

    /// Do not output a trailing newline
    #[arg(short, long)]
    no_newline: bool,

    /// Read input from a file
    #[arg(short, long, conflicts_with = "input")]
    file: Option<std::path::PathBuf>,

    /// Input string to encode or decode
    #[arg(short, long, conflicts_with = "file")]
    input: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let config = match cli.alphabet {
        Alphabet::Standard => Base64Config::standard(),
        Alphabet::Url => Base64Config::url(),
        Alphabet::Mime => Base64Config::mime(),
    };

    let input = if let Some(path) = &cli.file {
        std::fs::read(path).unwrap_or_else(|e| {
            eprintln!("base64: {}: {}", path.display(), e);
            std::process::exit(1);
        })
    } else if let Some(s) = &cli.input {
        s.as_bytes().to_vec()
    } else {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).unwrap_or_else(|e| {
            eprintln!("base64: {e}");
            std::process::exit(1);
        });
        buf
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();

    if cli.decode {
        match decode_to_vec(&config, &input) {
            Ok(bytes) => {
                out.write_all(&bytes).unwrap_or_else(|e| {
                    eprintln!("base64: {e}");
                    std::process::exit(1);
                });
                if !cli.no_newline {
                    out.write_all(b"\n").ok();
                }
            }
            Err(e) => {
                eprintln!("base64: decode error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        let encoded = encode_to_vec(&config, &input);
        out.write_all(&encoded).unwrap_or_else(|e| {
            eprintln!("base64: {e}");
            std::process::exit(1);
        });
        if !cli.no_newline {
            out.write_all(b"\n").ok();
        }
    }
}
