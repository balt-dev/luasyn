use std::error::Error;

use luasyn::{element::*, parse::Parseable, tokenize::TokenStream};

macro_rules! file {
    ($l: literal) => {
        [$l, include_str!($l)]
    };
}

static REAL_FILES: &[[&'static str; 2]] = &[file!("strontium.lua"), file!("omeganum.lua")];

#[test]
fn real_data() -> Result<(), Box<dyn Error>> {
    for [name, file] in REAL_FILES {
        let mut stream = match TokenStream::parse(*file) {
            Ok(v) => v,
            Err(e) => {
                let idx = e.index;
                let line_start = file[..idx].rfind('\n').map_or(0, |i| i + 1);
                let line_end = file[idx..].find('\n').unwrap_or(file.len() - idx) + idx;
                let lines = file[..line_start].chars().filter(|c| *c == '\n').count();
                panic!(
                    "tokenization error at {name}:{}:{}: {}\n{}\n{}^",
                    lines + 1,
                    idx - line_start,
                    e.message,
                    &file[line_start..line_end],
                    " ".repeat(idx - line_start)
                )
            }
        };
        if let Err(e) = Chunk::parse(&mut stream) {
            let idx = e.index;
            let line_start = file[..idx].rfind('\n').map_or(0, |i| i + 1);
            let line_end = file[idx..].find('\n').unwrap_or(file.len() - idx) + idx;
            let lines = file[..line_start].chars().filter(|c| *c == '\n').count();
            panic!(
                "parsing error at {name}:{}:{}: {}\n{}\n{}^",
                lines + 1,
                idx - line_start,
                e.message,
                &file[line_start..line_end],
                " ".repeat(idx - line_start)
            )
        }
    }
    Ok(())
}
