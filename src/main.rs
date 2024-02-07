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


#[derive(Debug, Display)]
#[display("format")]

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

enum tokenTypeEnum{
    PLUS, 
    MINUS, 
    IF_RW, 
    LOOP_RW, 
    END_RW, 
    L_PAREN, 
    R_PAREN,
    L_BRACKET, 
    R_BRACKET,
    NUMBER, 
    IDENTIFIER, 
    EOF
}

//This is the master struct for the lexer
struct Lexer {
    tokenType: tokenTypeEnum,
    inputFile: inFile,
    
}
impl Lexer{
    fn new(&mut self){
        println!("Created the lexer struct");
    }
}

//inFile Class
struct inFile{
    attatchFile: bool,
    fileName: String,
    lineCnt: usize,
    totalLines: usize,
    file : BufReader <File>,
}
impl inFile {
    //Init function, opens the file
    fn new(fileName: &str) -> inFile {
        let mut newFile = BufReader::new(File::open(fileName).unwrap());
        
        inFile {
            fileName: fileName.to_string(),
            attatchFile: false,
            lineCnt: 0,
            totalLines: 0,
            file: newFile,
        }

    }

    //Sets the gathered stats (probably unneccesary idk)
    fn setStats(&mut self, stats: Stats){
        self.totalLines = stats.lines;
    }

    //Prints the stats of the file (for debugging)
    fn printInfo(&self){
        println!("File Name: {}", self.fileName);
        println!("Lines: {}", self.lineCnt);
    }

    //A function to increment the current line
    fn incLineCnt(&mut self){
        self.lineCnt += 1;
    }

}

//A class for the tokenMark object (IDk what this means or what it is/does)
// struct tokenMark{
//     tmUnionType: mark1
// }


//Token class
struct token{
    tt: tokenType,
    tokStr: String,
    //To be completed later when I understand
    //tm: tokenMark,
}
impl token{
    fn new(&mut self){
        //self.tokenMark = NULL;
        println!("Created the token struct");
    }
}

fn main() -> Result<()> {
    
    let path = env::args().nth(1).expect("please supply an argument");
    let file = BufReader::new(File::open(&path)?);
    let stats = Stats::new(file)?;
    println!("File: {} characters: {}", path, stats.characters);

    println!("Creating inFile structure");
    let mut f = inFile::new(path.as_str());
    f.setStats(stats);
    //println!("inFile: {}", f.fileName);
    f.printInfo();
    f.lineCnt = 5;
    f.printInfo();


    Ok(())
}