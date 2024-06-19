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

//imports
use std::io::prelude::*;


// #[derive(Debug, Display)]
// #[display("format")]


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
    symTab: symbolTable,
    tokenList: Vec<Token>,
    // reservedWords: [&str; 10],

    
}
impl Lexer{
    fn new(fileName: &str) -> Lexer {
        println!("Beginning creation of Lexer");
        let newFile = inFile::new(fileName);
        println!("Lexer created successfully");
        let mut symTable = symbolTable::new();


        Lexer { 
            //tokenType: (), 
            inputFile: newFile,
            symTab: symTable,
            tokenList: Vec::new(),
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
                let newToken = self.symTab.hashLook(tokenString);
                //let newToken: Token = Token::new(tokType, tokenString);
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
                //self.inputFile.unGetChar();
                let mut newToken = self.symTab.hashLook(tokenString);
                if newToken.tt != tokType {
                    newToken.tt = tokType;
                }
                //let newToken: Token = Token::new(tokType, tokenString);
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
                    let newToken = Token::new(crate::tokenTypeEnum::LESS_EQUALS, tokenString);
                    return newToken;
                } else {
                    // println!("This is just a <");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::LESS, tokenString);
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

                    let newToken = Token::new(crate::tokenTypeEnum::GREATER, tokenString);
                    return newToken;
                } else {
                    // println!("This is just a >");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::GREATER_EQUALS, tokenString);
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

                    let newToken = Token::new(crate::tokenTypeEnum::CHECK_EQUALS, tokenString);
                    return newToken;
                } else if nextC == ' ' {
                    // println!("This is just a >");
                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::SET_EQUALS, tokenString);
                    return newToken;
                } else {
                    println!("ERROR");

                    self.inputFile.unGetChar();
                    let newToken = Token::new(crate::tokenTypeEnum::ERROR, tokenString);
                    return newToken;
                }
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
                let mut newToken = self.symTab.hashLook(tokenString);
                if newToken.tt != tokenTypeEnum::STRING {
                    newToken.tt = tokenTypeEnum::STRING;
                }
                //let newToken: Token = Token::new(tokType, tokenString);
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
                let newToken = Token::new(crate::tokenTypeEnum::UNACCOUNTED, tokenString);
                return newToken;
            }
            None => {
                // println!("This character is a None aka EOF");
                let newToken = Token::new(crate::tokenTypeEnum::EOF, "EOF".to_string());
                return newToken;
            }
        }
    }
    
    //Prints all of the tokens
    fn printTokenList(&mut self){
        for token in &self.tokenList {
            println!("< \"{}\" , {} >", token.tokenString, token.tt.to_string());
        }
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

        
        
    }

}


//This is the master struct for the parser
struct Parser {
    tokenList: Vec<Token>,
}
impl Parser{
    //Initialization function
    fn new(lexer: &mut Lexer) -> Parser {
        println!("Beginning creation of Parser");
        let tokenList = lexer.tokenList.clone();
        // let newFile = inFile::new(fileName);
        println!("Parser created successfully");


        Parser { 
            tokenList,
        }
    }

    fn parseThrough(&mut self) {
        parse(&mut self.tokenList);
    }

  

    //Prints all of the tokens
    fn printTokenList(&mut self){
        for token in &self.tokenList {
            println!("< \"{}\" , {} >", token.tokenString, token.tt.to_string());
        }
    }
}

fn parse(tokenList: &mut Vec<Token>) -> Stmt {
    println!("Beginning parsing");
    let numTokens: usize = tokenList.len();
    println!("Total number of tokens: {}", numTokens.to_string());

    let mut scope: i32 = 0;
    

    // Iterate through tokenList using an index
    for i in 0..tokenList.len() {
        //Gets next token
        let token = &tokenList[i];

        //Creates 

        match token.tt {
            tokenTypeEnum::PROGRAM => {
                println!("Found a program token");
                //This is for the beginning of the program
                if scope == 0{ 
                    println!("Current i: {}", i);
                    let isToken = &mut tokenList[i + 2];
                    if !(isToken.tt == tokenTypeEnum::IS) {
                        println!("ERROR");
                        isToken.printToken();
                        return Stmt::StringLiteral("Not Good".to_string());
                    } else {
                        println!("Program statemtnt good");
                        return Stmt::StringLiteral("Good".to_string());
                    }

                }
            }
            _ => {
                return Stmt::StringLiteral("Unwritten".to_string());
                // Handle any other token types that need specific handling
                // println!("Found something other than a word");
                // println!("< \"{}\" , {} >", token.tokenString, token.tt.to_string());
            }
        }
        return Stmt::StringLiteral("Unwritten".to_string());

        // Example: Move forward by one token
        // if i + 1 < self.tokenList.len() {
        //     let next_token = &self.tokenList[i + 1];
        //     // Handle next token if needed
        // }

        // Example: Move backward by one token
        // if i > 0 {
        //     let prev_token = &self.tokenList[i - 1];
        //     // Handle previous token if needed
        // }
    }
    println!("No elements in this token list");
    return Stmt::StringLiteral("ZERO ELEMENTS".to_string());
}





// Define the types of expressions
#[derive(Debug)]
pub enum Expr {
    IntLiteral(i64),            // Integer literal
    BinOp(Box<Expr>, BinOp, Box<Expr>),  // Binary operation: left operand, operator, right operand
    VarRef(String),             // Variable reference
}

// Define supported binary operators
#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

// These are the types of statements that are available
#[derive(Debug)]
pub enum Stmt {
    StringLiteral(String),
    Expr(Expr),                     // Expression statement
    Assign(String, Expr),           // Assignment statement: variable name, expression
    VarDecl(String),                // Variable declaration statement
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),  // If statement: condition, body, optional else body
    Block(Vec<Stmt>),               // Block statement: list of statements
}

// // This is the struct that defines the vector of statements
// #[derive(Debug)]
// pub struct Program {
//     pub statements: Vec<Stmt>,  // List of statements in the program
// }



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
#[derive(Clone)]
struct Token{
    tt: tokenTypeEnum,
    tokenString: String,
    //To be completed later when I understand
    //tm: tokenMark,
}
impl Token{
    //Init for the Token
    fn new(iden: tokenTypeEnum, tokenString: String) -> Token{
        //self.tokenMark = NULL;
        // println!("Created the Token struct");
        // println!("Created a Token of type: '{}'", iden.to_string());

        Token {
            tt: iden,
            tokenString: tokenString,
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
            ("if", Token::new(tokenTypeEnum::IF, "if".to_string())),
            ("else", Token::new(tokenTypeEnum::ELSE, "else".to_string())),
            ("procedure", Token::new(tokenTypeEnum::PROCEDURE, "procedure".to_string())),
            ("is", Token::new(tokenTypeEnum::IS, "is".to_string())),
            ("global", Token::new(tokenTypeEnum::GLOBAL, "global".to_string())),
            ("variable", Token::new(tokenTypeEnum::VARIABLE, "variable".to_string())),
            ("begin", Token::new(tokenTypeEnum::BEGIN, "begin".to_string())),
            ("then", Token::new(tokenTypeEnum::THEN, "then".to_string())),
            ("end", Token::new(tokenTypeEnum::END, "end".to_string())),
            ("program", Token::new(tokenTypeEnum::PROGRAM, "program".to_string())),

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
    fn hashLook(&mut self, mut lookupString: String) -> Token{
        // println!("Looking up the identifier of the string");
        if let Some(tokenResp) = self.symTab.get(&lookupString){
            // println!("Token found");
            return tokenResp.clone();
        } else {
            // println!("Token not found, creating");
            let newToken = Token::new(tokenTypeEnum::IDENTIFIER, lookupString);
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
fn main() -> Result<()> {
    
    let path = env::args().nth(1).expect("Please specify an input file");
    let mut myLexer: Lexer = Lexer::new(&path);
    println!("Lexer filename: {} \nCharacter count: {}", myLexer.inputFile.fileName, myLexer.inputFile.numChars);

    myLexer.scanThrough();

    // myLexer.printTokenList();

    let mut myParser: Parser = Parser::new(&mut myLexer);

    myParser.parseThrough();
    
    // println!("My parser token list: ");
    // myParser.printTokenList();
    

    Ok(())
}