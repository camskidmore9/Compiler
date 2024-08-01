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
// extern crate funcLib;

mod models;

//package imports
use {
    crate::models::{lexer::Lexer, parser::{Expr, Parser, *}, typechecker::{
        SymbolTable, SyntaxChecker
    }, compiler::*,
    }, anyhow::Result, inkwell::{builder::Builder, OptimizationLevel, passes::PassManager, context::Context, module::Module, types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum}, values::*, AddressSpace, FloatPredicate, IntPredicate}, parse_display::Display, std::{
        collections::HashMap, env::{self, args}, ffi::CString, fmt, rc::Rc
    }
};

///////////////////////// Setup /////////////////////////

use std::fs::{self, File};
//imports
use std::{io::prelude::*, path::Path};
use std::process::{self, Command};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple};
// use llvm_sys::target_machine::LLVMTargetMachineOptionsSetRelocMode;


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





//The main section of the code
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env::set_var("RUST_BACKTRACE", "full");
    // println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rustc-link-lib=dylib=funcLib");
    // // Specify the path if the library is not in a standard location
    // println!("cargo:rustc-link-search=native=./target/release/");
    // Link the Rust library
    // println!("cargo:rustc-link-lib=dylib=funcLib");
    
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

    programAst.display(0);

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

    let mut global_table: HashMap<String, PointerValue> = HashMap::new();

    // let input = env::args().nth(2).expect("Please specify an input value");



    //Creates the llvm context and the code generator struct
    let context = Context::create();
    let mut myGen = Compiler::new(programAst.clone(), &context, &mut global_table, "test".to_string(), "Program".to_string());

    

    println!("Created compiler");
    let ret = myGen.compileProgram();
    let mut finalMod: Module;
    match ret{
        Ok(module) => {
            println!("\n\nModule generated");
            // module.print_to_string();
            module.print_to_stderr();
            finalMod = module.clone();
        }
        Err(errMsg) => {
            println!("Error with generation: {}", errMsg);
            return Ok(());
        }
    }

    // Initialize LLVM targets
    Target::initialize_all(&InitializationConfig::default());

    // Define your target triple
    let target_triple = "x86_64-unknown-linux-gnu"; // Replace with your target triple
    let target_triple = TargetTriple::create(target_triple);


    let target = Target::from_triple(&target_triple).expect("Failed to get target");
    let targetMachineCheck = target.create_target_machine(
        &target_triple,
        "znver2",
        "",
        OptimizationLevel::None,
        RelocMode::Default,
        CodeModel::Default,

        // &target_triple,
        // "generic",
        // "generic",
        // &Default::default(),
        // &Default::default(),
        // &Default::default(),
    );

    let mut targetMachine: TargetMachine;
    match targetMachineCheck{
        Some(target) => {
            targetMachine = target;
        }
        None => {
            println!("no target machine");
            return Ok(());
        }
    }



    // Generate object file
    let path = Path::new("output.o");
    let result = targetMachine.write_to_file(&finalMod, inkwell::targets::FileType::Object, &path);
    if let Err(e) = result {
        println!("Error generating object file: {}", e);
    }

    let llvm_ir_path = Path::new("./out").with_extension("ll");
    finalMod.print_to_file(&llvm_ir_path).expect("Error printing ll file");

    let libPath = Path::new("./target/release/libfuncLib").with_extension("a");

    let output = Command::new("clang")
        .current_dir(env::current_dir().expect("failed to find current dir"))
        .arg(&llvm_ir_path)
        .arg(&libPath)
        .output()
        .expect("failed to execute linker");
    let status = output.status;
    if !status.success() {
        println!("link error: {}", std::str::from_utf8(&output.stderr).unwrap());
        process::exit(1);
    }

    fs::remove_file(llvm_ir_path).expect("failed to remove temporary ll file");




    Ok(())
}