//! A minimal toy Scheme: a VM-based interpreter.

mod compiler;
mod errors;
mod lexer;
mod parser;
mod symbol;
mod value;
mod vm;

use std::collections::HashMap;

use indexmap::IndexSet;

use crate::compiler::Compiler;
use crate::vm::VM;

pub use crate::errors::Error;
pub use crate::value::Value;

/// Runs Scheme source code, keeping definitions between calls.
pub struct Interpreter {
    compiler: Compiler,
    vm: VM,
    env: Vec<Vec<Value>>,
    constants: IndexSet<Value>,
}

impl Interpreter {
    /// Creates an interpreter with an empty global environment.
    pub fn new() -> Self {
        Self {
            compiler: Compiler::new(),
            vm: VM::new(HashMap::new()),
            env: Vec::new(),
            constants: IndexSet::new(),
        }
    }

    /// Evaluates every expression in `source` and returns the value of the
    /// last one, or `()` if `source` contains no expressions.
    ///
    /// # Errors
    ///
    /// Returns an error if `source` cannot be read or compiled, or if an
    /// expression fails at run time. Expressions before the failing one stay
    /// evaluated.
    pub fn eval(&mut self, source: &str) -> Result<Value, Error> {
        let mut result = Value::Nil;
        for expression in parser::read(source)? {
            let code = self
                .compiler
                .compile_toplevel(&expression, &mut self.constants)?;
            result = self
                .vm
                .interpret(code, &mut self.env, &mut self.constants)?;
        }
        Ok(result)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
