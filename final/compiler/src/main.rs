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
    }}, anyhow::Result, inkwell::{builder::Builder, context::Context, module::Module, types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum}, values::*, AddressSpace, FloatPredicate, IntPredicate}, parse_display::Display, std::{
        collections::HashMap, env::{self, args}, ffi::CString, fmt, rc::Rc
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
}

impl<'ctx> Compiler<'ctx> {
    // Initialize a new IRGen instance
    // The constructor
    pub fn new(
        programAst: Stmt,
        context: &'ctx Context,
        globalTable: &'ctx mut HashMap<String, PointerValue<'ctx>>,

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
        }
    }

    pub fn compileProgram(&mut self) -> Result<&Module<'ctx>, String>{
        match self.programAst.clone(){
            Stmt::Program(progName, headerBox, bodyBox, lineNum) => {
                //Adds the built ints
                self.addScanf();
                self.addPrintf();
                self.declareGetInt();
                self.declareGetBool();
                self.declareGetFloat();
                self.declareGetString();
                self.declarePutBool();
                self.declarePutFloat();
                self.declarePutInt();
                self.declarePutString();
                // self.declareSqrt();
                
                
                //Creates the main function
                let i32Type = self.context.i32_type();
                let mainType = i32Type.fn_type(&[], false);

                let mainFunc = self.module.add_function("main", mainType, None);

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
                        self.compileStmt(instr.clone(), &mainBuilder, &mut mainLocalTable);
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
                        let good = self.compileStmt(instr.clone(), &mainBuilder, &mut mainLocalTable);
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

    fn compileStmt(&mut self, stmt: Stmt, builder: &Builder<'ctx>, localTable: &mut HashMap<String, PointerValue<'ctx>>) -> bool{
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
                                return false;
                            }
                        }
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
                                return false;
                            }
                        }
                        localTable.insert(varName.clone(), localPtr);
                        
                        return true;
                    }
                    VarType::Int => {
                        let localType = self.context.i64_type();
                        let localName = varName.clone();
                        
                        
                        // let globVar = self.module.add_global(boolType.clone(), None, &boolName);
                        
                        let localVarCheck = builder.build_alloca(localType.clone(), &localName.clone());

                        let localPtr: PointerValue;
                        match localVarCheck{
                            Ok(ptr) => {
                                localPtr = ptr.clone();
                            }
                            Err(err) => {
                                println!("Error allocating local int variable {}: {}", localName.clone(), err.to_string());
                                return false;
                            }
                        }
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
                                return false;
                            }
                        }
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
                                return false;
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
                        
                        let globPtr = globVar.as_pointer_value();
                        self.globalTable.insert(varName.clone(), globPtr);
                        
                        return true;
                    }
                    VarType::Float => {
                        let varType = self.context.f32_type();
                        let globName = varName.clone();
                        let globVar = self.module.add_global(varType.clone(), None, &globName);
                        
                        let globPtr = globVar.as_pointer_value();
                        self.globalTable.insert(varName.clone(), globPtr);
                        
                        return true;
                    }
                    VarType::Int => {
                        let varType = self.context.i64_type();
                        let globName = varName.clone();
                        let globVar = self.module.add_global(varType.clone(), None, &globName);
                        let globPtr = globVar.as_pointer_value();
                        self.globalTable.insert(varName.clone(), globPtr);
                        
                        return true;
                    }
                    VarType::Str => {
                        let maxStringLen = 64 as u32 + 1;
                        let i8Type = self.context.i8_type();
                        let arrayType = i8Type.array_type(maxStringLen);
                        // let stringVal: Vec<IntValue> = 

                        
                        // let varType = self.context.f64_type();
                        let globName = varName.clone();
                        let globVar = self.module.add_global(arrayType.clone(), None, &globName);
                        let globPtr = globVar.as_pointer_value();
                        self.globalTable.insert(varName.clone(), globPtr);
                        
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
                                    return false;
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
                            return false;
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
                                    return false;
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
                                    return false;
                                }
                            }

                        }
                        _ => {
                            println!("Can only index by integer");
                            return false;
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
                            return false;
                        }
                    }
                    // let elementPtr = builder.
                
                }
                
                else {
                    println!("Cannot assing to a non variable");
                    return false;
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
                            return false;
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
                                    return false;
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
                                    return false;
                                }
                            }

                        }
                        _ => {
                            println!("Can only index by integer");
                            return false;
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
                            return false;
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
                            return false;
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
                            return false;
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
                    let good = self.compileStmt(instr.clone(), builder, localTable);
                    if (!good){
                        println!("Error in block:");
                        instr.display(0);
                        return false;
                    } else {
                        //continue
                    }
                }
                return true;
            }
            Stmt::Error(err, lineNum) => {
                println!("Somehow an error made it to the compiler");
                return false;
            }
            Stmt::Expr(exprStmt, lineNum) => {
                // println!("ExprStmt needs written");
                // return true;
                match (exprStmt.clone()){
                    _ => {
                        let checked = self.compileExpr(&exprStmt.clone(), builder, localTable);
                        match checked {
                            Ok(val) => {
                                println!("SOMETHING SHOULD BE DONE HERE, IDK");
                                return true;
                            }
                            Err(err) => {
                                println!("Error: {}", err.clone());
                                return false;
                            }
                        }
                    }
                }
            }
            Stmt::For(assignment, condExpr, body, lineNum) => {
                //Parses the assignment first
                let mut iInitVal: BasicValueEnum;
                let mut iName: String;
                let assignStmt = Rc::clone(&assignment);
                if let Stmt::Assign(varRef, val, lineNum) = &*assignStmt.clone() {
                    if let Expr::VarRef(varName) = varRef.clone(){
                        println!("for loop variable i {}", varName.clone());
                        iName = varName.clone();
                        let iteratorValCheck = self.compileExpr(&val.clone(), builder, localTable);
                        match iteratorValCheck{
                            Ok(val) => {
                                println!("Iterator value: {}", val.clone());
                                iInitVal = val;
                            }
                            Err(err) => {
                                println!("Error parsing for loop iterator assignment: {}", err.clone());
                                return false;
                            }
                        }
                    }
                    else {
                        println!("Error: For loop iterator must be a variable");
                        return false;
                    }
                }
                else {
                    println!("Error: For loop assignment must be a variable assignment");
                    return false
                }
                
                
                let intType = self.context.i32_type().clone();
                let fnType = intType.fn_type(&[], false);

                //set up the loop "function"
                let loopFunction = self.module.add_function("forLoop", fnType, None);
                let entry = self.context.append_basic_block(loopFunction, "ForEntry");
                let loopCond = self.context.append_basic_block(loopFunction, "forCond");
                let loopBody = self.context.append_basic_block(loopFunction, "forBody");
                let loopEnd = self.context.append_basic_block(loopFunction, "forEnd");
                let forBuilder = self.context.create_builder();
                
                // self.builder = b

                forBuilder.position_at_end(entry);


                //Initialize the iterator "i"
                let mut iPtr: PointerValue;
                let iPtrCheck = forBuilder.build_alloca(intType, &iName.clone());
                match iPtrCheck{
                    Ok(val) => {
                        iPtr = val.clone();
                    }
                    Err(err) => {
                        println!("Error allocating iterator pointer");
                        return false;
                    }
                }

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
                            return false;
                        }
                    }
                    match op1.clone(){
                        Expr::VarRef(varName) => {
                            println!("For loop condition op 1 variable: {}", varName.clone());
                            println!("iName: {}", iName.clone());
                            if varName.clone() == iName.clone(){
                                println!("They the same");
                                let iValCheck = forBuilder.build_load(iPtr, &iName.clone());
                                // let mut iVal: BasicValueEnum;
                                match iValCheck{
                                    Ok(val) => {
                                        condOp1Val = val.clone();
                                    }
                                    Err(err) => {
                                        println!("Error getting iterator value");
                                        return false;
                                    }
                                }
                            } else {
                                let valCheck = self.compileExpr(&op1.clone(), builder, localTable);
                                match valCheck{
                                    Ok(val) => {
                                        condOp1Val = val.clone();
                                    }
                                    Err(err) => {
                                        println!("Error getting for condition variable value");
                                        return false;
                                    }
                                }
                            }
                        }
                        _ => {
                            //First gets the values of both operands
                            let op1Res = self.compileExpr(&op1.clone(), builder, localTable);
                            // let mut op1Val: BasicValueEnum;
                            //Makes sure both results of checked operands are good
                            match op1Res{
                                Ok(res) => {
                                    condOp1Val = res;
                                }
                                Err(msg) => {
                                    println!("Error in for loop condition");
                                    return false;
                                }
                            }
                        }
                    }
                    match op2.clone(){
                        Expr::VarRef(varName) => {
                            println!("For loop condition op 2 variable: {}", varName.clone());
                            
                            if varName == iName.clone(){
                                let iValCheck = forBuilder.build_load(iPtr, &iName.clone());
                                // let mut iVal: BasicValueEnum;
                                match iValCheck{
                                    Ok(val) => {
                                        condOp2Val = val.clone();
                                    }
                                    Err(err) => {
                                        println!("Error getting iterator value");
                                        return false;
                                    }
                                }
                            } else {
                                println!("Not the same");
                                let valCheck = self.compileExpr(&op2.clone(), builder, localTable);
                                match valCheck{
                                    Ok(val) => {
                                        condOp2Val = val.clone();
                                    }
                                    Err(err) => {
                                        println!("Error getting for condition variable value {}: {}", varName.clone(), err.clone());
                                        return false;
                                    }
                                }
                            }
                        }
                        _ => {
                            //First gets the values of both operands
                            let op2Res = self.compileExpr(&op1.clone(), builder, localTable);
                            // let mut op1Val: BasicValueEnum;
                            //Makes sure both results of checked operands are good
                            match op2Res{
                                Ok(res) => {
                                    condOp2Val = res;
                                }
                                Err(msg) => {
                                    println!("Error in for loop condition");
                                    return false;
                                }
                            }
                        }
                    }


                } else {
                
                    println!("For loop condition must be a logical operation");
                    return false;
                }

                //Sets up the conditional
                let _ = forBuilder.build_store(iPtr, iInitVal.clone());
                let _ = forBuilder.build_unconditional_branch(loopCond);
                
                //Loop condition block
                let _ = forBuilder.position_at_end(loopCond);

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
                                return false;
                            }
                        }
                    }
                    _ => {
                        println!("For loop condition values must be numbers");
                        return false;
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
                                return false;
                            }
                        }
                    }
                    _ => {
                        println!("For loop condition values must be numbers");
                        return false;
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
                        return false;
                    }
                }

                let _ = forBuilder.build_conditional_branch(condition, loopBody, loopEnd);

                forBuilder.position_at_end(loopBody);

                //Populates the body with statements
                let bodyStmt = *body.clone();
                self.compileStmt(bodyStmt.clone(), &forBuilder, localTable);

                //Adds a conditional check to the end
                let _ = forBuilder.build_unconditional_branch(loopCond);

                //Moves builder to the end of the block
                forBuilder.position_at_end(loopEnd);
                let _ = forBuilder.build_return(None);

                println!("CREATED FOR LOOP ");

                let _ = builder.build_call(loopFunction.clone(), &[], "forLoopCall");
                println!("Inserted for loop");
                return true;
            }
            Stmt::If(condExpr, body, elseStmt, lineNum) => {
                //Sets up the function stuff
                let intType = self.context.i32_type().clone();
                let fnType = intType.fn_type(&[], false);
                let ifFunction = self.module.add_function("if", fnType, None);
                let ifEntry = self.context.append_basic_block(ifFunction, "ifEntry");
                // let ifCond = self.context.append_basic_block(ifFunction, "ifCondition");
                let ifBody = self.context.append_basic_block(ifFunction, "ifBody");
                let elseBody = self.context.append_basic_block(ifFunction, "elseBody");
                // let ifEnd = self.context.append_basic_block(ifFunction, "forEnd");
                let ifBuilder = self.context.create_builder();
                

                //Position at the beginning of the if statement
                ifBuilder.position_at_end(ifEntry);

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
                            return false;
                        }
                    }
                    

                    let op1Check = self.compileExpr(&op1.clone(), builder, localTable);
                    match op1Check{
                        Ok(val) => {
                            condOp1Val = val.clone();
                        }
                        Err(err) => {
                            println!("Error getting if condition op 1: {}", err.clone());
                            return false;
                        }
                    }
                    let op2Check = self.compileExpr(&op2.clone(), builder, localTable);
                    match op2Check{
                        Ok(val) => {
                            condOp2Val = val.clone();
                        }
                        Err(err) => {
                            println!("Error getting if condition op 2");
                            return false;
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
                    return false;
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
                                return false;
                            }
                        }
                    }
                    _ => {
                        println!("For loop condition values must be numbers");
                        return false;
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
                                return false;
                            }
                        }
                    }
                    _ => {
                        println!("For loop condition values must be numbers");
                        return false;
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
                        return false;
                    }
                }

                //Creates the condition
                let _ = ifBuilder.build_conditional_branch(condition, ifBody, elseBody);

                //Position at the end of the ifBody
                ifBuilder.position_at_end(ifBody);
                let bodyStmt = *body.clone();
                let checkedIfBody = self.compileStmt(bodyStmt.clone(), &ifBuilder, localTable);
                if checkedIfBody{
                    //continue
                } else {
                    println!("Error building if body");
                    return false;
                }

                //Move to the end of the else body
                ifBuilder.position_at_end(elseBody);


                //Checks if there is an else statement
                match elseStmt.clone(){
                    Some(elseVal) => {
                        println!("If statement with else");
                        let elseBlock = *elseVal.clone();
                        let checkedElse = self.compileStmt(elseBlock.clone(), &ifBuilder, localTable);
                        if checkedElse{
                            //continue
                        }
                        else {
                            println!("problem with else body");
                            return false;
                        }
                        //set up the loop "function"


                        

                    }
                    None => {
                        println!("If statement no else");
                    }
                }
                    
                //Moves builder to the end of the block
                ifBuilder.position_at_end(elseBody);
                let _ = ifBuilder.build_return(None);

                println!("CREATED if LOOP ");

                let _ = builder.build_call(ifFunction.clone(), &[], "ifStatementCall");
                println!("Inserted for loop");
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
                                            paramType = self.context.i64_type().as_basic_type_enum().clone();
                    
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
                                paramType = self.context.i64_type().as_basic_type_enum().clone();
        
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
                        procTypeEnum = self.context.i64_type().as_basic_type_enum().clone();

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

                //Creates the entrypoint at the procedure
                let procEntry = self.context.append_basic_block(procFunVal, "procEntry");
                procBuilder.position_at_end(procEntry);
                println!("Created entry point");
                

                let parmStmt = paramStmtBlock.clone();
                let checkParm = self.compileStmt(parmStmt.clone(), &procBuilder, &mut procLocTable);
                if checkParm{
                    //continue
                }
                else {
                    println!("Error declaring parameter vars");
                }


                //Goes through the header and adds each line to the module
                let header = headerBox.clone();
                let mut procHeader = *header;
                // Check if the variable is a Block and iterate through it
                if let Stmt::Block(ref instrs, lineNum) = procHeader.clone() {
                    for instr in instrs {
                        self.compileStmt(instr.clone(), &procBuilder, &mut procLocTable);
                    }
                } else {
                    println!("Problem with procedure AST: header must be a Block");
                    return false;
                }

                println!("procedure Header processed");

                

                println!("Time to go through body");
                //Goes through the body and adds each line to the module
                let newBodyBox = bodyBox.clone();
                let mut body = *newBodyBox;

                // Check if the variable is a Block and iterate through it
                if let Stmt::Block(ref instrs, lineNum) = body.clone() {
                    for instr in instrs {
                        let good = self.compileStmt(instr.clone(), &procBuilder, &mut procLocTable);
                    }
                } else {
                    println!("Problem with proc AST: body must be a Block");
                    return false;
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
                    if varName.clone() == "".to_string(){
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
                                return false;
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
                                return false;
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

        
    fn compileExpr(&self, expr: &Expr, builder: &Builder<'ctx>, localTable: &mut HashMap<String, PointerValue<'ctx>>) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expr::IntLiteral(value) => {
                let val = value.clone() as u64;
                let intType = self.context.i64_type().clone();
                let intVal = intType.const_int(val, false);
                return Ok(BasicValueEnum::IntValue(intVal));
            }
                
            Expr::FloatLiteral(value) => {
                // let val = value.clone() as f32;
                let floatType = self.context.f32_type().clone();
                let floatVal = floatType.const_float(value.clone());
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
                        let loadedVal = builder.build_load(varPtr.clone(), &varName.clone());
                        match loadedVal{
                            Ok(val) => {
                                return Ok(val.clone());
                            }
                            Err(err) => {
                                return Err(format!("Error with pointer to value {}", varName.clone()));
                            }
                        }
                    }
                    None => {
                        let checkGlobVar = self.globalTable.get(&varName.clone());
                            match checkGlobVar{
                                Some(varPtr) => {
                                    let loadedVal = builder.build_load(varPtr.clone(), &varName.clone());
                                    match loadedVal{
                                        Ok(val) => {
                                            return Ok(val.clone());
                                        }
                                        Err(err) => {
                                            return Err(format!("FFFError with pointer to value {}", varName.clone()));
                                        }
                                    }
                                }
                                None => {
                                    let errMsg = format!("Variable {} is not defined", varName.clone());
                                    return Err(errMsg);
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
                        return Err(errMsg.clone());
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
                                return Err(errMsg.clone());
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
                                return Err(errMsg.clone());
                            }
                        }
    
                    }
                    _ => {
                        let errMsg = format!("Can only index by integer");
                        return Err(errMsg.clone());
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
                        return Err(errMsg);
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
                        return Err(errMsg.clone());
                    }
                }
                // let elementPtr = builder.
                
            }
    
            Expr::ArthOp(op1, op, op2) => {
                // let context = &mut self.context;
                // let builder = &mut builder;
                
                let intType = self.context.i32_type().clone();
                let floatType = self.context.f64_type().clone();
    
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
                        return Err(msg.clone());
                    }
                }
                match op2Res.clone(){
                    Ok(res) => {
                        op2Val = res.clone();
                    }
                    Err(msg) => {
                        return Err(msg.clone());
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float add
                            let retOp = builder.build_float_add(op1Float, op2Float, "addFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::FloatValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float add
                            let retOp = builder.build_float_sub(op1Float, op2Float, "subFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::FloatValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float add
                            let retOp = builder.build_float_mul(op1Float, op2Float, "multiplyFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::FloatValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float add
                            let retOp = builder.build_float_div(op1Float, op2Float, "divideFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::FloatValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    _ => {
                        //This should never happen because of parsing and error checking
                        return Err("Improper operator for arthimatic operation".to_string());
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
                        return Err(msg.clone());
                    }
                }
                match op2Res{
                    Ok(res) => {
                        op2Val = res.clone();
                    }
                    Err(msg) => {
                        return Err(msg.clone());
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
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OEQ,op1Float, op2Float, "equalFloat");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
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
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for greater".to_string()),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OGT,op1Float, op2Float, "floatGreater");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
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
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OGE,op1Float, op2Float, "floatGreaterEqual");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
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
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OLT,op1Float, op2Float, "floatLess");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
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
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::OLE,op1Float, op2Float, "floatLessEqual");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
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
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op1Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for not equal".to_string()),
                            };
    
                            //Checks if op2 is float, casts it to float if not
                            let op2Float: FloatValue;
                            match op2Val {
                                BasicValueEnum::FloatValue(val) => {
                                    op2Float = val;
                                }
                                BasicValueEnum::IntValue(val) => {
                                    // Convert integer to float if necessary
                                    let resConv = builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                                    match resConv{
                                        Ok(val) => {
                                            op2Float = val;
                                        }
                                        Err(errMsg) => {
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            //Does the float equality check
                            let retOp = builder.build_float_compare(FloatPredicate::ONE,op1Float, op2Float, "floatNotEqual");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
                                }
                            }
                            
                        }
                    }
                    
                    _ => {
                        //This should never happen because of parsing and error checking
                        return Err("Improper operator for logical operation".to_string());
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
                        return Err(msg.clone());
                    }
                }
                match op2Res{
                    Ok(res) => {
                        op2Val = res;
                    }
                    Err(msg) => {
                        return Err(msg.clone());
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            let retOp = builder.build_and(op1Int, op2Int, "intAnd");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
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
                                            return Err(format!("{}", errMsg));
                                        }
                                    }
                                },
                                _ => return Err("Unsupported type for addition".to_string()),
                            };
    
                            let retOp = builder.build_or(op1Int, op2Int, "intOr");
                            match retOp{
                                Ok(result) => {
                                    return Ok(BasicValueEnum::IntValue(result.clone()));
                                }
                                Err(errMsg) => {
                                    return Err(format!("{}", errMsg));
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
                                    return Err(format!("{}", errMsg));
                                }
                            }
                            
                        }
                        
                    }
                    
                    _ => {
                        //This should never happen because of parsing and error checking
                        return Err("Improper operator for logical operation".to_string());
                    }
                }
                
            }
        
            Expr::ProcRef(procName, params) => {
                //Get the function
                let mut function: FunctionValue;
                let functionCheck = self.module.get_function(&procName.clone());
                match functionCheck{
                    Some(fun) => {
                        function = fun.clone();
                    }
                    None => {
                        let errMsg = format!("Function: {} not found", procName.clone());
                        return Err(errMsg);
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
                                return Err(errMsg.clone());
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
                        return Err(errMsg);
                    }
                }

           
            }
        }
    
    
    
    }
    
    //Defines the built ins
    fn declareGetInt(&mut self) {
        let procName = "getint";
        //Creates the local variable hash table
        let mut procLocTable: HashMap<String, PointerValue> = HashMap::new();
        
        //Creates the local builder
        let procBuilder = self.context.create_builder();
        
        //Creates a vec for the param types
        let mut paramTypes: Vec<BasicTypeEnum> = Vec::new();
        
        //Defines the return type
        let intType = self.context.i32_type().clone();
        
        let paramTypesSlice: Vec<BasicMetadataTypeEnum> = paramTypes.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice = &paramTypesSlice[..];
    
        let funcType = intType.fn_type(paramTypesSlice, false);
        let procFunVal = self.module.add_function(&procName, funcType.clone(), None);
    
        
        let builder = self.context.create_builder();
        let entry = self.context.append_basic_block(procFunVal, "entry");
        builder.position_at_end(entry);

        let scanf = self.module.get_function("scanf").unwrap();
        let formatStr = CString::new("%d").unwrap();
        let formatStr = self.context.const_string(formatStr.as_bytes(), false);
        let formatPtrCheck = builder.build_alloca(formatStr.get_type(), "format");
        let mut stringPtr: PointerValue;
        match formatPtrCheck{
            Ok(val) => {
                stringPtr = val.clone();
            }
            Err(errMsg) => {
                println!("Error allocating string in getInt");
                return;
            }
        }
        let _ = builder.build_store(stringPtr, formatStr);

        //Allocates space for the int
        let intValCheck = builder.build_alloca(intType.clone(), "int_val");
        let int_val: PointerValue;
        match intValCheck{
            Ok(val) =>{
                int_val = val.clone();
            }
            Err(err) => {
                println!("Error allocating space for int in getint");
                return;
            }
        }
        let args: &[inkwell::values::BasicMetadataValueEnum<'_>] = &[stringPtr.into()];
        let call = builder.build_call(scanf, &args, "scanf_call");

        //Returns the loaded val
        let loaded_val = builder.build_load(int_val, "loaded_val");
        match loaded_val{
            Ok(val) => {
                match val {
                    BasicValueEnum::IntValue(int_val) => {
                        let _ = procBuilder.build_return(Some(&int_val));
                    }
                    _ => {
                        println!("getBool can only take bool");
                        return;
                    }
                }
                return;
            }
            Err(err) => {
                println!("error loading int value in getint");
                return;
            }
        }
    
        
    }
    fn declareGetBool(&mut self) {
        let procName = "getbool";
        //Creates the local variable hash table
        let mut procLocTable: HashMap<String, PointerValue> = HashMap::new();
        
        //Creates the local builder
        let procBuilder = self.context.create_builder();
        
        //Creates a vec for the param types
        let mut paramTypes: Vec<BasicTypeEnum> = Vec::new();
        
        //Defines the return type
        let boolType = self.context.bool_type().clone();
        
        let paramTypesSlice: Vec<BasicMetadataTypeEnum> = paramTypes.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice = &paramTypesSlice[..];
    
        let funcType = boolType.fn_type(paramTypesSlice, false);
        let procFunVal = self.module.add_function(&procName, funcType.clone(), None);
    
        
        let builder = self.context.create_builder();
        let entry = self.context.append_basic_block(procFunVal, "entry");
        builder.position_at_end(entry);

        let scanf = self.module.get_function("scanf").unwrap();
        let formatStr = CString::new("%d").unwrap();
        let formatStr = self.context.const_string(formatStr.as_bytes(), false);
        let formatPtrCheck = builder.build_alloca(formatStr.get_type(), "format");
        let mut stringPtr: PointerValue;
        match formatPtrCheck{
            Ok(val) => {
                stringPtr = val.clone();
            }
            Err(errMsg) => {
                println!("Error allocating string in getInt");
                return;
            }
        }
        let _ = builder.build_store(stringPtr, formatStr);

        //Allocates space for the int
        let intValCheck = builder.build_alloca(boolType, "boolVal");
        let int_val: PointerValue;
        match intValCheck{
            Ok(val) =>{
                int_val = val.clone();
            }
            Err(err) => {
                println!("Error allocating space for int in getbool");
                return;
            }
        }
        let args: &[inkwell::values::BasicMetadataValueEnum<'_>] = &[stringPtr.into()];
        let call = builder.build_call(scanf, &args, "scanf_call");

        //Returns the loaded val
        let loaded_val = builder.build_load(int_val, "loaded_val");
        match loaded_val{
            Ok(val) => {
                match val {
                    BasicValueEnum::IntValue(int_val) => {
                        let _ = procBuilder.build_return(Some(&int_val));
                    }
                    _ => {
                        println!("getBool can only take bool");
                        return;
                    }
                }
                return;
            }
            Err(err) => {
                println!("error loading int value in getbool");
                return;
            }
        }
    
        
    }
    fn declareGetFloat(&mut self) {
        let procName = "getfloat";
        //Creates the local variable hash table
        let mut procLocTable: HashMap<String, PointerValue> = HashMap::new();
        
        //Creates the local builder
        let procBuilder = self.context.create_builder();
        
        //Creates a vec for the param types
        let mut paramTypes: Vec<BasicTypeEnum> = Vec::new();
        
        //Defines the return type
        let floatType = self.context.f32_type().clone();
        
        let paramTypesSlice: Vec<BasicMetadataTypeEnum> = paramTypes.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice = &paramTypesSlice[..];
    
        let funcType = floatType.fn_type(paramTypesSlice, false);
        let procFunVal = self.module.add_function(&procName, funcType.clone(), None);
    
        
        let builder = self.context.create_builder();
        let entry = self.context.append_basic_block(procFunVal, "entry");
        builder.position_at_end(entry);

        let scanf = self.module.get_function("scanf").unwrap();
        let formatStr = CString::new("%d").unwrap();
        let formatStr = self.context.const_string(formatStr.as_bytes(), false);
        let formatPtrCheck = builder.build_alloca(formatStr.get_type(), "format");
        let mut stringPtr: PointerValue;
        match formatPtrCheck{
            Ok(val) => {
                stringPtr = val.clone();
            }
            Err(errMsg) => {
                println!("Error allocating string in getInt");
                return;
            }
        }
        let _ = builder.build_store(stringPtr, formatStr);

        //Allocates space for the int
        let intValCheck = builder.build_alloca(floatType, "floatVal");
        let int_val: PointerValue;
        match intValCheck{
            Ok(val) =>{
                int_val = val.clone();
            }
            Err(err) => {
                println!("Error allocating space for float in getfloat");
                return;
            }
        }
        let args: &[inkwell::values::BasicMetadataValueEnum<'_>] = &[stringPtr.into()];
        let call = builder.build_call(scanf, &args, "scanf_call");

        //Returns the loaded val
        let loaded_val = builder.build_load(int_val, "loaded_val");
        match loaded_val{
            Ok(val) => {
                match val {
                    BasicValueEnum::FloatValue(int_val) => {
                        let _ = procBuilder.build_return(Some(&int_val));
                    }
                    _ => {
                        println!("getFloat can only take float");
                        return;
                    }
                }
                return;
            }
            Err(err) => {
                println!("error loading int value in getFloat");
                return;
            }
        }
    
        
    }
    fn declareGetString(&mut self) {
        // Define the function
        let proc_name = "getstring";
        // Create the local variable hash table
        let mut proc_loc_table: HashMap<String, PointerValue> = HashMap::new();

        // Create the local builder
        let procBuilder = self.context.create_builder();

        // Create a vec for the param types (empty in this case, as no parameters are used)
        let paramTypes: Vec<BasicTypeEnum> = Vec::new();

        // Define the return type
        let charType = self.context.i8_type(); // `i8` for chars
        let pointer_type = charType.array_type(20);

        let return_type = pointer_type;

        // Create function type
        let paramTypesSlice: Vec<BasicMetadataTypeEnum> = paramTypes.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice = &paramTypesSlice[..];


        let funcType = return_type.fn_type(&paramTypesSlice, false);
        let procFunVal = self.module.add_function(proc_name, funcType.clone(), None);

        // Create entry block and position the builder
        let entry = self.context.append_basic_block(procFunVal, "entry");
        procBuilder.position_at_end(entry);

        // Prepare for the scanf call
        let scanf = self.module.get_function("scanf").unwrap();
        let formatStr = CString::new("%s").unwrap(); // Use %s for strings
        let formatStr = self.context.const_string(formatStr.as_bytes(), false);
        let formatPtrCheck = procBuilder.build_alloca(formatStr.get_type(), "format");
        let formatPtr:PointerValue;
        match formatPtrCheck{
            Ok(val) => {
                formatPtr = val.clone();
            }
            Err(err) => {
                println!("error in allocating string space");
                return;
            }
        }
        let _ = procBuilder.build_store(formatPtr, formatStr);

        // Allocate space for the string (with a fixed-size buffer)
        let buffer_size = 256; // Or whatever size is appropriate
        let buffer = self.context.i8_type().array_type(buffer_size).const_zero();
        let bufferPtrCheck = procBuilder.build_alloca(buffer.get_type(), "buffer");
        let mut buffer_ptr: PointerValue;
        match bufferPtrCheck{
            Ok(val) => {
                buffer_ptr = val.clone();
            }
            Err(err) => {
                println!("error in getstring");
                return;
            }
        }
        let _ = procBuilder.build_store(buffer_ptr, buffer);

        // Build the scanf call
        let args: &[inkwell::values::BasicMetadataValueEnum<'_>] = &[formatPtr.into()];
        let _ = procBuilder.build_call(scanf, &args, "scanf_call");

        // Load the string value
        let loadedValCheck = procBuilder.build_load(buffer_ptr, "loaded_val");
       
       

        // Return the pointer to the string
        let return_value = BasicValueEnum::PointerValue(buffer_ptr.clone());
        let _ = procBuilder.build_return(Some(&return_value));
    
        
    }
    
    fn declarePutInt(&mut self) {
        // Define the function
        let proc_name = "putinteger";
        // Create the local variable hash table
        let mut procLocTable: HashMap<String, PointerValue> = HashMap::new();

        // Create the local builder
        let procBuilder = self.context.create_builder();

        // Create a vec for the param types
        let paramTypes: Vec<BasicTypeEnum> = vec![self.context.i32_type().into()];

        // Define the return type (boolean)
        let boolType = self.context.bool_type();
        let returnType = boolType;

        // Create function type
        // let param_types_slice: Vec<BasicTypeEnum> = param_types.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice: Vec<BasicMetadataTypeEnum> = paramTypes.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice = &paramTypesSlice[..];


        let funcType = returnType.fn_type(&paramTypesSlice, false);
        let procFunVal = self.module.add_function(proc_name, funcType.clone(), None);

        // Create entry block and position the builder
        let entry = self.context.append_basic_block(procFunVal, "entry");
        procBuilder.position_at_end(entry);

        // Prepare for the printf call
        let printf = self.module.get_function("printf").unwrap();
        let formatStr = CString::new("%d\n").unwrap(); // Format string for integer
        let formatStr = self.context.const_string(formatStr.as_bytes(), false);
        let formatPtrCheck = procBuilder.build_alloca(formatStr.get_type(), "format");
        let formatPtr:PointerValue;
        match formatPtrCheck{
            Ok(val) => {
                formatPtr = val.clone();
            }
            Err(err) => {
                println!("error in allocating string space");
                return;
            }
        }
        let _ = procBuilder.build_store(formatPtr, formatStr);

        // Get the parameter (integer value) passed to the function
        let int_param = procFunVal.get_nth_param(0).unwrap().into_int_value();

        // Build the printf call
        let args: &[inkwell::values::BasicMetadataValueEnum<'_>] = &[formatPtr.into()];
        let _ = procBuilder.build_call(printf, &args, "printf_call");

        // Return a boolean true (indicating success)
        let true_val = self.context.bool_type().const_int(1, false);
        let _ = procBuilder.build_return(Some(&true_val));
    }
    fn declarePutBool(&mut self) {
        // Define the function
        let proc_name = "putinteger";
        // Create the local variable hash table
        let mut procLocTable: HashMap<String, PointerValue> = HashMap::new();

        // Create the local builder
        let procBuilder = self.context.create_builder();

        // Create a vec for the param types
        let paramTypes: Vec<BasicTypeEnum> = vec![self.context.i32_type().into()];

        // Define the return type (boolean)
        let boolType = self.context.bool_type();
        let returnType = boolType;

        // Create function type
        // let param_types_slice: Vec<BasicTypeEnum> = param_types.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice: Vec<BasicMetadataTypeEnum> = paramTypes.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice = &paramTypesSlice[..];


        let funcType = returnType.fn_type(&paramTypesSlice, false);
        let procFunVal = self.module.add_function(proc_name, funcType.clone(), None);

        // Create entry block and position the builder
        let entry = self.context.append_basic_block(procFunVal, "entry");
        procBuilder.position_at_end(entry);

        // Prepare for the printf call
        let printf = self.module.get_function("printf").unwrap();
        let formatStr = CString::new("%d\n").unwrap(); // Format string for integer
        let formatStr = self.context.const_string(formatStr.as_bytes(), false);
        let formatPtrCheck = procBuilder.build_alloca(formatStr.get_type(), "format");
        let formatPtr:PointerValue;
        match formatPtrCheck{
            Ok(val) => {
                formatPtr = val.clone();
            }
            Err(err) => {
                println!("error in allocating string space");
                return;
            }
        }
        
        let _ = procBuilder.build_store(formatPtr, formatStr);

        // Get the parameter (integer value) passed to the function
        let int_param = procFunVal.get_nth_param(0).unwrap().into_int_value();

        // Build the printf call
        let args: &[inkwell::values::BasicMetadataValueEnum<'_>] = &[formatPtr.into()];
        let _ = procBuilder.build_call(printf, &args, "printf_call");

        // Return a boolean true (indicating success)
        let true_val = self.context.bool_type().const_int(1, false);
        let _ = procBuilder.build_return(Some(&true_val));
    }
    fn declarePutFloat(&mut self) {
        // Define the function
        let proc_name = "putFloat";
        // Create the local variable hash table
        let mut procLocTable: HashMap<String, PointerValue> = HashMap::new();

        // Create the local builder
        let procBuilder = self.context.create_builder();

        // Create a vec for the param types
        let paramTypes: Vec<BasicTypeEnum> = vec![self.context.i32_type().into()];

        // Define the return type (boolean)
        let boolType = self.context.bool_type();
        let returnType = boolType;

        // Create function type
        // let param_types_slice: Vec<BasicTypeEnum> = param_types.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice: Vec<BasicMetadataTypeEnum> = paramTypes.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice = &paramTypesSlice[..];


        let funcType = returnType.fn_type(&paramTypesSlice, false);
        let procFunVal = self.module.add_function(proc_name, funcType.clone(), None);

        // Create entry block and position the builder
        let entry = self.context.append_basic_block(procFunVal, "entry");
        procBuilder.position_at_end(entry);

        // Prepare for the printf call
        let printf = self.module.get_function("printf").unwrap();
        let formatStr = CString::new("%d\n").unwrap(); // Format string for integer
        let formatStr = self.context.const_string(formatStr.as_bytes(), false);
        let formatPtrCheck = procBuilder.build_alloca(formatStr.get_type(), "format");
        let formatPtr:PointerValue;
        match formatPtrCheck{
            Ok(val) => {
                formatPtr = val.clone();
            }
            Err(err) => {
                println!("error in allocating string space");
                return;
            }
        }
        
        let _ = procBuilder.build_store(formatPtr, formatStr);

        // Get the parameter (integer value) passed to the function
        let int_param = procFunVal.get_nth_param(0).unwrap().into_int_value();

        // Build the printf call
        let args: &[inkwell::values::BasicMetadataValueEnum<'_>] = &[formatPtr.into()];
        let _ = procBuilder.build_call(printf, &args, "printf_call");

        // Return a boolean true (indicating success)
        let true_val = self.context.bool_type().const_int(1, false);
        let _ = procBuilder.build_return(Some(&true_val));
    }
    fn declarePutString(&mut self) {
        // Define the function
        let proc_name = "putstring";
        // Create the local variable hash table
        let mut procLocTable: HashMap<String, PointerValue> = HashMap::new();

        // Create the local builder
        let procBuilder = self.context.create_builder();

        // Create a vec for the param types
        let paramTypes: Vec<BasicTypeEnum> = vec![self.context.i32_type().into()];

        // Define the return type (boolean)
        let boolType = self.context.bool_type();
        let returnType = boolType;

        // Create function type
        // let param_types_slice: Vec<BasicTypeEnum> = param_types.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice: Vec<BasicMetadataTypeEnum> = paramTypes.iter().map(|&ty| ty.into()).collect();
        let paramTypesSlice = &paramTypesSlice[..];


        let funcType = returnType.fn_type(&paramTypesSlice, false);
        let procFunVal = self.module.add_function(proc_name, funcType.clone(), None);

        // Create entry block and position the builder
        let entry = self.context.append_basic_block(procFunVal, "entry");
        procBuilder.position_at_end(entry);

        // Prepare for the printf call
        let printf = self.module.get_function("printf").unwrap();
        let formatStr = CString::new("%d\n").unwrap(); // Format string for integer
        let formatStr = self.context.const_string(formatStr.as_bytes(), false);
        let formatPtrCheck = procBuilder.build_alloca(formatStr.get_type(), "format");
        let formatPtr:PointerValue;
        match formatPtrCheck{
            Ok(val) => {
                formatPtr = val.clone();
            }
            Err(err) => {
                println!("error in allocating string space");
                return;
            }
        }
        
        let _ = procBuilder.build_store(formatPtr, formatStr);

        // Get the parameter (integer value) passed to the function
        let int_param = procFunVal.get_nth_param(0).unwrap().into_int_value();

        // Build the printf call
        let args: &[inkwell::values::BasicMetadataValueEnum<'_>] = &[formatPtr.into()];
        let _ = procBuilder.build_call(printf, &args, "printf_call");

        // Return a boolean true (indicating success)
        let true_val = self.context.bool_type().const_int(1, false);
        let _ = procBuilder.build_return(Some(&true_val));
    }
   
    
    

    fn addScanf(&mut self) {
        let i32Type = self.context.i32_type();
        let stringType = self.context.i8_type();
        let strPtrType = BasicMetadataTypeEnum::IntType(stringType);
        let scanfType = i32Type.fn_type(&[strPtrType], true);
        self.module.add_function("scanf", scanfType, None);
    }
    fn addPrintf(&mut self) {
        let i32Type = self.context.i32_type();
        let stringType = self.context.i8_type();
        let strPtrType = BasicMetadataTypeEnum::IntType(stringType);
        let scanfType = i32Type.fn_type(&[strPtrType], true);
        self.module.add_function("printf", scanfType, None);
    }
    
    
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


    //Creates the llvm context and the code generator struct
    let context = Context::create();
    let mut myGen = Compiler::new(programAst.clone(), &context, &mut global_table, "Program".to_string());

    println!("Created generator");
    let ret = myGen.compileProgram();
    match ret{
        Ok(module) => {
            println!("\n\nModule generated");
            // module.print_to_string();
            module.print_to_stderr();
        }
        Err(errMsg) => {
            println!("Error with generation: {}", errMsg);
        }
    }



    Ok(())
}