//Rules
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

mod models;

//package imports
use {
    crate::models::{lexer::Lexer, parser::{Expr, Parser, *}, typechecker::{
        SymbolTable, SyntaxChecker
    }}, anyhow::Result, inkwell::{builder::Builder, context::Context, module::Module, values::*, AddressSpace, OptimizationLevel}, parse_display::Display, std::{
        collections::HashMap, env, fmt, fs::{
            read_to_string, File
        }, 
        hash::Hash, 
        io::{
            prelude::*, BufRead, BufReader, Read
        }, 
        path::Path, 
        rc::Rc
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
    TRUE,
    FALSE,

    
    
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
            tokenTypeEnum::TRUE => "TRUE",
            tokenTypeEnum::FALSE => "FALSE",
            // tokenTypeEnum::OPERATOR => "OPERATOR",


        };
        write!(f, "{}", variant_str)
    }
}

///////////////////////// /Setup /////////////////////////




// The IR generator structure
pub struct Compiler<'ctx> {
    context: &'ctx Context,     //the llvm context
    module: Module<'ctx>,       //the llvm module
    builder: Builder<'ctx>,     //the llvm builder
    programAst: Stmt,           //the programAst that will be run through to generate llvm IR
    // programFunction: 
}

impl<'ctx> Compiler<'ctx> {
    // Initialize a new IRGen instance
    pub fn new(context: &'ctx Context, programAst: Stmt) -> Self {
        // let context = Context::create();
        let module = context.create_module("my_module");
        let builder = context.create_builder();

        Compiler {
            context,
            module,
            builder,
            programAst,
        }
    }

    pub fn compileProgram(&mut self) -> Result<&Module<'ctx>, String>{
        match self.programAst.clone(){
            Stmt::Program(progName, headerBox, bodyBox, lineNum) => {
                //Creates the main function
                let i32Type = self.context.i32_type();
                let mainType = i32Type.fn_type(&[], false);

                let mainFunc = self.module.add_function("main", mainType, None);

                println!("Program ast is a program");
                //Goes through the header and adds each line to the module
                let header = headerBox.clone();
                let mut progHeader = *header;
                // Check if the variable is a Block and iterate through it
                if let Stmt::Block(ref instrs, lineNum) = progHeader.clone() {
                    for instr in instrs {
                        self.compileStmt(instr.clone(), mainFunc);
                    }
                } else {
                    println!("Problem with AST: header must be a Block");
                }

                println!("Header processed");

                //Creates the entrypoint at the main function
                let mainBlock = self.context.append_basic_block(mainFunc, "entry");
                self.builder.position_at_end(mainBlock);
                println!("Created entry point");

                println!("Time to go through body");
                //Goes through the body and adds each line to the module
                let newBodyBox = bodyBox.clone();
                let mut body = *newBodyBox;
                // Check if the variable is a Block and iterate through it
                if let Stmt::Block(ref instrs, lineNum) = body.clone() {
                    for instr in instrs {
                        self.compileStmt(instr.clone(), mainFunc);
                    }
                } else {
                    println!("Problem with AST: header must be a Block");
                }
                let mainRet = i32Type.const_int(1, false);
                let _ = self.builder.build_return(Some(&mainRet));
            }
            _ => {
                let errMsg = format!("ProgramAst must be a Program Stmt");
                return Err(errMsg);
            }
        }
        
        // if let (Stmt::Program(progName, head, body, lineNum) = self.programAst.clone()) {
        //     println!("program good");
        // }
        
        return Ok(&self.module);
    }

    fn compileStmt(&mut self, stmt: Stmt, func: FunctionValue){
        match stmt.clone(){
            //For global variable declarations
            Stmt::GlobVarDecl(varName, varType, lineNum) => {
                match varType{
                    
                    VarType::Bool => {
                        let boolType = self.context.bool_type();
                        let boolName = varName.clone();
                        let globBool = self.module.add_global(boolType.clone(), None, &boolName);
                        return;
                    }
                    VarType::Float => {
                        let varType = self.context.f64_type();
                        let globName = varName.clone();
                        let globVar = self.module.add_global(varType.clone(), None, &globName);
                        return;
                    }
                    VarType::Int => {
                        let varType = self.context.i64_type();
                        let globName = varName.clone();
                        let globVar = self.module.add_global(varType.clone(), None, &globName);
                        return;
                    }
                    VarType::Str => {
                        let maxStringLen = 64 as u32 + 1;
                        let i8Type = self.context.i8_type();
                        let arrayType = i8Type.array_type(maxStringLen);
                        // let stringVal: Vec<IntValue> = 

                        
                        // let varType = self.context.f64_type();
                        let globName = varName.clone();
                        let globVar = self.module.add_global(arrayType.clone(), None, &globName);
                        return;
                    }
                    VarType::IntArray(size) => {
                        let arrSize = size as u32;
                        let i32Type = self.context.i32_type();
                        let arrayType = i32Type.array_type(arrSize);
                        let globName = varName.clone();


                        //Adds to the global variables
                        let globVar = self.module.add_global(arrayType.clone(), None, &globName);

                        
                    }
                }
            }
            Stmt::Assign(varName, newValue, lineNum) => {
                
                
                println!("Variable assignment NEEDS WRITTEn");
                return;
            }
            Stmt::Block(blockStmt, lineNum) => {
                println!("block stmt NEEDS WRITTEn");
                return;
            }
            Stmt::Error(err, lineNum) => {
                println!("error stmt NEEDS WRITTEn");
                return;
            }
            Stmt::Expr(exprStmt, lineNum) => {
                println!("expr stmt NEEDS WRITTEn");
                return;
            }
            Stmt::For(assignment, condition, body, lineNum) => {
                println!("for stmt NEEDS WRITTEn");
                return;
            }
            Stmt::If(condition, body, elseStmt, lineNum) => {
                println!("if statement NEEDS WRITTEn");
                return;
            }
            Stmt::ProcDecl(procType, procName, params, headerBox, bodyBox, lineNum) => {
                println!("procedure declaration NEEDS WRITTEn");
                return;
            }
            Stmt::StringLiteral(str, lineNum) => {
                println!("StringLiteral Stmt, this should never happe");
                return;
            }
            Stmt::VarDecl(varName, varType, lineNum) => {
                println!("local assignment NEEDS WRITTEn");
                return;
            }
            Stmt::Return(valueExpr, lineNum) => {
                println!("return stmt NEEDS WRITTEn");
                return;
            }
            Stmt::Program(name, headerBox, bodyBox, lineNum) => {
                println!("Program Stmt, this should never happen");
                return;
            }
            
        }
        
    }

    // fn generateExpr(&self, expr: &Expr) -> BasicValueEnum<'ctx> {
    //     match expr {
    //         Expr::IntLiteral(value) => self.context.i32_type().const_int(*value as u64, false).into(),
    //         Expr::FloatLiteral(value) => self.context.f32_type().const_float(*value as f64).into(),
    //         Expr::VarRef(_) => {
    //             // Handle variable reference
    //             unimplemented!()
    //         }
    //         Expr::ArthOp(left, op, right) => {
    //             let left_val = self.generateExpr(left);
    //             let right_val = self.generateExpr(right);
    //             match op {
    //                 Operator::Add => self.builder.build_int_add(left_val.into_int_value(), right_val.into_int_value(), "tmp_add").into(),
    //                 Operator::Sub => self.builder.build_int_sub(left_val.into_int_value(), right_val.into_int_value(), "tmp_sub").into(),
    //                 _ => unimplemented!(),
    //             }
    //         }
    //         _ => unimplemented!(),
    //     }
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

    // println!("Lexer reporting: {:?}", myLexer.reports.clone());
    if (myLexer.reports.status) {
        println!("Error in lexer: {:?}", myLexer.reports.clone());
        return Ok(());
    } else {
        println!("Lexer returned successfully");
    }

    // Initialize the parser
    let mut myParser = Parser::new(&mut myLexer);


    // // Print the parser's token list
    // // println!("\n\nMy parser token list: ");
    // // myParser.printTokenList();

    let mut programAst: Stmt;
    match myParser.startParse() {
        Ok((reporting, Some(stmt))) => {
            println!("Parsing completed successfully.");
            programAst = stmt;
        }
        Ok((reporting, None)) => {
            println!("\n\nParsing succeeded, but no programAST was returned.");
            return Ok(());
        }
        Err(reporting) => {
            eprintln!("\n\nParsing failed.");
            eprintln!("Reporting: {:?}", reporting);
            return Ok(());
        }
    }

    // programAst.display(0);

    let mut globalTable = SymbolTable::new();
    let mut myChecker = SyntaxChecker::new(programAst.clone(), &mut globalTable, "Main".to_string());
    println!("\n\nTypeChecker Created");
    let programValid: bool = myChecker.checkProgram();


    if(!programValid){
        println!("\n\nError in program");
        return Ok(());
    } else {
        println!("\n\nProgram is valid");
    }

    //Creates the llvm context and the code generator struct
    let context = Context::create();
    let mut myGen = Compiler::new(&context, programAst.clone());

    println!("Created generator");
    let ret = myGen.compileProgram();
    match ret{
        Ok(module) => {
            println!("Module generated");
            module.print_to_string();
            module.print_to_stderr();
        }
        Err(errMsg) => {
            println!("Error with generation: {}", errMsg);
        }
    }



    Ok(())
}