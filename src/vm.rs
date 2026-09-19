use std::collections::HashMap;

use indexmap::IndexSet;
use internment::Intern;

use crate::errors::InterpreterError;
use crate::value::{Func, Value};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OpCode {
    Return,
    LoadNil,
    LoadNumber(i32),
    LoadConst(usize),
    GlobalGet(usize),
    LocalGet(usize, usize),
    SetGlobal(usize),
    SetLocal(usize, usize),
    Apply(usize),
    Jump(usize),
    JumpIfFalse(usize),
    NewEnv(usize),
}

struct VM {
    global_env: HashMap<Intern<str>, Value>,
    stack: Vec<Value>,
}

impl VM {
    fn new(global_env: HashMap<Intern<str>, Value>) -> Self {
        Self {
            global_env,
            stack: vec![],
        }
    }

    fn interpret(
        &mut self,
        code: Vec<OpCode>,
        env: &mut Vec<Vec<Value>>,
        constants: &mut IndexSet<Value>,
    ) -> Result<Value, InterpreterError> {
        let mut ip: usize = 0;
        loop {
            match code.get(ip).unwrap() {
                OpCode::Return => {
                    return Ok(self.stack.pop().unwrap());
                }
                OpCode::LoadNil => self.stack.push(Value::Nil),
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
                OpCode::LocalGet(frame, index) => self
                    .stack
                    .push(env.get(*frame).unwrap().get(*index).unwrap().clone()),
                OpCode::GlobalGet(index) => {
                    if let Value::Symbol(ident) = constants.get_index(*index).unwrap() {
                        self.stack.push(self.global_env.get(ident).unwrap().clone());
                    } else {
                        unreachable!();
                    }
                }
                OpCode::SetLocal(frame, index) => {
                    env[*frame][*index] = self.stack.pop().unwrap().clone();
                }
                OpCode::SetGlobal(index) => {
                    if let Value::Symbol(ident) = constants.get_index(*index).unwrap() {
                        self.global_env
                            .insert(*ident, self.stack.pop().unwrap().clone());
                    } else {
                        unreachable!();
                    }
                }
                OpCode::NewEnv(capacity) => env.push(Vec::with_capacity(*capacity)),
                OpCode::Apply(args_len) => {
                    let mut args = vec![];
                    for _ in 0..*args_len {
                        args.push(self.stack.pop().unwrap());
                    }
                    let Value::Func(Func::Closure {
                        env_pointer,
                        arity,
                        code: body,
                    }) = self.stack.pop().unwrap()
                    else {
                        panic!();
                    };
                    if arity != *args_len {
                        panic!();
                    }
                    env.get_mut(env_pointer)
                        .unwrap()
                        .extend(args.into_iter().rev());
                    self.interpret(body, env, constants)?;
                    env.get_mut(env_pointer).unwrap().clear();
                }
            };
            ip += 1
        }
    }
}
