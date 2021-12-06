use std::collections::HashMap;
use std::rc::Rc;

use crate::code::Code;
use crate::code::FnDef;
use crate::code::Instr::*;
use crate::debug;
use crate::parse::CoParser;
use crate::value::Value;

pub enum CoRes {
    Ok,
    CompileErr,
    RuntimeErr,
}

pub struct CoVM;

impl CoVM {
    pub fn run(&mut self, _src: &str) -> CoRes {
        if cfg!(feature = "debug") {
            CoParser::parse(r##"
                # a comment
                # another line comment
                1 2 3.14
                a b abc
                true false
                "" "a" "foo"
                ()
            #last comment"##);

            let mut code = Code::new();
            code.add(OpUnit, 1);
            code.add(OpPrint, 1);
            code.add(OpPop, 1);
            code.add(OpTrue, 2);
            code.add(OpPrint, 2);
            code.add(OpPop, 2);
            code.add(OpFalse, 3);
            code.add(OpPrint, 3);
            code.add(OpPop, 3);
            debug::print(&code, "code");

            let mut def = FnDef::new();
            def.code = code;
            def.print();
            println!();

            let mut co = Coro::new(Rc::new(def));
            co.print();
            println!();

            if let Err(msg) = co.resume(Vec::new()) {
                eprintln!("[coro] error: {}", msg);
                return CoRes::RuntimeErr;
            }

            co.print();
            println!();
        }
        CoRes::Ok
    }
}

#[derive(Debug, PartialEq)]
pub enum CoStatus {
    Suspended,
    Running,
    Done,
}

pub struct Coro {
    status: CoStatus,
    ip: usize,
    fun: Rc<FnDef>,
    env: HashMap<String, Value>,
    stack: Vec<Value>,
}

impl Coro {
    pub fn new(fun: Rc<FnDef>) -> Self {
        Self {
            status: CoStatus::Suspended,
            ip: 0,
            fun,
            env: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn print(&self) {
        let status = format!("{:?}", self.status);
        let status = status.to_lowercase();
        print!("<co fn:{} status:{}>", self.fun.name(), status);
    }

    pub fn resume(&mut self, args: Vec<Value>) -> Result<Value, String> {
        self.check_status()?;
        self.handle_inputs(args)?;

        self.status = CoStatus::Running;
        if cfg!(feature = "debug") {
            self.print();
            println!();
        }

        self.exec()?;

        if self.ip >= self.fun.code.len() {
            self.status = CoStatus::Done;
        }

        let res = if !self.stack.is_empty() {
            self.stack.pop().unwrap()
        } else {
            Value::Unit
        };
        Ok(res)
    }

    fn exec(&mut self) -> Result<(), String> {
        let code_len = self.fun.code.len();
        while self.ip < code_len {
            let instr = self.fun.code.instr(self.ip);
            self.ip += 1;
            match instr {
                OpUnit => self.stack.push(Value::Unit),
                OpTrue => self.stack.push(Value::Bool(true)),
                OpFalse => self.stack.push(Value::Bool(false)),
                OpPrint => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(Value::Unit);
                    val.print();
                    println!();
                }
                OpPop => {
                    self.stack.pop();
                }
            }
        }
        Ok(())
    }

    fn check_status(&self) -> Result<(), String> {
        if self.status != CoStatus::Suspended {
            Err(format!("tried to resume a non-suspended coroutine"))
        } else {
            Ok(())
        }
    }

    fn check_arity(&self, arity: usize, args_len: usize) -> Result<(), String> {
        if arity != args_len {
            Err(format!(
                "expected {} arguments but got {} when resuming coroutine",
                arity,
                args_len
            ))
        } else {
            Ok(())
        }
    }

    fn handle_inputs(&mut self, args: Vec<Value>) -> Result<(), String> {
        if self.ip == 0 {
            // First time calling coroutine, so setup the function arguments.
            let arity = self.fun.arity();
            self.check_arity(arity, args.len())?;
            for (i, arg) in args.into_iter().enumerate() {
                let param = self.fun.params[i].clone();
                self.env.insert(param, arg);
            }
        } else {
            // Only one value should be yielded, and we push this onto the stack.
            self.check_arity(1, args.len())?;
            self.stack.push(args.into_iter().next().unwrap());
        }
        Ok(())
    }
}
