use crate::value::Value;

#[derive(Debug)]
pub enum Instr {
    OpUnit,
    OpTrue,
    OpFalse,
    OpReturn,
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
