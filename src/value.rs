#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Bool(bool),
    Num(f64),
    Str(String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unit, Self::Unit) => true,
            (Self::Bool(b1), Self::Bool(b2)) => b1 == b2,
            (Self::Num(n1), Self::Num(n2)) => n1 == n2,
            (Self::Str(s1), Self::Str(s2)) => s1 == s2,
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

    pub fn is_num(self) -> bool {
        matches!(self, Self::Num(..))
    }

    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(..))
    }

    pub fn print(&self) {
        match self {
            Self::Unit => print!("unit"),
            Self::Bool(b) => print!("{}", b),
            Self::Num(n) => print!("{}", n),
            Self::Str(s) => print!("{}", s),
        }
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
        assert_eq!(Value::Unit, Value::Unit);
        assert_ne!(Value::Unit, Value::Bool(true));

        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_ne!(Value::Bool(true), Value::Bool(false));
        assert_ne!(Value::Bool(true), Value::Num(1.2));

        assert_eq!(Value::Num(1.2), Value::Num(1.2));
        assert_ne!(Value::Num(1.2), Value::Bool(true));
        assert_ne!(Value::Num(1.2), Value::Str("foo".to_owned()));

        assert_eq!(Value::Str("foo".to_owned()), Value::Str("foo".to_owned()));
        assert_ne!(Value::Str("foo".to_owned()), Value::Str("bar".to_owned()));
        assert_ne!(Value::Str("foo".to_owned()), Value::Bool(true));
    }
}
