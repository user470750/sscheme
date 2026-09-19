use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexSet;

use crate::env::{Env, Frame};
use crate::errors::InterpreterError;
use crate::symbol::Symbol;
use crate::value::{Func, Proto, Value};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OpCode {
    Return,
    LoadNil,
    LoadBool(bool),
    LoadNumber(i32),
    LoadConst(usize),
    GlobalGet(Symbol),
    LocalGet(usize, usize),
    SetGlobal(Symbol),
    SetLocal(usize, usize),
    Apply(usize),
    Jump(usize),
    JumpIfFalse(usize),
    MakeClosure(Rc<Proto>),
}

pub(crate) struct VM {
    global_env: HashMap<Symbol, Value>,
    stack: Vec<Value>,
}

impl VM {
    pub(crate) fn new(global_env: HashMap<Symbol, Value>) -> Self {
        Self {
            global_env,
            stack: vec![],
        }
    }

    /// Runs top-level code. After an error the stack is cleared, so the VM
    /// can run the next expression.
    pub(crate) fn run(
        &mut self,
        code: &[OpCode],
        constants: &mut IndexSet<Value>,
    ) -> Result<Value, InterpreterError> {
        let result = self.interpret(code, None, constants);
        if result.is_err() {
            self.stack.clear();
        }
        result
    }

    fn interpret(
        &mut self,
        code: &[OpCode],
        env: Option<&Env>,
        constants: &mut IndexSet<Value>,
    ) -> Result<Value, InterpreterError> {
        let mut ip: usize = 0;
        loop {
            match code.get(ip).unwrap() {
                OpCode::Return => {
                    return Ok(self.stack.pop().unwrap());
                }
                OpCode::LoadNil => self.stack.push(Value::Nil),
                OpCode::LoadBool(boolean) => self.stack.push(Value::Bool(*boolean)),
                OpCode::LoadNumber(number) => self.stack.push(Value::Number(*number)),
                OpCode::LoadConst(index) => self
                    .stack
                    .push(constants.get_index(*index).unwrap().clone()),
                OpCode::JumpIfFalse(addr) => {
                    if let Value::Nil = self.stack.pop().unwrap() {
                        ip += addr;
                    }
                }
                OpCode::Jump(addr) => ip += addr,
                OpCode::LocalGet(depth, index) => {
                    self.stack.push(locals(env).get(*depth, *index));
                }
                OpCode::GlobalGet(ident) => {
                    let value = self
                        .global_env
                        .get(ident)
                        .ok_or(InterpreterError::UnboundVariable(*ident))?;
                    self.stack.push(value.clone());
                }
                OpCode::SetLocal(depth, index) => {
                    locals(env).set(*depth, *index, self.stack.pop().unwrap());
                }
                OpCode::SetGlobal(ident) => {
                    self.global_env.insert(*ident, self.stack.pop().unwrap());
                }
                OpCode::MakeClosure(proto) => self.stack.push(Value::Func(Func::Closure {
                    proto: proto.clone(),
                    env: env.cloned(),
                })),
                OpCode::Apply(args_len) => {
                    let args = self.stack.split_off(self.stack.len() - args_len);
                    let (callee, captured) = match self.stack.pop().unwrap() {
                        Value::Func(Func::Closure { proto, env }) => (proto, env),
                        other => return Err(InterpreterError::NotProcedure(other.to_string())),
                    };
                    if callee.arity != *args_len {
                        return Err(InterpreterError::ArityMismatch {
                            expected: callee.arity,
                            passed: *args_len,
                        });
                    }
                    let frame = Frame::new(args, captured);
                    let result = self.interpret(&callee.code, Some(&frame), constants)?;
                    self.stack.push(result);
                }
            };
            ip += 1
        }
    }
}

/// Returns the frame of the running function; top-level code has no locals.
fn locals(env: Option<&Env>) -> &Env {
    env.expect("compiler emits local access only inside functions")
}
