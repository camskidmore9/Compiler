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
    }}, anyhow::Result, inkwell::{builder::Builder, context::Context, module::Module, values::*, AddressSpace, OptimizationLevel, IntPredicate, FloatPredicate}, parse_display::Display, std::{
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
                        let good = self.compileStmt(instr.clone(), mainFunc);
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

    fn compileStmt(&mut self, stmt: Stmt, func: FunctionValue) -> bool{
        match stmt.clone(){
            //For global variable declarations
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
                    let checkLocVar = self.localTable.get(&targName.clone());
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
                else {
                    println!("Cannot assing to a non variable");
                    return false;
                }

                let checkNewValue = self.compileExpr(&newValue.clone());
                match checkNewValue.clone(){
                    Ok(value) => {
                        newEnumValue = value.clone();
                    }
                    Err(msg) => {
                        println!("{}", msg.clone());
                        return false;
                    }
                }
                
                let mut finalVal = newEnumValue.clone();

                let builder = &mut self.builder;

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

                    _ => {
                        println!("Not implemented for that type yet");
                        return true;
                    }
                }

                // self.builder.build_store(variablePtr, finalVal);
                // return true;
                    
            }
            Stmt::Block(blockStmt, lineNum) => {
                println!("block stmt NEEDS WRITTEn");
                return true;
            }
            Stmt::Error(err, lineNum) => {
                println!("error stmt NEEDS WRITTEn");
                return true;
            }
            Stmt::Expr(exprStmt, lineNum) => {
                println!("expr stmt NEEDS WRITTEn");
                return true;
            }
            Stmt::For(assignment, condition, body, lineNum) => {
                println!("for stmt NEEDS WRITTEn");
                return true;
            }
            Stmt::If(condition, body, elseStmt, lineNum) => {
                println!("if statement NEEDS WRITTEn");
                return true;
            }
            Stmt::ProcDecl(procType, procName, params, headerBox, bodyBox, lineNum) => {
                println!("procedure declaration NEEDS WRITTEn");
                return true;
            }
            Stmt::StringLiteral(str, lineNum) => {
                println!("StringLiteral Stmt, this should never happe");
                return true;
            }
            Stmt::VarDecl(varName, varType, lineNum) => {
                println!("local assignment NEEDS WRITTEn");
                return true;
            }
            Stmt::Return(valueExpr, lineNum) => {
                println!("return stmt NEEDS WRITTEn");
                return true;
            }
            Stmt::Program(name, headerBox, bodyBox, lineNum) => {
                println!("Program Stmt, this should never happen");
                return true;
            }
            
        }
        
    }

    fn compileExpr(&self, expr: &Expr) -> Result<BasicValueEnum<'ctx>, String> {
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
                let checkLocVar = self.localTable.get(&varName.clone());
                match checkLocVar{
                    Some(varPtr) => {
                        let loadedVal = self.builder.build_load(varPtr.clone(), &varName.clone());
                        match loadedVal{
                            Ok(val) => {
                                return Ok(val);
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
                                    let loadedVal = self.builder.build_load(varPtr.clone(), &varName.clone());
                                    match loadedVal{
                                        Ok(val) => {
                                            return Ok(val);
                                        }
                                        Err(err) => {
                                            return Err(format!("Error with pointer to value {}", varName.clone()));
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

            Expr::ArthOp(op1, op, op2) => {
                // let context = &mut self.context;
                // let builder = &mut self.builder;
                
                let intType = self.context.i32_type().clone();
                let floatType = self.context.f64_type().clone();

                //First gets the values of both operands
                let op1Res = self.compileExpr(&*op1.clone()).clone();
                let op2Res = self.compileExpr(&*op2.clone()).clone();
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
                                    let resConv = self.builder.build_signed_int_to_float(val, floatType, "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, floatType, "intToFloat");
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
                            let retOp = self.builder.build_float_add(op1Float, op2Float, "addFloat");
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
                            let retOp = self.builder.build_int_add(op1Int.clone(), op2Int.clone(), "addInt");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, floatType, "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, floatType, "intToFloat");
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
                            let retOp = self.builder.build_float_sub(op1Float, op2Float, "subFloat");
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
                            let retOp = self.builder.build_int_sub(op1Int.clone(), op2Int.clone(), "subInt");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, floatType, "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, floatType, "intToFloat");
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
                            let retOp = self.builder.build_float_mul(op1Float, op2Float, "multiplyFloat");
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
                            let retOp = self.builder.build_int_mul(op1Int.clone(), op2Int.clone(), "multiplyInt");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, floatType, "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, floatType, "intToFloat");
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
                            let retOp = self.builder.build_float_div(op1Float, op2Float, "divideFloat");
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
                            let retOp = self.builder.build_int_signed_div(op1Int.clone(), op2Int.clone(), "divideInt");
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
                let op1Res = self.compileExpr(&*op1.clone());
                let op2Res = self.compileExpr(&*op2.clone());
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                            let retOp = self.builder.build_float_compare(FloatPredicate::OEQ,op1Float, op2Float, "equalFloat");
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
                            let retOp = self.builder.build_int_compare(IntPredicate::EQ,op1Int, op2Int, "equalInt");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                            let retOp = self.builder.build_float_compare(FloatPredicate::OGT,op1Float, op2Float, "floatGreater");
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
                            let retOp = self.builder.build_int_compare(IntPredicate::SGT,op1Int, op2Int, "intGreater");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                            let retOp = self.builder.build_float_compare(FloatPredicate::OGE,op1Float, op2Float, "floatGreaterEqual");
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
                            let retOp = self.builder.build_int_compare(IntPredicate::SGE,op1Int, op2Int, "intGreaterEqual");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                            let retOp = self.builder.build_float_compare(FloatPredicate::OLT,op1Float, op2Float, "floatLess");
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
                            let retOp = self.builder.build_int_compare(IntPredicate::SLT,op1Int, op2Int, "intLess");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                            let retOp = self.builder.build_float_compare(FloatPredicate::OLE,op1Float, op2Float, "floatLessEqual");
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
                            let retOp = self.builder.build_int_compare(IntPredicate::SLE,op1Int, op2Int, "intLessEqual");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                                    let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
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
                            let retOp = self.builder.build_float_compare(FloatPredicate::ONE,op1Float, op2Float, "floatNotEqual");
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
                            let retOp = self.builder.build_int_compare(IntPredicate::NE,op1Int, op2Int, "intNotEqual");
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
                // println!("LogOp temporarily removed needs written");
                // let intType = self.context.i32_type();
                // let intval = intType.const_int(1, false);
                // return Ok(BasicValueEnum::IntValue(intval));
                
                //First gets the values of both operands
                let op1Res = self.compileExpr(&*op1.clone());
                let op2Res = self.compileExpr(&*op2.clone());
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
                                    let resConv = self.builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
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
                                    let resConv = self.builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
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

                            let retOp = self.builder.build_and(op1Int, op2Int, "intAnd");
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
                            let retOp = self.builder.build_and(op1Int, op2Int, "intAnd");
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
                                    let resConv = self.builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
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
                                    let resConv = self.builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
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

                            let retOp = self.builder.build_or(op1Int, op2Int, "intOr");
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
                            let retOp = self.builder.build_or(op1Int, op2Int, "intOr");
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
        
        
            
            _ => {
                println!("Not implemented expression");
                let val = 25;
                let intType = self.context.i64_type().clone();
                let intVal = intType.const_int(val, false);
                return Ok(BasicValueEnum::IntValue(intVal));
            },
        }
    
    
    
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