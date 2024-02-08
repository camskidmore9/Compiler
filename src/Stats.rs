//Stats class, breaks down a file and gets the stats from it
struct Stats {
    characters: usize,
    words: usize,
    lines: usize,
}


impl Stats {
    fn new<R: BufRead>(mut reader: R) -> Result<Self> {
        let mut stats = Stats {
            characters: 0,
            words: 1,
            lines: 1,
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
