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
    }}, anyhow::Result, inkwell::{
        builder::Builder, context::Context, llvm_sys::LLVMValue, module::Module, types::BasicTypeEnum, values::*, AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel
    }, llvm_sys::transforms::pass_builder::LLVMPassBuilderOptionsRef, parse_display::Display, std::{
        collections::HashMap, env, fmt, fs::{
            read_to_string, File
        }, hash::Hash, io::prelude::{*}, io::{
            prelude::{*}, BufRead, BufReader, Read
        }, path::Path, rc::Rc
    }, unicode_segmentation::UnicodeSegmentation, utf8_chars::BufReadCharsExt
    

};

///////////////////////// Setup /////////////////////////

//imports

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


// The main type checking structure
pub struct Compiler<'ctx> {
    pub programAst: Stmt, // The program AST
    pub scope: i32, // The current scope level

    pub localTable: HashMap<String, PointerValue<'ctx>>, // Local table for the current scope
    pub globalTable: &'ctx mut HashMap<String, PointerValue<'ctx>>, // Shared global table

    pub name: String, // The name of the program (or procedure if in a nested scope)

    // LLVM stuff
    context: &'ctx Context, // The LLVM context
    module: Module<'ctx>, // The LLVM module
    builder: Builder<'ctx>, // The LLVM builder
}
impl<'ctx> Compiler<'ctx> {
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
                        let good = self.compileStmt(instr.clone());
                    }
                } else {
                    println!("Problem with AST: header must be a Block");
                }

                println!("Header processed");

                //Creates the entrypoint at the main function
                // let mainBlock = self.context.append_basic_block(mainFunc, "entry");
                // self.builder.position_at_end(mainBlock);
                // println!("Created entry point");

                // println!("Time to go through body");
                // //Goes through the body and adds each line to the module
                // let newBodyBox = bodyBox.clone();
                // let mut body = *newBodyBox;
                // // Check if the variable is a Block and iterate through it
                // if let Stmt::Block(ref instrs, lineNum) = body.clone() {
                //     for instr in instrs {
                //         self.compileStmt(instr.clone());
                //     }
                // } else {
                //     println!("Problem with AST: header must be a Block");
                // }
                // let mainRet = i32Type.const_int(1, false);
                // let _ = self.builder.build_return(Some(&mainRet));

                // let retval = self.module.clone();
                

                return Ok(&self.module);
            }
            _ => {
                let errMsg = format!("ProgramAst must be a Program Stmt");
                return Err(errMsg);
            }
        }
        
        // if let (Stmt::Program(progName, head, body, lineNum) = self.programAst.clone()) {
        //     println!("program good");
        // }
        
        // return Ok(&self.module);
    }


    pub fn compileExpr(&mut self, mut checkExpr: Expr) -> Result<BasicValueEnum<'ctx>, String>{
        match checkExpr.clone(){
            //Literals
            //Literals
            Expr::IntLiteral(value) => {
                let intType = self.context.i64_type().clone();
                let intVal = intType.const_int(value.try_into().unwrap(), false);
                return Ok(BasicValueEnum::IntValue(intVal.clone()).clone());
            }

                
            Expr::FloatLiteral(value) => {
                let floatType = self.context.f64_type().clone();
                let floatVal = floatType.const_float(value);
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
            
            //References
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
            
            
            Expr::ProcRef(procName, params) => {
                println!("ProcRef Stmt needs written");
                let intType = self.context.i32_type();
                let intval = intType.const_int(1, false);
                return Ok(BasicValueEnum::IntValue(intval));
                // if (self.name.clone() == procName.clone()){
                //     return true;
                // } else {
                //     //Gets the type if defined in local scope
                //     let checkProc = self.localTable.get(&procName.clone());
                //     match checkProc.clone(){
                //         Some(proc) => {
                //             if let HashItemType::Procedure(procAst, procParamList, mut procSt) = proc.hashType.clone() {
                //                 //Proc found, need to check params now
                //                 match params.clone(){
                //                     Some(paramsVec) => {
                //                         if (procParamList.len() == paramsVec.len()) {
                //                             //the numbers are correct at least
                //                             let mut i = 0;
                //                             //Checks all of the params
                //                             for param in paramsVec.clone() {
                //                                 let targetTypeCheck = procSt.getType(&procParamList[i].clone());
                //                                 match targetTypeCheck{
                //                                     Some(targetType) => {
                //                                         let compatable = self.checkExprTypeCompatability(targetType.clone(), param.clone());
                //                                         if compatable {
                //                                             //Continue to checking next param
                //                                         } else {
                //                                             println!("Error with call to procedure {}: param {} is type {}, which is incompatible with given type {}", procName.clone(), procParamList[i].clone(), targetType.clone(), param.clone());
                //                                             return false;
                //                                         }
                //                                     }
                //                                     None => {
                //                                         println!("Some sort of error with the procedure symbol table. Could not located defined parameter in table");
                //                                         return false;
                //                                     }
                //                                 }
                //                                 i += 1;
                //                             }
                //                             return true;

                //                         } else {
                //                             println!("Error with call to procedure {}: {} params required, {} provided", procName.clone(), paramsVec.len().to_string(), procParamList.len().clone().to_string())
                //                         }
                //                     }
                //                     None => {
                //                         if (procParamList.len() == 0){
                //                             return true;
                //                         } else {
                //                             println!("Procedure call to {} missing parameters", procName.clone());
                //                             return false;
                //                         }
                //                     }
                //                 }
                //                 return true;
                //             } else {
                //                 println!("{} is not defined as a procedure", procName.clone());
                //                 return false;
                //             }
                //         }
                //         None => {
                //             //CHECK IN GLOBAL SCOPE GOES HERE
                //             //Gets the type if defined in local scope
                //             let checkGlobProc = self.globalTable.get(&procName.clone());
                //             match checkGlobProc.clone(){
                //                 Some(proc) => {
                //                     if let HashItemType::Procedure(procAst, procParamList, mut procSt) = proc.hashType.clone() {
                //                         //Proc found, need to check params now
                //                         match params.clone(){
                //                             Some(paramsVec) => {
                //                                 if (procParamList.len() == paramsVec.len()) {
                //                                     //the numbers are correct at least
                //                                     let mut i = 0;
                //                                     //Checks all of the params
                //                                     for param in paramsVec.clone() {
                //                                         let targetTypeCheck = procSt.getType(&procParamList[i].clone());
                //                                         match targetTypeCheck{
                //                                             Some(targetType) => {
                //                                                 let compatable = self.checkExprTypeCompatability(targetType.clone(), param.clone());
                //                                                 if compatable {
                //                                                     //Continue to checking next param
                //                                                 } else {
                //                                                     println!("Error with call to procedure {}: param {} is type {}, which is incompatible with given type {}", procName.clone(), procParamList[i].clone(), targetType.clone(), param.clone());
                //                                                     return false;
                //                                                 }
                //                                             }
                //                                             None => {
                //                                                 println!("Some sort of error with the procedure symbol table. Could not located defined parameter in table");
                //                                                 return false;
                //                                             }
                //                                         }
                //                                         i += 1;
                //                                     }
                //                                     return true;

                //                                 } else {
                //                                     println!("Error with call to procedure {}: {} params required, {} provided", procName.clone(), paramsVec.len().to_string(), procParamList.len().clone().to_string());
                //                                     return false;
                //                                 }
                //                             }
                //                             None => {
                //                                 if (procParamList.len() == 0){
                //                                     return true;
                //                                 } else {
                //                                     println!("Procedure call to {} missing parameters", procName.clone());
                //                                     return false;
                //                                 }
                //                             }
                //                         }
                //                     } else {
                //                         println!("{} is not defined as a procedure", procName.clone());
                //                         return false;
                //                     }
                //                 }
                //                 None => {
                //                     println!("Procedure {} is not defined", procName.clone());
                //                     return false;
                //                 }
                //             }
                //         }
                //     }
                // }
                
            }
            //Needs written
            Expr::ArrayRef(varName, indexExpr) => {
                println!("arrayref Stmt needs written");
                let intType = self.context.i32_type();
                let intval = intType.const_int(1, false);
                return Ok(BasicValueEnum::IntValue(intval));
            }
            
            //Operations
            Expr::ArthOp(op1, op, op2) => {
                println!("ArthOp temporarily removed needs written");
                let intType = self.context.i32_type();
                let intval = intType.const_int(1, false);
                return Ok(BasicValueEnum::IntValue(intval));   
                
                // let context = &mut self.context;
                // let builder = &mut self.builder;
                
                // let intType = context.i32_type().clone();
                // let floatType = context.f64_type().clone();

                // //First gets the values of both operands
                // let op1Res = self.compileExpr(*op1.clone()).clone();
                // let op2Res = self.compileExpr(*op2.clone()).clone();
                // let mut op1Val: BasicValueEnum;
                // let mut op2Val: BasicValueEnum;
                // //Makes sure both results of checked operands are good
                // match op1Res.clone(){
                //     Ok(res) => {
                //         op1Val = res.clone();
                //     }
                //     Err(msg) => {
                //         return Err(msg.clone());
                //     }
                // }
                // match op2Res.clone(){
                //     Ok(res) => {
                //         op2Val = res.clone();
                //     }
                //     Err(msg) => {
                //         return Err(msg.clone());
                //     }
                // }

                // //Checks if either value is a float
                // let op1IsFloat = match op1Val.clone(){
                //     BasicValueEnum::FloatValue(_) => true,
                //     _ => false,
                // };
                // let op2IsFloat = match op2Val.clone(){
                //     BasicValueEnum::FloatValue(_) => true,
                //     _ => false,
                // };

                // //a match case to handle the different types of operators
                // match op.clone(){
                //     Operator::Add => {
                //         //If either result is a float
                //         if op1IsFloat.clone() || op2IsFloat.clone() {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val.clone() {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val.clone();
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val.clone() {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val.clone();
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val.clone();
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float add
                //             let retOp = builder.build_float_add(op1Float, op2Float, "addFloat");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::FloatValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = builder.build_int_add(op1Int.clone(), op2Int.clone(), "addInt");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     }
                //     Operator::Sub => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val.clone();
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val;
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float add
                //             let retOp = builder.build_float_sub(op1Float, op2Float, "subFloat");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::FloatValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = builder.build_int_sub(op1Int.clone(), op2Int.clone(), "subInt");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     } 
                //     Operator::Mul => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val;
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float add
                //             let retOp = builder.build_float_mul(op1Float, op2Float, "multiplyFloat");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::FloatValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.clone().into_int_value();
                //             let op2Int = op2Val.clone().into_int_value();
                //             let retOp = builder.build_int_mul(op1Int.clone(), op2Int.clone(), "multiplyInt");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     }
                //     Operator::Div => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val;
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = builder.build_signed_int_to_float(val, floatType, "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float add
                //             let retOp = builder.build_float_div(op1Float, op2Float, "divideFloat");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::FloatValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = builder.build_int_signed_div(op1Int.clone(), op2Int.clone(), "divideInt");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     }
                //     _ => {
                //         //This should never happen because of parsing and error checking
                //         return Err("Improper operator for arthimatic operation".to_string());
                //     }
                // }
            
            
            }
            Expr::RelOp(op1, op, op2) => {
                println!("RelOp temporarily removed needs written");
                let intType = self.context.i32_type();
                let intval = intType.const_int(1, false);
                return Ok(BasicValueEnum::IntValue(intval));    
                
                // //First gets the values of both operands
                // let op1Res = self.compileExpr(*op1.clone());
                // let op2Res = self.compileExpr(*op2.clone());
                // let mut op1Val: BasicValueEnum;
                // let mut op2Val: BasicValueEnum;
                // //Makes sure both results of checked operands are good
                // match op1Res{
                //     Ok(res) => {
                //         op1Val = res.clone();
                //     }
                //     Err(msg) => {
                //         return Err(msg.clone());
                //     }
                // }
                // match op2Res{
                //     Ok(res) => {
                //         op2Val = res.clone();
                //     }
                //     Err(msg) => {
                //         return Err(msg.clone());
                //     }
                // }

                // //Checks if either value is a float
                // let op1IsFloat = match op1Val.clone(){
                //     BasicValueEnum::FloatValue(_) => true,
                //     _ => false,
                // };
                // let op2IsFloat = match op2Val.clone(){
                //     BasicValueEnum::FloatValue(_) => true,
                //     _ => false,
                // };

                // //a match case to handle the different types of operators
                // match op{
                //     Operator::Check_Equal => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val;
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float equality check
                //             let retOp = self.builder.build_float_compare(FloatPredicate::OEQ,op1Float, op2Float, "equalFloat");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = self.builder.build_int_compare(IntPredicate::EQ,op1Int, op2Int, "equalInt");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     }
                //     Operator::Greater => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for greater".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val;
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float equality check
                //             let retOp = self.builder.build_float_compare(FloatPredicate::OGT,op1Float, op2Float, "floatGreater");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = self.builder.build_int_compare(IntPredicate::SGT,op1Int, op2Int, "intGreater");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     }
                //     Operator::Greater_Equal => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val;
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float equality check
                //             let retOp = self.builder.build_float_compare(FloatPredicate::OGE,op1Float, op2Float, "floatGreaterEqual");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = self.builder.build_int_compare(IntPredicate::SGE,op1Int, op2Int, "intGreaterEqual");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     }
                //     Operator::Less => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val;
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float equality check
                //             let retOp = self.builder.build_float_compare(FloatPredicate::OLT,op1Float, op2Float, "floatLess");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = self.builder.build_int_compare(IntPredicate::SLT,op1Int, op2Int, "intLess");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     }
                //     Operator::Less_Equal => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val;
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float equality check
                //             let retOp = self.builder.build_float_compare(FloatPredicate::OLE,op1Float, op2Float, "floatLessEqual");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = self.builder.build_int_compare(IntPredicate::SLE,op1Int, op2Int, "intLessEqual");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     }
                //     Operator::Not_Equals => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to float if not
                //             let op1Float: FloatValue;
                //             match op1Val {
                //                 BasicValueEnum::FloatValue(val) => op1Float = val,
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for not equal".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Float: FloatValue;
                //             match op2Val {
                //                 BasicValueEnum::FloatValue(val) => {
                //                     op2Float = val;
                //                 }
                //                 BasicValueEnum::IntValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_signed_int_to_float(val, self.context.f64_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Float = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Does the float equality check
                //             let retOp = self.builder.build_float_compare(FloatPredicate::ONE,op1Float, op2Float, "floatNotEqual");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = self.builder.build_int_compare(IntPredicate::NE,op1Int, op2Int, "intNotEqual");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                //     }
                    
                //     _ => {
                //         //This should never happen because of parsing and error checking
                //         return Err("Improper operator for logical operation".to_string());
                //     }
                // }
            
            
            }
            Expr::LogOp(op1, op, op2) => {
                println!("LogOp temporarily removed needs written");
                let intType = self.context.i32_type();
                let intval = intType.const_int(1, false);
                return Ok(BasicValueEnum::IntValue(intval));
                
                // //First gets the values of both operands
                // let op1Res = self.compileExpr(*op1.clone());
                // let op2Res = self.compileExpr(*op2.clone());
                // let mut op1Val: BasicValueEnum;
                // let mut op2Val: BasicValueEnum;
                // //Makes sure both results of checked operands are good
                // match op1Res{
                //     Ok(res) => {
                //         op1Val = res;
                //     }
                //     Err(msg) => {
                //         return Err(msg.clone());
                //     }
                // }
                // match op2Res{
                //     Ok(res) => {
                //         op2Val = res;
                //     }
                //     Err(msg) => {
                //         return Err(msg.clone());
                //     }
                // }

                // //Checks if either value is a float
                // let op1IsFloat = match op1Val{
                //     BasicValueEnum::FloatValue(_) => true,
                //     _ => false,
                // };
                // let op2IsFloat = match op2Val{
                //     BasicValueEnum::FloatValue(_) => true,
                //     _ => false,
                // };

                // //a match case to handle the different types of operators
                // match op{
                //     Operator::And => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to int if not
                //             let op1Int: IntValue;
                //             match op1Val {
                //                 BasicValueEnum::IntValue(val) => op1Int = val,
                //                 BasicValueEnum::FloatValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Int = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Int: IntValue;
                //             match op2Val {
                //                 BasicValueEnum::IntValue(val) => op2Int = val,
                //                 BasicValueEnum::FloatValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Int = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             let retOp = self.builder.build_and(op1Int, op2Int, "intAnd");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }

                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = self.builder.build_and(op1Int, op2Int, "intAnd");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                        
                //     }
                //     Operator::Or => {
                //         //If either result is a float
                //         if op1IsFloat || op2IsFloat {
                            
                //             //Checks if op1 is float, casts it to int if not
                //             let op1Int: IntValue;
                //             match op1Val {
                //                 BasicValueEnum::IntValue(val) => op1Int = val,
                //                 BasicValueEnum::FloatValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op1Int = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             //Checks if op2 is float, casts it to float if not
                //             let op2Int: IntValue;
                //             match op2Val {
                //                 BasicValueEnum::IntValue(val) => op2Int = val,
                //                 BasicValueEnum::FloatValue(val) => {
                //                     // Convert integer to float if necessary
                //                     let resConv = self.builder.build_float_to_signed_int(val, self.context.i32_type(), "intToFloat");
                //                     match resConv{
                //                         Ok(val) => {
                //                             op2Int = val;
                //                         }
                //                         Err(errMsg) => {
                //                             return Err(format!("{}", errMsg));
                //                         }
                //                     }
                //                 },
                //                 _ => return Err("Unsupported type for addition".to_string()),
                //             };

                //             let retOp = self.builder.build_or(op1Int, op2Int, "intOr");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }

                //         } 
                //         // Both operands are integers
                //         else {
                //             let op1Int = op1Val.into_int_value();
                //             let op2Int = op2Val.into_int_value();
                //             let retOp = self.builder.build_or(op1Int, op2Int, "intOr");
                //             match retOp{
                //                 Ok(result) => {
                //                     return Ok(BasicValueEnum::IntValue(result.clone()));
                //                 }
                //                 Err(errMsg) => {
                //                     return Err(format!("{}", errMsg));
                //                 }
                //             }
                            
                //         }
                        
                //     }
                    
                //     _ => {
                //         //This should never happen because of parsing and error checking
                //         return Err("Improper operator for logical operation".to_string());
                //     }
                // }
                
            }
        
        
        
        }
    }
    
    //Checks each statement one at a time, returns a bool if there's an error
    pub fn compileStmt(&mut self, mut checkStmt: Stmt) -> bool{
        match (checkStmt){
            //For checking and declaring local variables
            Stmt::VarDecl(varName, varType, lineNum) => {
                // if self.scope != 0 {
                //     let defined = self.localTable.checkItem(&varName.clone());
                //     if(defined){
                //         println!("Error: variable: {} defined twice", varName.clone());
                //         return false;
                //     } else {
                //         let item = HashItem::newVar(varName.clone(), varType.clone());
                //         self.localTable.symTab.insert(varName.clone(), item.clone());
                //         return true;
                //     }
                // } else {
                //     let defined = self.globalTable.checkItem(&varName.clone());
                //     if(defined){
                //         println!("Error: variable: {} defined twice", varName.clone());
                //         return false;
                //     } else {
                //         let item = HashItem::newVar(varName.clone(), varType.clone());
                //         self.globalTable.symTab.insert(varName.clone(), item.clone());
                //         return true;
                //     }
                // }
                println!("Local variable declaration needs written");
                return true;
            }
            //For checking and declaring global variables
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
                        let varType = self.context.f64_type();
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
            _ => {
                println!("Not yet");
                return true;
            }
            // //For checking a procedure
            // Stmt::ProcDecl(retType, procName, params, header, body, lineNum) => {
            //     println!("ProcDecl needs written");
            //     return true;
            //     // // println!("procedure declaration");
            //     // let procAst = Stmt::Program(procName.clone(), header.clone(), body.clone(), lineNum.clone());
                
            //     // let mut paramStrings: Vec<String> = Vec::new();

            //     // let curScope = self.scope.clone();

            //     // let mut procChecker: SyntaxChecker = self.newScope(procAst, curScope, procName.clone());
            //     // //Iterates through the parameters, registering them in the Symboltable and copying the names to the list of params
            //     // if let Stmt::Block(ref instrs, lineNum) = *params.clone() {
            //     //     for instr in instrs {
            //     //         let good = procChecker.checkStmt(instr.clone());
            //     //         if (!good){
            //     //             println!("Error in Procedure parameter definition on line {}:", lineNum.clone());
            //     //             // instr.display(0);
            //     //             return false;
            //     //         } else {
            //     //             match instr.clone(){
            //     //                 Stmt::VarDecl(varName, VarType, lineNum) => {
            //     //                     paramStrings.push(varName.clone());
            //     //                 }
            //     //                 _ => {
            //     //                     println!("Error with procedure {} declaration on line {}:\n Procedure parameters must be variable declarations in the following format:\n    variable <identifier> : <type_mark>", procName.clone(), lineNum.clone());
            //     //                     return false;
            //     //                 }
            //     //             }
            //     //         }
            //     //     }
            //     // } else {
            //     //     println!("Error in Procedure parameter definition on line {}:", lineNum.clone());
            //     //     // instr.display(0);
            //     //     return false;
            //     // }


            //     // //Checks the procedure to make sure its all good
            //     // let procGood = procChecker.checkProgram();


            //     // //If the procedure is good, appends to the symboltable and moved on
            //     // if(!procGood){
            //     //     println!("Error in procedure {} defined on line {}", procName.clone(), lineNum.clone());
            //     //     return false;
            //     // } else {
            //     //     if curScope != 0 {
            //     //         //Sets up the things and inserts the procedure into the symboltable
            //     //         let mut procItemType = HashItemType::newProcItem(body.clone(), paramStrings.clone(), procChecker.localTable.clone());
            //     //         let mut procItem: HashItem = HashItem::newProc(procName.clone(), retType.clone(), procItemType);
            //     //         self.localTable.symTab.insert(procName.clone(), procItem.clone());
                        
            //     //         return true;
            //     //     } else {
            //     //         //Sets up the things and inserts the procedure into the symboltable
            //     //         let mut procItemType = HashItemType::newProcItem(body.clone(), paramStrings.clone(), procChecker.localTable.clone());
            //     //         let mut procItem: HashItem = HashItem::newProc(procName.clone(), retType.clone(), procItemType);
            //     //         self.globalTable.symTab.insert(procName.clone(), procItem.clone());
            //     //         return true;
            //     //     }
            //     // }
            // }
            // //For checking a variable assignment
            // Stmt::Assign(variable, newValue, lineNum) => {
            //     let mut variablePtr: PointerValue;
            //     let mut newEnumValue: BasicValueEnum;
            //     let mut varName: String;

            //     if let Expr::VarRef(ref targName) = variable {
            //         varName = targName.clone();
            //         let checkLocVar = self.localTable.get(&targName.clone());
            //         match checkLocVar{
            //             Some(ptr) => {
            //                 println!("Assigninig local variable {} at location {}", targName.clone(), ptr.clone());
            //                 variablePtr = ptr.clone();
            //             }
            //             None => {
            //                 let checkGlobVar = self.globalTable.get(&targName.clone());
            //                 match checkGlobVar{
            //                     Some(ptr) => {
            //                         println!("Assigninig global variable {} at location {}", targName.clone(), ptr.clone());
            //                         variablePtr = ptr.clone();
            //                     }
            //                     None => {
            //                         println!("variable {} not found", targName.clone());
            //                         return false;
            //                     }
            //                 }
            //             }
            //         }
            //     }
            //     else {
            //         println!("Cannot assing to a non variable");
            //         return false;
            //     }

            //     let checkNewValue = self.compileExpr(newValue.clone());
            //     match checkNewValue.clone(){
            //         Ok(value) => {
            //             match value{
            //                 BasicValueEnum::IntValue(val) => {
            //                     let valClone = val.clone();
            //                     let newbasicInt = BasicValueEnum::IntValue(valClone.clone());
            //                     newEnumValue = newbasicInt.clone();
            //                 }
            //                 _ => {
            //                     println!("fuck you");
            //                     return true;
            //                 }
            //             }
            //         }
            //         Err(msg) => {
            //             println!("{}", msg.clone());
            //             return false;
            //         }
            //     }
                
            //     let mut finalVal = newEnumValue.clone();

            //     let builder = &mut self.builder;

            //     // let mut finalVal: IntValue;
            //     match finalVal{
            //         BasicValueEnum::IntValue(intVal) => {
            //             println!("Stored value {} in variable {}",intVal.clone(), varName.clone());
            //             let _ = builder.build_store(variablePtr, intVal.clone());
            //             return true;
            //         }
            //         _ => {
            //             println!("Not implemented for that type yet");
            //             return true;
            //         }
            //     }

            //     // self.builder.build_store(variablePtr, finalVal);
            //     // return true;
                    
            // }
            

            // //For Stmts that are just Exprs
            // Stmt::Expr(expr, lineNum) => {
            //     println!("ExprStmt needs written");
            //     return true;
            //     // match (expr){
            //     //     _ => {
            //     //         let checked = self.compileExpr(expr.clone());
            //     //         if checked {
            //     //             return true;
            //     //         } else {
            //     //             println!("Error with expression statement on line {}", lineNum.clone());
            //     //             return false;
            //     //         }
            //     //     }
            //     // }
            // }
            // //For checking if statements
            // Stmt::If(condition, body, elseBody, lineNum) => {
            //     println!("if needs written");
            //     return true;
            //     // //Checks the condition
            //     // match condition.clone() {
            //     //     Expr::IntArrayLiteral(size, array) => {
            //     //         println!("Error with if condition on line {}:\n Cannot use array as condition", lineNum.clone());
            //     //         return false;
            //     //     }
            //     //     Expr::FloatLiteral(val) => {
            //     //         println!("Error with if condition on line {}:\n Cannot use float as condition", lineNum.clone());
            //     //         return false;
            //     //     }
            //     //     Expr::StringLiteral(val) => {
            //     //         println!("Error with if condition on line {}:\n Cannot use string as condition", lineNum.clone());
            //     //         return false;
            //     //     }
                    
                    
            //     //     Expr::ProcRef(procName, params) => {
            //     //         let mut procType: VarType;
            //     //         //Checks if procedure is defined
            //     //         let checkLocProc = self.localTable.getType(&procName.clone());
            //     //         match checkLocProc{
            //     //             Some(proc) => {
            //     //                 procType = proc;
            //     //             }
            //     //             None => {
            //     //                 let checkGlobProc = self.localTable.getType(&procName.clone());
            //     //                 match checkGlobProc{
            //     //                     Some(proc) => {
            //     //                         procType = proc
            //     //                     }
            //     //                     None => {
            //     //                         println!("Error on line {}:\n Procedure {} is not defined", lineNum.clone(), procName.clone());
            //     //                         return false;
                                        
            //     //                     }
            //     //                 }
            //     //             }
            //     //         }
                    
            //     //         //Checks procedure type compatability with int
            //     //         match procType{
            //     //             VarType::Bool =>{
            //     //                 println!("Procedure type bool");
            //     //             }
            //     //             VarType::Int =>{
            //     //                 println!("Procedure type int");
            //     //             }
            //     //             VarType::Float =>{
            //     //                 println!("Error with if condition on line {}:\n Cannot use float procedure as condition", lineNum.clone());
            //     //         return false;
            //     //             }
            //     //             _ => {
            //     //                 println!("Error on line {}:\n Cannot use procedure of type {} as if condition", lineNum.clone(), procType.clone());
            //     //                 return false;
            //     //             }
            //     //         }

            //     //         //Checks if the condition is good
            //     //         let goodCond = self.compileExpr(condition.clone());
            //     //         //If the condition is bad, fails here
            //     //         if (!goodCond){
            //     //             println!("Error in if condition on line {}", lineNum.clone());
            //     //             return false;
            //     //         //If the condition is good, checks the rest of the if statement
            //     //         } else {
            //     //             //Checks the if body
            //     //             let goodIfBody = self.compileStmt(*body);
            //     //             //If the body if good
            //     //             if(goodIfBody){
            //     //                 //Checks if there is an else
            //     //                 match elseBody{
            //     //                     //Checks the else
            //     //                     Some(elseStmt) => {
            //     //                         let goodElse = self.compileStmt(*elseStmt.clone());
            //     //                         if(!goodElse){
            //     //                             println!("Error with else in if statement on line {}", lineNum.clone());
            //     //                             return false;
            //     //                         } else {
            //     //                             return true;
            //     //                         }
            //     //                     }
            //     //                     //If statement is good here if no else
            //     //                     None => {
            //     //                         return true;
            //     //                     }

            //     //                 }
            //     //             } else {
            //     //                 println!("Error with body of if statement on line: {}", lineNum.clone());
            //     //                 return false;
            //     //             }
            //     //         }


            //     //     }   
                    
            //     //     Expr::VarRef(varCondName) => {
            //     //         println!("Assigning: variable {}", varCondName.clone());
            //     //         let mut ifCondType: VarType;
            //     //         //Checks if variable is defined
            //     //         let checkLocVar = self.localTable.getType(&varCondName.clone());
            //     //         match checkLocVar{
            //     //             Some(var) => {
            //     //                 println!("variable exists locally");
            //     //                 ifCondType = var;
            //     //             }
            //     //             None => {
            //     //                 println!("Variable does not exist locally, checking global");
            //     //                 let checkGlobVar = self.localTable.getType(&varCondName.clone());
            //     //                 match checkGlobVar{
            //     //                     Some(var) => {
            //     //                         println!("Variable exists globally");
            //     //                         ifCondType = var
            //     //                     }
            //     //                     None => {
            //     //                         println!("Error on line {}:\n Variable {} is not defined", lineNum.clone(), varCondName.clone());
            //     //                         return false;
                                        
            //     //                     }
            //     //                 }
            //     //             }
            //     //         }
                    
            //     //         //Checks variable type compatability with int
            //     //         match ifCondType{
            //     //             VarType::Bool =>{
            //     //                 println!("Variable type bool");
            //     //             }
            //     //             VarType::Int =>{
            //     //                 println!("Variable type int");
            //     //             }
            //     //             VarType::Float =>{
            //     //                 println!("Error on line {}:\n Cannot use variable of type float as if condition", lineNum.clone());
            //     //                 return false;
            //     //             }
            //     //             _ => {
            //     //                 println!("Error on line {}:\n Cannot use variable of type {} for if condition", lineNum.clone(), ifCondType.clone());
            //     //                 return false;
            //     //             }
            //     //         }

            //     //         //Checks if the condition is good
            //     //         let goodCond = self.compileExpr(condition.clone());
            //     //         //If the condition is bad, fails here
            //     //         if (!goodCond){
            //     //             println!("Error in if condition on line {}", lineNum.clone());
            //     //             return false;
            //     //         //If the condition is good, checks the rest of the if statement
            //     //         } else {
            //     //             //Checks the if body
            //     //             let goodIfBody = self.compileStmt(*body);
            //     //             //If the body if good
            //     //             if(goodIfBody){
            //     //                 //Checks if there is an else
            //     //                 match elseBody{
            //     //                     //Checks the else
            //     //                     Some(elseStmt) => {
            //     //                         let goodElse = self.compileStmt(*elseStmt.clone());
            //     //                         if(!goodElse){
            //     //                             println!("Error with else in if statement on line {}", lineNum.clone());
            //     //                             return false;
            //     //                         } else {
            //     //                             return true;
            //     //                         }
            //     //                     }
            //     //                     //If statement is good here if no else
            //     //                     None => {
            //     //                         return true;
            //     //                     }

            //     //                 }
            //     //             } else {
            //     //                 println!("Error with body of if statement on line: {}", lineNum.clone());
            //     //                 return false;
            //     //             }
            //     //         }
            //     //     }
                

                    
            //     //     //All of the good conditions
            //     //     _ => {
            //     //         //Checks if the condition is good
            //     //         let goodCond = self.compileExpr(condition.clone());
            //     //         //If the condition is bad, fails here
            //     //         if (!goodCond){
            //     //             println!("Error in if condition on line {}", lineNum.clone());
            //     //             return false;
            //     //         //If the condition is good, checks the rest of the if statement
            //     //         } else {
            //     //             //Checks the if body
            //     //             let goodIfBody = self.compileStmt(*body);
            //     //             //If the body if good
            //     //             if(goodIfBody){
            //     //                 //Checks if there is an else
            //     //                 match elseBody{
            //     //                     //Checks the else
            //     //                     Some(elseStmt) => {
            //     //                         let goodElse = self.compileStmt(*elseStmt.clone());
            //     //                         if(!goodElse){
            //     //                             println!("Error with else in if statement on line {}", lineNum.clone());
            //     //                             return false;
            //     //                         } else {
            //     //                             return true;
            //     //                         }
            //     //                     }
            //     //                     //If statement is good here if no else
            //     //                     None => {
            //     //                         return true;
            //     //                     }

            //     //                 }
            //     //             } else {
            //     //                 println!("Error with body of if statement on line: {}", lineNum.clone());
            //     //                 return false;
            //     //             }
            //     //         }
            //     //     }
            //     // }
            
                
            // }    
            // Stmt::For(assignment, condition, body, lineNum) => {
            //     println!("for needs written");
            //     return true;
            //     // //Checks if the condition is valid
            //     // let checked = self.compileExpr(condition.clone());
            //     // if checked {
            //     //     //Continue
            //     // } else {
            //     //     println!("Error with for condition on line {}", lineNum.clone());
            //     //     return false;
            //     // }

            //     // //Ensures for condition is the correct type
            //     // match condition.clone() {
            //     //     Expr::IntArrayLiteral(size, array) => {
            //     //         println!("Error with if condition on line {}:\n Cannot use array as condition", lineNum.clone());
            //     //         return false;
            //     //     }
            //     //     Expr::FloatLiteral(val) => {
            //     //         println!("Error with if condition on line {}:\n Cannot use float as condition", lineNum.clone());
            //     //         return false;
            //     //     }
            //     //     Expr::StringLiteral(val) => {
            //     //         println!("Error with if condition on line {}:\n Cannot use string as condition", lineNum.clone());
            //     //         return false;
            //     //     }
                    
                    
            //     //     Expr::ProcRef(procName, params) => {
            //     //         println!("If condition procedure {}", procName.clone());
            //     //         let mut procType: VarType;
            //     //         //Checks if procedure is defined
            //     //         let checkLocProc = self.localTable.getType(&procName.clone());
            //     //         match checkLocProc{
            //     //             Some(proc) => {
            //     //                 procType = proc;
            //     //             }
            //     //             None => {
            //     //                 let checkGlobProc = self.localTable.getType(&procName.clone());
            //     //                 match checkGlobProc{
            //     //                     Some(proc) => {
            //     //                         procType = proc
            //     //                     }
            //     //                     None => {
            //     //                         println!("Error on line {}:\n Procedure {} is not defined", lineNum.clone(), procName.clone());
            //     //                         return false;
                                        
            //     //                     }
            //     //                 }
            //     //             }
            //     //         }
                    
            //     //         //Checks procedure type compatability with int
            //     //         match procType{
            //     //             VarType::Bool =>{
            //     //                 println!("Procedure type bool");
            //     //             }
            //     //             VarType::Int =>{
            //     //                 println!("Procedure type int");
            //     //             }
            //     //             VarType::Float =>{
            //     //                 println!("Error with for condition on line {}:\n Cannot use float procedure as condition", lineNum.clone());
            //     //         return false;
            //     //             }
            //     //             _ => {
            //     //                 println!("Error on line {}:\n Cannot use procedure of type {} as for condition", lineNum.clone(), procType.clone());
            //     //                 return false;
            //     //             }
            //     //         }
            //     //     }   
                    
            //     //     Expr::VarRef(varCondName) => {
            //     //         println!("Assigning: variable {}", varCondName.clone());
            //     //         let mut forCondType: VarType;
            //     //         //Checks if variable is defined
            //     //         let checkLocVar = self.localTable.getType(&varCondName.clone());
            //     //         match checkLocVar{
            //     //             Some(var) => {
            //     //                 println!("variable exists locally");
            //     //                 forCondType = var;
            //     //             }
            //     //             None => {
            //     //                 println!("Variable does not exist locally, checking global");
            //     //                 let checkGlobVar = self.localTable.getType(&varCondName.clone());
            //     //                 match checkGlobVar{
            //     //                     Some(var) => {
            //     //                         println!("Variable exists globally");
            //     //                         forCondType = var
            //     //                     }
            //     //                     None => {
            //     //                         println!("Error on line {}:\n Variable {} is not defined", lineNum.clone(), varCondName.clone());
            //     //                         return false;
                                        
            //     //                     }
            //     //                 }
            //     //             }
            //     //         }
                    
            //     //         //Checks variable type compatability with int
            //     //         match forCondType{
            //     //             VarType::Bool =>{
            //     //                 println!("Variable type bool");
            //     //             }
            //     //             VarType::Int =>{
            //     //                 println!("Variable type int");
            //     //             }
            //     //             VarType::Float =>{
            //     //                 println!("Error on line {}:\n Cannot use variable of type float as for condition", lineNum.clone());
            //     //                 return false;
            //     //             }
            //     //             _ => {
            //     //                 println!("Error on line {}:\n Cannot use variable of type {} as for condition", lineNum.clone(), forCondType.clone());
            //     //                 return false;
            //     //             }
            //     //         }
            //     //     }
                

                    
            //     //     //All of the good conditions
            //     //     _ => {
            //     //         //Checks if the condition is good
            //     //         let goodCond = self.compileExpr(condition.clone());
            //     //         //If the condition is bad, fails here
            //     //         if (!goodCond){
            //     //             println!("Error in if condition on line {}", lineNum.clone());
            //     //             return false;
            //     //         //If the condition is good, checks the rest of the if statement
            //     //         } else {
            //     //             //continue
            //     //         }
            //     //     }
            //     // }

            //     // //Checks the for body
            //     // let forBodyCheck = self.compileStmt(*body);
            //     // //If the body for good
            //     // if(forBodyCheck){
            //     //     //Checks for there is an else
            //     //     return true;
            //     // } else {
            //     //     println!("Error with body of for statement on line: {}", lineNum.clone());
            //     //     return false;
            //     // }
            
            
            
            // }  
            // Stmt::Block(stmts, lineNum) => {
            //     for instr in stmts {
            //         let good = self.compileStmt(instr.clone());
            //         if (!good){
            //             println!("Error in header:");
            //             instr.display(0);
            //             return false;
            //         } else {
            //             //continue
            //         }
            //     }
            //     return true;
            // }
            // Stmt::Error(report, errMsg) => {
            //     println!("Error found in AST: {}", errMsg);
            //     return false;
            // }
            // Stmt::Program(name, header, body, lineNum) => {
            //     return true;
            // }
            // Stmt::Return(retVal, lineNum) => {
            //     let checked = self.compileExpr(retVal.clone());
            //     match checked{
            //         Ok(res) => {
            //             println!("Return good, needs written");
            //             return true;
            //         }
            //         Err(err) => {
            //             println!("{}", err);
            //             return false;
            //         }
            //     }
            // }
            // Stmt::StringLiteral(val, lineNum) => {
            //     return true;
            // }
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
    

    let context = Context::create();
    // let mut module = context.create_module("my_module");
    // let mut builder = context.create_builder();

    //Creates the llvm context and the code generator struct

    let progAst = programAst.clone();
    // let program_ast = /* Initialize your AST here */;

    let mut compiler = Compiler::new(
        progAst,
        &context,
        &mut global_table,
        "Program".to_string()
    );


    println!("Created generator");
    let ret = compiler.compileProgram();
    match ret{
        Ok(module) => {
            println!("Module generated");
            // module.print_to_string();
            // module.print_to_stderr();
        }
        Err(errMsg) => {
            println!("Error with generation: {}", errMsg);
        }
    }

    drop(compiler);
    // drop(builder);
    // drop(module);


    Ok(())
}