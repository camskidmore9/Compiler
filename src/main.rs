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
        fmt,


    },
    utf8_chars::BufReadCharsExt,
};

//imports
use std::io::prelude::*;


// #[derive(Debug, Display)]
// #[display("format")]


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
    INT,
    FLOAT, 
    IDENTIFIER, 
    LESS,
    GREATER,
    LESS_EQUALS,
    GREATER_EQUALS,
    EOF,
    LETTER,
    UNACCOUNTED,
    WORD
}
impl fmt::Display for tokenTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant_str = match self {
            tokenTypeEnum::PLUS => "PLUS",
            tokenTypeEnum::MINUS => "MINUS",
            tokenTypeEnum::IF_RW => "IF_RW",
            tokenTypeEnum::LOOP_RW => "LOOP_RW",
            tokenTypeEnum::END_RW => "END_RW",
            tokenTypeEnum::L_PAREN => "L_PAREN",
            tokenTypeEnum::R_PAREN => "R_PAREN",
            tokenTypeEnum::L_BRACKET => "L_BRACKET",
            tokenTypeEnum::R_BRACKET => "R_BRACKET",
            tokenTypeEnum::INT => "INT",
            tokenTypeEnum::FLOAT => "FLOAT",
            tokenTypeEnum::IDENTIFIER => "IDENTIFIER",
            tokenTypeEnum::LESS => "LESS",
            tokenTypeEnum::GREATER => "GREATER",
            tokenTypeEnum::LESS_EQUALS => "LESS_EQUALS",
            tokenTypeEnum::GREATER_EQUALS => "GREATER_EQUALS",
            tokenTypeEnum::EOF => "EOF",
            tokenTypeEnum::LETTER => "LETTER",
            tokenTypeEnum::UNACCOUNTED => "UNACCOUNTED",
            tokenTypeEnum::WORD => "WORD",

        };
        write!(f, "{}", variant_str)
    }
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
    fn scan(&mut self) -> token{
        //Gets the next character
        let mut currChar = self.inputFile.getChar();

        //Looks for the filler characters and removes them
        while let Some(c) = currChar {
            if c == '\n' || c == '\t' || c == '\r' || c == ' ' {
                //println!("Filler character found: '{}'", c);
                
                if c == '\n' {
                    self.inputFile.incLineCnt();
                }
                currChar = self.inputFile.getChar();
            } else {
                break; // Exit the loop if the character is not a filler character
            }
        }

        //A segment to parse/ignore comments goes here:
        if let Some('/') = currChar {
            currChar = self.inputFile.getChar();
            let Some(c) = currChar else { todo!() };
            if c == '/' {
                // println!("Comment line found");
                while let Some(c) = currChar {
                    if c == '\n' {
                        self.inputFile.incLineCnt();
                        currChar = self.inputFile.getChar();
                        break;
                    } else {
                        currChar = self.inputFile.getChar();
                    }
                }
            }
        }

        //A switch case to handle the different things that it could be to look ahead
        //println!("{}", currChar);
        let mut tokenString: String = "".to_string();
        match currChar {
            //If the character is a letter
            Some(ch) if ch.is_ascii_alphabetic() => {
                //Starts the tokenString
                tokenString.push(ch);
                let mut tokType: tokenTypeEnum = tokenTypeEnum::WORD;
                //Iterates through until it stops finding numbers
                while let Some(numC) = currChar {
                    if numC.is_ascii_alphabetic() {
                        tokenString.push(numC);
                        currChar = self.inputFile.getChar();
                    } else {
                        break;
                    }
                }
                self.inputFile.unGetChar();
                let newToken: token = token::new(tokType, tokenString);
                return newToken;
            }

            //If the character is a number
            Some(ch) if ch.is_ascii_digit() => {
                //Starts the tokenString
                tokenString.push(ch);
                let mut tokType: tokenTypeEnum = tokenTypeEnum::INT;
                //Iterates through until it stops finding numbers
                while let Some(numC) = currChar {
                    if numC.is_ascii_digit() {
                        tokenString.push(numC);
                        currChar = self.inputFile.getChar();
                    //If the number has a decimal, meaning its a float
                    } else if numC == '.' {
                        tokenString.push(ch);
                        tokType = tokenTypeEnum::FLOAT;
                        currChar = self.inputFile.getChar();
                    } else {
                        break;
                    }
                }
                self.inputFile.unGetChar();
                let newToken: token = token::new(tokType, tokenString);
                return newToken;
            }

            Some('<') => {
                //println!("This character is a <.");
                let mut nextNextChar = self.inputFile.getChar();
                let Some(nextC) = nextNextChar else { todo!() };
                if nextC == '=' {
                    // println!("This is a <=");
                    let newToken = token::new(crate::tokenTypeEnum::LESS_EQUALS, tokenString);
                    return newToken;
                } else {
                    // println!("This is just a <");
                    self.inputFile.unGetChar();
                    let newToken = token::new(crate::tokenTypeEnum::LESS, tokenString);
                    return newToken;
                }
            }

            Some('>') => {
                //println!("This character is a <.");
                let mut nextNextChar = self.inputFile.getChar();
                let Some(nextC) = nextNextChar else { todo!() };
                if nextC == '=' {
                    // println!("This is a >=");
                    let newToken = token::new(crate::tokenTypeEnum::GREATER, tokenString);
                    return newToken;
                } else {
                    // println!("This is just a >");
                    self.inputFile.unGetChar();
                    let newToken = token::new(crate::tokenTypeEnum::GREATER_EQUALS, tokenString);
                    return newToken;
                }
            }

            Some(c) => {
                // println!("This character is unaccounted for '{}'", c);
                let newToken = token::new(crate::tokenTypeEnum::UNACCOUNTED, tokenString);
                return newToken;
            }
            None => {
                // println!("This character is an None");
                let newToken = token::new(crate::tokenTypeEnum::EOF, tokenString);
                return newToken;
            }
        }
    }



    //A function to scan through entire file
    fn scanThrough(&mut self){
        while self.inputFile.currentCharIndex < self.inputFile.numChars{
            let newToken: token = self.scan();
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
    //"ungets" the next character by decrementing the current index. Used for looking ahead then going back
    fn unGetChar(&mut self) {
        self.currentCharIndex -= 1;
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
    tokenString: String,
    //To be completed later when I understand
    //tm: tokenMark,
}
impl token{
    //Init for the token
    fn new(iden: tokenTypeEnum, tokenString: String) -> token{
        //self.tokenMark = NULL;
        // println!("Created the token struct");
        println!("Created a token of type: '{}'", iden.to_string());

        token {
            tt: iden,
            tokenString: tokenString,
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