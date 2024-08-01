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

// mod models;

//package imports
use {
    crate::models::{lexer::Lexer, parser::{Expr, Parser, *}, typechecker::{
        SymbolTable, SyntaxChecker
    }}, anyhow::Result, core::panic, inkwell::{builder::Builder, context::{self, Context}, module::Module, types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType}, values::*, AddressSpace, FloatPredicate, IntPredicate}, parse_display::Display, std::{
        array, collections::HashMap, env::{self, args}, ffi::CString, fmt, rc::Rc
    }
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
    scope: i32,
    pub localTable: HashMap<String, PointerValue<'ctx>>, // Local table for the current scope
    pub globalTable: &'ctx mut HashMap<String, PointerValue<'ctx>>, // Shared global table
    pub name: String,
    pub stdIn: String,
}

impl<'ctx> Compiler<'ctx> {
    // Initialize a new IRGen instance
    // The constructor
    pub fn new(
        programAst: Stmt,
        context: &'ctx Context,
        globalTable: &'ctx mut HashMap<String, PointerValue<'ctx>>,
        stdIn: String,
        name: String
    ) -> Compiler<'ctx> {
        let mut module = context.create_module("my_module");
        let mut builder = context.create_builder();

        Compiler {
            programAst,
            scope: 0,
            localTable: HashMap::new(),
            globalTable,
            name,
            context,
            module,
            builder,
            stdIn,
        }
    }

    pub fn compileProgram(&mut self) -> Result<&Module<'ctx>, String>{
        match self.programAst.clone(){
            Stmt::Program(progName, headerBox, bodyBox, lineNum) => {
                //Adds the built ints
                self.defineGetInt();
                self.definePutInt();
                self.defineGetFloat();
                self.definePutBool();
                self.definePutFloat();
                
                
                //Creates the main function
                let i32Type = self.context.i32_type();
                let mainType = i32Type.fn_type(&[], false);

                let mut mainFunc = self.module.add_function("main", mainType, None);

                println!("Program ast is a program");
                //Goes through the header and adds each line to the module
                let header = headerBox.clone();
                let mut progHeader = *header;
                // Check if the variable is a Block and iterate through it
                let mainBuilder = self.context.create_builder();

                
                

                let mut mainLocalTable: HashMap<String, PointerValue<'ctx>> = HashMap::new();
                // self.builder = mainBuilder;
                if let Stmt::Block(ref instrs, lineNum) = progHeader.clone() {
                    for instr in instrs {
                        self.compileStmt(instr.clone(), &mainBuilder, &mut mainLocalTable, mainFunc);
                    }
                } else {
                    println!("Problem with AST: header must be a Block");
                }

                println!("Header processed");

                //Creates the entrypoint at the main function
                let mainBlock = self.context.append_basic_block(mainFunc, "entry");
                mainBuilder.position_at_end(mainBlock);
                println!("Created entry point");

                println!("Time to go through body");
                //Goes through the body and adds each line to the module
                let newBodyBox = bodyBox.clone();
                let mut body = *newBodyBox;

                // Check if the variable is a Block and iterate through it
                if let Stmt::Block(ref instrs, lineNum) = body.clone() {
                    for instr in instrs {
                        let good = self.compileStmt(instr.clone(), &mainBuilder, &mut mainLocalTable, mainFunc);
                    }
                } else {
                    println!("Problem with AST: header must be a Block");
                    
                }
                let mainRet = i32Type.const_int(0, false);
                let _ = mainBuilder.build_return(Some(&mainRet));
            }
            _ => {
                let errMsg = format!("ProgramAst must be a Program Stmt");
                panic!("{}", errMsg);
            }
        }
        
        // if let (Stmt::Program(progName, head, body, lineNum) = self.programAst.clone()) {
        //     println!("program good");
        // }
        
        return Ok(&self.module);
    }

    fn compileStmt(&mut self, stmt: Stmt, builder: &Builder<'ctx>, localTable: &mut HashMap<String, PointerValue<'ctx>>, function: FunctionValue) -> bool{
        match stmt.clone(){
            //For global variable declarations
            Stmt::VarDecl(varName, varType, lineNum) => {
                match varType{
                    VarType::Bool => {
                        let localType = self.context.bool_type();
                        let localName = varName.clone();
                        
                        
                        // let globVar = self.module.add_global(boolType.clone(), None, &boolName);
                        
                        let localVarCheck = builder.build_alloca(localType.clone(), &localName.clone());

                        let localPtr: PointerValue;
                        match localVarCheck{
                            Ok(ptr) => {
                                localPtr = ptr.clone();
                            }
                            Err(err) => {
                                println!("Error allocating local bool variable {}", localName.clone());
                                panic!();
                            }
                        }

                        let initVal = localType.const_int(0, false);
                        let _ = builder.build_store(localPtr, initVal);

                        localTable.insert(varName.clone(), localPtr);
                        
                        return true;
                    }
                    VarType::Float => {
                        let localType = self.context.f32_type();
                        let localName = varName.clone();
                        
                        
                        // let globVar = self.module.add_global(boolType.clone(), None, &boolName);
                        
                        let localVarCheck = builder.build_alloca(localType.clone(), &localName.clone());

                        let localPtr: PointerValue;
                        match localVarCheck{
                            Ok(ptr) => {
                                localPtr = ptr.clone();
                            }
                            Err(err) => {
                                println!("Error allocating local float variable {}", localName.clone());
                                panic!();
                            }
                        }

                        let initVal = localType.const_float(0.0);
                        let _ = builder.build_store(localPtr, initVal);

                        localTable.insert(varName.clone(), localPtr);
                        
                        return true;
                    }
                    VarType::Int => {
                        let localType = self.context.i32_type();
                        let localName = varName.clone();
                        
                        
                        // let globVar = self.module.add_global(boolType.clone(), None, &boolName);
                        
                        let localVarCheck = builder.build_alloca(localType.clone(), &localName.clone());

                        let localPtr: PointerValue;
                        match localVarCheck{
                            Ok(ptr) => {
                                localPtr = ptr.clone();
                            }
                            Err(err) => {
                                println!("Error allocating local int variable {}: {}", localName.clone(), err);
                                panic!();
                            }
                        }

                        let initVal = localType.const_int(0, false);
                        let _ = builder.build_store(localPtr, initVal);

                        localTable.insert(varName.clone(), localPtr);
                        
                        return true;
                    }
                    VarType::Str => {
                        let maxStringLen = 64 as u32 + 1;
                        let i8Type = self.context.i8_type();
                        let arrayType = i8Type.array_type(maxStringLen);
                        // let stringVal: Vec<IntValue> = 

                        
                        let localVarCheck = builder.build_alloca(arrayType.clone(), &varName.clone());

                        let localPtr: PointerValue;
                        match localVarCheck{
                            Ok(ptr) => {
                                localPtr = ptr.clone();
                            }
                            Err(err) => {
                                println!("Error allocating local str variable {}", varName.clone());
                                panic!();
                            }
                        }

                        let string = "EMPTY";
                        let stringBytes = string.as_bytes();
                        let arrayVal = self.context.const_string(stringBytes, false).clone();
            
            
                        // Wrap the array constant in a BasicValueEnum
                        let initVal = BasicValueEnum::ArrayValue(arrayVal);
                        let _ = builder.build_store(localPtr, initVal);

                        localTable.insert(varName.clone(), localPtr);
                        
                        return true;
                    }
                    VarType::IntArray(size) => {
                        let arrSize = size as u32;
                        let i32Type = self.context.i32_type();
                        let arrayType = i32Type.array_type(arrSize);
                        let globName = varName.clone();


                        //Adds to the local variables
                        let localVarCheck = builder.build_alloca(arrayType.clone(), &varName.clone());

                        let localPtr: PointerValue;
                        match localVarCheck{
                            Ok(ptr) => {
                                localPtr = ptr.clone();
                            }
                            Err(err) => {
                                println!("Error allocating local str variable {}", varName.clone());
                                panic!();
                            }
                        }
                        localTable.insert(varName.clone(), localPtr);
                        
                        return true;
                        
                    }
                }
                
            }
            
            Stmt::GlobVarDecl(varName, varType, lineNum) => {
                match varType{
                    VarType::Bool => {
                        let boolType = self.context.bool_type();
                        let boolName = varName.clone();
                        let globVar = self.module.add_global(boolType.clone(), None, &boolName);
                        let _ =  globVar.set_initializer(&boolType.const_int(0, false));
                        let globPtr = globVar.as_pointer_value();
                        self.globalTable.insert(varName.clone(), globPtr);
                        
                        return true;
                    }
                    VarType::Float => {
                        let varType = self.context.f32_type();
                        let globName = varName.clone();
                        let globVar = self.module.add_global(varType.clone(), None, &globName);
                        let _ =  globVar.set_initializer(&varType.const_float(0.0));
                        
                        let globPtr = globVar.as_pointer_value();
                        self.globalTable.insert(varName.clone(), globPtr);
                        
                        return true;
                    }
                    VarType::Int => {
                        let varType = self.context.i32_type();
                        let globName = varName.clone();
                        let globVar = self.module.add_global(varType.clone(), None, &globName);
                        
                        let _ =  globVar.set_initializer(&varType.const_int(0, false));
                        
                        let globPtr = globVar.as_pointer_value();
                        self.globalTable.insert(varName.clone(), globPtr);
                        
                        return true;
                    }
                    VarType::Str => {

                        // Define the array type [65 x i8]
                        let max_string_len = 65;
                        let i8_type = self.context.i8_type();
                        let array_type = i8_type.array_type(max_string_len);

                        // Create a string that fits exactly 65 characters with padding and null terminator
                        let string_value = "A".repeat(64) + "\0"; // 64 spaces + null terminator

                        // Convert the string into a byte array
                        let string_bytes = string_value.into_bytes();
                        
                        // // Create the constant array with the string bytes
                        // let const_array = array_type.const_array(
                        //     &string_bytes.iter().map(|&byte| i8_type.const_int(byte as u64, false).into()).collect::<Vec<BasicValueEnum>>()
                        // );

                        // Declare the global variable and initialize it
                        let glob_name = varName.clone();
                        let glob_var = self.module.add_global(array_type, Some(AddressSpace::default()), &glob_name);
                        let globPtr = glob_var.as_pointer_value();
                        // let test = unsafe { ArrayValue::new(string_value) };
                        let test = array_type.const_zero();
                        self.globalTable.insert(glob_name.clone(), globPtr);
                        glob_var.set_initializer(&test);
                        
                        return true;
                    }
                    VarType::IntArray(size) => {
                        let arrSize = size as u32;
                        let i32Type = self.context.i32_type();
                        let arrayType = i32Type.array_type(arrSize);
                        let globName = varName.clone();


                        //Adds to the global variables
                        let globVar = self.module.add_global(arrayType.clone(), None, &globName);
                        let globPtr = globVar.as_pointer_value();
                        self.globalTable.insert(varName.clone(), globPtr);
                        
                        return true;
                        
                    }
                }
                
            }
            Stmt::Assign(variable, newValue, lineNum) => {
                let mut variablePtr: PointerValue;
                let mut newEnumValue: BasicValueEnum;
                let mut varName: String;

                if let Expr::VarRef(ref targName) = variable {
                    varName = targName.clone();
                    let checkLocVar = localTable.get(&targName.clone());
                    match checkLocVar{
                        Some(ptr) => {
                            println!("Assigninig local variable {} at location {}", targName.clone(), ptr.clone());
                            variablePtr = ptr.clone();
                        }
                        None => {
                            let checkGlobVar = self.globalTable.get(&targName.clone());
                            match checkGlobVar{
                                Some(ptr) => {
                                    println!("Assigninig global variable {} at location {}", targName.clone(), ptr.clone());
                                    variablePtr = ptr.clone();
                                }
                                None => {
                                    println!("variable {} not found", targName.clone());
                                    panic!();
                                }
                            }
                        }
                    }
                }
                
                else if let Expr::ArrayRef(ref targName, indexExpr) = variable{
                    // println!("ASsigning")
                    varName = targName.clone();
                    let arrSize = 64 as u32;
                    let i32Type = self.context.i32_type().clone();
                    let arrayType = i32Type.array_type(arrSize).clone();
                    
                    //Gets the value of the index expression
                    let indexExprCheck = self.compileExpr(&*&indexExpr.clone(), builder, localTable);
                    let mut indexVal: BasicValueEnum;
                    match indexExprCheck{
                        Ok(val) => {
                            indexVal = val.clone();
                        }
                        Err(err) => {
                            println!("{}", err.clone());
                            panic!();
                        }
                    }

                    
                    let mut arrayPtr: PointerValue;
                    let checkLocVar = localTable.get(&targName.clone());
                    match checkLocVar{
                        Some(ptr) => {
                            println!("Assigninig local variable {} at location {}", targName.clone(), ptr.clone());
                            arrayPtr = ptr.clone();
                        }
                        None => {
                            let checkGlobVar = self.globalTable.get(&targName.clone());
                            match checkGlobVar{
                                Some(ptr) => {
                                    println!("Assigninig global array variable {} at location {}", targName.clone(), ptr.clone());
                                    arrayPtr = ptr.clone();
                                }
                                None => {
                                    println!("variable {} not found", targName.clone());
                                    panic!();
                                }
                            }
                        }
                    }

                    let mut indexInt: IntValue;
                    match indexVal{
                        BasicValueEnum::IntValue(val) => {
                            indexInt = val.clone();
                        }
                        BasicValueEnum::FloatValue(val) => {
                            let intType = self.context.i32_type().clone();
                            let intValue = builder.build_float_to_signed_int(val.clone(), intType, "float_to_int");
                            match intValue{
                                Ok(iVal) => {
                                    indexInt = iVal.clone();
                                }
                                Err(err) => {
                                    println!("Error converting float to int");
                                    panic!();
                                }
                            }

                        }
                        _ => {
                            println!("Can only index by integer");
                            panic!();
                        }
                    }
                
                    // Get the pointer to the desired index
                    // let variablePtr: PointerValue;
                    let intType = self.context.i32_type().clone();
                    let zero = intType.const_int(0, false);
                    let indexList = [zero, indexInt];
                    let checkIndexPtr = unsafe { builder.build_gep(arrayPtr, &indexList, "arrayIndexLoad") };
                    match checkIndexPtr{
                        Ok(ptr) => {
                            println!("GOT ARRAY INDEX PTR");
                            variablePtr = ptr.clone();
                        }
                        Err(err) => {
                            println!("Error getting array index ptr");
                            panic!();
                        }
                    }
                    // let elementPtr = builder.
                
                }
                
                else {
                    println!("Cannot assing to a non variable");
                    panic!();
                }

                if let Expr::ArrayRef(ref targName, indexExpr) = newValue.clone() {
                    println!("array reference");
                    // let targName = name.clone();
                    let arrSize = 64 as u32;
                    let i32Type = self.context.i32_type().clone();
                    let arrayType = i32Type.array_type(arrSize).clone();
                    
                    //Gets the value of the index expression
                    let indexExprCheck = self.compileExpr(&*&indexExpr.clone(), builder, localTable);
                    let mut indexVal: BasicValueEnum;
                    match indexExprCheck{
                        Ok(val) => {
                            indexVal = val.clone();
                        }
                        Err(err) => {
                            println!("{}", err.clone());
                            println!("Could error with index {}", err.clone());
                            panic!();
                        }
                    }

                    
                    let mut arrayPtr: PointerValue;
                    let checkLocVar = localTable.get(&targName.clone());
                    match checkLocVar{
                        Some(ptr) => {
                            arrayPtr = ptr.clone();
                        }
                        None => {
                            let checkGlobVar = self.globalTable.get(&targName.clone());
                            match checkGlobVar{
                                Some(ptr) => {
                                    arrayPtr = ptr.clone();
                                }
                                None => {
                                    println!("variable {} not found", targName.clone());
                                    panic!();
                                }
                            }
                        }
                    }

                    let mut indexInt: IntValue;
                    match indexVal{
                        BasicValueEnum::IntValue(val) => {
                            indexInt = val.clone();
                        }
                        BasicValueEnum::FloatValue(val) => {
                            let intType = self.context.i32_type().clone();
                            let intValue = builder.build_float_to_signed_int(val.clone(), intType, "float_to_int");
                            match intValue{
                                Ok(iVal) => {
                                    indexInt = iVal.clone();
                                }
                                Err(err) => {
                                    println!("Error converting float to int");
                                    panic!();
                                }
                            }

                        }
                        _ => {
                            println!("Can only index by integer");
                            panic!();
                        }
                    }
                
                    // Get the pointer to the desired index
                    let variablePtr: PointerValue;
                    let checkIndexPtr = unsafe { builder.build_in_bounds_gep(arrayPtr, &[indexInt], "test") };
                    match checkIndexPtr{
                        Ok(ptr) => {
                            variablePtr = ptr.clone();
                        }
                        Err(err) => {
                            println!("Error getting array index ptr");
                            panic!();
                        }
                    }

                    //Gets the value at that pointer
                    let retValCheck = builder.build_load(variablePtr, "arrayIndexReference");
                    match retValCheck{
                        Ok(val) => {
                            println!("ARRAY INDEX VALUE GOT {}", val.clone());
                            newEnumValue = val.clone();
                        }
                        Err(msg) => {
                            println!("Error getting array index value");
                            panic!();
                        }
                    }
                    // let elementPtr = builder.
                
                }
                else {
                    let checkNewValue = self.compileExpr(&newValue.clone(), builder, localTable);
                    match checkNewValue.clone(){
                        Ok(value) => {
                            
                            newEnumValue = value.clone();
                        }
                        Err(msg) => {
                            println!("{}", msg.clone());
                            panic!();
                        }
                    }
                }
                
                
                let mut finalVal = newEnumValue.clone();

                // let builder = &mut builder;

                // let mut finalVal: IntValue;
                match finalVal{
                    BasicValueEnum::IntValue(intVal) => {
                        println!("Stored int value {} in variable {}",intVal.clone(), varName.clone());
                        let _ = builder.build_store(variablePtr, intVal.clone());
                        return true;
                    }
                    BasicValueEnum::FloatValue(intVal) => {
                        

                        

                        
                        println!("Stored int value {} in variable {}",intVal.clone(), varName.clone());
                        let _ = builder.build_store(variablePtr, intVal.clone());
                        return true;
                    }
                    BasicValueEnum::ArrayValue(val) => {
                        

                        println!("ARRAY {}", val.clone());
                        let _ = builder.build_store(variablePtr, val.clone());

                        return true;
                    }

                    _ => {
                        println!("Not implemented for that type yet");
                        return true;
                    }
                }

                // builder.build_store(variablePtr, finalVal);
                // return true;
                    
            }
            Stmt::Block(blockStmt, lineNum) => {
                for instr in blockStmt.clone() {
                    let good = self.compileStmt(instr.clone(), builder, localTable, function);
                    if (!good){
                        println!("Error in block:");
                        instr.display(0);
                        panic!();
                    } else {
                        //continue
                    }
                }
                return true;
            }
            Stmt::Error(err, lineNum) => {
                println!("Somehow an error made it to the compiler");
                panic!();
            }
            Stmt::Expr(exprStmt, lineNum) => {
                // println!("ExprStmt needs written");
                // return true;
                match (exprStmt.clone()){
                    _ => {
                        let checked = self.compileExpr(&exprStmt.clone(), builder, localTable);
                        match checked {
                            Ok(val) => {
                                // println!("Called expr{}")
                                println!("SOMETHING SHOULD BE DONE HERE, IDK");
                                return true;
                            }
                            Err(err) => {
                                println!("Error: {}", err.clone());
                                panic!();
                            }
                        }
                    }
                }
            }
            Stmt::For(assignment, condExpr, body, lineNum) => {
                //Creates the local builder
                let forBuilder = builder;

                
                //Parses the assignment first
                let mut iInitVal: BasicValueEnum;
                let mut iName: String;
                let assignStmt = Rc::clone(&assignment);
                if let Stmt::Assign(varRef, val, lineNum) = &*assignStmt.clone() {
                    if let Expr::VarRef(varName) = varRef.clone(){
                        println!("for loop variable i {}", varName.clone());
                        iName = varName.clone();
                        let iteratorValCheck = self.compileExpr(&val.clone(), &forBuilder, localTable);
                        match iteratorValCheck{
                            Ok(val) => {
                                println!("Iterator value: {}", val.clone());
                                iInitVal = val;
                            }
                            Err(err) => {
                                println!("Error parsing for loop iterator assignment: {}", err.clone());
                                panic!();
                            }
                        }
                    }
                    else {
                        println!("Error: For loop iterator must be a variable");
                        panic!();
                    }
                }
                else {
                    println!("Error: For loop assignment must be a variable assignment");
                    return false
                }
                
                
                let intType = self.context.i32_type().clone();
                let fnType = intType.fn_type(&[], false);

                //set up the loop "function"
                let loopFunction = function;
                // let forEntry = self.context.append_basic_block(loopFunction, "ForEntry");
                let loopCond = self.context.append_basic_block(loopFunction, "forCond");
                let loopBody = self.context.append_basic_block(loopFunction, "forBody");
                let mergeFor = self.context.append_basic_block(loopFunction, "mergeFor");
                

                
                //Sets up the conditional
                // let _ = forBuilder.build_store(iPtr, iInitVal.clone());
                let _ = forBuilder.build_unconditional_branch(loopCond);
                
                //Loop condition block
                let _ = forBuilder.position_at_end(loopCond); 

                //Parse the condition
                let mut condOp1Val: BasicValueEnum;
                let mut condOp2Val: BasicValueEnum;
                let mut condOp: IntPredicate; 
                if let Expr::RelOp(op1Box, op, op2Box) = condExpr{
                   let op1 = *op1Box.clone();
                   let op2 = *op2Box.clone();
                    match op{
                        Operator::Greater => {
                            condOp = IntPredicate::SGT;
                        }
                        Operator::Greater_Equal => {
                            condOp = IntPredicate::SGE;
                        }
                        Operator::Less => {
                            condOp = IntPredicate::SLT;
                        }
                        Operator::Less_Equal => {
                            condOp = IntPredicate::SLE;
                        }
                        Operator::Check_Equal => {
                            condOp = IntPredicate::EQ;
                        }
                        Operator::Not_Equals => {
                            condOp = IntPredicate::NE;
                        }
                        _ => {
                            println!("For condition operator must be logical operator");
                            panic!();
                        }
                    }
                    //First gets the values of both operands
                    let op1Res = self.compileExpr(&op1.clone(), &forBuilder, localTable);
                    //Makes sure both results of checked operands are good
                    match op1Res{
                        Ok(res) => {
                            condOp1Val = res;
                        }
                        Err(msg) => {
                            panic!("Error in for loop condition");
                        }
                    }
                    //First gets the values of both operands
                    let op2Res = self.compileExpr(&op2.clone(), &forBuilder, localTable);
                    //Makes sure both results of checked operands are good
                    match op2Res{
                        Ok(res) => {
                            condOp2Val = res;
                        }
                        Err(msg) => {
                            panic!("Error in for loop condition");
                        }
                    }
                    
                   
                } else {
                
                    panic!("For loop condition must be a logical operation");
                }

                
                
                //Checks/converts the values of the 2 operands
                let mut op1Int: IntValue;
                let mut op2Int: IntValue;
                match condOp1Val{
                    BasicValueEnum::IntValue(val) => {
                        op1Int = val.clone();
                    }
                    BasicValueEnum::FloatValue(val) => {
                        let intType = self.context.i32_type().clone();
                        let intVal = forBuilder.build_float_to_signed_int(val.clone(), intType.clone(), "floatToInt");
                        match intVal{
                            Ok(val) => {
                                op1Int = val.clone()
                            }
                            Err(msg) => {
                                println!("Error converting float to int");
                                panic!();
                            }
                        }
                    }
                    _ => {
                        println!("For loop condition values must be numbers");
                        panic!();
                    }
                }
                match condOp2Val{
                    BasicValueEnum::IntValue(val) => {
                        op2Int = val.clone();
                    }
                    BasicValueEnum::FloatValue(val) => {
                        let intType = self.context.i32_type().clone();
                        let intVal = forBuilder.build_float_to_signed_int(val.clone(), intType.clone(), "floatToInt");
                        match intVal{
                            Ok(val) => {
                                op2Int = val.clone()
                            }
                            Err(msg) => {
                                println!("Error converting float to int");
                                panic!();
                            }
                        }
                    }
                    _ => {
                        println!("For loop condition values must be numbers");
                        panic!();
                    }
                }

                //Creates the condition
                let conditionCheck = forBuilder.build_int_compare(condOp.clone(), op1Int.clone(), op2Int.clone(), "forLoopCondition");
                let condition: IntValue;
                match conditionCheck{
                    Ok(val) => {
                        condition = val.clone();
                    }
                    Err(msg) => {
                        println!("Error creating condition");
                        panic!();
                    }
                }

                //Set up the conditional branch, determining if loop is taken or not
                let _ = forBuilder.build_conditional_branch(condition, loopBody, mergeFor);

                //Move builder to the loop body to populate it
                forBuilder.position_at_end(loopBody);

                //Populates the body with statements
                let bodyStmt = *body.clone();
                self.compileStmt(bodyStmt.clone(), &forBuilder, localTable, function);

                //Adds a conditional check to the end
                let _ = forBuilder.build_unconditional_branch(loopCond);

                //Moves builder to the end of the block
                forBuilder.position_at_end(mergeFor);

                println!("CREATED FOR LOOP ");

                // let _ = builder.build_call(loopFunction.clone(), &[], "forLoopCall");
                // let _ = builder.build_unconditional_branch(forEntry);
                println!("Inserted for loop");
                return true;
            }
            Stmt::If(condExpr, body, elseStmt, lineNum) => {
                
                
                //Sets up the function stuff
                let voidType = self.context.void_type().clone();
                let fnType = voidType.fn_type(&[], false);
                let ifFunction = function;
                // let ifEntry = self.context.append_basic_block(ifFunction, "ifEntry");
                // let ifCond = self.context.append_basic_block(ifFunction, "ifCondition");
                let ifBody = self.context.append_basic_block(ifFunction, "ifBody");
                let elseBody = self.context.append_basic_block(ifFunction, "elseBody");
                let mergeBack = self.context.append_basic_block(ifFunction, "ifMerge");
                
                // let ifEnd = self.context.append_basic_block(ifFunction, "forEnd");
                let ifBuilder = builder;
                

                //Position at the beginning of the if statement
                // ifBuilder.position_at_end(ifEntry);

                //Parse the condition
                let mut condOp1Val: BasicValueEnum;
                let mut condOp2Val: BasicValueEnum;
                let mut condOp: IntPredicate; 
                if let Expr::RelOp(op1Box, op, op2Box) = condExpr.clone(){
                   let op1 = *op1Box.clone();
                   let op2 = *op2Box.clone();
                    match op{
                        Operator::Greater => {
                            condOp = IntPredicate::SGT;
                        }
                        Operator::Greater_Equal => {
                            condOp = IntPredicate::SGE;
                        }
                        Operator::Less => {
                            condOp = IntPredicate::SLT;
                        }
                        Operator::Less_Equal => {
                            condOp = IntPredicate::SLE;
                        }
                        Operator::Check_Equal => {
                            condOp = IntPredicate::EQ;
                        }
                        Operator::Not_Equals => {
                            condOp = IntPredicate::NE;
                        }
                        _ => {
                            println!("For condition operator must be logical operator");
                            panic!();
                        }
                    }
                    

                    let op1Check = self.compileExpr(&op1.clone(), builder, localTable);
                    match op1Check{
                        Ok(val) => {
                            condOp1Val = val.clone();
                        }
                        Err(err) => {
                            println!("Error getting if condition op 1: {}", err.clone());
                            panic!();
                        }
                    }
                    let op2Check = self.compileExpr(&op2.clone(), builder, localTable);
                    match op2Check{
                        Ok(val) => {
                            condOp2Val = val.clone();
                        }
                        Err(err) => {
                            println!("Error getting if condition op 2");
                            panic!();
                        }
                    } 

                } else if let Expr::BoolLiteral(boolVal) = condExpr.clone() {
                    let intBool = boolVal.clone() as u64;
                    let intVal = self.context.bool_type();
                    let boolConst = intVal.const_int(intBool.clone(), false);
                    let boolVal = BasicValueEnum::IntValue(boolConst.clone());

                    condOp1Val = boolVal.clone();
                    condOp2Val = boolVal.clone();
                    condOp = IntPredicate::EQ;

                } else {
                    println!("If loop condition must be a logical operation");
                    panic!();
                }
                
                //Parses operand returns
                let mut op1Int: IntValue;
                let mut op2Int: IntValue;
                match condOp1Val{
                    BasicValueEnum::IntValue(val) => {
                        op1Int = val.clone();
                    }
                    BasicValueEnum::FloatValue(val) => {
                        let intType = self.context.i32_type().clone();
                        let intVal = ifBuilder.build_float_to_signed_int(val.clone(), intType.clone(), "floatToInt");
                        match intVal{
                            Ok(val) => {
                                op1Int = val.clone()
                            }
                            Err(msg) => {
                                println!("Error converting float to int");
                                panic!();
                            }
                        }
                    }
                    _ => {
                        println!("For loop condition values must be numbers");
                        panic!();
                    }
                }
                match condOp2Val{
                    BasicValueEnum::IntValue(val) => {
                        op2Int = val.clone();
                    }
                    BasicValueEnum::FloatValue(val) => {
                        let intType = self.context.i32_type().clone();
                        let intVal = ifBuilder.build_float_to_signed_int(val.clone(), intType.clone(), "floatToInt");
                        match intVal{
                            Ok(val) => {
                                op2Int = val.clone()
                            }
                            Err(msg) => {
                                println!("Error converting float to int");
                                panic!();
                            }
                        }
                    }
                    _ => {
                        println!("For loop condition values must be numbers");
                        panic!();
                    }
                }

                //Creates the condition
                let conditionCheck = ifBuilder.build_int_compare(condOp.clone(), op1Int.clone(), op2Int.clone(), "ifCondition");
                let condition: IntValue;
                match conditionCheck{
                    Ok(val) => {
                        condition = val.clone();
                    }
                    Err(msg) => {
                        println!("Error creating condition");
                        panic!("Invalid condition");
                        
                    }
                }

                //Creates the condition
                let _ = builder.build_conditional_branch(condition, ifBody, elseBody);

                //Position at the end of the ifBody
                ifBuilder.position_at_end(ifBody);
                
                //Add to the if body
                let mut ifRet: bool = false;
                let bodyStmt = *body.clone();
                match bodyStmt.clone(){
                    Stmt::Block(stmtVec, lineNum) => {
                        for stmt in stmtVec.clone(){
                            println!("COMPILING IF STATEMENT");
                            match stmt.clone(){
                                Stmt::Return(val, lineNum) => {
                                    println!("IF RETURN");
                                    let checkedIfBody = self.compileStmt(stmt.clone(), builder, localTable, function);
                                    if checkedIfBody{
                                        //continue
                                    } else {
                                        println!("Error building if body");
                                        panic!();
                                    }
                                    ifRet = true;
                                    break;
                                }
                                _ => {
                                    println!("Not if return");
                                    let checkedIfBody = self.compileStmt(stmt.clone(), builder, localTable, function);
                                    if checkedIfBody{
                                        //continue
                                    } else {
                                        println!("Error building if body");
                                        panic!();
                                    }
                                    ifRet = false;
                                }
                            }
                        }
                    }
                    _ => {
                        panic!("If body must be a block");
                    }
                }
                    
                if !ifRet{
                    let _ = builder.build_unconditional_branch(mergeBack);
                }

                //Move to the end of the else body
                ifBuilder.position_at_end(elseBody);

                let mut elseRet: bool = false;
                //Checks if there is an else statement
                match elseStmt.clone(){
                    Some(elseVal) => {
                        let elseStmt = *elseVal.clone();
                        println!("If statement with else");
                        match elseStmt{
                            Stmt::Return(val, lineNum) => {
                                let checkedIfBody = self.compileStmt(bodyStmt.clone(), &ifBuilder, localTable, function);
                                if checkedIfBody{
                                    //continue
                                } else {
                                    println!("Error building if body");
                                    panic!();
                                }
                                elseRet = true;
                            }
                            Stmt::Block(stmtVec, lineNum) => {
                                for stmt in stmtVec.clone(){
                                    match stmt.clone(){
                                        Stmt::Return(val, lineNum) => {
                                            let checkedIfBody = self.compileStmt(bodyStmt.clone(), &ifBuilder, localTable, function);
                                            if checkedIfBody{
                                                //continue
                                            } else {
                                                println!("Error building if body");
                                                panic!();
                                            }
                                            elseRet = true;
                                        }
                                        _ => {
                                            let checkedIfBody = self.compileStmt(bodyStmt.clone(), &ifBuilder, localTable, function);
                                            if checkedIfBody{
                                                //continue
                                            } else {
                                                println!("Error building if body");
                                                panic!();
                                            }
                                            elseRet = false;
                                        }
                                    }
                                }
                            }
                            _ => {
                                let checkedIfBody = self.compileStmt(bodyStmt.clone(), &ifBuilder, localTable, function);
                                if checkedIfBody{
                                    //continue
                                } else {
                                    println!("Error building if body");
                                    panic!();
                                }
                                elseRet = false;
                            }
                        }
                    }
                    None => {
                        println!("If statement no else");
                        // let _ = builder.build_unconditional_branch(mergeBack);
                        elseRet = false;
                    }
                }

                if !elseRet {
                    let _ = builder.build_unconditional_branch(mergeBack);
                }
                    
                //Moves builder to the end of the block
                ifBuilder.position_at_end(mergeBack);

                println!("CREATED if LOOP ");

                // let _ = builder.build_call(ifFunction.clone(), &[], "ifStatementCall");
                // let _ = builder.build_unconditional_branch(ifEntry);
                return true;
                
            }
            Stmt::ProcDecl(procRetType, procName, params, headerBox, bodyBox, lineNum) => {
                println!("DECLARING A PROCEDURE");
                //Creates the local variable hash table
                let mut procLocTable: HashMap<String, PointerValue<'ctx>> = HashMap::new();
                
                //Creates the local builder
                let procBuilder = self.context.create_builder();
                
                //Creates a vec for the param types
                let mut paramTypes: Vec<BasicTypeEnum> = Vec::new();

                //Parses the params
                let paramStmtBlock = *params.clone();
                match paramStmtBlock.clone(){
                    Stmt::Block(params, lineNum) => {
                        for param in params{
                            match param.clone(){
                                Stmt::VarDecl(varName, varType, lineNum) => {
                                    let mut paramType: BasicTypeEnum;
                                    match varType{
                                        VarType::Bool => {
                                            paramType = self.context.bool_type().as_basic_type_enum().clone();
                                        }
                                        VarType::Float => {
                                            paramType = self.context.f32_type().as_basic_type_enum().clone();
                    
                                        }
                                        VarType::Int => {
                                            paramType = self.context.i32_type().as_basic_type_enum().clone();
                    
                                        }
                                        VarType::IntArray(size) => {
                                            let arrSize = size as u32;
                                            let i32Type = self.context.i32_type();
                                            let arrayType = i32Type.array_type(arrSize);
                                            
                                            
                                            paramType = self.context.i32_type().as_basic_type_enum().clone();
                    
                                        }
                                        VarType::Str => {
                                            paramType = self.context.i8_type().as_basic_type_enum().clone();
                                            
                                        }
                                    }
                                    paramTypes.push(paramType.clone());
                                    
                                }
                                _ => {
                                    println!("Function delcaration parameters can only be local variable declarations");
                                    false;
                                }
                            }
                        }
                    }
                    Stmt::VarDecl(varName, varType, lineNum) => {
                        let mut paramType: BasicTypeEnum;
                        match varType{
                            VarType::Bool => {
                                paramType = self.context.bool_type().as_basic_type_enum().clone();
                            }
                            VarType::Float => {
                                paramType = self.context.f32_type().as_basic_type_enum().clone();
        
                            }
                            VarType::Int => {
                                paramType = self.context.i32_type().as_basic_type_enum().clone();
        
                            }
                            VarType::IntArray(size) => {
                                let arrSize = size as u32;
                                let i32Type = self.context.i32_type();
                                let arrayType = i32Type.array_type(arrSize);
                                
                                
                                paramType = self.context.i32_type().as_basic_type_enum().clone();
        
                            }
                            VarType::Str => {
                                paramType = self.context.i8_type().as_basic_type_enum().clone();
                                
                            }
                        }
                        paramTypes.push(paramType.clone());
                        
                    }
                    _ => {
                        println!("Function delcaration parameters can only be local variable declarations");
                        false;
                    }
                }
                    
                println!("Created param list");

                
                //Creates the main function
                let mut procTypeEnum: BasicTypeEnum;
                match procRetType{
                    VarType::Bool => {
                        procTypeEnum = self.context.bool_type().as_basic_type_enum().clone();
                    }
                    VarType::Float => {
                        procTypeEnum = self.context.f32_type().as_basic_type_enum().clone();

                    }
                    VarType::Int => {
                        procTypeEnum = self.context.i32_type().as_basic_type_enum().clone();

                    }
                    VarType::IntArray(size) => {
                        let arrSize = size as u32;
                        let i32Type = self.context.i32_type();
                        let arrayType = i32Type.array_type(arrSize);
                        
                        
                        procTypeEnum = self.context.i32_type().as_basic_type_enum().clone();

                    }
                    VarType::Str => {
                        procTypeEnum = self.context.i8_type().as_basic_type_enum().clone();
                        
                    }
                }
                let paramTypesSlice: Vec<BasicMetadataTypeEnum> = paramTypes.iter().map(|&ty| ty.into()).collect();
                let paramTypesSlice = &paramTypesSlice[..];

                let funcType = procTypeEnum.fn_type(paramTypesSlice, false);

                let intType = self.context.i32_type().clone();
                // let procType = intType.fn_type(&[], false);

                let procFunVal = self.module.add_function(&procName.clone(), funcType, None);

                let function = procFunVal;

                //Creates the entrypoint at the procedure
                let procEntry = self.context.append_basic_block(procFunVal, "procEntry");
                procBuilder.position_at_end(procEntry);
                println!("Created entry point");
                

                let parmStmt = paramStmtBlock.clone();
                // let checkParm = self.compileStmt(parmStmt.clone(), &procBuilder, &mut procLocTable, function);
                match parmStmt{
                    Stmt::VarDecl(varName, varType, lineNum) => {
                        let params = procFunVal.get_params();
                        let paramValue = params[1];
                        let paramName = varName.clone();
                        let paramType = paramValue.get_type();
                        //Allocates space
                        let allocaRes = procBuilder.build_alloca(paramType.clone(), &paramName);
                        let paramPtr: PointerValue;
                        match allocaRes{
                            Ok(val) => {
                                paramPtr = val;
                            }
                            Err(err) => {
                                panic!("Error allocating param space {}", err);
                            }
                        }

                        //Stores the parameter
                        let _ = procBuilder.build_store(paramPtr, paramValue.clone());

                        //Adds location to the hash table
                        procLocTable.insert(paramName.clone(), paramPtr.clone());


                    }
                    Stmt::Block(stmtVec, lineNum) => {
                        let mut i = 0;
                        for paramStmt in stmtVec.clone(){
                            let curStmt = paramStmt.clone();
                            match curStmt{
                                Stmt::VarDecl(varName, varType, lineNum) => {
                                    let params = procFunVal.get_params();
                                    let paramValue = params[i];
                                    let paramName = varName.clone();
                                    let paramType = paramValue.get_type();
                                    //Allocates space
                                    let allocaRes = procBuilder.build_alloca(paramType.clone(), &paramName);
                                    let paramPtr: PointerValue;
                                    match allocaRes{
                                        Ok(val) => {
                                            paramPtr = val;
                                        }
                                        Err(err) => {
                                            panic!("Error allocating param space {}", err);
                                        }
                                    }

                                    //Stores the parameter
                                    let _ = procBuilder.build_store(paramPtr, paramValue.clone());

                                    //Adds location to the hash table
                                    procLocTable.insert(paramName.clone(), paramPtr.clone());


                                }
                                _ => {
                                    panic!("Parameters must be variable declaration or block");
                                }
                            }
                            i += 1;
                        }
                    }
                    _ => {
                        panic!("Parameters must be variable declaration or block");
                    }
                }

                procBuilder.position_at_end(procEntry);

                //Goes through the header and adds each line to the module
                let header = headerBox.clone();
                let mut procHeader = *header;
                // Check if the variable is a Block and iterate through it
                if let Stmt::Block(ref instrs, lineNum) = procHeader.clone() {
                    for instr in instrs {
                        self.compileStmt(instr.clone(), &procBuilder, &mut procLocTable, function);
                    }
                } else {
                    println!("Problem with procedure AST: header must be a Block");
                    panic!();
                }

                println!("procedure Header processed");

                let procBody = self.context.append_basic_block(procFunVal, "procBody");

                let _ = procBuilder.build_unconditional_branch(procBody);


                procBuilder.position_at_end(procBody);

                println!("Time to go through body");
                //Goes through the body and adds each line to the module
                let newBodyBox = bodyBox.clone();
                let mut body = *newBodyBox;

                // Check if the variable is a Block and iterate through it
                if let Stmt::Block(ref instrs, lineNum) = body.clone() {
                    for instr in instrs {
                        println!("Proc expressions");
                        let good = self.compileStmt(instr.clone(), &procBuilder, &mut procLocTable, function);
                    }
                } else {
                    println!("Problem with proc AST: body must be a Block");
                    panic!();
                }
                
                println!("Procedure created");
                
                return true;
             
            }
            Stmt::StringLiteral(str, lineNum) => {
                println!("StringLiteral Stmt, this should never happe");
                return true;
            }
            Stmt::Return(valueExpr, lineNum) => {
                let retValExpr = valueExpr.clone();
                if let Expr::VarRef(varName) = retValExpr.clone(){
                    println!("RETURN EXPRESSION");
                    if varName.clone() == ""{
                        let _ = builder.build_return(None);
                        return true;
                    } else {
                        let exprCheck = self.compileExpr(&retValExpr.clone(), builder, localTable);
                        match exprCheck {
                            Ok(val) => {
                                match val {
                                    BasicValueEnum::IntValue(int_val) => {
                                        let _ = builder.build_return(Some(&int_val));
                                    }
                                    BasicValueEnum::FloatValue(float_val) => {
                                        let _ = builder.build_return(Some(&float_val));
                                    }
                                    BasicValueEnum::PointerValue(ptr_val) => {
                                        let _ = builder.build_return(Some(&ptr_val));
                                    }
                                    BasicValueEnum::ArrayValue(array_val) => {
                                        let _ = builder.build_return(Some(&array_val));
                                    }
                                    BasicValueEnum::StructValue(struct_val) => {
                                        let _ = builder.build_return(Some(&struct_val));
                                    }
                                    BasicValueEnum::VectorValue(vector_val) => {
                                        let _ = builder.build_return(Some(&vector_val));
                                    }
                                }
                                return true;
                            }
                            Err(e) => {
                                // Handle the error case
                                println!("Failed get return value: {}", e);
                                panic!();
                            }
                        }
                        
                    }

                }
                else {
                    let exprCheck = self.compileExpr(&retValExpr.clone(), builder, localTable);
                        match exprCheck {
                            Ok(val) => {
                                match val {
                                    BasicValueEnum::IntValue(int_val) => {
                                        let _ = builder.build_return(Some(&int_val));
                                    }
                                    BasicValueEnum::FloatValue(float_val) => {
                                        let _ = builder.build_return(Some(&float_val));
                                    }
                                    BasicValueEnum::PointerValue(ptr_val) => {
                                        let _ = builder.build_return(Some(&ptr_val));
                                    }
                                    BasicValueEnum::ArrayValue(array_val) => {
                                        let _ = builder.build_return(Some(&array_val));
                                    }
                                    BasicValueEnum::StructValue(struct_val) => {
                                        let _ = builder.build_return(Some(&struct_val));
                                    }
                                    BasicValueEnum::VectorValue(vector_val) => {
                                        let _ = builder.build_return(Some(&vector_val));
                                    }
                                }
                                return true;
                            }
                            Err(e) => {
                                // Handle the error case
                                println!("Failed get return value: {}", e);
                                panic!();
                            }
                        }
                }
                
            }
            Stmt::Program(name, headerBox, bodyBox, lineNum) => {
                println!("Program Stmt, this should never happen");
                return true;
            }
            
        }
        
    }

        
    fn compileExpr(&mut self, expr: &Expr, builder: &Builder<'ctx>, localTable: &mut HashMap<String, PointerValue<'ctx>>) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expr::IntLiteral(value) => {
                let val = value.clone() as u64;
                let intType = self.context.i32_type().clone();
                let intVal = intType.const_int(val, false);
                return Ok(BasicValueEnum::IntValue(intVal));
            }
                
            Expr::FloatLiteral(value) => {
                // let val = value.clone() as f32;
                let floatType = self.context.f32_type().clone();
                let floatVal = floatType.const_float(value.clone().into());
                return Ok(BasicValueEnum::FloatValue(floatVal.clone()));
            }
            
            Expr::StringLiteral(string) => {
                let stringBytes = string.as_bytes();
    
    
                let arrayVal = self.context.const_string(stringBytes, false).clone();
    
    
                // Wrap the array constant in a BasicValueEnum
                let basicArrayVal = BasicValueEnum::ArrayValue(arrayVal);
                return Ok(basicArrayVal.clone());
    
            }
            Expr::IntArrayLiteral(size, values) => {
                // let byte_vec = string.as_bytes().to_vec();
                // let max_len = 65 as u32;
                // let i8_type = self.context.i8_type(); // Define the element type (8-bit integer for characters)
                // let array_type = i8_type.array_type(max_len); // Define the array type with the maximum length
    
                
    
                // return basic_value_enum;
                println!("intarray NEEDS WRITTEN");
                let i32_type = self.context.i32_type();
                let intValue = i32_type.const_int(0, false);                
                return Ok(BasicValueEnum::IntValue(intValue));
            }
            Expr::BoolLiteral(boolVal) => {
                let boolType = self.context.custom_width_int_type(1).clone();
                let trueVal = BasicValueEnum::IntValue(boolType.const_int(1, false));
                let falseVal = BasicValueEnum::IntValue(boolType.const_int(0, false));
                match boolVal{
                    true => {
                        return Ok(trueVal);
                    }
                    false => {
                        return Ok(falseVal);
                    }
                }
            }
            
            Expr::VarRef(varName) => {
                //Gets the type if defined in local scope
                let checkLocVar = localTable.get(&varName.clone());
                match checkLocVar{
                    Some(varPtr) => {
                        println!("Loading local value {} at location {}", varName.clone(), varPtr.clone());
                        let loadedVal = builder.build_load(varPtr.clone(), &varName.clone());
                        match loadedVal{
                            Ok(val) => {
                                return Ok(val.clone());
                            }
                            Err(err) => {
                                panic!("{}", format!("Error with pointer to value {}", varName.clone()));
                            }
                        }
                    }
                    None => {
                        let checkGlobVar = self.globalTable.get(&varName.clone());
                            match checkGlobVar{
                                Some(varPtr) => {
                                    println!("Loading local value {} at location {}", varName.clone(), varPtr.clone());
                                    
                                    let loadedVal = builder.build_load(varPtr.clone(), &varName.clone());
                                    match loadedVal{
                                        Ok(val) => {
                                            return Ok(val.clone());
                                        }
                                        Err(err) => {
                                            panic!("{}", format!("FFFError with pointer to value {}", varName.clone()));
                                        }
                                    }
                                }
                                None => {
                                    let errMsg = format!("Variable {} is not defined", varName.clone());
                                    panic!("{}", errMsg);
                                }
                            }
                    }
                }
                
            }
            Expr::ArrayRef(name, indexExpr) => {
                println!("array reference");
                let targName = name.clone();
                let arrSize = 64 as u32;
                let i32Type = self.context.i32_type().clone();
                let arrayType = i32Type.array_type(arrSize).clone();
                
                //Gets the value of the index expression
                let indexExprCheck = self.compileExpr(&*&indexExpr.clone(), builder, localTable);
                let mut indexVal: BasicValueEnum;
                match indexExprCheck{
                    Ok(val) => {
                        indexVal = val.clone();
                    }
                    Err(err) => {
                        println!("{}", err.clone());
                        let errMsg = format!("Could error with index {}", err.clone());
                        panic!("{}", errMsg.clone());
                    }
                }
    
                
                let mut arrayPtr: PointerValue;
                let checkLocVar = localTable.get(&targName.clone());
                match checkLocVar{
                    Some(ptr) => {
                        println!("getting local array {} at location {}", targName.clone(), ptr.clone());
                        arrayPtr = ptr.clone();
                    }
                    None => {
                        let checkGlobVar = self.globalTable.get(&targName.clone());
                        match checkGlobVar{
                            Some(ptr) => {
                                println!("Gettting global array index  {} at location {}", targName.clone(), ptr.clone());
                                arrayPtr = ptr.clone();
                            }
                            None => {
                                let errMsg = format!("variable {} not found", targName.clone());
                                panic!("{}", errMsg.clone());
                            }
                        }
                    }
                }
    
                let mut indexInt: IntValue;
                match indexVal{
                    BasicValueEnum::IntValue(val) => {
                        indexInt = val.clone();
                    }
                    BasicValueEnum::FloatValue(val) => {
                        let intType = self.context.i32_type().clone();
                        let intValue = builder.build_float_to_signed_int(val.clone(), intType, "float_to_int");
                        match intValue{
                            Ok(iVal) => {
                                indexInt = iVal.clone();
                            }
                            Err(err) => {
                                let errMsg = format!("Error converting float to int");
                                panic!("{}", errMsg.clone());
                            }
                        }
    
                    }
                    _ => {
                        let errMsg = format!("Can only index by integer");
                        panic!("{}", errMsg.clone());
                    }
                }
            
                // Get the pointer to the desired index
                let variablePtr: PointerValue;
                let intType = self.context.i32_type().clone();
                let zero = intType.const_int(0, false);
                let indexList = [zero, indexInt];
                let checkIndexPtr = unsafe { builder.build_gep(arrayPtr, &indexList, "arrayIndexLoad") };
                match checkIndexPtr{
                    Ok(ptr) => {
                        variablePtr = ptr.clone();
                    }
                    Err(err) => {
                        let errMsg = format!("Error getting array index ptr");
                        panic!("{}", errMsg);
                    }
                }
    
                //Gets the value at that pointer
                let retValCheck = builder.build_load(variablePtr, "arrayIndexReference");
                match retValCheck{
                    Ok(val) => {
                        // match val{
                        //     BasicValueEnum::IntValue(val) => {
                        //         println!("GOT INT FROM ARRAY");
                        //     }
                        //     BasicValueEnum::ArrayValue(val) => {
                        //         println!("GOT ARRAY TYPE");
                        //     }
                        //     _ => {
                        //         println!("GOT NOT INT FROM ARRAY");
                        //     }
                        // }
                        return Ok(val.clone());
                    }
                    Err(msg) => {
                        let errMsg = format!("Error getting array index value");
                        panic!("{}", errMsg.clone());
                    }
                }
                // let elementPtr = builder.
                
            }
    
            Expr::ArthOp(op1, op, op2) => {
                // let context = &mut self.context;
                // let builder = &mut builder;
                
                let intType = self.context.i32_type().clone();
                let floatType = self.context.f32_type().clone();
    
                //First gets the values of both operands
                let op1Res = self.compileExpr(&*op1.clone(), builder, localTable).clone();
                let op2Res = self.compileExpr(&*op2.clone(), builder, localTable).clone();
                let mut op1Val: BasicValueEnum;
                let mut op2Val: BasicValueEnum;
                //Makes sure both results of checked operands are good
                match op1Res.clone(){
                    Ok(res) => {
                        op1Val = res.clone();
                    }
                    Err(msg) => {
                        panic!("{}", msg.clone());
                    }
                }
                match op2Res.clone(){
                    Ok(res) => {
                        op2Val = res.clone();
                    }
                    Err(msg) => {
                        panic!("{}", msg.clone());
                    }
                }
    
                //Checks if either value is a float
                let op1IsFloat = match op1Val.clone(){
                    BasicValueEnum::FloatValue(_) => true,
                    _ => false,
                };
                let op2IsFloat = match op2Val.clone(){
                    BasicValueEnum::FloatValue(_) => true,
                    _ => false,
                };
    
                //a match case to handle the different types of operators
                match op.clone(){
                    Operator::Add => {
                        //If either result is a float
                        if op1IsFloat.clone() || op2IsFloat.clone() {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val.clone() {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val.clone();
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val.clone() {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val.clone();
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val.clone();
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float add
                            let retOp = builder.build_float_add(op1Float, op2Float, "addFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::FloatValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_int_add(op1Int.clone(), op2Int.clone(), "addInt");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    Operator::Sub => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val.clone();
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float add
                            let retOp = builder.build_float_sub(op1Float, op2Float, "subFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::FloatValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_int_sub(op1Int.clone(), op2Int.clone(), "subInt");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    } 
                    Operator::Mul => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float add
                            let retOp = builder.build_float_mul(op1Float, op2Float, "multiplyFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::FloatValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.clone().into_int_value();
                            let op2Int = op2Val.clone().into_int_value();
                            let retOp = builder.build_int_mul(op1Int.clone(), op2Int.clone(), "multiplyInt");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    Operator::Div => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float add
                            let retOp = builder.build_float_div(op1Float, op2Float, "divideFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::FloatValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_int_signed_div(op1Int.clone(), op2Int.clone(), "divideInt");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    _ => {
                        //This should never happen because of parsing and error checking
                        panic!("Improper operator for arthimatic operation");
                    }
                }
            
            }
            Expr::RelOp(op1, op, op2) => {   
                
                //First gets the values of both operands
                let op1Res = self.compileExpr(&*op1.clone(), builder, localTable);
                let op2Res = self.compileExpr(&*op2.clone(), builder, localTable);
                let mut op1Val: BasicValueEnum;
                let mut op2Val: BasicValueEnum;
                //Makes sure both results of checked operands are good
                match op1Res{
                    Ok(res) => {
                        op1Val = res.clone();
                    }
                    Err(msg) => {
                        panic!("{}", msg.clone());
                    }
                }
                match op2Res{
                    Ok(res) => {
                        op2Val = res.clone();
                    }
                    Err(msg) => {
                        panic!("{}", msg.clone());
                    }
                }
    
                //Checks if either value is a float
                let op1IsFloat = match op1Val.clone(){
                    BasicValueEnum::FloatValue(_) => true,
                    _ => false,
                };
                let op2IsFloat = match op2Val.clone(){
                    BasicValueEnum::FloatValue(_) => true,
                    _ => false,
                };
    
                //a match case to handle the different types of operators
                match op{
                    Operator::Check_Equal => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OEQ,op1Float, op2Float, "equalFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_int_compare(IntPredicate::EQ,op1Int, op2Int, "equalInt");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    Operator::Greater => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for greater"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OGT,op1Float, op2Float, "floatGreater");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_int_compare(IntPredicate::SGT,op1Int, op2Int, "intGreater");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    Operator::Greater_Equal => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OGE,op1Float, op2Float, "floatGreaterEqual");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_int_compare(IntPredicate::SGE,op1Int, op2Int, "intGreaterEqual");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    Operator::Less => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OLT,op1Float, op2Float, "floatLess");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_int_compare(IntPredicate::SLT,op1Int, op2Int, "intLess");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    Operator::Less_Equal => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OLE,op1Float, op2Float, "floatLessEqual");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_int_compare(IntPredicate::SLE,op1Int, op2Int, "intLessEqual");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    Operator::Not_Equals => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to float if not
                            let op1Float: FloatValue;
                            match op1Val {
                                BasicValueEnum::FloatValue(val) => op1Float = val,
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for not equal"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::ONE,op1Float, op2Float, "floatNotEqual");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_int_compare(IntPredicate::NE,op1Int, op2Int, "intNotEqual");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    
                    _ => {
                        //This should never happen because of parsing and error checking
                        panic!("Improper operator for logical operation");
                    }
                }
            
            }
            Expr::LogOp(op1, op, op2) => {

                
                //First gets the values of both operands
                let op1Res = self.compileExpr(&*op1.clone(), builder, localTable);
                let op2Res = self.compileExpr(&*op2.clone(), builder, localTable);
                let mut op1Val: BasicValueEnum;
                let mut op2Val: BasicValueEnum;
                //Makes sure both results of checked operands are good
                match op1Res{
                    Ok(res) => {
                        op1Val = res;
                    }
                    Err(msg) => {
                        panic!("{}", msg.clone());
                    }
                }
                match op2Res{
                    Ok(res) => {
                        op2Val = res;
                    }
                    Err(msg) => {
                        panic!("{}", msg.clone());
                    }
                }
    
                //Checks if either value is a float
                let op1IsFloat = match op1Val{
                    BasicValueEnum::FloatValue(_) => true,
                    _ => false,
                };
                let op2IsFloat = match op2Val{
                    BasicValueEnum::FloatValue(_) => true,
                    _ => false,
                };
    
                //a match case to handle the different types of operators
                match op{
                    Operator::And => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to int if not
                            let op1Int: IntValue;
                            match op1Val {
                                BasicValueEnum::IntValue(val) => op1Int = val,
                                BasicValueEnum::FloatValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Int = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Int: IntValue;
                            match op2Val {
                                BasicValueEnum::IntValue(val) => op2Int = val,
                                BasicValueEnum::FloatValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Int = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            let retOp = builder.build_and(op1Int, op2Int, "intAnd");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
    
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_and(op1Int, op2Int, "intAnd");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                        
                    }
                    Operator::Or => {
                        //If either result is a float
                        if op1IsFloat || op2IsFloat {
                            
                            //Checks if op1 is float, casts it to int if not
                            let op1Int: IntValue;
                            match op1Val {
                                BasicValueEnum::IntValue(val) => op1Int = val,
                                BasicValueEnum::FloatValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Int = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Int: IntValue;
                            match op2Val {
                                BasicValueEnum::IntValue(val) => op2Int = val,
                                BasicValueEnum::FloatValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Int = val;
                                        }
                                        Err(errMsg) => {
                                            panic!("{}", format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => panic!("Unsupported type for addition"),
                            };
    
                            let retOp = builder.build_or(op1Int, op2Int, "intOr");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
    
                        } 
                        // Both operands are integers
                        else {
                            let op1Int = op1Val.into_int_value();
                            let op2Int = op2Val.into_int_value();
                            let retOp = builder.build_or(op1Int, op2Int, "intOr");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    panic!("{}", format!("{}", errMsg));
                                }
                            }
                            
                        }
                        
                    }
                    
                    _ => {
                        //This should never happen because of parsing and error checking
                        panic!("Improper operator for logical operation");
                    }
                }
                
            }
        
            Expr::ProcRef(procName, params) => {
                // self.scope += 1;
                
                //Get the function
                let mut function: FunctionValue;
                let functionCheck = self.module.get_function(&procName.clone());
                match functionCheck{
                    Some(fun) => {
                        function = fun.clone();
                    }
                    None => {
                        let errMsg = format!("Function: {} not found", procName.clone());
                        panic!("{}", errMsg);
                    }
                }

                //Compile arguments
                let mut compiledParams: Vec<BasicValueEnum> = Vec::new();
                if let Some(paramExprs) = params.clone(){
                    for param in paramExprs{
                        let paramCheck = self.compileExpr(&param.clone(), builder, localTable);
                        match paramCheck{
                            Ok(val) => {
                                compiledParams.push(val.clone());
                            }
                            Err(err) => {
                                let errMsg = format!("Error parsing function call param: {}", err.clone());
                                panic!("{}", errMsg.clone());
                            }
                        }
                    }
                }

                //COnvert params to correct type
                let metadata_values: Vec<BasicMetadataValueEnum> = compiledParams.into_iter()
                    .map(|val| val.into())
                    .collect();

                // Convert `Vec<BasicMetadataValueEnum>` to a slice
                let parmVals = metadata_values.as_slice();

                //Create the call
                let procCallRes = builder.build_call(function, parmVals, "callProc");
                match procCallRes{
                    Ok(val) => {
                        let retVal = val.try_as_basic_value().left().unwrap();
                        return Ok(retVal.clone());
                    }
                    Err(err) => {
                        let errMsg = format!("Error calling procedure");
                        panic!("{}", errMsg);
                    }
                }

           
            }
        }
    
    
    
    }

    fn definePutInt(&mut self) {
        let intType = self.context.i32_type();
        let retType = self.context.bool_type();
        let paramTypes = vec![BasicMetadataTypeEnum::from(intType)];
        let parmVals = paramTypes.as_slice();
        let printFnType = retType.fn_type(parmVals, false);
        let putInt = self.module.add_function("putinteger", printFnType, None);        
    }
    fn definePutBool(&mut self) {
        let boolType = self.context.bool_type();
        let retType = self.context.bool_type();
        let paramTypes = vec![BasicMetadataTypeEnum::from(boolType)];
        let parmVals = paramTypes.as_slice();
        let printFnType = retType.fn_type(parmVals, false);
        let putInt = self.module.add_function("putbool", printFnType, None);        
    }
    fn definePutFloat(&mut self) {
        let floatType = self.context.f32_type();
        let retType = self.context.bool_type();
        let paramTypes = vec![BasicMetadataTypeEnum::from(floatType)];
        let parmVals = paramTypes.as_slice();
        let printFnType = retType.fn_type(parmVals, false);
        let putInt = self.module.add_function("putfloat", printFnType, None);        
    }
    fn definePutStr(&mut self) {
        let i8_type = self.context.i8_type();
        let array_type = i8_type.array_type(65); // Assuming the array size is 65
        let string_type = array_type.ptr_type(AddressSpace::default());
        let retType = self.context.bool_type();
        let paramTypes = vec![BasicMetadataTypeEnum::from(string_type)];
        let parmVals = paramTypes.as_slice();
        let printFnType = retType.fn_type(parmVals, false);
        let putInt = self.module.add_function("putfloat", printFnType, None);
    }


    fn defineGetInt(&mut self) {
        let intType = self.context.i32_type();
        // let retType = self.context.bool_type();
        let paramTypes = vec![];
        let parmVals = paramTypes.as_slice();
        let getIntType = intType.fn_type(parmVals, false);
        let putInt = self.module.add_function("getinteger", getIntType, None);
    }


    fn defineGetFloat(&mut self) {
        let intType = self.context.f32_type();
        // let retType = self.context.bool_type();
        let paramTypes = vec![];
        let parmVals = paramTypes.as_slice();
        let getIntType = intType.fn_type(parmVals, false);
        let putInt = self.module.add_function("getfloat", getIntType, None);
    }

}

#[no_mangle]
pub extern "C" fn putinteger(val: i32) -> bool {
    println!("{}", val);
    return true;
}
