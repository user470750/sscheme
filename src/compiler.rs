use std::rc::Rc;

use crate::errors::CompilerError;
use crate::symbol::Symbol;
use crate::value::{Proto, Value};
use crate::vm::OpCode;
use indexmap::IndexSet;

pub(crate) struct Compiler {
    /// Parameter names of the `lambda`s being compiled, innermost last.
    scopes: Vec<Vec<Symbol>>,
}

impl Compiler {
    pub(crate) fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub(crate) fn compile_toplevel(
        &mut self,
        source: &Value,
        constants: &mut IndexSet<Value>,
    ) -> Result<Vec<OpCode>, CompilerError> {
        let mut code = vec![];
        self.compile(source, constants, &mut code)?;
        code.push(OpCode::Return);
        Ok(code)
    }

    fn compile(
        &mut self,
        source: &Value,
        constants: &mut IndexSet<Value>,
        code: &mut Vec<OpCode>,
    ) -> Result<(), CompilerError> {
        match source {
            Value::Nil => code.push(OpCode::LoadNil),
            Value::Number(num) => code.push(OpCode::LoadNumber(*num)),
            Value::Symbol(ident) => code.push(self.compile_identifier(*ident)),
            Value::List(list) => self.compile_list(list, constants, code)?,
            _ => unreachable!(),
        }
        Ok(())
    }

    fn compile_identifier(&self, ident: Symbol) -> OpCode {
        match self.resolve(ident) {
            Some((depth, index)) => OpCode::LocalGet(depth, index),
            None => OpCode::GlobalGet(ident),
        }
    }

    /// Finds a local variable as the number of frames to go up and its index
    /// in that frame, or `None` for a global.
    fn resolve(&self, ident: Symbol) -> Option<(usize, usize)> {
        self.scopes
            .iter()
            .rev()
            .enumerate()
            .find_map(|(depth, scope)| {
                let index = scope.iter().rposition(|local| *local == ident)?;
                Some((depth, index))
            })
    }

    fn compile_list(
        &mut self,
        list: &[Value],
        constants: &mut IndexSet<Value>,
        code: &mut Vec<OpCode>,
    ) -> Result<(), CompilerError> {
        match list.first().ok_or(CompilerError::NilCombination)? {
            Value::Symbol(ident) => match ident.as_ref() {
                "quote" => {
                    if list.len() != 2 {
                        return Err(CompilerError::ArityError {
                            exp_name: "quote",
                            expected: "1",
                            passed: list.len() - 1,
                        });
                    }
                    code.push(OpCode::LoadConst(
                        constants.insert_full(list.get(1).unwrap().clone()).0,
                    ));
                }
                "set" => self.compile_set_exp(list, constants, code)?,
                "lambda" => self.compile_closure(list, constants, code)?,
                "if" => self.compile_if_exp(list, constants, code)?,
                _ => self.compile_call(list, constants, code)?,
            },
            _ => self.compile_call(list, constants, code)?,
        }
        Ok(())
    }

    fn compile_set_exp(
        &mut self,
        exp: &[Value],
        constants: &mut IndexSet<Value>,
        code: &mut Vec<OpCode>,
    ) -> Result<(), CompilerError> {
        if exp.len() != 3 {
            return Err(CompilerError::ArityError {
                exp_name: "set",
                expected: "2",
                passed: exp.len() - 1,
            });
        }
        let Value::Symbol(ident) = exp.get(1).unwrap() else {
            return Err(CompilerError::WrongArgument {
                exp_name: "set",
                expected: "identifier",
            });
        };
        self.compile(exp.get(2).unwrap(), constants, code)?;
        match self.resolve(*ident) {
            Some((depth, index)) => code.push(OpCode::SetLocal(depth, index)),
            None => code.push(OpCode::SetGlobal(*ident)),
        }
        code.push(OpCode::LoadNil);
        Ok(())
    }

    fn compile_closure(
        &mut self,
        lambda: &[Value],
        constants: &mut IndexSet<Value>,
        code: &mut Vec<OpCode>,
    ) -> Result<(), CompilerError> {
        if lambda.len() != 3 {
            return Err(CompilerError::ArityError {
                exp_name: "lambda",
                expected: "2",
                passed: lambda.len() - 1,
            });
        }
        let params: &[Value] = match lambda.get(1).unwrap() {
            Value::Nil => &[],
            Value::List(params) => params,
            _ => {
                return Err(CompilerError::WrongArgument {
                    exp_name: "lambda",
                    expected: "list of identifiers",
                });
            }
        };
        let mut args = vec![];
        for ident in params {
            let Value::Symbol(arg_name) = ident else {
                return Err(CompilerError::WrongArgument {
                    exp_name: "lambda",
                    expected: "list of identifiers",
                });
            };
            args.push(*arg_name);
        }
        let mut body = vec![];
        self.scopes.push(args);
        let compiled = self.compile(lambda.get(2).unwrap(), constants, &mut body);
        self.scopes.pop();
        compiled?;
        body.push(OpCode::Return);
        code.push(OpCode::MakeClosure(Rc::new(Proto {
            arity: params.len(),
            code: body,
        })));
        Ok(())
    }

    fn compile_if_exp(
        &mut self,
        if_exp: &[Value],
        constants: &mut IndexSet<Value>,
        code: &mut Vec<OpCode>,
    ) -> Result<(), CompilerError> {
        if if_exp.len() < 3 || if_exp.len() > 4 {
            return Err(CompilerError::ArityError {
                exp_name: "if",
                expected: "2 or 3",
                passed: if_exp.len() - 1,
            });
        }
        self.compile(if_exp.get(1).unwrap(), constants, code)?;
        code.push(OpCode::JumpIfFalse(0));

        let mut start_len = code.len();

        self.compile(if_exp.get(2).unwrap(), constants, code)?;
        code.push(OpCode::Jump(1));

        let mut end_len = code.len();
        code[start_len - 1] = OpCode::JumpIfFalse(end_len - start_len);
        if let Some(alt) = if_exp.get(3) {
            start_len = code.len();
            self.compile(alt, constants, code)?;

            end_len = code.len();
            code[start_len - 1] = OpCode::Jump(end_len - start_len);
        } else {
            code.push(OpCode::LoadNil);
        }
        Ok(())
    }

    fn compile_call(
        &mut self,
        list: &[Value],
        constants: &mut IndexSet<Value>,
        code: &mut Vec<OpCode>,
    ) -> Result<(), CompilerError> {
        self.compile(list.first().unwrap(), constants, code)?;
        for arg in &list[1..] {
            self.compile(arg, constants, code)?;
        }
        code.push(OpCode::Apply(list.len() - 1));
        Ok(())
    }
}
