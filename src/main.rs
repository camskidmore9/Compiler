//Rules
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(dead_code)]

extern crate anyhow;
extern crate parse_display;
extern crate utf8_chars;

use {
    anyhow::Result,
    parse_display::Display,
    std::{
        env,
        fs::{
            File,
            read_to_string,
        },
        path::Path,
        io::{
            Read,
            prelude::*,
            BufReader,
            BufRead
        },


    },
    utf8_chars::BufReadCharsExt,
};

//imports
use std::io::prelude::*;


#[derive(Clone, Debug, Display)]
#[display("format")]



//inFile Class
struct inFile{
    attatchFile: bool,
    fileName: String,
    lineCnt: usize,
    file : BufReader <File>,
}
impl inFile {
    fn new(fileName: &str) -> inFile {
        let mut newFile = BufReader::new(File::open(fileName).unwrap());
        
        inFile {
            fileName: fileName.to_string(),
            attatchFile: false,
            lineCnt: 0,
            file: newFile,
        }

    }
}


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

fn main() -> Result<()> {
    
    let path = env::args().nth(1).expect("please supply an argument");
    let file = BufReader::new(File::open(&path)?);
    let stats = Stats::new(file)?;
    println!("File: {} characters: {}", path, stats.characters);

    println!("Creating inFile structure");
    let f = inFile::new(path.as_str(), stats.lines);
    println!("inFile: {}", f.fileName);


    Ok(())
}