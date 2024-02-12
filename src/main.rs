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

//package imports
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


//The enumeration for saving token types, this is a list of every type of token there is
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

//The enumeration for character type
enum charType{
    LETTER,
    NUMBER,
    SYMBOL,
    EOF,
}

//This is the master struct for the lexer
struct Lexer {
    //tokenType: tokenTypeEnum,
    inputFile: inFile,
    
}
impl Lexer{
    fn new(fileName: &str) -> Lexer {
        println!("Beginning creation of Lexer");
        let newFile = inFile::new(fileName);
        println!("Lexer created successfully");


        Lexer { 
            //tokenType: (), 
            inputFile: newFile, 
        }
    }
    //The main function if the lexer
    //Returns one token
    fn scan(&mut self) -> bool{
        //Gets the next character in the file string
        let mut nextChar = self.inputFile.getChar();
        
        //Checks if it is a filler character or not
        let mut nextChar = self.inputFile.getChar();
        while let Some(c) = nextChar {
            if c == '\n' || c == '\t' || c == '\r' {
                if c == '\n' {
                    self.inputFile.incLineCnt();
                }
                nextChar = self.inputFile.getChar();
            } else {
                break; // Exit the loop if the character is not a filler character
            }
        }

        //A segment to parse/ignore comments goes here:
        //
        //
        //

        //A switch case to handle the different things that it could be to look ahead
        //println!("{}", nextChar);
        match nextChar {
            Some(ch) if ch.is_ascii_alphabetic() => {
                println!("The character is a letter.");
                return true;
            }
            Some(ch) if ch.is_ascii_digit() => {
                println!("The character is a number.");
                return true;
            }
            Some(_) => {
                println!("The character is a symbol.");
                return true;
            }
            None => {
                println!("This character is an None");
                return false;
            }
        }
    }



    //A function to scan through entire file
    fn scanThrough(&mut self){
        while self.scan(){

        };
        println!("EOF Reached")
    }

}

//inFile Class
struct inFile{
    attatchFile: bool,
    fileName: String,
    fileContents: String,
    lineCnt: usize,
    numChars: usize,
    totalLines: usize,
    file : BufReader <File>,
    currentCharIndex: usize,
}
impl inFile {
    //Init function, opens the file
    fn new(fileName: &str) -> inFile {
        let mut newFile = BufReader::new(File::open(fileName).unwrap());
        let fileContentsString = std::fs::read_to_string(fileName).expect("Unable to read file");
        let numChars = fileContentsString.len();
        println!("Creating the inFile structure");
        
        inFile {
            fileName: fileName.to_string(),
            attatchFile: false,
            lineCnt: 0,
            currentCharIndex: 0,
            totalLines: 0,
            file: newFile,
            fileContents: fileContentsString,
            numChars: numChars,

        }

    }

    //Prints the stats of the file (for debugging)
    fn printInfo(&self){
        println!("File Name: {}", self.fileName);
        println!("Lines: {}", self.lineCnt);
    }

    //Gets the next character in the file string
    fn getChar(&mut self) -> Option<char> {
        if let Some(current_char) = self.fileContents.chars().nth(self.currentCharIndex) {
            self.currentCharIndex += 1;
            Some(current_char)
        } else {
            None
        }
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
    
    let path = env::args().nth(1).expect("Please specify an input file");
    let mut myLexer: Lexer = Lexer::new(&path);
    println!("Lexer filename: {} \nCharacter count: {}", myLexer.inputFile.fileName, myLexer.inputFile.numChars);

    myLexer.scanThrough();


    // for mut char in myLexer.inputFile.fileContents.clone().chars(){
    //     myLexer.scan();
    // }
    
    
    // let file = BufReader::new(File::open(&path)?);
    // let stats = Stats::new(file)?;
    // println!("File: {} characters: {}", path, stats.characters);

    // //println!("Creating inFile structure");
    // let mut f = inFile::new(path.as_str());
    // f.setStats(stats);
    // //println!("inFile: {}", f.fileName);
    // f.printInfo();
    // f.lineCnt = 5;
    // f.printInfo();


    Ok(())
}