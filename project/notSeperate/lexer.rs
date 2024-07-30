#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_variables)]

//Crate imports
extern crate anyhow;
extern crate parse_display;
extern crate utf8_chars;
extern crate unicode_segmentation;

//package imports
use {
    anyhow::Result, parse_display::Display, std::{
        collections::HashMap, env, fmt, fs::{
            read_to_string, File
        }, hash::Hash, io::{
            prelude::*, BufRead, BufReader, Read
        }, path::Path, rc::Rc


    }, unicode_segmentation::UnicodeSegmentation, utf8_chars::BufReadCharsExt
};

///////////////////////// Setup /////////////////////////

//imports
use std::io::prelude::*;

//The enumeration for saving Token types, this is a list of every type of Token there is
#[derive(Clone, PartialEq)]
pub enum tokenTypeEnum{
    //Operators
    PLUS, 
    MINUS,
    LESS,
    GREATER,
    LESS_EQUALS,
    GREATER_EQUALS,
    SET_EQUALS,
    CHECK_EQUALS,
    NOT_EQUALS,
    MULTIPLY,
    DIVIDE,
    AND,
    OR,
    NOT,
    // OPERATOR,
    
    
    //Variable types
    INT,
    FLOAT, 
    STRING,

    //Word types
    IDENTIFIER, 
    
    //Keywords
    IF,
    ELSE,
    GLOBAL,
    VARIABLE,
    THEN,
    END,
    

    IF_RW, 
    LOOP_RW, 
    END_RW, 
    L_PAREN, 
    R_PAREN,
    L_BRACKET, 
    R_BRACKET,
    
    EOF,
    LETTER,
    UNACCOUNTED,
    WORD,
    RETURN,
    ERROR,
    PROGRAM,
    IS,
    BEGIN,
    PROCEDURE,
    SEMICOLON,
    COLON,
    PERIOD,
    END_PROGRAM,
    END_PROCEDURE,
    END_IF,
    END_FOR,
    COMMA,
    FOR,

    PROCEDURE_CALL,

    
    
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
            tokenTypeEnum::STRING => "STRING",
            tokenTypeEnum::RETURN => "RETURN",
            tokenTypeEnum::SET_EQUALS => "SET_EQUALS",
            tokenTypeEnum::CHECK_EQUALS => "CHECK_EQUALS",
            tokenTypeEnum::ERROR => "ERROR",
            tokenTypeEnum::PROGRAM => "PROGRAM",
            tokenTypeEnum::IS => "IS",
            tokenTypeEnum::BEGIN => "BEGIN",
            tokenTypeEnum::PROCEDURE => "PROCEDURE",
            tokenTypeEnum::IF => "IF",
            tokenTypeEnum::ELSE => "ELSE",
            tokenTypeEnum::GLOBAL => "GLOBAL",
            tokenTypeEnum::VARIABLE => "VARIABLE",
            tokenTypeEnum::THEN => "THEN",
            tokenTypeEnum::END => "END",
            tokenTypeEnum::SEMICOLON => "SEMICOLON",
            tokenTypeEnum::COLON => "COLON",
            tokenTypeEnum::PERIOD => "PERIOD",
            tokenTypeEnum::END_PROCEDURE => "END_PROCEDURE",
            tokenTypeEnum::END_PROGRAM => "END_PROGRAM",
            tokenTypeEnum::END_IF => "END_IF",
            tokenTypeEnum::MULTIPLY => "MULTIPLY",
            tokenTypeEnum::DIVIDE => "DIVIDE",
            tokenTypeEnum::COMMA => "COMMA",
            tokenTypeEnum::END_FOR => "END_FOR",
            tokenTypeEnum::FOR => "FOR",
            tokenTypeEnum::PROCEDURE_CALL => "PROCEDURE_CALL",
            tokenTypeEnum::AND => "AND",
            tokenTypeEnum::OR => "OR",
            tokenTypeEnum::NOT => "NOT",
            tokenTypeEnum::NOT_EQUALS => "NOT_EQUALS",



            // tokenTypeEnum::OPERATOR => "OPERATOR",


        };
        write!(f, "{}", variant_str)
    }
}

///////////////////////// /Setup /////////////////////////



///////////////////////// LEXER SECTION /////////////////////////
//This section contains all of the necessary code for the Lexical analysis section of the code.
//This includes all of the structs and functions that make up token definitions and such.

//This is the master struct for the lexer
struct Lexer {
    //tokenType: tokenTypeEnum,
    inputFile: inFile,
    symTab: tokenTable,
    tokenList: Vec<Token>,
    reports: Reporting,
    // reservedWords: [&str; 10],

    
}
impl Lexer{
    fn new(fileName: &str) -> Lexer {
        println!("Beginning creation of Lexer");
        let newFile = inFile::new(fileName);
        println!("Lexer created successfully");
        let mut symTable = tokenTable::new();
        let mut report: Reporting = Reporting::new();


        Lexer { 
            //tokenType: (), 
            inputFile: newFile,
            symTab: symTable,
            tokenList: Vec::new(),
            reports: report,
        }
    }
    
    //The main function if the lexer
    //Returns one Token
    fn scan(&mut self) -> Token{
        //Gets the next character
        let mut currChar = self.inputFile.getChar();


        //Looks for the filler characters and removes them
        while let Some(c) = currChar {
            
            if c == '\n' || c == '\t' || c == '\r' || c == ' ' || c == '\u{0009}' {
                // println!("Filler character found: '{}'", c);
                
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
                        
                        // currChar = self.inputFile.getChar();
                        break;
                    } else {
                        currChar = self.inputFile.getChar();
                    }
                }
            } else if c == '*' {
                // println!("multiline comment");
                let mut nested: usize = 1;
                // println!("Comment line found");
                while let Some(c) = currChar {
                    if c == '/' {
                        // println!("scope +1 nested");
                        currChar = self.inputFile.getChar();
                        let Some(ch) = currChar else { todo!() };
                        if ch == '*' {
                            nested += 1;
                            currChar = self.inputFile.getChar();
                        }
                    } else if c == '*' {
                        currChar = self.inputFile.getChar();
                        let Some(ch) = currChar else { todo!() };
                        if ch == '/' {
                            nested -= 1;
                            if nested == 0 {
                                currChar = self.inputFile.getChar();
                                // println!("End of nested comment");
                                break;
                            } else {
                                currChar = self.inputFile.getChar();
                                // println!("Not end of internal nested comment");
                            }
                        }
                    } else if c == '\n' {
                        self.inputFile.incLineCnt();
                        currChar = self.inputFile.getChar();

                          
                    } else {
                        currChar = self.inputFile.getChar();
                    }
                }
            } else if c == ' ' {
                let tokenString = '/';
                let newToken = Token::new(crate::tokenTypeEnum::DIVIDE,tokenString.to_string(), self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                return newToken;
            }
        }

        //A switch case to handle the different things that it could be to look ahead
        //println!("{}", currChar);
        let mut tokenString: String = "".to_string();
        match currChar {
            //If the character is a letter
            Some(ch) if ch.is_ascii_alphabetic() => {
                //Starts the tokenString
                //tokenString.push(ch);

                let mut tokType: tokenTypeEnum = tokenTypeEnum::WORD;
                //Iterates through until it stops finding numbers
                while let Some(numC) = currChar {
                    if (numC.is_ascii_alphabetic() || numC.is_ascii_digit() || numC == '_')  {
                        tokenString.push(numC);
                        currChar = self.inputFile.getChar();
                    } else {
                        break;
                    }
                }
                self.inputFile.unGetChar();
                tokenString = tokenString.to_ascii_lowercase();
                let mut newToken = self.symTab.hashLook(tokenString, self.inputFile.lineCnt.to_string());
                newToken.lineNum = self.inputFile.lineCnt.to_string();
                //let newToken: Token = Token::new(tokType,tokenString, self.inputFile.lineCnt.to_string());
                return newToken;
            }

            //If the character is a number
            Some(ch) if ch.is_ascii_digit() => {
                //Starts the tokenString
                //tokenString.push(ch);
                let mut tokType: tokenTypeEnum = tokenTypeEnum::INT;
                //Iterates through until it stops finding numbers
                while let Some(numC) = currChar {
                    if numC.is_ascii_digit() {
                        tokenString.push(numC);
                        currChar = self.inputFile.getChar();
                    //If the number has a decimal, meaning its a float
                    } else if numC == '.' {
                        tokenString.push('.');
                        tokType = tokenTypeEnum::FLOAT;
                        currChar = self.inputFile.getChar();
                    } else {
                        break;
                    }
                }
                self.inputFile.unGetChar();
                let mut newToken = self.symTab.hashLook(tokenString, self.inputFile.lineCnt.to_string());
                newToken.lineNum = self.inputFile.lineCnt.to_string();
                if newToken.tt != tokType {
                    newToken.tt = tokType;
                }
                //let newToken: Token = Token::new(tokType,tokenString, self.inputFile.lineCnt.to_string());
                return newToken;
            }

            //If the character is a <
            Some('<') => {
                //println!("This character is a <.");
                let mut nextNextChar = self.inputFile.getChar();
                tokenString.push('<');
                let Some(nextC) = nextNextChar else { todo!() };
                if nextC == '=' {
                    // println!("This is a <=");
                    tokenString.push('=');
                    let newToken = Token::new(crate::tokenTypeEnum::LESS_EQUALS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                } else {
                    // println!("This is just a <");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::LESS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                }
            }

            //If the character is a >
            Some('>') => {
                //println!("This character is a <.");
                tokenString.push('>');
                let mut nextNextChar = self.inputFile.getChar();
                let Some(nextC) = nextNextChar else { todo!() };
                if nextC == '=' {
                    // println!("This is a >=");
                    tokenString.push('=');

                    let newToken = Token::new(crate::tokenTypeEnum::GREATER_EQUALS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                } else {
                    // println!("This is just a >");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::GREATER,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                }
            }

            //If the character is a =
            Some('=') => {
                //println!("This character is a <.");
                tokenString.push('=');
                let mut nextNextChar = self.inputFile.getChar();
                let Some(nextC) = nextNextChar else { todo!() };
                if nextC == '=' {
                    // println!("This is a >=");
                    tokenString.push('=');

                    let newToken = Token::new(crate::tokenTypeEnum::CHECK_EQUALS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                } else if nextC == ' ' {
                    // println!("This is just a >");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::SET_EQUALS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                } else {
                    println!("ERROR");

                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::ERROR,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OTHER);
                    return newToken;
                }
            }


            //If the character is a =
            Some('!') => {
                //println!("This character is a <.");
                tokenString.push('!');
                let mut nextNextChar = self.inputFile.getChar();
                let Some(nextC) = nextNextChar else { todo!() };
                if nextC == '=' {
                    // println!("This is a >=");
                    tokenString.push('=');

                    let newToken = Token::new(crate::tokenTypeEnum::NOT_EQUALS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                } else {
                    // println!("ERROR");

                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::ERROR,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OTHER);
                    return newToken;
                }
            }

            //If the character is a ;
            Some(';') => {
                // println!("Current line: {}", self.inputFile.lineCnt.to_string());
                tokenString.push(';');
                let newToken = Token::new(crate::tokenTypeEnum::SEMICOLON,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::SYMBOL);
                return newToken;
            }

            //If the character is a :
            Some(':') => {
                // /println!("This character is a <.");
                tokenString.push(':');
                let mut nextNextChar = self.inputFile.getChar();
                let Some(nextC) = nextNextChar else { todo!() };
                if nextC == '=' {
                    // println!("This is a :=");
                    tokenString.push('=');

                    let newToken = Token::new(crate::tokenTypeEnum::SET_EQUALS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                } else {
                    // println!("This is just a >");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::COLON,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                }
            }


            //If the character is a :
            Some('[') => {
                tokenString.push('[');
                let newToken = Token::new(crate::tokenTypeEnum::L_BRACKET,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::SYMBOL);
                return newToken;
            }

            //If the character is a :
            Some(']') => {
                tokenString.push(']');
                let newToken = Token::new(crate::tokenTypeEnum::R_BRACKET,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::SYMBOL);
                return newToken;
            }

            //If the character is a :
            Some('(') => {
                tokenString.push('(');
                let newToken = Token::new(crate::tokenTypeEnum::L_PAREN,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::SYMBOL);
                return newToken;
            }

            //If the character is a :
            Some(')') => {
                tokenString.push(')');
                let newToken = Token::new(crate::tokenTypeEnum::R_PAREN,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::SYMBOL);
                return newToken;
            }

            Some('+') => {
                tokenString.push('+');
                let newToken = Token::new(crate::tokenTypeEnum::PLUS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                return newToken;
            }

            Some('-') => {
                // println!("This character is a -.");
                tokenString.push('-');
                let mut nextNextChar = self.inputFile.getChar();
                let Some(nextC) = nextNextChar else { todo!() };
                
                // println!("This is just a -");
                self.inputFile.unGetChar();
                let newToken = Token::new(crate::tokenTypeEnum::MINUS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                return newToken;
                
            }


            Some('*') => {
                tokenString.push('*');
                let newToken = Token::new(crate::tokenTypeEnum::MULTIPLY,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                return newToken;
            }

            Some(',') => {
                tokenString.push(',');
                let newToken = Token::new(crate::tokenTypeEnum::COMMA,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::SYMBOL);
                return newToken;
            }

            Some('/') => {
                tokenString.push('/');
                let newToken = Token::new(crate::tokenTypeEnum::DIVIDE,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                return newToken;
            }

            //If the character is a :
            Some('.') => {
                tokenString.push('.');
                let newToken = Token::new(crate::tokenTypeEnum::PERIOD,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::SYMBOL);
                return newToken;
            }

            //If the character is a &
            Some('&') => {
                tokenString.push('&');
                let newToken = Token::new(crate::tokenTypeEnum::AND,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                return newToken;
            }

            //If the character is a |
            Some('|') => {
                tokenString.push('|');
                let newToken = Token::new(crate::tokenTypeEnum::OR,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                return newToken;
            }

            // //If the character is a |
            // Some('|') => {
            //     tokenString.push('|');
            //     let newToken = Token::new(crate::tokenTypeEnum::OR,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
            //     return newToken;
            // }
            

            // Some(',') => {
            //     tokenString.push(',');
            //     let newToken = Token::new(crate::tokenTypeEnum::COMMA,tokenString, self.inputFile.lineCnt.to_string());
            //     return newToken;
            // }

            //If the character is a "
            Some('"') => {
                currChar = self.inputFile.getChar();
                let mut tokType: tokenTypeEnum = tokenTypeEnum::WORD;
                // println!("Comment line found");
                while let Some(numC) = currChar {
                    if numC == '"' {
                        break;
                    } else {
                        tokenString.push(numC);
                    }
                    currChar = self.inputFile.getChar();

                }
                //self.inputFile.unGetChar();
                let mut newToken = self.symTab.hashLook(tokenString, self.inputFile.lineCnt.to_string());
                newToken.lineNum = self.inputFile.lineCnt.to_string();
                if newToken.tt != tokenTypeEnum::STRING {
                    newToken.tt = tokenTypeEnum::STRING;
                }
                //let newToken: Token = Token::new(tokType,tokenString, self.inputFile.lineCnt.to_string());
                return newToken;
            }
            
            //Somehow a \n makes it here, just runs it through another scan to get the next thing
            Some('\n') => {
                let newToken = self.scan();
                return newToken;
            }
            Some(c) => {
                // println!("This character is unaccounted for '{}'", c);
                tokenString.push(c);
                let newToken = Token::new(crate::tokenTypeEnum::UNACCOUNTED,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OTHER);
                return newToken;
            }
            None => {
                // println!("This character is a None aka EOF");
                let newToken = Token::new(crate::tokenTypeEnum::EOF, "EOF".to_string(), self.inputFile.lineCnt.to_string(), tokenGroup::SYMBOL);
                return newToken;
            }
        }
    }
    
    //Prints all of the tokens
    fn printTokenList(&mut self){
        for token in &self.tokenList {
            println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
        }
    }

    fn secondPass(&mut self) -> Vec<Token>{
        let mut newTokList = Vec::new(); 
        let mut i: usize = 0;
        while i < self.tokenList.len() {
            let token = &self.tokenList[i];
            match token.tt {
                tokenTypeEnum::END => {
                    // println!("End found");
                    let nextToken = &self.tokenList[i+1];
                    if nextToken.tt == tokenTypeEnum::PROGRAM {
                        // println!("Combining end and program");
                        let newToken = Token::new(crate::tokenTypeEnum::END_PROGRAM,"END_PROGRAM".to_string(), nextToken.lineNum.to_string(), tokenGroup::OTHER);
                        newTokList.push(newToken.clone());
                        i = i + 1;
                    } else if nextToken.tt == tokenTypeEnum::PROCEDURE {
                        // println!("Combining end and procedure");
                        let newToken = Token::new(crate::tokenTypeEnum::END_PROCEDURE,"END_PROCEDURE".to_string(), nextToken.lineNum.to_string(), tokenGroup::OTHER);
                        newTokList.push(newToken.clone());
                        i = i + 1;
                    } else if nextToken.tt == tokenTypeEnum::IF {
                        // println!("Combining end and if");
                        let newToken = Token::new(crate::tokenTypeEnum::END_IF,"END_IF".to_string(), nextToken.lineNum.to_string(), tokenGroup::OTHER);
                        newTokList.push(newToken.clone());
                        i = i + 1;
                    } else if nextToken.tt == tokenTypeEnum::FOR {
                        // println!("Combining end and if");
                        let newToken = Token::new(crate::tokenTypeEnum::END_FOR,"END_FOR".to_string(), nextToken.lineNum.to_string(), tokenGroup::OTHER);
                        newTokList.push(newToken.clone());
                        i = i + 1;
                    } else {
                        // println!("other end with type: {}", nextToken.tt);
                        newTokList.push(token.clone());

                    }
                }
                tokenTypeEnum::IDENTIFIER => {
                    let nextToken = &self.tokenList[i+1];
                    if nextToken.tt == tokenTypeEnum::L_PAREN {
                        // println!("Combining end and if");
                        let newToken = Token::new(crate::tokenTypeEnum::PROCEDURE_CALL, token.tokenString.clone(), nextToken.lineNum.to_string(), tokenGroup::SYMBOL);
                        newTokList.push(newToken.clone());
                        i = i + 1;
                    } else {
                        // println!("other end with type: {}", nextToken.tt);
                        newTokList.push(token.clone());

                    }
                }
                tokenTypeEnum::UNACCOUNTED => {
                    println!("Skipping unaccounted");
                    println!("Unaccounted: {}", token.tokenString);
                    let nextToken = &self.tokenList[i+1];
                    println!("Next token: {}", nextToken.tokenString);
                    newTokList.push(nextToken.clone());
                    i = i + 1;
                }
                tokenTypeEnum::MINUS => {
                    let nextToken = &self.tokenList[i+1];
                    let prevToken = &self.tokenList[i-1];
                    if (nextToken.tg == tokenGroup::VARIABLE) && (prevToken.tg == tokenGroup::OPERATOR) {
                        // println!("Found a neg number");
                        let newString = format!("-{}", nextToken.tokenString.clone());
                        let newToken = Token::new(nextToken.tt.clone(), newString, nextToken.lineNum.to_string(), tokenGroup::NUMBER);
                        newTokList.push(newToken.clone());
                        i = i + 1;
                    } else {
                        // println!("Just a minus");
                        // println!("Previous group: {} nextGrou: {} nextString: {}", prevToken.tg, nextToken.tg, nextToken.tokenString);
                        // println!("other end with type: {}", nextToken.tt);
                        newTokList.push(token.clone());

                    }
                }
                _ => {
                    // Handle other token types
                    newTokList.push(token.clone());
                }
            }
            i = i + 1;
        }
        return newTokList;
    }

    //A function to scan through entire file
    fn scanThrough(&mut self){


        println!("\nBeginning scan:");

        //Scans the first token and initializes the newToken variable
        let mut newToken: Token = self.scan();
        self.tokenList.push(newToken.clone());
        // println!("First token: < \"{}\" , {} >", newToken.tokenString, newToken.tt.to_string());




        while newToken.tokenString != "EOF".to_string(){
            newToken = self.scan();
            self.tokenList.push(newToken.clone());
            // println!("< \"{}\" , {} >", newToken.tokenString, newToken.tt.to_string());
        };
        // println!("\n\nEOF Reached");

        // println!("Starting second pass");
        let newTokList = self.secondPass();
        self.tokenList = newTokList;
        println!("Second pass finished");

        
        
    }

}

//inFile Class, this is where the file to be compiled is loaded
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
            lineCnt: 1,
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

//Token class, this is where tokens are defined and setup
#[derive(Clone, PartialEq)]
struct Token{
    tt: tokenTypeEnum,
    tokenString: String,
    tg: tokenGroup,
    pub lineNum: String,
    //To be completed later when I understand
    //tm: tokenMark,
}
impl Token{
    //Init for the Token
    fn new(iden: tokenTypeEnum, tokenString: String, line: String, group: tokenGroup) -> Token{
        Token {
            tt: iden,
            tokenString: tokenString,
            lineNum: line,
            tg: group,
        }
    }
    //Used for setting the Token type
    fn setTokenType(&mut self, newType: tokenTypeEnum){
        self.tt = newType;
    }

    fn printToken(&mut self){
        println!("< \"{}\" , {} >", self.tokenString, self.tt.to_string());
    }
}

//Token Function class, derived from Token class
struct tokenFunction{
    parent: Token,
    tokStr: String,
    argList: Token<>,
    returnType: Token,
}

//The structure for the SymbolTable. This holds all of the IDENTIFIERS of the program as well as their scope and information
struct tokenTable{
    // For now you can simply use a single hash table of tokens. As we move forward to parsing, the symbol table
    // structure will have to be augmented to permit the recording of entering/exiting program scopes as well as
    // the scope that an IDENTIFIER is declared. In general when you exit a scope the symbol table will remove
    // any symbols defined in that scope from the symbol table. Again, we will solve this problem later; the
    // example methods for scope entry/exit are here to deomonstrate what we will probably want in the future
    tokTab: HashMap<String, Token>,
}
impl tokenTable{
    // The symbol table hashLook function should automatically create a new entry and mark it as an
    // IDENTIFIER Token for any IDENTIFIER string that is not already in the symbol table. In some languages
    // case does not matter to the uniqueness of the symbol. In this case, an easy place to solve this is to simply
    // upper case or lower case all strings in the symbol table API functions (and storage)
    fn new() -> tokenTable {
        //Creates the empty hash map
        let mut symHash: HashMap<String, Token> = HashMap::new();

        //List of all of the tokens that should be in the symbol table when initializes. Like all of the reserved words and such
        let tokens = vec![
            ("if", Token::new(tokenTypeEnum::IF, "if".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("else", Token::new(tokenTypeEnum::ELSE, "else".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("procedure", Token::new(tokenTypeEnum::PROCEDURE, "procedure".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("is", Token::new(tokenTypeEnum::IS, "is".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("global", Token::new(tokenTypeEnum::GLOBAL, "global".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("variable", Token::new(tokenTypeEnum::VARIABLE, "variable".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("begin", Token::new(tokenTypeEnum::BEGIN, "begin".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("then", Token::new(tokenTypeEnum::THEN, "then".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("end", Token::new(tokenTypeEnum::END, "end".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("program", Token::new(tokenTypeEnum::PROGRAM, "program".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("return", Token::new(tokenTypeEnum::RETURN, "return".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("for", Token::new(tokenTypeEnum::FOR, "for".to_string(), "0".to_string(), tokenGroup::KEYWORD)),
            ("not", Token::new(tokenTypeEnum::NOT, "not".to_string(), "0".to_string(), tokenGroup::OPERATOR)),



        ];

        for (key, value) in tokens {
            symHash.insert(key.to_string(), value);
        }

        // println!("token table created and seeded");
        // for (key, token) in &mut symHash {
        //     println!("Key: {}, Token: {:?}", key, token.printToken());
        // }


        tokenTable{
            tokTab: symHash,
        }
    }
    //Returns the Token for a given string
    fn hashLook(&mut self, mut lookupString: String, line: String) -> Token{
        // println!("Looking up the identifier of the string");
        if let Some(tokenResp) = self.tokTab.get(&lookupString){
            // println!("Token found");
            return tokenResp.clone();
        } else {
            // println!("Token not found, creating");
            let newToken = Token::new(tokenTypeEnum::IDENTIFIER, lookupString, line.to_string(), tokenGroup::VARIABLE);
            self.tokTab.insert(newToken.tokenString.clone(), newToken.clone());
            return newToken;
        }
    }
    // fn enterScope(){
    //     println!("To be used in the future");
    // }
    // fn exitScope(){
    //     println!("To be used in the future");
    // }
}

//An enum used in conjunction with tokenType for parsing purposes
#[derive(Clone, PartialEq)]
pub enum tokenGroup{
    OPERATOR,
    KEYWORD,
    VARIABLE,
    OTHER,
    SYMBOL,
    NUMBER,
}
//Display for tokenGroup
impl fmt::Display for tokenGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant_str = match self {
            &tokenGroup::OPERATOR => "OPERATOR",
            &tokenGroup::KEYWORD => "KEYWORD",
            &tokenGroup::VARIABLE => "VARIABLE",
            &tokenGroup::OTHER => "OTHER",
            &tokenGroup::SYMBOL => "SYMBOL",
            &tokenGroup::NUMBER => "NUMBER",

        };
        write!(f, "{}", variant_str)
    }
}

///////////////////////// /LEXER SECTION /////////////////////////



///////////////////////// HELPER SECTION /////////////////////////

//Structure for reporting errors and warnings
#[derive(Debug, Clone, PartialEq)]
pub struct Reporting {
    error_status: bool,
    warnings: Vec<String>,
    errors: Vec<String>,
}

impl Reporting {
    pub fn new() -> Reporting {
        Reporting {
            error_status: false,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn reportError(&mut self, message: String) {
        self.errors.push(message.clone());
    }

    pub fn reportWarning(&mut self, message: String) {
        self.warnings.push(message.clone());
    }
}

impl std::fmt::Display for Reporting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Errors: {:?}, Warnings: {:?}", self.errors, self.warnings)
    }
}

impl From<String> for Reporting {
    fn from(error: String) -> Self {
        let mut reporting = Reporting::new();
        reporting.reportError(error);
        reporting
    }
}

fn printTokList(tokList: &Vec<Token>){
    for token in tokList {
        println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
    }
}

///////////////////////// /HELPER SECTION /////////////////////////
