//Rules
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_variables)]

extern crate anyhow;
extern crate parse_display;
extern crate utf8_chars;
extern crate unicode_segmentation;

use {
    unicode_segmentation::UnicodeSegmentation,
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
    //The main function if the lexer
    //Returns one token
    fn scan(&mut self){
        //Gets the next character in the file string
        let mut nextChar: char = self.inputFile.getChar();
        //Checks if it is a filler character or not
        while((nextChar == '\n') || (nextChar == '\t') || (nextChar == '\r')){
            if nextChar == '\n'{
                self.inputFile.incLineCnt();
            }
            nextChar = self.inputFile.getChar();
        }
        //A segment to parse/ignore comments goes here:
        //
        //
        //

        //A switch case to handle the different things that it could be to look ahead
    }
}

//inFile Class
struct inFile{
    attatchFile: bool,
    fileName: String,
    fileContents: String,
    lineCnt: usize,
    totalLines: usize,
    file : BufReader <File>,
    currentCharIndex: usize,
}
impl inFile {
    //Init function, opens the file
    fn new(fileName: &str) -> inFile {
        let mut newFile = BufReader::new(File::open(fileName).unwrap());
        let fileContentsString = std::fs::read_to_string(fileName).expect("Unable to read file");
        inFile {
            fileName: fileName.to_string(),
            attatchFile: false,
            lineCnt: 0,
            currentCharIndex: 0,
            totalLines: 0,
            file: newFile,
            fileContents: fileContentsString,
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

    fn getChar(&mut self) -> char{
        let mut currentChar: char = self.fileContents.chars().nth(self.currentCharIndex). unwrap();
        self.currentCharIndex += 1;
        return currentChar;
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
    tt: tokenTypeEnum,
    //To be completed later when I understand
    //tm: tokenMark,
}
impl token{
    //Init for the token
    fn new(&mut self) -> token{
        //self.tokenMark = NULL;
        println!("Created the token struct");

        token {
            tt: crate::tokenTypeEnum::IDENTIFIER,
        }
    }
    //Used for setting the token type
    fn setTokenType(&mut self, newType: tokenTypeEnum){
        self.tt = newType;
    }
}

//Token ID class, derives from token class
struct tokenId {
    parent: token,
    tokStr: String,
}

//Token Function class, derived from token class
struct tokenFunction{
    parent: token,
    tokStr: String,
    argList: token<>,
    returnType: token,
}

//Structure for reporting errors and warnings
struct reporting{
    errorStatus: bool,
}
impl reporting{
    fn new(&mut self) -> reporting{
        self.errorStatus = false;

        reporting{
            errorStatus: false,
        }
    }
    fn reportError(message: String){
        println!("reporting error: {}", message);
    }
    fn reportWarning(message: String){
        println!("reporting warning: {}", message);
    }
}

//The structure for the SymbolTable. This holds all of the IDENTIFIERS of the program as well as their scope and information
struct symbolTable{
    // For now you can simply use a single hash table of tokens. As we move forward to parsing, the symbol table
    // structure will have to be augmented to permit the recording of entering/exiting program scopes as well as
    // the scope that an IDENTIFER is declared. In general when you exit a scope the symbol table will remove
    // any symbols defined in that scope from the symbol table. Again, we will solve this problem later; the
    // example methods for scope entry/exit are here to deomonstrate what we will probably want in the future
    //symTab: hashTable<token>,
}
impl symbolTable{
    // The symbol table hashLook function should automatically create a new entry and mark it as an
    // IDENTIFER token for any IDENTIFIER string that is not already in the symbol table. In some languages
    // case does not matter to the uniqueness of the symbol. In this case, an easy place to solve this is to simply
    // upper case or lower case all strings in the symbol table API functions (and storage)
    fn hashLook(mut lookupString: String){
        println!("Looking up the idenntifier of the string");
    }
    fn enterScope(){
        println!("To be used in the future");
    }
    fn exitScope(){
        println!("To be used in the future");
    }
}



//The main section of the code
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