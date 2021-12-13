use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::code::Code;
use crate::vm::Coro;

#[derive(Clone)]
pub enum Value {
    Unit,
    Bool(bool),
    Num(f64),
    Str(String),
    Fn(Rc<FnDef>),
    Co(Rc<RefCell<Coro>>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Num(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "\"{}\"", s),
            Self::Fn(def) => def.fmt(f),
            Self::Co(coro) => coro.borrow().fmt(f),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unit, Self::Unit) => true,
            (Self::Bool(b1), Self::Bool(b2)) => b1 == b2,
            (Self::Num(n1), Self::Num(n2)) => n1 == n2,
            (Self::Str(s1), Self::Str(s2)) => s1 == s2,
            (Self::Fn(f1), Self::Fn(f2)) => Rc::ptr_eq(f1, f2),
            (Self::Co(c1), Self::Co(c2)) => Rc::ptr_eq(c1, c2),
            _ => false,
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Self::Unit => true,
            Self::Bool(b) => !b,
            _ => false,
        }
    }

    pub fn is_num(&self) -> bool {
        matches!(self, Self::Num(..))
    }

    pub fn as_num(self) -> f64 {
        match self {
            Self::Num(n) => n,
            _ => panic!(),
        }
    }

    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(..))
    }

    pub fn is_fn(&self) -> bool {
        matches!(self, Self::Fn(..))
    }

    pub fn is_co(&self) -> bool {
        matches!(self, Self::Co(..))
    }
}

pub struct FnDef {
    name: String,
    pub params: Vec<String>,
    pub code: Code,
}

impl fmt::Display for FnDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn name:{} arity:{}>", self.name(), self.arity())
    }
}

impl FnDef {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            params: Vec::new(),
            code: Code::new(),
        }
    }

    pub fn name(&self) -> &str {
        if self.name.is_empty() {
            "__main__"
        } else {
            &self.name
        }
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn false_values() {
        assert!(Value::Unit.is_falsey());
        assert!(Value::Bool(false).is_falsey());
        assert_eq!(false, Value::Bool(true).is_falsey());
        assert_eq!(false, Value::Num(1.2).is_falsey());
        assert_eq!(false, Value::Str("foo".to_owned()).is_falsey());
    }

    #[test]
    fn num_values() {
        assert!(Value::Num(2.3).is_num());
        assert_eq!(false, Value::Unit.is_num());
    }

    #[test]
    fn str_values() {
        assert!(Value::Str("foo".to_owned()).is_str());
        assert_eq!(false, Value::Unit.is_str());
    }

    #[test]
    fn equality() {
        assert!(Value::Unit == Value::Unit);
        assert!(Value::Unit != Value::Bool(true));

        assert!(Value::Bool(true) == Value::Bool(true));
        assert!(Value::Bool(true) != Value::Bool(false));
        assert!(Value::Bool(true) != Value::Num(1.2));

        assert!(Value::Num(1.2) == Value::Num(1.2));
        assert!(Value::Num(1.2) != Value::Bool(true));
        assert!(Value::Num(1.2) != Value::Str("foo".to_owned()));

        assert!(Value::Str("foo".to_owned()) == Value::Str("foo".to_owned()));
        assert!(Value::Str("foo".to_owned()) != Value::Str("bar".to_owned()));
        assert!(Value::Str("foo".to_owned()) != Value::Bool(true));
    }
}
