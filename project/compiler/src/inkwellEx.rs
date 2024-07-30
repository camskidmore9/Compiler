use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter::Peekable;
use std::ops::DerefMut;
use std::str::Chars;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::FloatPredicate;

use inkwell_internals::llvm_versions;

use crate::Token::*;

const ANONYMOUS_FUNCTION_NAME: &str = "anonymous";

// ======================================================================================
// LEXER ================================================================================
// ======================================================================================

/// Represents a primitive syntax token.
#[derive(Debug, Clone)]
pub enum Token {
    Binary,
    Comma,
    Comment,
    Def,
    Else,
    EOF,
    Extern,
    For,
    Ident(String),
    If,
    In,
    LParen,
    Number(f64),
    Op(char),
    RParen,
    Then,
    Unary,
    Var,
}

/// Defines an error encountered by the `Lexer`.
pub struct LexError {
    pub error: &'static str,
    pub index: usize,
}

impl LexError {
    pub fn new(msg: &'static str) -> LexError {
        LexError { error: msg, index: 0 }
    }

    pub fn with_index(msg: &'static str, index: usize) -> LexError {
        LexError { error: msg, index }
    }
}

/// Defines the result of a lexing operation; namely a
/// `Token` on success, or a `LexError` on failure.
pub type LexResult = Result<Token, LexError>;

/// Defines a lexer which transforms an input `String` into
/// a `Token` stream.
pub struct Lexer<'a> {
    input: &'a str,
    chars: Box<Peekable<Chars<'a>>>,
    pos: usize,
}

impl<'a> Lexer<'a> {
}

impl<'a> Iterator for Lexer<'a> {
}

// ======================================================================================
// PARSER ===============================================================================
// ======================================================================================

/// Defines a primitive expression.
#[derive(Debug)]
pub enum Expr {
}

/// Defines the prototype (name and parameters) of a function.
#[derive(Debug)]
pub struct Prototype {
}

/// Defines a user-defined or external function.
#[derive(Debug)]
pub struct Function {
}

/// Represents the `Expr` parser.
pub struct Parser<'a> {
}

// I'm ignoring the 'must_use' lint in order to call 'self.advance' without checking
// the result when an EOF is acceptable.
#[allow(unused_must_use)]
impl<'a> Parser<'a> {
}

// ======================================================================================
// COMPILER =============================================================================
// ======================================================================================

/// Defines the `Expr` compiler.
pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub function: &'a Function,

    variables: HashMap<String, PointerValue<'ctx>>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    /// Gets a defined function given its name.
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

    /// Returns the `FunctionValue` representing the function being compiled.
    #[inline]
    fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }

    /// Creates a new stack allocation instruction in the entry block of the function.
    fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
    }

    #[llvm_versions(..=14)]
    pub fn build_load(&self, ptr: PointerValue<'ctx>, name: &str) -> BasicValueEnum<'ctx> {
        self.builder.build_load(ptr, name).unwrap()
    }

    #[llvm_versions(15..)]
    pub fn build_load(&self, ptr: PointerValue<'ctx>, name: &str) -> BasicValueEnum<'ctx> {
        self.builder.build_load(self.context.f64_type(), ptr, name).unwrap()
    }

    /// Compiles the specified `Expr` into an LLVM `FloatValue`.
    fn compile_expr(&mut self, expr: &Expr) -> Result<FloatValue<'ctx>, &'static str> {
    }

    /// Compiles the specified `Prototype` into an extern LLVM `FunctionValue`.
    fn compile_prototype(&self, proto: &Prototype) -> Result<FunctionValue<'ctx>, &'static str> {
    }

    /// Compiles the specified `Function` into an LLVM `FunctionValue`.
    fn compile_fn(&mut self) -> Result<FunctionValue<'ctx>, &'static str> {
    }

    /// Compiles the specified `Function` in the given `Context` and using the specified `Builder` and `Module`.
    pub fn compile(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        module: &'a Module<'ctx>,
        function: &Function,
    ) -> Result<FunctionValue<'ctx>, &'static str> {
        let mut compiler = Compiler {
            context,
            builder,
            module,
            function,
            fn_value_opt: None,
            variables: HashMap::new(),
        };

        compiler.compile_fn()
    }
}