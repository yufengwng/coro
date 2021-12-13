use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Instr {
    /// Push a unit value onto stack.
    OpUnit,
    /// Push a true value onto stack.
    OpTrue,
    /// Push a false value onto stack.
    OpFalse,
    /// (idx) Lookup constant value using `idx` and push onto stack.
    OpConst(usize),
    /// Pop 2 operands and push sum onto stack.
    OpAdd,
    /// Pop 2 operands and push difference onto stack.
    OpSub,
    /// Pop 2 operands and push product onto stack.
    OpMul,
    /// Pop 2 operands and push quotient onto stack.
    OpDiv,
    /// Pop an operand and push its numeric negation onto stack.
    OpNeg,
    /// Pop an operand and push its boolean negation onto stack.
    OpNot,
    /// Pop 2 operands, compare less, and push boolean onto stack.
    OpLt,
    /// Pop 2 operands, compare equals, and push boolean onto stack.
    OpEq,
    /// (offset) Jump backwards with `offset` amount of instructions.
    OpLoop(usize),
    /// (offset) Jump forwards with `offset` amount of instructions.
    OpJump(usize),
    /// (offset) Conditional forward jump if top of stack is false.
    OpBranch(usize),
    /// (idx) Lookup name using `idx` and push onto stack the value bound in env.
    OpLoad(usize),
    /// (idx) Lookup name using `idx`, write top of stack to env, and push unit onto stack.
    OpStore(usize),
    /// (idx) Lookup function using `idx`, write to env, and push unit onto stack.
    OpDefine(usize),
    /// (idx) Lookup name of function using `idx`, and push a new coroutine onoto stack.
    OpCreate(usize),
    /// (num) Resume coroutine using `num` arguments from stack. Returned/yielded value will be top of stack.
    OpResume(usize),
    /// Suspend current coroutine and yield top of stack.
    OpYield,
    /// Pop top of stack, print value, and push unit onto stack.
    OpPrint,
    /// Pop the top of stack.
    OpPop,
    /// Exit coroutine, and return top of stack or unit.
    OpRet,
}

pub struct Code {
    instrs: Vec<Instr>,
    consts: Vec<Value>,
    lines: Vec<usize>,
}

impl Code {
    pub fn new() -> Self {
        Self {
            instrs: Vec::new(),
            consts: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.instrs.len()
    }

    pub fn line(&self, idx: usize) -> usize {
        self.lines[idx]
    }

    pub fn instr(&self, idx: usize) -> &Instr {
        &self.instrs[idx]
    }

    pub fn constant(&self, idx: usize) -> &Value {
        &self.consts[idx]
    }

    pub fn add(&mut self, instr: Instr, line: usize) -> usize {
        let idx = self.instrs.len();
        self.instrs.push(instr);
        self.lines.push(line);
        idx
    }

    pub fn add_const(&mut self, value: Value) -> usize {
        let idx = self.consts.len();
        for (i, val) in self.consts.iter().enumerate() {
            if val == &value {
                return i;
            }
        }
        self.consts.push(value);
        idx
    }

    pub fn patch(&mut self, idx: usize, instr: Instr) {
        self.instrs[idx] = instr;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_add_returns_index() {
        let mut code = Code::new();
        assert_eq!(0, code.add(Instr::OpUnit, 1));
        assert_eq!(1, code.add(Instr::OpTrue, 1));
        assert_eq!(2, code.add(Instr::OpFalse, 1));
        assert_eq!(3, code.len());
    }

    #[test]
    fn code_add_const_returns_index() {
        let mut code = Code::new();
        assert_eq!(0, code.add_const(Value::Unit));
        assert_eq!(1, code.add_const(Value::Bool(true)));
        assert_eq!(2, code.consts.len());
    }

    #[test]
    fn code_add_const_stores_unique() {
        let mut code = Code::new();
        assert_eq!(0, code.add_const(Value::Str("foo".to_owned())));
        assert_eq!(0, code.add_const(Value::Str("foo".to_owned())));
        assert_eq!(1, code.add_const(Value::Str("bar".to_owned())));
        assert_eq!(2, code.consts.len());
    }
}
