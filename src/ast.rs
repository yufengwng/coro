#[derive(Debug)]
pub struct Ast {
    pub items: Vec<Bind>,
}

impl Ast {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

#[derive(Debug)]
pub enum Bind {
    Def(DefBind),
    Let(LetBind),
    Cmd(Cmd),
}

#[derive(Debug)]
pub struct DefBind {
    pub name: String,
    pub params: Vec<String>,
    pub body: Cmd,
}

impl DefBind {
    pub fn new(name: String, params: Vec<String>, body: Cmd) -> Self {
        Self { name, params, body }
    }
}

#[derive(Debug)]
pub struct LetBind {
    pub name: String,
    pub init: Cmd,
}

impl LetBind {
    pub fn new(name: String, init: Cmd) -> Self {
        Self { name, init }
    }
}

#[derive(Debug)]
pub enum Cmd {
    Print(Expr),
    Create(String),
    Resume(Expr, Vec<Expr>),
    Yield(Expr),
    While(Expr, Expr),
    If(Expr, Expr, Expr),
    Expr(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Lt(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Block(Vec<Bind>),
    Group(Box<Cmd>),
    Ident(String),
    Bool(bool),
    Num(f64),
    Str(String),
    Unit,
}
