use crate::errors::CompilerError;
use crate::value::{Func, Value};
use crate::vm::OpCode;
use indexmap::IndexSet;
use internment::Intern;

struct Compiler {
    env_pointer: Option<usize>,
    env: Vec<(Option<usize>, Vec<Intern<str>>)>,
}

impl Compiler {
    fn new() -> Self {
        Self {
            env_pointer: Some(0),
            env: vec![],
        }
    }

    fn compile_toplevel(
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
            Value::Symbol(ident) => code.push(self.compile_identifier(*ident, constants)),
            Value::List(list) => self.compile_list(list, constants, code)?,
            _ => unreachable!(),
        }
        Ok(())
    }

    fn compile_identifier(&self, ident: Intern<str>, constants: &mut IndexSet<Value>) -> OpCode {
        let mut pointer = self.env_pointer;
        while let Some(p) = pointer {
            let frame = self.env.get(p).unwrap(); // pointer is always valid
            pointer = frame.0;
            for (i, local) in frame.1.iter().enumerate().rev() {
                if *local == ident {
                    return OpCode::LocalGet(p, i);
                }
            }
        }
        OpCode::GlobalGet(constants.insert_full(Value::Symbol(ident)).0)
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
                            expected: 1,
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
                expected: 2,
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
        let mut pointer = self.env_pointer;
        while let Some(p) = pointer {
            let frame = self.env.get(p).unwrap(); // pointer is always valid
            pointer = frame.0;
            for (i, local) in frame.1.iter().enumerate().rev() {
                if local == ident {
                    code.push(OpCode::SetLocal(p, i));
                    return Ok(());
                }
            }
        }
        code.push(OpCode::SetGlobal(
            constants.insert_full(Value::Symbol(*ident)).0,
        ));
        code.push(OpCode::LoadNil);
        Ok(())
    }

    fn compile_closure(
        &mut self,
        lambda: &[Value],
        constants: &mut IndexSet<Value>,
        code: &mut Vec<OpCode>
    ) -> Result<(), CompilerError> {
        if lambda.len() != 3 {
            return Err(CompilerError::ArityError {
                exp_name: "lambda",
                expected: 2,
                passed: lambda.len() - 1,
            });
        }
        let Value::List(idenifier_list) = lambda.get(1).unwrap() else {
            return Err(CompilerError::WrongArgument {
                exp_name: "lambda",
                expected: "list of identifiers",
            });
        };
        let mut args = vec![];
        for ident in idenifier_list {
            if let Value::Symbol(arg_name) = ident {
                args.push(*arg_name);
            }
        }
        let pre_env_pointer = self.env_pointer;
        self.env.push((pre_env_pointer, args));
        self.env_pointer = Some(self.env.len() - 1);
        let mut body = vec![];
        self.compile(lambda.get(2).unwrap(), constants, &mut body)?;
        body.push(OpCode::Return);
        let closure = constants
            .insert_full(Value::Func(Func::Closure {
                env_pointer: self.env.len() - 1,
                arity: idenifier_list.len(),
                code: body,
            }))
            .0;
        self.env_pointer = pre_env_pointer;
        code.push(OpCode::NewEnv(idenifier_list.len()));
        code.push(OpCode::LoadConst(closure));
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
                expected: 2,
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
