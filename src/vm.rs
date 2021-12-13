//! This is the main entry point to executing Coro code.
//! 
//! The main design is the `Coro` struct, which represents a coroutine. In order to support the
//! resume/yield functionalities of coroutines, we designed  coroutine objects to be mostly
//! "self-contained". This means that `Coro` objects manage their own state and executes their
//! own code. Thus, the "virtual machine" (VM) here is just a wrapper/helper for invoking
//! coroutines.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::cgen::CoGen;
use crate::code::Instr::*;
use crate::debug;
use crate::parse::CoParser;
use crate::value::FnDef;
use crate::value::Value;

pub enum CoRes {
    Ok,
    CompileErr,
    RuntimeErr,
}

pub struct CoVM;

impl CoVM {
    pub fn build(src: &str) -> Result<Coro, String> {
        let def = Self::compile(src)?;
        Ok(Coro::new(def))
    }

    pub fn compile(src: &str) -> Result<Rc<FnDef>, String> {
        let ast = match CoParser::parse(src) {
            Ok(tree) => tree,
            Err(e) => return Err(format!("{}", e)),
        };

        if cfg!(feature = "ast") {
            eprintln!("{:?}", ast);
        }

        let code = CoGen::compile(ast);
        let mut def = FnDef::new();
        def.code = code;

        if cfg!(feature = "dbg") {
            eprintln!("{}", def);
        }
        if cfg!(feature = "instr") {
            debug::print(&def.code, def.name());
        }

        Ok(Rc::new(def))
    }

    // Replace with function, reset state, while keeping env.
    // Useful for things like the REPL.
    pub fn rewind(co: &mut Coro, fun: Rc<FnDef>) {
        co.ip = 0;
        co.fun = fun;
        co.status = CoStatus::Suspended;
        co.stack.clear();
    }

    pub fn run(co: &mut Coro) -> Result<Value, String> {
        co.resume(Vec::new())
    }

    pub fn eval(src: &str) -> CoRes {
        let mut co = match Self::build(src) {
            Ok(co) => co,
            Err(e) => {
                eprintln!("[coro] compile error:\n{}", e);
                return CoRes::CompileErr;
            }
        };

        let val = match Self::run(&mut co) {
            Ok(val) => val,
            Err(msg) => {
                eprintln!("[coro] runtime error: {}", msg);
                return CoRes::RuntimeErr;
            }
        };

        if cfg!(feature = "dbg") {
            println!("[coro] value: {}", val);
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
    ip: usize,
    fun: Rc<FnDef>,
    status: CoStatus,
    env: HashMap<String, Value>,
    stack: Vec<Value>,
}

impl fmt::Display for Coro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = format!("{:?}", self.status);
        let status = status.to_lowercase();
        write!(f, "<coro fn: {} status: {}>", self.fun.name(), status)
    }
}

impl Coro {
    pub fn new(fun: Rc<FnDef>) -> Self {
        Self {
            ip: 0,
            fun,
            status: CoStatus::Suspended,
            env: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn resume(&mut self, args: Vec<Value>) -> Result<Value, String> {
        self.check_status()?;
        self.handle_inputs(args)?;

        self.status = CoStatus::Running;
        if cfg!(feature = "dbg") {
            println!("{}", self);
        }

        let res = self.exec()?;
        if self.ip >= self.fun.code.len() {
            self.status = CoStatus::Done;
        }

        if cfg!(feature = "stack") {
            self.debug_stack();
        }
        if cfg!(feature = "dbg") {
            println!("{}", self);
        }

        Ok(res)
    }

    pub fn debug_stack(&self) {
        eprint!("<ip: {:04} stack: [", self.ip);
        for value in &self.stack {
            match value {
                Value::Str(_) => eprint!(" <str>"),
                Value::Fn(_) => eprint!(" <fn>"),
                Value::Co(_) => eprint!(" <co>"),
                _ => eprint!(" {}", value),
            }
        }
        eprintln!(" ]>");
    }

    fn exec(&mut self) -> Result<Value, String> {
        let code_len = self.fun.code.len();
        while self.ip < code_len {
            if cfg!(feature = "stack") {
                self.debug_stack();
            }
            let instr = self.fun.code.instr(self.ip);
            let instr = instr.clone();
            self.ip += 1;
            match instr {
                OpUnit => self.stack.push(Value::Unit),
                OpTrue => self.stack.push(Value::Bool(true)),
                OpFalse => self.stack.push(Value::Bool(false)),
                OpConst(idx) => {
                    let val = self.fun.code.constant(idx);
                    self.stack.push(val.clone());
                }
                OpAdd => {
                    self.check_bin_operands()?;
                    let rhs = self.stack.pop().unwrap().into_num();
                    let lhs = self.stack.pop().unwrap().into_num();
                    let val = Value::Num(lhs + rhs);
                    self.stack.push(val);
                }
                OpSub => {
                    self.check_bin_operands()?;
                    let rhs = self.stack.pop().unwrap().into_num();
                    let lhs = self.stack.pop().unwrap().into_num();
                    let val = Value::Num(lhs - rhs);
                    self.stack.push(val);
                }
                OpMul => {
                    self.check_bin_operands()?;
                    let rhs = self.stack.pop().unwrap().into_num();
                    let lhs = self.stack.pop().unwrap().into_num();
                    let val = Value::Num(lhs * rhs);
                    self.stack.push(val);
                }
                OpDiv => {
                    self.check_bin_operands()?;
                    let rhs = self.stack.pop().unwrap().into_num();
                    let lhs = self.stack.pop().unwrap().into_num();
                    if rhs == 0.0 {
                        return Err("cannot divide by zero".to_owned());
                    }
                    let val = Value::Num(lhs / rhs);
                    self.stack.push(val);
                }
                OpNeg => {
                    self.check_uni_operands()?;
                    let val = self.stack.pop().unwrap().into_num();
                    let val = Value::Num(-val);
                    self.stack.push(val);
                }
                OpNot => {
                    let val = self.stack.pop().unwrap();
                    let val = Value::Bool(val.is_falsey());
                    self.stack.push(val);
                }
                OpLt => {
                    self.check_bin_operands()?;
                    let rhs = self.stack.pop().unwrap().into_num();
                    let lhs = self.stack.pop().unwrap().into_num();
                    let val = Value::Bool(lhs < rhs);
                    self.stack.push(val);
                }
                OpEq => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let val = Value::Bool(lhs == rhs);
                    self.stack.push(val);
                }
                OpLoop(offset) => {
                    self.ip -= offset;
                }
                OpJump(offset) => {
                    self.ip += offset;
                }
                OpBranch(offset) => {
                    if self.peek(0).is_falsey() {
                        self.ip += offset;
                    }
                }
                OpLoad(idx) => {
                    let name = self.fun.code.constant(idx);
                    let name = name.as_str_ref();
                    match self.env.get(name) {
                        Some(val) => self.stack.push(val.clone()),
                        None => return Err(format!("no binding for name '{}'", name)),
                    }
                }
                OpStore(idx) => {
                    let name = self.fun.code.constant(idx);
                    let name = name.clone().into_str();
                    let val = self.stack.pop().unwrap();
                    self.env.insert(name, val);
                    self.stack.push(Value::Unit);
                }
                OpDefine(idx) => {
                    let def = self.fun.code.constant(idx);
                    let def = def.clone().into_fn();
                    let name = def.name().to_owned();
                    let val = Value::Fn(def);
                    self.env.insert(name, val);
                    self.stack.push(Value::Unit);
                }
                OpCreate(idx) => {
                    let name = self.fun.code.constant(idx);
                    let name = name.as_str_ref();
                    let val = match self.env.get(name) {
                        Some(val) => val,
                        None => return Err(format!("no binding for name '{}'", name)),
                    };
                    if !val.is_fn() {
                        return Err(format!("'{}' is not a function", name));
                    }
                    let def = val.clone().into_fn();
                    let coro = Self::new(def);
                    let coro = Rc::new(RefCell::new(coro));
                    self.stack.push(Value::Co(coro))
                }
                OpResume(num) => {
                    let mut args = Vec::with_capacity(num);
                    for _ in 0..num {
                        let val = self.stack.pop().unwrap();
                        args.insert(0, val);
                    }
                    let coro = self.stack.pop().unwrap();
                    if !coro.is_co() {
                        return Err(format!("only coroutines can be resumed"));
                    }
                    let coro = coro.into_co();
                    self.status = CoStatus::Suspended;
                    let val = coro.borrow_mut().resume(args)?;
                    self.status = CoStatus::Running;
                    self.stack.push(val);
                }
                OpYield => {
                    let val = self.stack.pop().unwrap();
                    self.status = CoStatus::Suspended;
                    return Ok(val);
                }
                OpPrint => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(Value::Unit);
                    println!("{}", val);
                }
                OpPop => {
                    self.stack.pop();
                }
                OpRet => {
                    let val = if !self.stack.is_empty() {
                        self.stack.pop().unwrap()
                    } else {
                        Value::Unit
                    };
                    self.status = CoStatus::Done;
                    return Ok(val);
                }
            }
        }
        Ok(Value::Unit)
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - distance - 1]
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
                arity, args_len
            ))
        } else {
            Ok(())
        }
    }

    fn check_uni_operands(&self) -> Result<(), String> {
        let val = self.peek(0);
        if !val.is_num() {
            Err("operand must be a number".to_owned())
        } else {
            Ok(())
        }
    }

    fn check_bin_operands(&self) -> Result<(), String> {
        let lhs = self.peek(1);
        let rhs = self.peek(0);
        if !lhs.is_num() || !rhs.is_num() {
            Err("operands must be numbers".to_owned())
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
                let param = self.fun.param(i).clone();
                self.env.insert(param, arg);
            }
        } else {
            // At most one value (unit if none), and we push this onto the stack.
            let arg = if !args.is_empty() {
                self.check_arity(1, args.len())?;
                args.into_iter().next().unwrap()
            } else {
                Value::Unit
            };
            self.stack.push(arg);
        }
        Ok(())
    }
}
