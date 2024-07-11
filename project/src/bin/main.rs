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
        collections::HashMap,
        rc::Rc,


    },
    utf8_chars::BufReadCharsExt,
};

///////////////////////// Setup /////////////////////////

//imports
use std::io::prelude::*;

//The enumeration for saving Token types, this is a list of every type of Token there is
#[derive(Clone)]
#[derive(PartialEq)]
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
    MULTIPLY,
    DIVIDE,
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

            // tokenTypeEnum::OPERATOR => "OPERATOR",


        };
        write!(f, "{}", variant_str)
    }
}

#[derive(Clone)]
#[derive(PartialEq)]
pub enum tokenGroup{
    OPERATOR,
    KEYWORD,
    VARIABLE,
    OTHER,
    SYMBOL,
    NUMBER,
}

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


///////////////////////// /Setup /////////////////////////





///////////////////////// LEXER SECTION /////////////////////////
//This section contains all of the necessary code for the Lexical analysis section of the code.
//This includes all of the structs and functions that make up token definitions and such.

//This is the master struct for the lexer
struct Lexer {
    //tokenType: tokenTypeEnum,
    inputFile: inFile,
    symTab: symbolTable,
    tokenList: Vec<Token>,
    reports: Reporting,
    // reservedWords: [&str; 10],

    
}
impl Lexer{
    fn new(fileName: &str) -> Lexer {
        println!("Beginning creation of Lexer");
        let newFile = inFile::new(fileName);
        println!("Lexer created successfully");
        let mut symTable = symbolTable::new();
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
                        
                        currChar = self.inputFile.getChar();
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
                                break;
                            } else {
                                currChar = self.inputFile.getChar();
                            }
                        }
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
                    if (numC.is_ascii_alphabetic() || numC.is_ascii_digit())  {
                        tokenString.push(numC);
                        currChar = self.inputFile.getChar();
                    } else {
                        break;
                    }
                }
                self.inputFile.unGetChar();
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
                if nextC.is_ascii_digit() {
                    // println!("Found a neg number");
                    tokenString.push(nextC);
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
                            // self.inputFile.unGetChar();
                            break;
                        }
                    }
                    let mut newToken = self.symTab.hashLook(tokenString, self.inputFile.lineCnt.to_string());
                    newToken.lineNum = self.inputFile.lineCnt.to_string();
                    if newToken.tt != tokType {
                        newToken.tt = tokType;
                    }
                    //let newToken: Token = Token::new(tokType,tokenString, self.inputFile.lineCnt.to_string());
                    return newToken;
                } else {
                    // println!("This is just a -");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::MINUS,tokenString, self.inputFile.lineCnt.to_string(), tokenGroup::OPERATOR);
                    return newToken;
                }
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
#[derive(Clone)]
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

///////////////////////// /LEXER SECTION /////////////////////////




///////////////////////// PARSER SECTION /////////////////////////

//This is the master struct for the parser
struct Parser {
    pub tokenList: Vec<Token>,
    reports: Reporting,
}
impl Parser{
    //Initialization function
    fn new(lexer: &mut Lexer) -> Parser {
        // println!("\nBeginning creation of Parser");
        let tokenList = lexer.tokenList.clone();
        // let newFile = inFile::new(fileName);
        println!("\n\nParser created");
        let mut report: Reporting = Reporting::new();


        Parser { 
            tokenList,
            reports: report,
        }
    }  


    fn startParse(&mut self) -> Result<(Reporting, Option<Stmt>), Reporting> {
        println!("Starting master parse");
        let mut tokList = self.tokenList.clone();
        let parsed = self.parse(&mut tokList);

        match parsed {
            Ok((Some(stmt))) => {
                println!("program parsed successfully");
                // stmt.display(0);
                // headerBlock.push_to_block(stmt.clone());
                // headerStmt = stmt;
                return Ok((self.reports.clone(), Some(stmt)));
                // headerReporting = reporting;
            },
            Ok((None)) => {
                println!("program parsed but no statement returned.");
                return Ok((self.reports.clone(), None));
                // headerReporting = reporting;
                // headerStmt = Stmt::StringLiteral("No header returned".to_string());
            },
            Err(reporting) => {
                // println!("Error parsing header: {:?}", reporting);
                let errMsg = format!("Error parsing program: {:?}", self.reports);
                return Err(self.reports.clone()); // Propagate the error up the call stack
            },
        }

    }

    fn processBlock(&mut self, block: &Stmt) -> Result<Expr, String> {
        match block {
            Stmt::Block(stmts) => {
                if let Some(first_stmt) = stmts.first() {
                    match first_stmt {
                        Stmt::Expr(expr) => {
                            // If the first statement is an Expr, return it
                            Ok(expr.clone()) // Cloning to return a new instance
                        },
                        _ => Err("First statement in Block is not an Expr".to_string()),
                    }
                } else {
                    Err("Block is empty".to_string())
                }
            },
            _ => Err("Expected Stmt::Block, but received a different Stmt type".to_string()),
        }
    }


    //Parses an expressions and returns the Expr
    fn parseExpr(&mut self, tokenList: &mut Vec<Token>) -> Result<Expr, String> {
        // println!("Parsing expression");


        //Initializes the variable that is being referenced first
        let mut varRef:Expr = Expr::StringLiteral(("NONE".to_string()));

        //Initializes values for finding the end of the expression
        let mut k = 0;
        let mut nextTok = &tokenList[k];
        let mut curStmt: Vec<Token> = vec![];
        
        //Finds the end of the expression
        while k < tokenList.len() {
            let nextTok = &tokenList[k];
            curStmt.push(nextTok.clone());
        
            if (nextTok.tt == tokenTypeEnum::SEMICOLON){
                break; // Stop loop when semicolon or parentheses is found
            }
        
            k += 1;
        }

        // println!("Full statement:");
        // printTokList(&curStmt);


        //Checks if the first value is an array reference or not
        if(curStmt[1].tt == tokenTypeEnum::L_BRACKET) {
            // println!("Array reference found");
            let varName = curStmt[0].tokenString.clone();
            // let varRef = Expr::VarRef((varName));

            //Finds the end of the brackets
            let mut brackInd = 0;
            let mut nextIndToken = &curStmt[brackInd];
            let mut indexList: Vec<Token> = vec![];
            
            //Finds the end of the index statement
            while brackInd < curStmt.len() {
                let nextTok = &curStmt[brackInd];
                indexList.push(nextTok.clone());
            
                if (nextTok.tt == tokenTypeEnum::R_BRACKET) {
                    break; // Stop loop when semicolon or parentheses is found
                }
            
                brackInd += 1;
            }

            //If the end of the index was not found, error
            if brackInd == curStmt.len() {
                println!("THIS IS AN ERROR in finding end of array index, add error and reporting");
                tokenList.drain(..k+1);
                let errMsg = format!("Error finding the end of the array index on line {}", curStmt[1].lineNum);
                return(Err(errMsg));
            }

            //Removes the index assignment so there is just the ] left for proper parsing of index expression
            indexList.drain(0..2);
            // indexList.drain(indexList)


            // println!("Here is the index list:");
            // printTokList(&indexList);


            let parsedExpr = self.parseExpr(&mut indexList);
            let mut indexExpr: Expr;

            //Extracts the resulting expression
            match parsedExpr {
                Ok(expr) => {
                    indexExpr = expr;
                }
                Err(err) => {
                    // println!("Error parsing expression");
                    let errMsg = format!("Error on line {}: {}", tokenList[0].lineNum, err);
                    self.reports.reportError(errMsg);
                    return Err("Error with expression".to_string());
                }
            }

            // println!("New value expressions: {}", newValueExpr);

            let indexBox = Box::new(indexExpr);
            varRef = Expr::ArrayRef((varName), (indexBox));

            //Removes the array reference so there is just the ] left
            curStmt.drain(0..brackInd);
        } else if (curStmt[0].tg == tokenGroup::VARIABLE){
            //If not an array
            varRef = Expr::VarRef(curStmt[0].tokenString.clone());

        } else if (curStmt[0].tt == tokenTypeEnum::L_PAREN) { 
            // println!("Parentheses found!!!!");
            let mut scope = 0;

            //Initializes values for finding the end of the expression
            let mut parenInd = 1;
            // let mut nextParenTok = &curStmt[1];
            let mut parStmt: Vec<Token> = vec![];
            
            //Finds the end of the expression
            while parenInd < curStmt.len() {
                let nextParenTok = &curStmt[parenInd];
                parStmt.push(nextParenTok.clone());
                
                if (nextParenTok.tt == tokenTypeEnum::L_PAREN){
                    scope += 1;
                    // println!("Scope increase");
                }

                if (nextParenTok.tt == tokenTypeEnum::R_PAREN){
                    if(scope != 0){
                        // println!("Scope decrease");

                        scope -= 1; //decrease scope if not the final
                    } else {
                        // println!("found the end");

                        break; // Stop loop when final parentheses is found
                    }
                }
            
                parenInd += 1;
            }

            // println!("Final paren: {}", parStmt[parenInd - 1].tokenString);

            parStmt[parenInd - 1].tokenString = ";".to_string();
            parStmt[parenInd - 1].tt = tokenTypeEnum::SEMICOLON;
            parStmt[parenInd - 1].tg = tokenGroup::SYMBOL;



            let scanned = self.parseExpr(&mut parStmt);                            
            match scanned {
                Ok(stmt) => {
                    varRef = stmt;
                },
                Err(reporting) => {
                    println!("Error parsing paren expression: {:?}", reporting);
                    let errMsg = format!("Error parsing paren expr: {:?}", self.reports);

                    return Err(errMsg);
                },
            }
            // println!("extracted paren statement:");
            // printTokList(&parStmt);
            // println!("Parsed paren expression: {}", varRef);



            // varRef = Expr::IntLiteral((0));
            curStmt.drain(..parStmt.len());

        } else if (curStmt[0].tt == tokenTypeEnum::PROCEDURE_CALL) {
            println!("Procedure call");
            let procName = curStmt[0].tokenString.clone();
            if (curStmt[1].tt != tokenTypeEnum::R_PAREN) {
                println!("There are params");
                let mut paramInd = 0;
                let mut params: Vec<Expr> = Vec::new();
                let mut p = 1;
                let mut pToken;
                while p < curStmt.len() {
                    pToken = curStmt[p].clone();
                    if(pToken.tt == tokenTypeEnum::R_PAREN) {
                        break;
                    }
                    let paramExpr = Expr::new(pToken.tt.clone(), Some(pToken.tokenString.clone()));
                    match paramExpr{
                        Ok(expr) => {
                            params.push(expr);
                            p += 1;
                        }
                        _ => {
                            println!("Something wrong with parameter");
                            p += 1;
                        }
                    }
                }

                // println!("Extracted parameters:");
                // for param in params.clone(){
                //     println!("{}", param);
                // }
                let procCall = Expr::ProcCall((procName), (Some(params.clone())));
                varRef = procCall;
                curStmt.drain(0..p);


            } else {
                println!("No params");
                let procCall = Expr::ProcCall((procName), (None));
                varRef = procCall;
                curStmt.drain(0..1);
            }
        }
        
        else {
            // println!("Expression first not a variable reference");
            match &varRef {
                Expr::StringLiteral(s) if s == "NONE" => {
                    println!("Uninitializes varRef");
                    let empty:Expr = Expr::StringLiteral(("NONE".to_string())); 
                    let valRef = Expr::new(curStmt[0].tt.clone(), Some(curStmt[0].tokenString.clone()));
                    match valRef {
                        Ok(expr) => {
                            varRef = expr;
                        } Err(err) => {
                            let errMsg = format!("error on line {}: {}", curStmt[0].lineNum.to_string(), err);
                            return(Err(errMsg));
                        }
                    }
                }
                _ => {
                    println!("Initialized");
                }
            }
            
            
            
        }


        // println!("We are doing something with this value: {}", varRef);
        // // // println!("First after variable: {}", curStmt[1].tokenString);
        // println!("Remaining items: {}", curStmt.len().to_string());
        // printTokList(&curStmt);

        // match tokenList


        if(curStmt.len() > 2){
            // println!("Complex expression");

            let operand1 = varRef;
            let operatorRes = BinOp::new(curStmt[1].tt.clone());
            let mut operator: BinOp;
            match operatorRes {
                Ok(op) => {
                    operator = op;
                },
                Err(reporting) => {
                    println!("Error parsing op: {:?}", reporting);
                    println!("The fucked up guy in question: {}", curStmt[1].tokenString);
                    let errMsg = format!("Error parsing operator on line {}: {:?}", curStmt[1].lineNum.to_string(), self.reports);
                    return Err(errMsg);
                },
            }

            let mut subList = curStmt.clone();
            subList.drain(0..2); 
            // println!("First token: {}", subList[0].tokenString);
            let mut parsedExpr: Expr;
            let scanned = self.parseExpr(&mut subList);                            
            let mut headerStmt:Expr;
            // let mut headerReporting = Reporting::new();
            match scanned {
                Ok(stmt) => {
                    parsedExpr = stmt;
                },
                Err(reporting) => {
                    println!("Error parsing expression TEST: {:?}", reporting);
                    let errMsg = format!("Error parsing body: {:?}", self.reports);

                    return Err(errMsg);
                },
            }

            let op1Box = Box::new(operand1);
            let op2Box = Box::new(parsedExpr);
            let retExpr = Expr::BinOp((op1Box), (operator), (op2Box));
            // println!("Expression parsed: {}", retExpr);
            // let retStmt = Stmt::Assign(varName.to_string(), parsedExpr);
            
            // parsedStmt.display(0);
            // tokenList.drain(0..k+1);
            return Ok(retExpr);            
        } else if (curStmt.len() == 2) {
            // println!("Simple expressions");
            
            if(curStmt[0].tt == tokenTypeEnum::R_PAREN){
                return(Ok(varRef));
            }


            let valueRes = Expr::new(tokenList[0].tt.clone(), Some(tokenList[0].tokenString.clone()));
            let mut valueExpr:Expr; 
            match valueRes {
                Ok(expr) => {
                    valueExpr = expr;
                }
                Err(err) => {
                    println!("Error parsing expression");
                    let errMsg = format!("Error on line {}: {}", tokenList[0].lineNum, err);
                    self.reports.reportError(errMsg);
                    return Err("Error with expression".to_string());
                }
            }

            tokenList.drain(0..k);
            return Ok(valueExpr);
        } else {
            // println!("ERROR: no expression to parse");
            // return(Err("No expression to parse".to_string()));
            return(Ok(varRef));
        }

        
    }

    fn parse(&mut self, tokenList: &mut Vec<Token>) -> Result<Option<Stmt>, String> {
        // let mut tokenList = &mut self.tokenList;

        // println!("Beginning individual parse");
            
        // let mut i: usize = 0;
        let tokLen: usize = tokenList.len();
        // Iterate through tokenList using an index
        
        // while i < tokLen {
        //     //Gets next token
        //     let token = &tokenList[i];
        //     // println!("current i: {} on token: {}", i.to_string(), token.tokenString);

        let mut token = &tokenList[0];
            
        match token.tt {
            tokenTypeEnum::PROGRAM => {
                //If program is just starting, check it.
                //Checks the first line
                let firstToken = &tokenList[0];
                if let tokenTypeEnum::PROGRAM = firstToken.tt {
                    let thirdToken = &tokenList[2];
                    if let tokenTypeEnum::IS = thirdToken.tt {
                        //Gets the program name
                        let programName: String = tokenList[1].tokenString.clone();                                
                        
                        //Removes the program statement
                        tokenList.drain(0..3);

                        //Finds where the header ends and the body begins
                        let mut beginInt = 0;
                        let mut beginScope = 0;
                        let iterTokList = tokenList.clone();
                        for token in iterTokList{
                            // println!("Current token: {}", token.tokenString);
                            if (token.tt == tokenTypeEnum::BEGIN) && (beginScope == 0){
                                // println!("Found the end");
                                break;
                            } else if (token.tt == tokenTypeEnum::PROCEDURE) {
                                beginScope = beginScope + 1;
                                beginInt = beginInt + 1;
                            } else if (token.tt == tokenTypeEnum::END_PROCEDURE){
                                beginInt = beginInt + 1;
                                beginScope = beginScope - 1;
                            } else {
                                beginInt = beginInt + 1;
                            }
                        }

                        // println!("begin index: {}", beginInt);

                        //Splits into two lists to parse seperately
                        let mut bodyList = tokenList.split_off(beginInt);

                        //Parses the header
                        let mut newHeader: Vec<Token> = tokenList.iter().cloned().map(|t| t.clone()).collect();
                        let mut headerBlock = Stmt::Block(Vec::new());
                        let mut headerI = 0;
                        let headerLen = newHeader.len();
                        while(!newHeader.is_empty()){
                            if(headerI > headerLen){
                                self.reports.reportError("Infinite loop in header".to_string());
                                println!("infinite loop in header");
                                return Err("infinite loop in header".to_string());
                            }
                            let scanned = self.parse(&mut newHeader);                            
                            let mut headerStmt:Stmt;
                            // let mut headerReporting = Reporting::new();
                            match scanned {
                                Ok((Some(stmt))) => {
                                    // println!("Header statement parsed successfully");
                                    // stmt.display(0);
                                    let _ = headerBlock.push_to_block(stmt.clone());
                                    // let _ = headerStmt = stmt;
                                    // headerReporting = reporting;
                                },
                                Ok((None)) => {
                                    println!("Parsed header statement but no statement returned.");
                                    // headerReporting = reporting;
                                    // headerStmt = Stmt::StringLiteral("No header returned".to_string());
                                },
                                Err(reporting) => {
                                    println!("Error parsing header: {:?}", reporting);
                                    let errMsg = format!("Error parsing header: {:?}", self.reports);
                                    return Err(errMsg);
                                    // return Err(("Error parsing header: {:?}", reporting).to_string()); // Propagate the error up the call stack
                                },
                            }
                        }

                        // println!("Finished parsing header: ");
                        // headerBlock.display(0);

                        

                        let mut newBody: Vec<Token> = bodyList.iter().cloned().map(|t| t.clone()).collect();
                        newBody.drain(0..1);
                        // println!("First in body: {}", newBody[0].tokenString);
                        let mut bodyBlock = Stmt::Block(Vec::new());
                        let mut bodyI = 0;
                        let bodyLen = newBody.len();
                        while(!newBody.is_empty()){
                            if(bodyI > bodyLen){
                                self.reports.reportError("Infinite loop in body".to_string());
                                println!("infinite loop in body");
                                return Err("infinite loop in body".to_string());
                            }
                            let scanned = self.parse(&mut newBody);                            
                            let mut headerStmt:Stmt;
                            // let mut headerReporting = Reporting::new();
                            match scanned {
                                Ok((Some(stmt))) => {
                                    // println!("Header statement parsed successfully");
                                    // stmt.display(0);
                                    let _ = bodyBlock.push_to_block(stmt.clone());
                                    bodyI = bodyI + 1;
                                    // let _ = headerStmt = stmt;
                                    // headerReporting = reporting;
                                },
                                Ok((None)) => {
                                    // println!("Parsed body statement but no statement returned.");
                                    bodyI = bodyI + 1;

                                    // headerReporting = reporting;
                                    // headerStmt = Stmt::StringLiteral("No header returned".to_string());
                                },
                                Err(reporting) => {
                                    println!("Error parsing body: {:?}", reporting);
                                    let errMsg = format!("Error parsing body: {:?}", self.reports);
                                    // bodyI = bodyI + 1;

                                    return Err(errMsg);
                                    // return Err(("Error parsing header: {:?}", reporting).to_string()); // Propagate the error up the call stack
                                },
                            }
                            
                        }

                        // println!("Finished parsing body: ");
                        // bodyBlock.display(0);

                        // Turns the statements into boxes
                        let boxHeader: Box<Stmt> = Box::new(headerBlock);
                        let boxBody: Box<Stmt> = Box::new(bodyBlock);


                        let programAst = Stmt::Program(programName.clone(), boxHeader, boxBody);
                        // programAst.display(0);

                        return Ok((Some(programAst)));

                    } else {
                        self.reports.reportError("Program declaration incorrect. \n Program must start with: 'program [Program name] is'".to_string());
                        println!("Error with program delcaration");
                        return Err("Error with program declaration".to_string());
                    }
                    // scope = 1;
                    // i = 3;
                } else {
                    self.reports.reportError("Program declaration incorrect. \n Program must start with: 'program [Program name] is'".to_string());
                    println!("Error with program delcaration");
                    return Err("Error with program declaration".to_string());
                }
                
            }
            tokenTypeEnum::VARIABLE => {
                let mut retStmt:Stmt;
                
                let mut k = 1;
                let mut nextTok = &tokenList[k];
                // println!("Found a variable token");
                let mut curStmt: Vec<&Token> = vec![];
                curStmt.push(token);
                while nextTok.tt != tokenTypeEnum::SEMICOLON {
                    curStmt.push(nextTok);
                    k = k + 1;
                    nextTok = &tokenList[k];

                    // println!("iterating");
                }
                curStmt.push(nextTok);
                // println!("Found the semicolon");
                let varName = &curStmt[1].tokenString;
                // println!("\nCurrent variable declaration name: {}", varName);
                
                // for token in &curStmt {
                //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
                // }

                //Error checking
                if curStmt[2].tt != tokenTypeEnum::COLON {
                    // println!("{}", curStmt[2].tokenString);
                    let errMsg = format!("In line: {}, Array variable declaration incorrect. \n Must be in this format: 'variable [Variable name] : [variable type]'", curStmt[3].lineNum,);
                    self.reports.reportError(errMsg);
                    return Err("Error with variable declaration".to_string());
                } else {
                    if (curStmt[4].tt != tokenTypeEnum::SEMICOLON) {
                        if curStmt[4].tt != tokenTypeEnum::L_BRACKET {
                            self.reports.reportError(format!(
                                "In line: {}, Array variable declaration incorrect. \n Must be in this format: 'variable [Variable name] : integer[arraySize]'", 
                                curStmt[3].lineNum, 
                            ));
                            return Err("Error with variable declaration".to_string());
                        } else {
                            if curStmt[3].tokenString == "integer" {
                                if curStmt[5].tt == tokenTypeEnum::INT {
                                    let arSizeStr = curStmt[5].tokenString.clone();
                                    if let Ok(arSize) = arSizeStr.parse::<usize>() {
                                        let newVar = Stmt::VarDecl(varName.clone(), VarType::IntArray(vec![0; arSize]));
                                        retStmt = newVar;
                                    } else {
                                        self.reports.reportError(format!(
                                            "In line: {}, Invlaid array size", 
                                            curStmt[3].lineNum, 
                                        ));
                                        return Err("Error with variable declaration".to_string());
                                    }
                                } else {
                                    self.reports.reportError(format!(
                                        "In line: {}, Array variable declaration incorrect. \n Must be in this format: 'variable [Variable name] : integer[arraySize]'", 
                                        curStmt[3].lineNum, 
                                    ));
                                    return Err("Error with variable declaration".to_string());
                                }
                            } else {
                                self.reports.reportError(format!(
                                    "In line: {}, '{}' is not a valid variable type", 
                                    curStmt[3].lineNum, 
                                    curStmt[3].tokenString
                                ));
                                return Err("Error with variable declaration".to_string());
                            }
                        }
                    } else if curStmt[3].tokenString == "string" {
                        let newVar = Stmt::VarDecl(varName.clone(), VarType::Str("".to_string()));
                        retStmt = newVar;
                    } else if curStmt[3].tokenString == "integer" {
                        let newVar = Stmt::VarDecl(varName.clone(), VarType::Int(0));
                        retStmt = newVar;

                    }  else if curStmt[3].tokenString == "bool" {
                        let newVar = Stmt::VarDecl(varName.clone(), VarType::Bool(false));
                        retStmt = newVar;

                    }  else if curStmt[3].tokenString == "float" {
                        let newVar = Stmt::VarDecl(varName.clone(), VarType::Float(0.0));
                        retStmt = newVar;
                    } else {
                        self.reports.reportError(format!(
                            "In line: {}, '{}' is not a valid variable type", 
                            curStmt[3].lineNum, 
                            curStmt[3].tokenString
                        ));
                        return Err("Error with variable declaration".to_string());
                    }
                }

                // let newVar = Stmt::VarDecl(varName, );
                
                k = k + 1;
                // i = k;

                tokenList.drain(0..k);
                return Ok(Some(retStmt));
                // println!("Variable initialized");
                

                
            }

            tokenTypeEnum::GLOBAL => {
                let mut retStmt:Stmt;
                
                let mut k = 1;
                let mut nextTok = &tokenList[k];
                // println!("Found a global");
                let mut curStmt: Vec<&Token> = vec![];
                // curStmt.push(token);
                while nextTok.tt != tokenTypeEnum::SEMICOLON {
                    curStmt.push(nextTok);
                    k = k + 1;
                    nextTok = &tokenList[k];

                    // println!("iterating");
                }
                curStmt.push(nextTok);
                // println!("Found the semicolon");
                let varName = &curStmt[1].tokenString;
                // println!("\nCurrent variable declaration name: {}", varName);
                
                // for token in &curStmt {
                //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
                // }

                //Error checking
                if curStmt[2].tt != tokenTypeEnum::COLON {
                    // println!("{}", curStmt[2].tokenString);
                    self.reports.reportError(format!(
                        "In line: {}, Array variable declaration incorrect. \n Must be in this format: 'variable [Variable name] : [variable type]'", 
                        curStmt[3].lineNum, 
                    ));
                    return Err("Error with global variable declaration".to_string());
                } else {
                    if (curStmt[4].tt != tokenTypeEnum::SEMICOLON) {
                        if curStmt[4].tt != tokenTypeEnum::L_BRACKET {
                            self.reports.reportError(format!(
                                "In line: {}, Array variable declaration incorrect. \n Must be in this format: 'variable [Variable name] : integer[arraySize]'", 
                                curStmt[3].lineNum, 
                            ));
                            return Err("Error with global variable declaration".to_string());
                        } else {
                            if curStmt[3].tokenString == "integer" {
                                if curStmt[5].tt == tokenTypeEnum::INT {
                                    let arSizeStr = curStmt[5].tokenString.clone();
                                    if let Ok(arSize) = arSizeStr.parse::<usize>() {
                                        let newVar = Stmt::GlobVarDecl(varName.clone(), VarType::IntArray(vec![0; arSize]));
                                        retStmt = newVar;
                                    } else {
                                        self.reports.reportError(format!(
                                            "In line: {}, Invlaid array size", 
                                            curStmt[3].lineNum, 
                                        ));
                                        return Err("Error with variable declaration".to_string());
                                    }
                                } else {
                                    self.reports.reportError(format!(
                                        "In line: {}, Array variable declaration incorrect. \n Must be in this format: 'variable [Variable name] : integer[arraySize]'", 
                                        curStmt[3].lineNum, 
                                    ));
                                    return Err("Error with variable declaration".to_string());
                                }
                            } else {
                                self.reports.reportError(format!(
                                    "In line: {}, '{}' is not a valid variable type", 
                                    curStmt[3].lineNum, 
                                    curStmt[3].tokenString
                                ));
                                return Err("Error with variable declaration".to_string());
                            }
                        }
                    } else if curStmt[3].tokenString == "string" {
                        let newVar = Stmt::GlobVarDecl(varName.clone(), VarType::Str("".to_string()));
                        retStmt = newVar;
                    } else if curStmt[3].tokenString == "integer" {
                        let newVar = Stmt::GlobVarDecl(varName.clone(), VarType::Int(0));
                        retStmt = newVar;

                    }  else if curStmt[3].tokenString == "bool" {
                        let newVar = Stmt::GlobVarDecl(varName.clone(), VarType::Bool(false));
                        retStmt = newVar;

                    }  else if curStmt[3].tokenString == "float" {
                        let newVar = Stmt::GlobVarDecl(varName.clone(), VarType::Float(0.0));
                        retStmt = newVar;
                    } else {
                        self.reports.reportError(format!(
                            "In line: {}, '{}' is not a valid variable type", 
                            curStmt[3].lineNum, 
                            curStmt[3].tokenString
                        ));
                        return Err("Error with variable declaration".to_string());
                    }
                }

                // let newVar = Stmt::VarDecl(varName, );
                
                k = k + 1;
                // i = k;

                tokenList.drain(0..k);
                return Ok(Some(retStmt));
                // println!("Variable initialized");
                

                
            }
            
            tokenTypeEnum::IDENTIFIER => {
                //Initializes the variable that is being referenced first
                let mut varRef:Expr;
                //Initializes the return statement (I DONT THINK THIS IS NEEDED)
                let mut retStmt:Stmt;

                //Initializes values for finding the end of the expression
                let mut k = 0;
                let mut nextTok = &tokenList[k];
                let mut curStmt: Vec<&Token> = vec![];
                
                //Finds the end of the expression
                while k < tokenList.len() {
                    let nextTok = &tokenList[k];
                    curStmt.push(nextTok);
                
                    if (nextTok.tt == tokenTypeEnum::SEMICOLON) {
                        break; // Stop loop when semicolon or parentheses is found
                    }
                
                    k += 1;
                }


                //Checks if the first value is an array reference or not
                if(curStmt[1].tt == tokenTypeEnum::L_BRACKET) {
                    // println!("Array reference found");
                    let varName = curStmt[0].tokenString.clone();
                    // let varRef = Expr::VarRef((varName));

                    //Finds the end of the brackets
                    let mut brackInd = 0;
                    let mut nextIndToken = &curStmt[brackInd];
                    let mut indexList: Vec<Token> = vec![];
                    
                    //Finds the end of the index statement
                    while brackInd < curStmt.len() {
                        let nextTok = curStmt[brackInd];
                        indexList.push(nextTok.clone());
                    
                        if (nextTok.tt == tokenTypeEnum::R_BRACKET) {
                            break; // Stop loop when semicolon or parentheses is found
                        }
                    
                        brackInd += 1;
                    }

                    //If the end of the index was not found, error
                    if brackInd == curStmt.len() {
                        println!("THIS IS AN ERROR in finding end of array index, add error and reporting");
                        tokenList.drain(..k+1);
                        return(Ok(None));
                    }

                    //Removes the index assignment so there is just the ] left for proper parsing of index expression
                    indexList.drain(0..2);
                    // indexList.drain(indexList)


                    // println!("Here is the index list:");
                    // printTokList(&indexList);


                    let parsedExpr = self.parseExpr(&mut indexList);
                    let mut indexExpr: Expr;

                    //Extracts the resulting expression
                    match parsedExpr {
                        Ok(expr) => {
                            indexExpr = expr;
                        }
                        Err(err) => {
                            // println!("Error parsing expression");
                            let errMsg = format!("Error on line {}: {}", tokenList[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with expression".to_string());
                        }
                    }

                    // println!("New value expressions: {}", newValueExpr);

                    let indexBox = Box::new(indexExpr);
                    varRef = Expr::ArrayRef((varName), (indexBox));

                    //Removes the array reference so there is just the ] left
                    curStmt.drain(0..brackInd);
                } else if (curStmt[0].tg == tokenGroup::VARIABLE){
                    //If not an array
                    varRef = Expr::VarRef(curStmt[0].tokenString.clone());

                } else {
                    println!("Identifier but not a variable reference");
                    varRef = Expr::StringLiteral(("Something has gone fucky with variables").to_string());
                }


                // println!("We are doing something with this variable: {}", varRef);
                // println!("First after variable: {}", curStmt[1].tokenString);

                // //Looks ahead to see what comes next and parses accordingly
                match curStmt[1].tg {
                    tokenGroup::OPERATOR => {
                        // println!("OPERATOR");
                        match curStmt[1].tt {
                            tokenTypeEnum::SET_EQUALS =>{
                                let varName = curStmt[0].tokenString.clone();

                                let mut newValueList: Vec<Token> = curStmt.iter().cloned().map(|t| t.clone()).collect();



                                // let mut newValueList = curStmt.clone();
                                newValueList.drain(..2);

                                // println!("Next token in set equal expression: {}", newValueList[0].tokenString);

                                let parsedExpr = self.parseExpr(&mut newValueList);
                                let mut newValueExpr: Expr;

                                //Extracts the resulting expression
                                match parsedExpr {
                                    Ok(expr) => {
                                        newValueExpr = expr;
                                    }
                                    Err(err) => {
                                        // println!("Error parsing expression");
                                        let errMsg = format!("Error on line {}: {}", tokenList[0].lineNum, err);
                                        self.reports.reportError(errMsg);
                                        return Err("Error with expression".to_string());
                                    }
                                }
                                
                                // println!("New value expressions: {}", newValueExpr);

                                let varAssignment = Stmt::Assign((varRef), (newValueExpr));
                                tokenList.drain(..k+1);
                                return Ok(Some(varAssignment));

                            }
                            tokenTypeEnum::L_PAREN => {
                                println!("Procedure call");

                                tokenList.drain(..k+1);
                                return(Ok(None));

                            }
                            _ => {
                                println!("Other");
                                println!("Unknown operator: {}", curStmt[1].tokenString);
                            }
                        }
                        

                    }
                    tokenGroup::NUMBER => {
                        println!("NUMBER");
                        
                        return Ok(None);
                    }
                    tokenGroup::SYMBOL => {
                        println!("SYMBOL");
                        match curStmt[1].tt {
                            tokenTypeEnum::L_BRACKET =>{
                                



                            }
                            _ => {
                                println!("Other symbol");
                            }
                        }
                        

                    }
                    _ => {
                        println!("UNACCOUNTED IN MATCH CASE");
                    }
                }
            
                tokenList.drain(..k+1);
                return Ok(None);
                
            }
            
            
            tokenTypeEnum::IF => {
                //Finds the end of the IF statement
                let mut k = 0;
                let mut nextTok = &tokenList[k];
                // println!("\n\nFound an if");
                let mut curStmt: Vec<Token> = vec![];
                let mut ifInd = 0;
                let ifLen = tokenList.len();
            
                // Finds the end of the if
                // curStmt.push(token.clone());
                while nextTok.tt != tokenTypeEnum::END_IF {
                    if(ifInd > ifLen) {
                        // println!("ERROR IN IF CONDITION");
                        let errMsg = format!("For If on line {}, no end if found", token.lineNum);
                        self.reports.reportError(errMsg);
                        return Err("No end if".to_string());
                    }
                    curStmt.push(nextTok.clone());
                    k = k + 1;
                    ifInd = ifInd + 1;
                    nextTok = &tokenList[k];
                }
                curStmt.push(nextTok.clone());

                let mut condInt;

                let mut ifCondition: Expr;
                // // Extract the condition if it exists
                if curStmt[1].tt == tokenTypeEnum::L_PAREN {
                    let mut j = 1;
                    let mut nextTok = &curStmt[j];
                    let mut condStmt: Vec<Token> = vec![];
                
                    // Finds the end of the condition by findind the then
                    while nextTok.tt != tokenTypeEnum::THEN {
                        condStmt.push(nextTok.clone());
                        j = j + 1;
                        nextTok = &curStmt[j];
                    }
                    condInt = j;

                    condStmt.drain(0..1);

                    

                    // let scanned = self.parse(&mut condStmt);                            
                    // let mut headerStmt:Stmt;

                    let mut parsedExpr: Expr;

                    // printTokList(&condStmt);
                    let scanned = self.parse(&mut condStmt);                            
                    let mut headerStmt:Expr;

                    // let mut headerReporting = Reporting::new();
                    match scanned {
                        Ok((Some(stmt))) => {
                            let parsed = stmt.extractExpr();
                            match parsed {
                                Ok(expr) => {
                                    parsedExpr = expr
                                },
                                Err(msg) => {
                                    println!("Error parsing expression from if condition");
                                    let errMsg = format!("Error parsing if condition: {:?}", self.reports);
                                    parsedExpr = Expr::IntLiteral(0);
                                }
                            }
                                        
                            
                        },
                        Ok((None)) => {
                            println!("Parsed if expression but no statement returned.");
                            parsedExpr = Expr::IntLiteral(0);
                        },
                        Err(err) => {
                            println!("Error parsing if condition: {:?}", err);
                            let errMsg = format!("Error parsing if condition: {}", err);
                            self.reports.reportError(errMsg);
                            return Err("Error with if condition".to_string());
                        },
                    }
                    // println!("Expression parsed: {}", parsedExpr);
                    ifCondition = parsedExpr;
                } else {
                    println!("ERROR IN IF CONDITION");
                    let errMsg = format!("Error in if statement on line: {},\nIf statement declarations must follow this format: if([condition]) then", token.lineNum);
                    self.reports.reportError(errMsg);
                    return Err("Error with if condition".to_string());
                }

                // println!("\n\nIf Condition: {}", ifCondition);


                //Checks for an else statement
                let mut elseInd: usize = 0;
                let mut holder = 0;
                curStmt.drain(0..condInt+1);
                // println!("Curstmt before iteration:");
                // printTokList(&curStmt);
                for token in &curStmt {
                    // println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
                    if(token.tt == tokenTypeEnum::ELSE){
                        // println!("Found else");
                        elseInd = holder;
                    }
                    holder = holder + 1;
                }

                if elseInd != 0 {
                    // println!("Else found");
                    //Splits into two lists to parse seperately
                    let mut ifList = curStmt.clone();
                    ifList.drain(elseInd..);
                    

                    //Parses the header
                    let mut newIf: Vec<Token> = ifList.iter().cloned().map(|t| t.clone()).collect();
                    let mut ifBlock = Stmt::Block(Vec::new());
                    let mut ifI = 0;
                    let ifLen = newIf.len();
                    // println!("newIf len {}", ifLen.to_string());
                    while(!newIf.is_empty()){
                        if(ifI > ifLen){
                            self.reports.reportError("Infinite loop in if statement".to_string());
                            println!("infinite loop in if");
                            return Err("infinite loop in if".to_string());
                        }
                        ifI = ifI + 1;
                        let scanned = self.parse(&mut newIf);                            
                        let mut ifStmt:Stmt;
                        // let mut headerReporting = Reporting::new();
                        match scanned {
                            Ok((Some(stmt))) => {
                                // println!("Header statement parsed successfully");
                                // stmt.display(0);
                                let _ = ifBlock.push_to_block(stmt.clone());
                                // let _ = headerStmt = stmt;
                                // headerReporting = reporting;
                            },
                            Ok((None)) => {
                                println!("Parsed if statement but no statement returned.");
                                // headerReporting = reporting;
                                // headerStmt = Stmt::StringLiteral("No header returned".to_string());
                            },
                            Err(reporting) => {
                                println!("Error parsing if: {:?}", reporting);
                                let errMsg = format!("Error parsing if: {:?}", self.reports);
                                return Err(errMsg);
                                // return Err(("Error parsing header: {:?}", reporting).to_string()); // Propagate the error up the call stack
                            },
                        }
                    }

                    // println!("Finished parsing if: ");
                    // ifBlock.display(0);

                    
                    //Parses the else block
                    let mut elseList = curStmt.split_off(elseInd);                    
                    let mut newElse: Vec<Token> = elseList.iter().cloned().map(|t| t.clone()).collect();
                    newElse.drain(0..1);
                    newElse.drain(newElse.len() - 1..);
                    // println!("First in else: {}", newElse[0].tokenString);
                    let mut elseBlock = Stmt::Block(Vec::new());
                    let mut elseI = 0;
                    let elseLen = newElse.len();
                    while(!newElse.is_empty()){
                        if(elseI > elseLen){
                            self.reports.reportError("Infinite loop in else".to_string());
                            println!("infinite loop in else");
                            return Err("infinite loop in else".to_string());
                        }
                        let scanned = self.parse(&mut newElse);                            
                        let mut elseStmt:Stmt;
                        // let mut headerReporting = Reporting::new();
                        match scanned {
                            Ok((Some(stmt))) => {
                                // println!("Header statement parsed successfully");
                                // stmt.display(0);
                                let _ = elseBlock.push_to_block(stmt.clone());
                                elseI = elseI + 1;
                                // let _ = headerStmt = stmt;
                                // headerReporting = reporting;
                            },
                            Ok((None)) => {
                                println!("Parsed else statement but no statement returned.");
                                elseI = elseI + 1;

                                // headerReporting = reporting;
                                // headerStmt = Stmt::StringLiteral("No header returned".to_string());
                            },
                            Err(reporting) => {
                                println!("Error parsing else: {:?}", reporting);
                                let errMsg = format!("Error parsing else: {:?}", self.reports);
                                // bodyI = bodyI + 1;

                                return Err(errMsg);
                                // return Err(("Error parsing header: {:?}", reporting).to_string()); // Propagate the error up the call stack
                            },
                        }
                        
                    }

                    // println!("Finished parsing else: ");
                    // elseBlock.display(0);

                    //Converts the blocks to boxes
                    let ifBox = Box::new(ifBlock);
                    let elseBox = Box::new(elseBlock);

                    //Finishes up and returns
                    let retStmt = Stmt::If(ifCondition, ifBox, Some(elseBox));
                    tokenList.drain(0..k+2);
                    return Ok(Some(retStmt));
                } else {
                    // println!("Else not found");
                    let mut ifList = curStmt.clone();
                    ifList.drain(ifList.len() - 1..);                    

                    //Parses the header
                    let mut newIf: Vec<Token> = ifList.iter().cloned().map(|t| t.clone()).collect();
                    let mut ifBlock = Stmt::Block(Vec::new());
                    let mut ifI = 0;
                    let ifLen = newIf.len();
                    // println!("newIf len {}", ifLen.to_string());
                    while(!newIf.is_empty()){
                        if(ifI > ifLen){
                            self.reports.reportError("Infinite loop in if statement".to_string());
                            println!("infinite loop in if");
                            return Err("infinite loop in if".to_string());
                        }
                        ifI = ifI + 1;
                        let scanned = self.parse(&mut newIf);                            
                        let mut ifStmt:Stmt;
                        // let mut headerReporting = Reporting::new();
                        match scanned {
                            Ok((Some(stmt))) => {
                                // println!("Header statement parsed successfully");
                                // stmt.display(0);
                                let _ = ifBlock.push_to_block(stmt.clone());
                                // let _ = headerStmt = stmt;
                                // headerReporting = reporting;
                            },
                            Ok((None)) => {
                                println!("Parsed if statement but no statement returned.");
                                // headerReporting = reporting;
                                // headerStmt = Stmt::StringLiteral("No header returned".to_string());
                            },
                            Err(reporting) => {
                                println!("Error parsing if: {:?}", reporting);
                                let errMsg = format!("Error parsing if: {:?}", self.reports);
                                return Err(errMsg);
                                // return Err(("Error parsing header: {:?}", reporting).to_string()); // Propagate the error up the call stack
                            },
                        }
                    }

                    // println!("Finished parsing if: ");
                    // ifBlock.display(0);

                    //Converts the blocks to boxes
                    let ifBox = Box::new(ifBlock);

                    //Finishes up and returns
                    let retStmt = Stmt::If(ifCondition, ifBox, None);
                    tokenList.drain(0..k+2);
                    return Ok(Some(retStmt));
                }
            }
            tokenTypeEnum::FOR => {
                //Finds the end of the FOR statement
                let mut k = 0;
                let mut nextTok = &tokenList[k];
                // println!("\n\nFound a for");
                let mut curStmt: Vec<Token> = vec![];
                let mut forInd = 0;
                let forLen = tokenList.len();
            
                // Finds the end of the for
                // curStmt.push(token.clone());
                while nextTok.tt != tokenTypeEnum::END_FOR {
                    if(forInd > forLen) {
                        // println!("ERROR IN FOR CONDITION");
                        let errMsg = format!("For for on line {}, no end for found", token.lineNum);
                        self.reports.reportError(errMsg);
                        return Err("No end for".to_string());
                    }
                    curStmt.push(nextTok.clone());
                    k = k + 1;
                    forInd = forInd + 1;
                    nextTok = &tokenList[k];
                }

                // println!("ENDFOR found");

                curStmt.push(nextTok.clone());

                let mut condInt;

                let mut forDecl: Stmt;
                let mut forCond: Expr;
                // // Extract the condition if it exists
                if curStmt[1].tt == tokenTypeEnum::L_PAREN {
                    let mut j = 1;
                    let mut nextTok = &curStmt[j];
                    let mut condStmt: Vec<Token> = vec![];
                
                    // Finds the end of the condition by findind the paren
                    while nextTok.tt != tokenTypeEnum::R_PAREN {
                        condStmt.push(nextTok.clone());
                        j = j + 1;
                        nextTok = &curStmt[j];
                    }
                    condInt = j;

                    condStmt.push(nextTok.clone());
                    condStmt.drain(0..1);

                    


                    let mut parsedStmt: Stmt;

                    // println!("For condition statement: ");
                    // printTokList(&condStmt);
                    let scanned = self.parse(&mut condStmt);                            
                    // let mut headerStmt:Stmt;

                    // let mut headerReporting = Reporting::new();
                    match scanned {
                        Ok((Some(stmt))) => {
                            // let parsed = stmt.extractExpr();
                            // println!("Parsed condition: ");
                            // stmt.display(0);
                            parsedStmt = stmt;
                            // match parsed {
                            //     Ok(expr) => {
                            //         parsedExpr = expr
                            //     },
                            //     Err(err) => {
                            //         // println!("Error parsing for condition: {:?}", err);
                            //         let errMsg = format!("Error parsing for condition: {}", err);
                            //         self.reports.reportError(errMsg);
                            //         return Err("Error with for condition".to_string());
                            //     }
                            // }
                                        
                            
                        },
                        Ok((None)) => {
                            println!("Parsed for condition expression but no statement returned.");
                            parsedStmt = Stmt::StringLiteral("None".to_string());
                        },
                        Err(err) => {
                            // println!("Error parsing for condition: {:?}", err);
                            let errMsg = format!("Error parsing for condition: {}", err);
                            self.reports.reportError(errMsg);
                            return Err("Error with for condition".to_string());
                        },
                    }
                    // println!("Expression parsed: {}", parsedExpr);
                    forDecl = parsedStmt;




                    // println!("Remaining condition tokens:");
                    // printTokList(&condStmt);

                    let scanned = self.parseExpr(&mut condStmt);                            
                    match scanned {
                        Ok((stmt)) => {
                            forCond = stmt;
                        },
                        Err(err) => {
                            // println!("Error parsing for condition: {:?}", err);
                            let errMsg = format!("Error parsing for condition: {}", err);
                            self.reports.reportError(errMsg);
                            return Err("Error with for condition".to_string());
                        },
                    }
                    // println!("Expression parsed: {}", parsedExpr);
                    // forCond = parsedStmt;





                } else {
                    println!("ERROR IN FOR CONDITION");
                    let errMsg = format!("Error in FOR statement on line: {},\nFor statement declarations must follow this format: for([condition]) then", token.lineNum);
                    self.reports.reportError(errMsg);
                    return Err("Error with for condition".to_string());
                }

                // println!("\n\nFor Condition:");
                // forDecl.display(0);
                // println!("For expression: {}", forCond);


                let mut forList = curStmt.clone();
                forList.drain(0..condInt+1);
                
                let mut newForLen = forList.len() - 1;
                forList.drain(newForLen..);             


                // println!("Remaining for list:");
                // printTokList(&forList);

                // println!("test1");
                let mut newFor: Vec<Token> = forList.iter().cloned().map(|t| t.clone()).collect();
                let mut forBlock = Stmt::Block(Vec::new());
                let mut ifI = 0;
                let ifLen = newFor.len();
                // println!("newFor len {}", ifLen.to_string());
                while(!newFor.is_empty()){
                    if(ifI > ifLen){
                        self.reports.reportError("Infinite loop in if statement".to_string());
                        println!("infinite loop in if");
                        return Err("infinite loop in if".to_string());
                    }
                    ifI = ifI + 1;
                    let scanned = self.parse(&mut newFor);                            
                    let mut ifStmt:Stmt;
                    // let mut headerReporting = Reporting::new();
                    match scanned {
                        Ok((Some(stmt))) => {
                            // println!("Header statement parsed successfully");
                            // println!("Found for body statement");
                            // stmt.display(0);

                            let _ = forBlock.push_to_block(stmt.clone());
                            // let _ = headerStmt = stmt;
                            // headerReporting = reporting;
                        },
                        Ok((None)) => {
                            println!("Parsed if statement but no statement returned.");
                            // headerReporting = reporting;
                            // headerStmt = Stmt::StringLiteral("No header returned".to_string());
                        },
                        Err(reporting) => {
                            println!("Error parsing if: {:?}", reporting);
                            let errMsg = format!("Error parsing if: {:?}", self.reports);
                            return Err(errMsg);
                            // return Err(("Error parsing header: {:?}", reporting).to_string()); // Propagate the error up the call stack
                        },
                    }
                }

                // println!("Finished parsing for: ");
                // forBlock.display(0);

                //Converts the blocks to boxes
                let forBox = Box::new(forBlock);

                //Finishes up and returns
                let retStmt = Stmt::For(forDecl.into(), forCond, forBox);


                // println!("Final for:");
                // retStmt.display(0);


                tokenList.drain(0..k+2);

                // println!("Next token after for: {}", tokenList[0].tokenString);

                // println!("for returned");

                // println
                return Ok(Some(retStmt));
            }
            tokenTypeEnum::PROCEDURE => {
                //Finds the end of the procedure
                let mut retStmt:Stmt;
                
                let mut k = 1;
                let mut nextTok = &tokenList[k];
                let mut scope = 0;
                // println!("Found a variable token");
                let mut curStmt: Vec<Token> = vec![];
                // println!("First procedure token: {}", nextTok.tokenString);
                curStmt.push(token.clone());
                while (nextTok.tt != tokenTypeEnum::END_PROCEDURE) || (scope != 0) {
                    if(nextTok.tt == tokenTypeEnum::PROCEDURE){
                        // println!("PROCEDURE");
                        scope = scope + 1;
                    } else if ((nextTok.tt == tokenTypeEnum::END_PROCEDURE) && (scope != 1)){
                        scope = scope - 1;
                        // println!("end procedure");

                    }
                    curStmt.push(nextTok.clone());
                    k = k + 1;
                    nextTok = &tokenList[k];

                    // println!("iterating");
                }
                curStmt.push(nextTok.clone());
                
                // println!("procedure tokens:");
                // printTokList(&curStmt);

                let procId = &curStmt[1].tokenString.clone();
                let procType = VarType::new(&curStmt[3].tokenString);    
                let mut procedureType:VarType;
                // Gets the procedure type
                match procType {
                    Ok(varType) => {
                        // println!("Procedure type: {:?}", varType);
                        // println!("Procedure id: {}", procId);
                        procedureType = varType;

                    }
                    Err(err) => {
                        println!("Error determining procedure type: {}", err);
                        println!("WRITE REPORTING");
                        return Err("Error with procedure type".to_string());
                    }
                }

                let mut paramList = Stmt::Block(Vec::new());
                
                let mut j = 4;
                //Finds and extracts the parameters
                if(curStmt[3].tt != tokenTypeEnum::PROCEDURE_CALL){
                    let errMsg = format!("Invalid procedure declaration: {} on line {}", &curStmt[4].tt, &curStmt[4].lineNum);
                    println!("{}, {}", curStmt[4].tokenString, curStmt[4].tt);
                    


                    println!("WRITE REPORTING FOR L PAREN PROCEDURE");
                    return Err(errMsg);
                } else {
                    let mut nextTok = &curStmt[j];
                    let mut paramTokens: Vec<Token> = vec![];
                    let decLine = curStmt[4].lineNum.clone();
                    // Finds the end of the if
                    // curStmt.push(token);
                    while nextTok.tt != tokenTypeEnum::R_PAREN  {
                        if(nextTok.lineNum != decLine){
                            println!("No right parent, make error");
                        } else {
                            paramTokens.push(nextTok.clone());
                            j = j + 1;
                            nextTok = &curStmt[j];
                        }
                    }

                    // printTokList(&paramTokens);

                    


                    // let mut params: Stmt::Block;
                    let mut curParam: Vec<Token> = vec![];
                    for curToken in &paramTokens {
                        if(curToken.tt == tokenTypeEnum::COMMA) {
                            //Parse the parameter
                            let tokenString: String = ";".to_string();
                            let semicolon = Token::new(crate::tokenTypeEnum::SEMICOLON,tokenString, decLine.to_string(), tokenGroup::SYMBOL);
                            curParam.push(semicolon.clone());
                            let mut newCurParam: Vec<Token> = curParam.iter().cloned().map(|t| t.clone()).collect();
                            // println!("test");
                            // println!("Found all parameters:");
                            // for token in &newCurParam {
                            //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
                            // }
                            let scanParam = self.parse(&mut newCurParam);
                            let mut paramStmt: Stmt;
                            // println!("PENIS");
                            match scanParam {
                                Ok((Some(stmt))) => {
                                    // Add your logic to handle the parsed condition statement here
                                    // For example:
                                    // println!("Good if: {:?}", stmt);
                                    paramStmt = stmt; // Assuming Stmt is the type of your condition
                                    // println!("parm extracted");
                                    let _ = paramList.push_to_block(paramStmt);
                                    // Add condition to your newBlock or handle it as needed
                                },
                                Ok((None)) => {
                                    println!("Parsed parameter but no statement returned.");
                                    self.reports.reportError(format!(
                                        "In line: {}, Error with parameter", curStmt[0].lineNum
                                    ));
                                    return Ok(None);
                                },
                                Err(reporting) => {
                                    println!("Error parsing condition: {:?}", reporting);
                                    println!("Parsed condition but no statement returned.");
                                    // paramBlock = None; // Assuming Stmt is the type of your condition
                                    self.reports.reportError(format!(
                                        "In line: {}, Error with condition", curStmt[0].lineNum
                                    ));
                                    return(Err("Error with parameter".to_string()));
                                },
                            }


                            // println!("Params extracted:");
                            // paramList.display(0);
                            curParam = vec![];
                        } else {
                            let _ = &curParam.push(curToken.clone());
                        }
                    }
                    if((paramTokens.len().clone() as i32) != 0){
                        //Parse the parameter
                        let tokenString: String = ";".to_string();
                        let semicolon = Token::new(crate::tokenTypeEnum::SEMICOLON,tokenString, decLine.to_string(), tokenGroup::SYMBOL);
                        curParam.push(semicolon.clone());
                        let mut newCurParam: Vec<Token> = curParam.iter().cloned().map(|t| t.clone()).collect();
                        // println!("test");
                        // println!("Found all parameters:");
                        // for token in &newCurParam {
                        //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
                        // }
                        let scanParam = self.parse(&mut newCurParam);
                        let mut paramStmt: Stmt;
                        // println!("PENIS");
                        match scanParam {
                            Ok((Some(stmt))) => {
                                // Add your logic to handle the parsed condition statement here
                                // For example:
                                // println!("Good if: {:?}", stmt);
                                paramStmt = stmt; // Assuming Stmt is the type of your condition
                                // println!("parm extracted");
                                let _ = paramList.push_to_block(paramStmt);
                                // Add condition to your newBlock or handle it as needed
                            },
                            Ok((None)) => {
                                println!("Parsed parameter but no statement returned.");
                                self.reports.reportError(format!(
                                    "In line: {}, Error with parameter", curStmt[0].lineNum
                                ));
                                return Ok(None);
                            },
                            Err(reporting) => {
                                println!("Error parsing condition: {:?}", reporting);
                                println!("Parsed condition but no statement returned.");
                                // paramBlock = None; // Assuming Stmt is the type of your condition
                                self.reports.reportError(format!(
                                    "In line: {}, Error with condition", curStmt[0].lineNum
                                ));
                                return(Err("Error with parameter".to_string()));
                            },
                        }


                        // println!("Params extracted:");
                        // paramList.display(0);
                        // curParam = vec![];
                    }
                }
                // println!("All Params:");
                // paramList.display(0);

                curStmt.drain(0..j+1);

                //Finds where the header ends and the body begins
                let mut beginInt = 0;
                let mut beginScope = 0;
                let iterTokList = curStmt.clone();
                for token in iterTokList{
                    // println!("Current token: {}", token.tokenString);
                    if (token.tt == tokenTypeEnum::BEGIN) && (beginScope == 0){
                        // println!("Found the end");
                        break;
                    } else if (token.tt == tokenTypeEnum::PROCEDURE) {
                        beginScope = beginScope + 1;
                        beginInt = beginInt + 1;
                    } else if (token.tt == tokenTypeEnum::END_PROCEDURE){
                        beginInt = beginInt + 1;
                        beginScope = beginScope - 1;
                    } else {
                        beginInt = beginInt + 1;
                    }
                }

                        

                println!("begin index: {}", beginInt);

                //Splits into two lists to parse seperately
                let mut bodyList = curStmt.split_off(beginInt);

                //Parses the header
                let mut newHeader: Vec<Token> = curStmt.iter().cloned().map(|t| t.clone()).collect();
                let mut headerBlock = Stmt::Block(Vec::new());
                let mut headerI = 0;
                // println!("First in procedure header: {}", newHeader[0].tokenString);

                let headerLen = newHeader.len();
                while(!newHeader.is_empty()){
                    if(headerI > headerLen){
                        self.reports.reportError("Infinite loop in heprocedure header".to_string());
                        println!("infinite loop in procedure header");
                        return Err("infinite loop in procedure header".to_string());
                    }
                    let scanned = self.parse(&mut newHeader);                            
                    let mut headerStmt:Stmt;
                    // let mut headerReporting = Reporting::new();
                    match scanned {
                        Ok((Some(stmt))) => {
                            // println!("Header statement parsed successfully");
                            // stmt.display(0);
                            let _ = headerBlock.push_to_block(stmt.clone());
                            // let _ = headerStmt = stmt;
                            // headerReporting = reporting;
                        },
                        Ok((None)) => {
                            println!("Parsed procedure header statement but no statement returned.");
                            // headerReporting = reporting;
                            // headerStmt = Stmt::StringLiteral("No header returned".to_string());
                        },
                        Err(reporting) => {
                            println!("Error parsing procedure header: {:?}", reporting);
                            let errMsg = format!("Error parsing header: {:?}", self.reports);
                            return Err(errMsg);
                            // return Err(("Error parsing header: {:?}", reporting).to_string()); // Propagate the error up the call stack
                        },
                    }
                }     
                
                // println!("Finished parsing procedure header: ");
                // headerBlock.display(0);

                let mut newBody: Vec<Token> = bodyList.iter().cloned().map(|t| t.clone()).collect();
                newBody.drain(0..1);
                // println!("First in body: {}", newBody[0].tokenString);
                let mut bodyBlock = Stmt::Block(Vec::new());
                let mut bodyI = 0;
                let bodyLen = newBody.len();
                while(!newBody.is_empty()){
                    if(bodyI > bodyLen){
                        self.reports.reportError("Infinite loop in body".to_string());
                        println!("infinite loop in body");
                        return Err("infinite loop in body".to_string());
                    }
                    let scanned = self.parse(&mut newBody);                            
                    let mut headerStmt:Stmt;
                    // let mut headerReporting = Reporting::new();
                    match scanned {
                        Ok((Some(stmt))) => {
                            // println!("Header statement parsed successfully");
                            // stmt.display(0);
                            let _ = bodyBlock.push_to_block(stmt.clone());
                            bodyI = bodyI + 1;
                            // let _ = headerStmt = stmt;
                            // headerReporting = reporting;
                        },
                        Ok((None)) => {
                            println!("Parsed procedure body statement but no statement returned.");
                            bodyI = bodyI + 1;

                            // headerReporting = reporting;
                            // headerStmt = Stmt::StringLiteral("No header returned".to_string());
                        },
                        Err(reporting) => {
                            println!("Error parsing proedure body: {:?}", reporting);
                            let errMsg = format!("Error parsing body: {:?}", self.reports);
                            // bodyI = bodyI + 1;

                            return Err(errMsg);
                            // return Err(("Error parsing header: {:?}", reporting).to_string()); // Propagate the error up the call stack
                        },
                    }
                    
                }

                // println!("Finished parsing procedure body: ");
                // bodyBlock.display(0);

                // let procedureStmt:Stmt = Stmt::Procedure

                // Turns the statements into boxes
                let boxHeader: Box<Stmt> = Box::new(headerBlock);
                let boxBody: Box<Stmt> = Box::new(bodyBlock);
                let boxParams: Box<Stmt> = Box::new(paramList);


                let procedureAst = Stmt::Procedure(procedureType, procId.clone(), boxParams, boxHeader, boxBody);
                // procedureAst.display(0);
                
                tokenList.drain(0..);
                return Ok(Some(procedureAst));
            }

            tokenTypeEnum::RETURN => {
                if tokenList[1].tt != tokenTypeEnum::SEMICOLON {
                    let retValue = Expr::VarRef(tokenList[1].tokenString.clone());
                    let retStmt = Stmt::Return(retValue);
                    tokenList.drain(0..3);
                    // println!("first one return");


                    return(Ok(Some(retStmt)));
                } else {
                    let retValue = Expr::VarRef("".to_string());
                    let retStmt = Stmt::Return(retValue);
                    // let _ = newBlock.push_to_block(retStmt);
                    tokenList.drain(0..3);
                    // println!("else return");

                    return(Ok(Some(retStmt)));
                }
            }

            
            tokenTypeEnum::END_PROGRAM => {
                // println!("End program");
                let len = tokenList.len();
                tokenList.drain(0..len);
                return Ok((None));
            }
            tokenTypeEnum::END_PROCEDURE => {
                // println!("End program");
                let len = tokenList.len();
                tokenList.drain(0..len);
                return Ok((None));
            }
            tokenTypeEnum::INT => {
                let mut retStmt:Stmt;
                
                let mut k = 0;
                let mut nextTok = &tokenList[k];
                let mut curStmt: Vec<&Token> = vec![];
                // curStmt.push(token);
                while k < tokenList.len() {
                    let nextTok = &tokenList[k];
                    curStmt.push(nextTok);
                
                    if (nextTok.tt == tokenTypeEnum::SEMICOLON) || (nextTok.tt == tokenTypeEnum::R_PAREN) {
                        break; // Stop loop when semicolon or parentheses is found
                    }
                
                    k += 1;
                }
                
                // println!("Found an expression of type: {}", curStmt[1].tokenString);
                // println!("Expressions length: {}", curStmt.len());
                if(curStmt.len() == 4) {
                    // println!("Simple expression");
                    let operand1 = Expr::new(curStmt[0].tt.clone(), Some(curStmt[0].tokenString.clone()));
                    let mut op1Expr: Expr;
                    match operand1 {
                        Ok(expr) => {
                            op1Expr = expr;
                        }
                        Err(err) => {
                            println!("Error parsing operand 1");
                            let errMsg = format!("Error with operand 1 on line {}: {}", curStmt[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with operand 1".to_string());
                        }
                    }
                    
                    let operand2 = Expr::new(curStmt[2].tt.clone(), Some(curStmt[2].tokenString.clone()));
                    let mut op2Expr: Expr;
                    match operand2 {
                        Ok(expr) => {
                            op2Expr = expr;
                        }
                        Err(err) => {
                            println!("Error parsing operand 2");
                            let errMsg = format!("Error with operand 2 on line {}: {}", curStmt[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with operand 2".to_string());
                        }
                    }
                
                    let operator = BinOp::new(curStmt[1].tt.clone());
                    let mut opBin:BinOp; 
                    match operator {
                        Ok(expr) => {
                            opBin = expr;
                        }
                        Err(err) => {
                            println!("Error creating expression");
                            let errMsg = format!("Error with operator on line {}: {}", curStmt[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with operator".to_string());
                        }
                    }
                    
                    let finalExpr = Expr::BinOp(Box::new(op1Expr), opBin, Box::new(op2Expr));

                    let retStmt = Stmt::Expr(finalExpr);
                    tokenList.drain(0..k+1);
                    return Ok(Some(retStmt));

                } else if (curStmt.len() > 4) {
                    // println!("Complex expressions");
                    // println!("First complex expression token: {}", curStmt[0].tokenString);

                    //Parses the first operand
                    let operand1 = Expr::new(curStmt[0].tt.clone(), Some(curStmt[0].tokenString.clone()));
                    let mut op1Expr: Expr;
                    match operand1 {
                        Ok(expr) => {
                            op1Expr = expr;
                        }
                        Err(err) => {
                            println!("Error parsing operand 1");
                            let errMsg = format!("Error with operand 1 on line {}: {}", curStmt[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with operand 1".to_string());
                        }
                    }

                    // println!("Operand 1: {}", op1Expr);
                    
                    let mut subList = tokenList.clone();
                    subList.drain(0..2);
                    // println!("First new token expression token: {}", subList[0].tokenString);

                    let mut parsedExpr: Expr;
                    let scanned = self.parse(&mut subList);                            
                        let mut headerStmt:Expr;
                        // let mut headerReporting = Reporting::new();
                        match scanned {
                            Ok((Some(stmt))) => {
                                let parsed = stmt.extractExpr();
                                match parsed {
                                    Ok(expr) => {
                                        parsedExpr = expr
                                    },
                                    Err(msg) => {
                                        println!("Error parsing expression from statment");
                                        let errMsg = format!("Error parsing body: {:?}", self.reports);
                                        parsedExpr = Expr::IntLiteral(0);
                                    }
                                }
                                            
                                
                            },
                            Ok((None)) => {
                                println!("Parsed complex expression but no statement returned.");
                                parsedExpr = Expr::IntLiteral(0);
                            },
                            Err(reporting) => {
                                println!("Error parsing expression: {:?}", reporting);
                                let errMsg = format!("Error parsing body: {:?}", self.reports);

                                return Err(errMsg);
                            },
                        }
                    // println!("Expression parsed: {}", parsedExpr);
                    let op2Expr = parsedExpr;
                    // println!("Operand 2: {}", op2Expr);


                    let operator = BinOp::new(curStmt[1].tt.clone());
                    let mut opBin:BinOp; 
                    match operator {
                        Ok(expr) => {
                            opBin = expr;
                        }
                        Err(err) => {
                            println!("Error creating expression");
                            let errMsg = format!("Error with operator on line {}: {}", curStmt[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with operator".to_string());
                        }
                    }

                    // println!("Operator: {}", opBin);

                    
                    let finalExpr = Expr::BinOp(Box::new(op1Expr), opBin, Box::new(op2Expr));

                    // println!("Final complex expression: {}", finalExpr);

                    let retStmt = Stmt::Expr(finalExpr);
                    tokenList.drain(0..k+1);
                    return Ok(Some(retStmt));
                } else {
                    println!("Fucked up expressions");
                        // println!("{}", curStmt[1].tt);
                        self.reports.reportError(format!(
                            "In line: {}, expression is too short'", curStmt[1].lineNum));
                        return Err("Error with expression".to_string());
                }
            }
            tokenTypeEnum::FLOAT => {
                let mut retStmt:Stmt;
                
                let mut k = 0;
                let mut nextTok = &tokenList[k];
                let mut curStmt: Vec<&Token> = vec![];
                // curStmt.push(token);
                while k < tokenList.len() {
                    let nextTok = &tokenList[k];
                    curStmt.push(nextTok);
                
                    if (nextTok.tt == tokenTypeEnum::SEMICOLON) || (nextTok.tt == tokenTypeEnum::R_PAREN) {
                        break; // Stop loop when semicolon or parentheses is found
                    }
                
                    k += 1;
                }
                
                // println!("Found an expression of type: {}", curStmt[1].tokenString);
                // println!("Expressions length: {}", curStmt.len());
                if(curStmt.len() == 4) {
                    // println!("Simple expression");
                    let operand1 = Expr::new(curStmt[0].tt.clone(), Some(curStmt[0].tokenString.clone()));
                    let mut op1Expr: Expr;
                    match operand1 {
                        Ok(expr) => {
                            op1Expr = expr;
                        }
                        Err(err) => {
                            println!("Error parsing operand 1");
                            let errMsg = format!("Error with operand 1 on line {}: {}", curStmt[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with operand 1".to_string());
                        }
                    }
                    
                    let operand2 = Expr::new(curStmt[2].tt.clone(), Some(curStmt[2].tokenString.clone()));
                    let mut op2Expr: Expr;
                    match operand2 {
                        Ok(expr) => {
                            op2Expr = expr;
                        }
                        Err(err) => {
                            println!("Error parsing operand 2");
                            let errMsg = format!("Error with operand 2 on line {}: {}", curStmt[0].lineNum, err);
                            
                            // let errMsg = format!("Error with operand 2 on line {}: {}", curStmt[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with operand 2".to_string());
                        }
                    }
                
                    let operator = BinOp::new(curStmt[1].tt.clone());
                    let mut opBin:BinOp; 
                    match operator {
                        Ok(expr) => {
                            opBin = expr;
                        }
                        Err(err) => {
                            println!("Error creating expression");
                            let errMsg = format!("Error with operator on line {}: {}", curStmt[0].lineNum, err);                            
                            self.reports.reportError(errMsg);
                            return Err("Error with operator".to_string());
                        }
                    }
                    
                    let finalExpr = Expr::BinOp(Box::new(op1Expr), opBin, Box::new(op2Expr));

                    let retStmt = Stmt::Expr(finalExpr);
                    tokenList.drain(0..k+1);
                    return Ok(Some(retStmt));

                } else if (curStmt.len() > 4) {
                    // println!("Complex expressions");
                    // println!("First complex expression token: {}", curStmt[0].tokenString);

                    //Parses the first operand
                    let operand1 = Expr::new(curStmt[0].tt.clone(), Some(curStmt[0].tokenString.clone()));
                    let mut op1Expr: Expr;
                    match operand1 {
                        Ok(expr) => {
                            op1Expr = expr;
                        }
                        Err(err) => {
                            println!("Error parsing operand 1");
                            let errMsg = format!("Error with operand1 on line {}: {}", curStmt[0].lineNum, err);                            
                            
                            // let errMsg = format!("Error with operand 1 on line {}: {}", curStmt[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with operand 1".to_string());
                        }
                    }

                    // println!("Operand 1: {}", op1Expr);
                    
                    let mut subList = tokenList.clone();
                    subList.drain(0..2);
                    // println!("First new token expression token: {}", subList[0].tokenString);

                    let mut parsedExpr: Expr;
                    let scanned = self.parse(&mut subList);                            
                    let mut headerStmt:Expr;
                    // let mut headerReporting = Reporting::new();
                    match scanned {
                        Ok((Some(stmt))) => {
                            let parsed = stmt.extractExpr();
                            match parsed {
                                Ok(expr) => {
                                    parsedExpr = expr
                                },
                                Err(msg) => {
                                    println!("Error parsing expression from statment");
                                    let errMsg = format!("Error parsing body {:?}", self.reports);                            
                                    parsedExpr = Expr::IntLiteral(0);
                                }
                            }
                                        
                            
                        },
                        Ok((None)) => {
                            println!("Parsed complex expression but no statement returned.");
                            parsedExpr = Expr::IntLiteral(0);
                        },
                        Err(reporting) => {
                            println!("Error parsing expression: {:?}", reporting);
                            let errMsg = format!("Error parsing body: {:?}", self.reports);

                            return Err(errMsg);
                        },
                    }
                    // println!("Expression parsed: {}", parsedExpr);
                    let op2Expr = parsedExpr;
                    // println!("Operand 2: {}", op2Expr);


                    let operator = BinOp::new(curStmt[1].tt.clone());
                    let mut opBin:BinOp; 
                    match operator {
                        Ok(expr) => {
                            opBin = expr;
                        }
                        Err(err) => {
                            println!("Error creating expression");
                            let errMsg = format!("Error with operator on line {}: {}", curStmt[0].lineNum, err);
                            self.reports.reportError(errMsg);
                            return Err("Error with operator".to_string());
                        }
                    }

                    // println!("Operator: {}", opBin);

                    
                    let finalExpr = Expr::BinOp(Box::new(op1Expr), opBin, Box::new(op2Expr));

                    // println!("Final complex expression: {}", finalExpr);

                    let retStmt = Stmt::Expr(finalExpr);
                    tokenList.drain(0..k+1);
                    return Ok(Some(retStmt));
                } else {
                    println!("Fucked up expressions");
                    // println!("{}", curStmt[1].tt);
                    self.reports.reportError(format!(
                        "In line: {}, expression is too short'", curStmt[3].lineNum));
                    return Err("Error with expression".to_string());
                }
            }
            tokenTypeEnum::PROCEDURE_CALL => {
                println!("PROCEDURE CALL");
                tokenList.drain(0..1);
                return Ok((None));
            }
            
            _ => {
                // i = i + 1;
                // return(Ok());
                println!("Unaccounted token: {}", token.tokenString);
                tokenList.drain(0..1);
                return Ok((None));

            }
        }
    }

    //Prints all of the tokens
    fn printTokenList(&mut self){
        for token in &self.tokenList {
            println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
        }
    }
    
}




#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Greater,
    Less,
    Greater_Equal,
    Less_Equal,
    Check_Equal,
}

impl BinOp {
    pub fn new(op_str: tokenTypeEnum) -> Result<Self, String> {
        match op_str {
            tokenTypeEnum::PLUS => Ok(BinOp::Add),
            tokenTypeEnum::MINUS => Ok(BinOp::Sub),
            tokenTypeEnum::MULTIPLY => Ok(BinOp::Mul),
            tokenTypeEnum::DIVIDE => Ok(BinOp::Div),
            tokenTypeEnum::GREATER => Ok(BinOp::Greater),
            tokenTypeEnum::LESS => Ok(BinOp::Less),
            tokenTypeEnum::GREATER_EQUALS => Ok(BinOp::Greater_Equal),
            tokenTypeEnum::LESS_EQUALS => Ok(BinOp::Less_Equal),
            tokenTypeEnum::CHECK_EQUALS => Ok(BinOp::Check_Equal),
            _ => Err(format!("Unsupported operator: {}", op_str)),
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Greater => write!(f, ">"),
            BinOp::Less => write!(f, "<"),
            BinOp::Greater_Equal => write!(f, ">="),
            BinOp::Less_Equal => write!(f, "<="),
            BinOp::Check_Equal => write!(f, "=="),

        }
    }
}

// Define types of expressions
#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64),                        //An integer literal (int value)
    FloatLiteral(f64),                      //A float literal (float value)
    StringLiteral(String),                  //A string literal (the string)
    VarRef(String),                         //A reference to a variable (variable name)
    ArrayRef(String, Box<Expr>),            //A reference to an array index (array name, Box of the index value)
                                            //                               This is a box because it can be an intliteral or BinOp

    BinOp(Box<Expr>, BinOp, Box<Expr>),     //A binary Operation, (Operand 1, an instance of the BinOp enum, Operand 2)
                                            //                      These are boxes because they can contain more BinOps within themselves     
    // ArrayAssign(Box<Expr>, Box<Expr>),
    ProcCall(String, Option<Vec<Expr>>),    //Procedure calls: the name of the procedure, an optional box of a Block of Exprs for the parameters 

    
}

impl Expr {
    pub fn new(expr_type: tokenTypeEnum, param1: Option<String>) -> Result<Self, String> {
        match expr_type {
            tokenTypeEnum::INT => {
                let value_str = param1.ok_or("IntLiteral requires an integer parameter".to_string())?;
                let value = value_str.parse::<i64>().map_err(|e| format!("Failed to parse integer: {}", e))?;
                Ok(Expr::IntLiteral(value))
            },
            tokenTypeEnum::FLOAT => {
                let value_str = param1.ok_or("Float requires a float parameter".to_string())?;
                let value = value_str.parse::<f64>().map_err(|e| format!("Failed to parse integer: {}", e))?;
                Ok(Expr::FloatLiteral(value))
            },
            tokenTypeEnum::STRING => {
                let value = param1.ok_or("StringLiteral requires a string parameter".to_string())?.to_string();
                Ok(Expr::StringLiteral(value))
            },
            // "BinOp" => {
            //     let left = param3.ok_or("BinOp requires a left operand".to_string())?;
            //     let op = match *left {
            //         Expr::BinOp(_, ref op, _) => op,
            //         _ => return Err("BinOp requires a BinOp enum as the left operand".to_string()),
            //     };
            //     let right = param4.ok_or("BinOp requires a right operand".to_string())?;

            //     Ok(Expr::BinOp(left.clone(), op.clone(), right.clone()))
            // },
            tokenTypeEnum::IDENTIFIER => {
                let var_name = param1.ok_or("VarRef requires a variable name".to_string())?.to_string();
                Ok(Expr::VarRef(var_name))
            },
            _ => Err("Invalid expression type".to_string()),
        }
    }
}


impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::IntLiteral(i) => write!(f, "{}", i),
            Expr::StringLiteral(s) => write!(f, "{}", s),
            Expr::FloatLiteral(n) => write!(f, "{}", n),
            Expr::BinOp(left, op, right) => write!(f, "({} {} {})", left, op, right),
            Expr::VarRef(var) => write!(f, "{}", var),
            Expr::ArrayRef(var, index) => write!(f, "({}[{}])", var, index),
            Expr::ProcCall(name, Some(params)) => {
                let params_str = params.iter().map(|expr| format!("{}", expr)).collect::<Vec<_>>().join(", ");
                write!(f, "{}({})", name, params_str)
            },
            Expr::ProcCall(name, None) => write!(f, "{}()", name),
        }
    }
}

// Define supported variable types
#[derive(Debug, Clone)]
pub enum VarType {
    Int(i64),
    Bool(bool),
    Float(f64),
    Str(String),
    IntArray(Vec<i32>),
}
impl VarType {
    pub fn new(typeStr: &str) -> Result<Self, String> {
        match typeStr {
            "integer" => Ok(VarType::Int(0)),
            "bool" => Ok(VarType::Bool(false)),
            "float" => Ok(VarType::Float(0.0)),
            "string" => Ok(VarType::Str("".to_string())),
            "int[]" => Ok(VarType::IntArray(Vec::new())),
            
            _ => Err(format!("Unsupported var type: {}", typeStr)),
        }
    }
}


impl fmt::Display for VarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarType::Int(i) => write!(f, "Int({})", i),
            VarType::Bool(b) => write!(f, "Bool({})", b),
            VarType::Float(fl) => write!(f, "Float({})", fl),
            VarType::Str(s) => write!(f, "Str({})", s),
            VarType::IntArray(arr) => write!(f, "IntArray({:?})", arr),
        }
    }
}


// These are the types of statements that are available
#[derive(Debug, Clone)]
pub enum Stmt {
    StringLiteral(String),
    Expr(Expr),                     // Expression statement
    Assign(Expr, Expr),           // Assignment statement: variable refernce, expression to assign to
    VarDecl(String, VarType),       // Variable declaration statement
    GlobVarDecl(String, VarType),       // Variable declaration statement
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),  // If statement: condition, body, optional else body
    For(Rc<Stmt>, Expr, Box<Stmt>),          // For statement: assignment, condition, Box of commands for statement
    Block(Vec<Stmt>),               // Block statement: list of statements
    // Procedure(String, VarType, Box<Stmt>),  //Procedure statement: Name of procedure, return type, statements 
    Error(Reporting),
    Return(Expr),
    Program(String, Box<Stmt>, Box<Stmt>), //The program AST: Name, the statements
    Procedure(VarType, String, Box<Stmt>, Box<Stmt>, Box<Stmt>), //Procedure AST: type, Name, parameter, Header, body
}


impl Stmt {
    // Function to push a statement into a Block variant
    pub fn push_to_block(&mut self, stmt: Stmt) -> Result<(), String> {
        match self {
            Stmt::Block(stmts) => {
                stmts.push(stmt);
                Ok(())
            },
            _ => Err("Cannot push to a non-Block statement".to_string())
        }
    }

    pub fn display(&self, indent: usize) {
        let indentation = " ".repeat(indent);
        match self {
            Stmt::StringLiteral(s) => println!("{}StringLiteral({})", indentation, s),
            Stmt::Expr(expr) => println!("{}Expr({})", indentation, expr),
            Stmt::Assign(var, expr) => println!("{}Assign({}, {})", indentation, var, expr),
            
            // Stmt::ArrayAssign(var, index, expr) => println!("{}Index Assign({}[{}], {})", indentation, var, index, expr),
            
            
            Stmt::VarDecl(var, vartype) => println!("{}VarDecl({}, {})", indentation, var, vartype),
            Stmt::GlobVarDecl(var, vartype) => println!("{}GlobVarDecl({}, {})", indentation, var, vartype),
            Stmt::If(cond, body, else_body) => {
                println!("{}If (", indentation);
                println!("{}  Condition: {}", indentation, cond);
                println!("{}  Body: ", indentation);
                body.display(indent + 2);
                if let Some(else_stmt) = else_body {
                    println!("{}  Else: ", indentation);
                    else_stmt.display(indent + 2);
                }
                println!("{})", indentation);
            }

            Stmt::For(assignment, cond, body) => {
                println!("{}For (", indentation);
                println!("{}  Assignment: ", indentation);
                assignment.display(indent + 3);
                println!("{}  Condition: {}", indentation, cond);
                println!("{}  Body: ", indentation);
                body.display(indent + 3);
                println!("{})", indentation);
            }



            Stmt::Block(stmts) => {
                println!("{}Block([", indentation);
                for stmt in stmts {
                    stmt.display(indent + 2);
                }
                println!("{}])", indentation);
            },
            Stmt::Error(reporting) => println!("{}Error({:?})", indentation, reporting),
            Stmt::Return(expr) => println!("{}Return({})", indentation, expr),
            Stmt::Program(name, header, body) => {
                println!("{}{}:(", indentation,name);
                println!(" {}Header:",indentation);
                header.display(indent + 1);
                println!(" {}Body:",indentation);
                body.display(indent + 1);
                println!("{})", indentation);
            }


            
            

            Stmt::Procedure(procType, name, params, header, body) => {
                println!("{}{} {}:(", indentation,procType,name);
                println!(" {}Params:",indentation);
                params.display(indent + 1);
                
                println!(" {}Header:",indentation);
                header.display(indent + 1);
                println!(" {}Body:",indentation);
                body.display(indent + 1);
                println!("{})", indentation);
            }
            
        }
    }

    pub fn extractExpr(&self) -> Result<Expr, String> {
        match self {
            Stmt::Expr(expr) => Ok(expr.clone()), // Clone the expression to return it
            _ => Err("Provided statement is not an expression.".to_string()),
        }
    }
}

// This is the struct that defines the vector of statements
#[derive(Debug)]
pub struct Program {
    pub name: String,           //The name of the program
    // pub header: Vec<Stmt>,      //The statments that make up the header of the program:
    //                             //  -Variable inits
    //                             //  -Functions
    //                             //  -That stuff
    pub statements: Vec<Stmt>,        //The actual main part of the code
    pub report: Reporting,
    // pub statements: Vec<Stmt>,  // List of statements in the program
}

///////////////////////// /PARSER SECTION /////////////////////////

fn printTokList(tokList: &Vec<Token>){
    for token in tokList {
        println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
    }
}
    




//Structure for reporting errors and warnings
#[derive(Debug, Clone)]
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

//The structure for the SymbolTable. This holds all of the IDENTIFIERS of the program as well as their scope and information
struct symbolTable{
    // For now you can simply use a single hash table of tokens. As we move forward to parsing, the symbol table
    // structure will have to be augmented to permit the recording of entering/exiting program scopes as well as
    // the scope that an IDENTIFIER is declared. In general when you exit a scope the symbol table will remove
    // any symbols defined in that scope from the symbol table. Again, we will solve this problem later; the
    // example methods for scope entry/exit are here to deomonstrate what we will probably want in the future
    symTab: HashMap<String, Token>,
}
impl symbolTable{
    // The symbol table hashLook function should automatically create a new entry and mark it as an
    // IDENTIFIER Token for any IDENTIFIER string that is not already in the symbol table. In some languages
    // case does not matter to the uniqueness of the symbol. In this case, an easy place to solve this is to simply
    // upper case or lower case all strings in the symbol table API functions (and storage)
    fn new() -> symbolTable {
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



        ];

        for (key, value) in tokens {
            symHash.insert(key.to_string(), value);
        }

        println!("Symbol table created and seeded");
        // for (key, token) in &mut symHash {
        //     println!("Key: {}, Token: {:?}", key, token.printToken());
        // }


        symbolTable{
            symTab: symHash,
        }
    }
    //Returns the Token for a given string
    fn hashLook(&mut self, mut lookupString: String, line: String) -> Token{
        // println!("Looking up the identifier of the string");
        if let Some(tokenResp) = self.symTab.get(&lookupString){
            // println!("Token found");
            return tokenResp.clone();
        } else {
            // println!("Token not found, creating");
            let newToken = Token::new(tokenTypeEnum::IDENTIFIER, lookupString, line.to_string(), tokenGroup::VARIABLE);
            self.symTab.insert(newToken.tokenString.clone(), newToken.clone());
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



//The main section of the code
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the path from command line arguments
    let path = env::args().nth(1).expect("Please specify an input file");
    let mut myLexer = Lexer::new(&path);
    println!("Lexer filename: {} \nCharacter count: {}", myLexer.inputFile.fileName, myLexer.inputFile.numChars);

    // Scan through the input
    myLexer.scanThrough();

    // Initialize the parser
    let mut myParser = Parser::new(&mut myLexer);


    // Print the parser's token list
    // println!("\n\nMy parser token list: ");
    // myParser.printTokenList();

    // println!("\n\nParsing now");
    // Call the parse function and handle the result
    match myParser.startParse() {
        Ok((reporting, Some(stmt))) => {
            println!("\n\nParsing completed successfully.");
            println!("Reporting: {:?}", reporting);
            stmt.display(0);
            // Continue with normal flow
        }
        Ok((reporting, None)) => {
            println!("\n\nParsing succeeded, but no statement was returned.");
            println!("Reporting: {:?}", reporting);
            // Continue with normal flow
        }
        Err(reporting) => {
            eprintln!("\n\nParsing failed.");
            eprintln!("Reporting: {:?}", reporting);
            // Handle the error gracefully, log, recover, etc.
        }
    }

    


    Ok(())
}