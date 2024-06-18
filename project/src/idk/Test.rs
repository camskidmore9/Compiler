use {
    anyhow::Result,
    parse_display::Display,
    std::{
        env,
        fs::File,
        io::{BufRead, BufReader},
    },
    utf8_chars::BufReadCharsExt,
};

extern crate anyhow;
extern crate parse_display;
extern crate utf8_chars;

#[derive(Clone, Debug, Display)]
#[display("{characters} {words} {lines}")]
struct Stats {
    characters: usize,
    words: usize,
    lines: usize,
}

impl Stats {
    fn new<R: BufRead>(mut reader: R) -> Result<Self> {
        let mut stats = Stats {
            characters: 0,
            words: 0,
            lines: 0,
        };
        let mut in_word = false;

        for c in reader.chars_raw() {
            let c = c?;

            if c != '\0' {
                stats.characters += 1;
            }

            if !c.is_whitespace() {
                in_word = true;
            } else if in_word {
                stats.words += 1;
                in_word = false;
            }

            if c == '\n' {
                stats.lines += 1;
            }
        }

        Ok(stats)
    }
}

fn main() -> Result<()> {
    for path in env::args().skip(1) {
        let file = BufReader::new(File::open(&path)?);
        let stats = Stats::new(file)?;
        println!("{} {}", stats, path);
    }

    Ok(())
}