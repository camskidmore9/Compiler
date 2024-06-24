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


    },
    utf8_chars::BufReadCharsExt,
};

///////////////////////// Setup /////////////////////////

//imports
use std::io::prelude::*;

//The enumeration for saving Token types, this is a list of every type of Token there is
#[derive(Clone)]
#[derive(PartialEq)]
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
    WORD,
    STRING,
    SET_EQUALS,
    CHECK_EQUALS,
    RETURN,
    ERROR,
    PROGRAM,
    IS,
    BEGIN,
    PROCEDURE,
    IF,
    ELSE,
    GLOBAL,
    VARIABLE,
    THEN,
    END,
    SEMICOLON,
    COLON,
    PERIOD,
    END_PROGRAM,
    END_PROCEDURE,
    END_IF,

    
    
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
            
            if c == '\n' || c == '\t' || c == '\r' || c == ' ' {
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
                    let newToken = Token::new(crate::tokenTypeEnum::LESS_EQUALS,tokenString, self.inputFile.lineCnt.to_string());
                    return newToken;
                } else {
                    // println!("This is just a <");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::LESS,tokenString, self.inputFile.lineCnt.to_string());
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

                    let newToken = Token::new(crate::tokenTypeEnum::GREATER,tokenString, self.inputFile.lineCnt.to_string());
                    return newToken;
                } else {
                    // println!("This is just a >");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::GREATER_EQUALS,tokenString, self.inputFile.lineCnt.to_string());
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

                    let newToken = Token::new(crate::tokenTypeEnum::CHECK_EQUALS,tokenString, self.inputFile.lineCnt.to_string());
                    return newToken;
                } else if nextC == ' ' {
                    // println!("This is just a >");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::SET_EQUALS,tokenString, self.inputFile.lineCnt.to_string());
                    return newToken;
                } else {
                    println!("ERROR");

                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::ERROR,tokenString, self.inputFile.lineCnt.to_string());
                    return newToken;
                }
            }

            //If the character is a ;
            Some(';') => {
                // println!("Current line: {}", self.inputFile.lineCnt.to_string());
                tokenString.push(';');
                let newToken = Token::new(crate::tokenTypeEnum::SEMICOLON,tokenString, self.inputFile.lineCnt.to_string());
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

                    let newToken = Token::new(crate::tokenTypeEnum::SET_EQUALS,tokenString, self.inputFile.lineCnt.to_string());
                    return newToken;
                } else {
                    // println!("This is just a >");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::COLON,tokenString, self.inputFile.lineCnt.to_string());
                    return newToken;
                }
            }


            //If the character is a :
            Some('[') => {
                tokenString.push('[');
                let newToken = Token::new(crate::tokenTypeEnum::L_BRACKET,tokenString, self.inputFile.lineCnt.to_string());
                return newToken;
            }

            //If the character is a :
            Some(']') => {
                tokenString.push(']');
                let newToken = Token::new(crate::tokenTypeEnum::R_BRACKET,tokenString, self.inputFile.lineCnt.to_string());
                return newToken;
            }

            //If the character is a :
            Some('(') => {
                tokenString.push('(');
                let newToken = Token::new(crate::tokenTypeEnum::L_PAREN,tokenString, self.inputFile.lineCnt.to_string());
                return newToken;
            }

            //If the character is a :
            Some(')') => {
                tokenString.push(')');
                let newToken = Token::new(crate::tokenTypeEnum::R_PAREN,tokenString, self.inputFile.lineCnt.to_string());
                return newToken;
            }

            //If the character is a :
            Some('.') => {
                tokenString.push('.');
                let newToken = Token::new(crate::tokenTypeEnum::PERIOD,tokenString, self.inputFile.lineCnt.to_string());
                return newToken;
            }

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
                let newToken = Token::new(crate::tokenTypeEnum::UNACCOUNTED,tokenString, self.inputFile.lineCnt.to_string());
                return newToken;
            }
            None => {
                // println!("This character is a None aka EOF");
                let newToken = Token::new(crate::tokenTypeEnum::EOF, "EOF".to_string(), self.inputFile.lineCnt.to_string());
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
                    println!("End found");
                    let nextToken = &self.tokenList[i+1];
                    if nextToken.tt == tokenTypeEnum::PROGRAM {
                        println!("Combining end and program");
                        let newToken = Token::new(crate::tokenTypeEnum::END_PROGRAM,"END_PROGRAM".to_string(), nextToken.lineNum.to_string());
                        newTokList.push(newToken.clone());
                        i = i + 1;
                    } else if nextToken.tt == tokenTypeEnum::PROCEDURE {
                        println!("Combining end and procedure");
                        let newToken = Token::new(crate::tokenTypeEnum::END_PROCEDURE,"END_PROCEDURE".to_string(), nextToken.lineNum.to_string());
                        newTokList.push(newToken.clone());
                        i = i + 1;
                    } else if nextToken.tt == tokenTypeEnum::IF {
                        println!("Combining end and if");
                        let newToken = Token::new(crate::tokenTypeEnum::END_IF,"END_IF".to_string(), nextToken.lineNum.to_string());
                        newTokList.push(newToken.clone());
                        i = i + 1;
                    } else {
                        println!("other end with type: {}", nextToken.tt);
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


        println!("\n\nBeginning scan:");

        //Scans the first token and initializes the newToken variable
        let mut newToken: Token = self.scan();
        self.tokenList.push(newToken.clone());
        // println!("First token: < \"{}\" , {} >", newToken.tokenString, newToken.tt.to_string());




        while newToken.tokenString != "EOF".to_string(){
            newToken = self.scan();
            self.tokenList.push(newToken.clone());
            // println!("< \"{}\" , {} >", newToken.tokenString, newToken.tt.to_string());
        };
        println!("\n\nEOF Reached");

        println!("Starting second pass");
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

//Token class, this is where tokens are defined and setup
#[derive(Clone)]
struct Token{
    tt: tokenTypeEnum,
    tokenString: String,
    pub lineNum: String,
    //To be completed later when I understand
    //tm: tokenMark,
}
impl Token{
    //Init for the Token
    fn new(iden: tokenTypeEnum, tokenString: String, line: String) -> Token{
        Token {
            tt: iden,
            tokenString: tokenString,
            lineNum: line,
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
        println!("Beginning creation of Parser");
        let tokenList = lexer.tokenList.clone();
        // let newFile = inFile::new(fileName);
        println!("Parser created successfully");
        let mut report: Reporting = Reporting::new();


        Parser { 
            tokenList,
            reports: report,
        }
    }  


    fn startParse(&mut self) -> Result<(Reporting, Option<Stmt>), Reporting> {
        let mut tokList = self.tokenList.clone();
        return self.parse(tokList, 0);
    }

    fn parse(&mut self, mut tokenList: Vec<Token>, mut scope: i32) -> Result<(Reporting, Option<Stmt>), Reporting> {
        
        let mut newBlock = Stmt::Block(Vec::new());

        // let mut tokenList = &mut self.tokenList;
        println!("Beginning parsing");
        let numTokens: usize = tokenList.len();
        println!("Total number of tokens: {}", numTokens.to_string());
    
        // println!("Current scope: {}", scope.to_string());
            
        let mut i: usize = 0;
        let tokLen: usize = tokenList.len();
        // Iterate through tokenList using an index
        
        while i < tokLen {
            //Gets next token
            let token = &tokenList[i];
            // println!("current i: {} on token: {}", i.to_string(), token.tokenString);

            
            match token.tt {
                tokenTypeEnum::PROGRAM => {
                    //If program is just starting, check it.
                    if scope == 0{
                        //Checks the first line
                        let firstToken = &tokenList[0];
                        if let tokenTypeEnum::PROGRAM = firstToken.tt {
                            let thirdToken = &tokenList[2];
                            if let tokenTypeEnum::IS = thirdToken.tt {
                                let programName: &String = &tokenList[1].tokenString;
                                println!("Program declaration good");
                            } else {
                                self.reports.reportError("Program declaration incorrect. \n Program must start with: 'program [Program name] is'".to_string());
                                return Err(self.reports.clone());
                            }
                            // println!("TEST");
                            scope = 1;
                            i = 3;
                            // println!("Yes");
                        } else {
                            self.reports.reportError("Program declaration incorrect. \n Program must start with: 'program [Program name] is'".to_string());
                            return Err(self.reports.clone());
                        }
                    } else {
                        // println!("PROGRAM but not the first one");
                        i = i + 1;
                    }
                }
                tokenTypeEnum::VARIABLE => {
                    let mut k = i + 1;
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
                        self.reports.reportError(format!(
                            "In line: {}, Array variable declaration incorrect. \n Must be in this format: 'variable [Variable name] : [variable type]'", 
                            curStmt[3].lineNum, 
                        ));
                        return Err(self.reports.clone());
                    } else {
                        if curStmt[4].tt != tokenTypeEnum::SEMICOLON {
                            if curStmt[4].tt != tokenTypeEnum::L_BRACKET {
                                self.reports.reportError(format!(
                                    "In line: {}, Array variable declaration incorrect. \n Must be in this format: 'variable [Variable name] : integer[arraySize]'", 
                                    curStmt[3].lineNum, 
                                ));
                                return Err(self.reports.clone());
                            } else {
                                if curStmt[3].tokenString == "integer" {
                                    if curStmt[5].tt == tokenTypeEnum::INT {
                                        let arSizeStr = curStmt[5].tokenString.clone();
                                        if let Ok(arSize) = arSizeStr.parse::<usize>() {
                                            let newVar = Stmt::VarDecl(varName.clone(), VarType::IntArray(vec![0; arSize]));
                                            let _ = newBlock.push_to_block(newVar);
                                        } else {
                                            self.reports.reportError(format!(
                                                "In line: {}, Invlaid array size", 
                                                curStmt[3].lineNum, 
                                            ));
                                            return Err(self.reports.clone());
                                        }
                                    } else {
                                        self.reports.reportError(format!(
                                            "In line: {}, Array variable declaration incorrect. \n Must be in this format: 'variable [Variable name] : integer[arraySize]'", 
                                            curStmt[3].lineNum, 
                                        ));
                                        return Err(self.reports.clone());
                                    }
                                } else {
                                    self.reports.reportError(format!(
                                        "In line: {}, '{}' is not a valid variable type", 
                                        curStmt[3].lineNum, 
                                        curStmt[3].tokenString
                                    ));
                                    return Err(self.reports.clone());
                                }
                            }
                        } else if curStmt[3].tokenString == "string" {
                            let newVar = Stmt::VarDecl(varName.clone(), VarType::Str("".to_string()));
                            let _ = newBlock.push_to_block(newVar);
                        } else if curStmt[3].tokenString == "integer" {
                            let newVar = Stmt::VarDecl(varName.clone(), VarType::Int(0));
                            let _ = newBlock.push_to_block(newVar);
                        }  else if curStmt[3].tokenString == "bool" {
                            let newVar = Stmt::VarDecl(varName.clone(), VarType::Bool(false));
                            let _ = newBlock.push_to_block(newVar);
                        }  else if curStmt[3].tokenString == "float" {
                            let newVar = Stmt::VarDecl(varName.clone(), VarType::Float(0.0));
                            let _ = newBlock.push_to_block(newVar);
                        }
                    }

                    // let newVar = Stmt::VarDecl(varName, );
                    
                    k = k + 1;
                    i = k;
                    println!("Variable initialized");
                    

                    
                }
                // tokenTypeEnum::BEGIN => {
                //     let mut k = i + 1;
                //     let mut nextTok = &tokenList[k];
                //     println!("\nFound a program begin");
                //     let mut curStmt: Vec<&Token> = vec![];
                //     curStmt.push(token);
                //     while nextTok.tt != tokenTypeEnum::END_PROGRAM {
                //         curStmt.push(nextTok);
                //         k = k + 1;
                //         nextTok = &tokenList[k];

                //         // println!("iterating");
                //     }
                //     curStmt.push(nextTok);
                //     println!("Found the end program");
                    
                //     curStmt.remove(0);

                //     for token in &curStmt {
                //         println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
                //     }

                //     let progBlock = self.parse(curStmt);
                    


                //     i = i + 1;
                // }
                tokenTypeEnum::IDENTIFIER => {
                    let mut k = i + 1;
                    let mut nextTok = &tokenList[k];
                    println!("\n\nFound an identifier");
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

                    // println!("{}", curStmt[1].tokenString);
                    if(curStmt[1].tt == tokenTypeEnum::SET_EQUALS) {
                        let varName = &curStmt[0].tokenString;
                        // println!("command length: {}", &curStmt.len().to_string());
                        if (curStmt.len() == 4) {
                            println!("\n\nSimple set equals found");
                            if(curStmt[2].tt == tokenTypeEnum::INT){
                                let newExpr = Expr::IntLiteral(curStmt[2].tokenString.parse().unwrap());
                                let newVar = Stmt::Assign(varName.clone(), newExpr);
                                let _ = newBlock.push_to_block(newVar);
                            } else if(curStmt[2].tt == tokenTypeEnum::STRING){
                                let strValue = &curStmt[0].tokenString;
                                let newExpr = Expr::StringLiteral(strValue.clone());
                                let newVar = Stmt::Assign(varName.clone(), newExpr);
                                let _ = newBlock.push_to_block(newVar);
                            }
                        }
                        
                    }

                    // for token in &curStmt {
                    //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
                    // }

                    i = i + 1;
                }
                _ => {
                    // return Ok((self.reports.clone(), Some(Stmt::StringLiteral("Unwritten".to_string()))));
                    // println!("Unwritten");
                    i = i + 1;
                }
            }
        }
        // println!("No elements in this token list");
        println!("\n\nHere is the block: {}", newBlock);
        Ok((self.reports.clone(), Some(Stmt::StringLiteral("ZERO ELEMENTS".to_string()))))
    }

    //Prints all of the tokens
    fn printTokenList(&mut self){
        for token in &self.tokenList {
            println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
        }
    }
    
}


// Define the types of expressions
#[derive(Debug)]
pub enum Expr {
    IntLiteral(i64),                       // Integer literal
    StringLiteral(String),                //A string value
    BinOp(Box<Expr>, BinOp, Box<Expr>),  // Binary operation: left operand, operator, right operand
    VarRef(String),             // Variable reference
}
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::IntLiteral(i) => write!(f, "{}", i),
            Expr::StringLiteral(i) => write!(f, "{}", i),
            Expr::BinOp(left, op, right) => write!(f, "({} {} {})", left, op, right),
            Expr::VarRef(var) => write!(f, "{}", var),
        }
    }
}


// Define supported binary operators
#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}
impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
        }
    }
}

//Defines the supported variable types
#[derive(Debug)]
pub enum VarType {
    Int(i64),
    Bool(bool),
    Float(f64),
    Str(String),
    IntArray(Vec<i32>),
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
#[derive(Debug)]
pub enum Stmt {
    StringLiteral(String),
    Expr(Expr),                     // Expression statement
    Assign(String, Expr),           // Assignment statement: variable name, expression
    VarDecl(String, VarType),       // Variable declaration statement
    // If(Expr, Box<Stmt>, Option<Box<Stmt>>),  // If statement: condition, body, optional else body
    Block(Vec<Stmt>),               // Block statement: list of statements
    Error(Reporting),
}
impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::StringLiteral(s) => write!(f, "StringLiteral({})", s),
            Stmt::Expr(expr) => write!(f, "Expr({})", expr),
            Stmt::Assign(var, expr) => write!(f, "Assign({}, {})", var, expr),
            Stmt::VarDecl(var, vartype) => write!(f, "VarDecl({}, {})", var, vartype),
            // Stmt::If(cond, body, else_body) => {
            //     write!(f, "If({}, {}, {})",
            //         cond,
            //         body.as_ref().map_or("None".to_string(), |stmt| stmt.to_string()),
            //         else_body.as_ref().map_or("None".to_string(), |stmt| stmt.to_string())
            //     )
            // },
            Stmt::Block(stmts) => {
                write!(f, "Block(")?;
                for stmt in stmts {
                    write!(f, "\n{}; ", stmt)?;
                }
                write!(f, ")")
            },
            Stmt::Error(reporting) => write!(f, "Error({:?})", reporting),
        }
    }
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





//Structure for reporting errors and warnings
#[derive(Debug, Clone)]
pub struct Reporting{
    errorStatus: bool,
    warnings: Vec<String>,
    errors: Vec<String>,
}
impl Reporting{
    fn new() -> Reporting{
        Reporting{
            errorStatus: false,
            warnings: Vec::new(),
            errors: Vec::new(),

        }
    }
    fn reportError(&mut self, message: String){
        self.errors.push(message.clone());
        // println!("reporting error: {}", message);
    }
    fn reportWarning(&mut self, message: String){
        self.warnings.push(message.clone());
        // println!("reporting warning: {}", message);
    }
}

//The structure for the SymbolTable. This holds all of the IDENTIFIERS of the program as well as their scope and information
struct symbolTable{
    // For now you can simply use a single hash table of tokens. As we move forward to parsing, the symbol table
    // structure will have to be augmented to permit the recording of entering/exiting program scopes as well as
    // the scope that an IDENTIFER is declared. In general when you exit a scope the symbol table will remove
    // any symbols defined in that scope from the symbol table. Again, we will solve this problem later; the
    // example methods for scope entry/exit are here to deomonstrate what we will probably want in the future
    symTab: HashMap<String, Token>,
}
impl symbolTable{
    // The symbol table hashLook function should automatically create a new entry and mark it as an
    // IDENTIFER Token for any IDENTIFIER string that is not already in the symbol table. In some languages
    // case does not matter to the uniqueness of the symbol. In this case, an easy place to solve this is to simply
    // upper case or lower case all strings in the symbol table API functions (and storage)
    fn new() -> symbolTable {
        //Creates the empty hash map
        let mut symHash: HashMap<String, Token> = HashMap::new();

        //List of all of the tokens that should be in the symbol table when initializes. Like all of the reserved words and such
        let tokens = vec![
            ("if", Token::new(tokenTypeEnum::IF, "if".to_string(), "0".to_string())),
            ("else", Token::new(tokenTypeEnum::ELSE, "else".to_string(), "0".to_string())),
            ("procedure", Token::new(tokenTypeEnum::PROCEDURE, "procedure".to_string(), "0".to_string())),
            ("is", Token::new(tokenTypeEnum::IS, "is".to_string(), "0".to_string())),
            ("global", Token::new(tokenTypeEnum::GLOBAL, "global".to_string(), "0".to_string())),
            ("variable", Token::new(tokenTypeEnum::VARIABLE, "variable".to_string(), "0".to_string())),
            ("begin", Token::new(tokenTypeEnum::BEGIN, "begin".to_string(), "0".to_string())),
            ("then", Token::new(tokenTypeEnum::THEN, "then".to_string(), "0".to_string())),
            ("end", Token::new(tokenTypeEnum::END, "end".to_string(), "0".to_string())),
            ("program", Token::new(tokenTypeEnum::PROGRAM, "program".to_string(), "0".to_string())),

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
            let newToken = Token::new(tokenTypeEnum::IDENTIFIER, lookupString, line.to_string());
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

    println!("\n\nParsing now");
    // Call the parse function and handle the result
    match myParser.startParse() {
        Ok((reporting, Some(stmt))) => {
            println!("\n\nParsing succeeded.");
            println!("Reporting: {:?}", reporting);
            println!("Parsed Statement: {:?}", stmt);
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

    // Print the parser's token list
    // println!("\n\nMy parser token list: ");
    // myParser.printTokenList();


    Ok(())
}