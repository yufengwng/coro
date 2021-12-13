use crate::ast::*;
use crate::code::Code;
use crate::code::Instr::*;
use crate::value::Value;

pub struct CoGen {}

impl CoGen {
    pub fn compile(ast: Ast) -> Code {
        let mut code = Code::new();
        for bind in ast.items {
            emit_bind(&mut code, bind);
        }
        code
    }
}

fn emit_bind(code: &mut Code, bind: Bind) {
    match bind {
        Bind::Def(def_bind) => emit_def(code, def_bind),
        Bind::Let(let_bind) => emit_let(code, let_bind),
        Bind::Cmd(cmd) => emit_cmd(code, cmd),
    }
}

fn emit_let(code: &mut Code, let_bind: LetBind) {
    todo!()
}

fn emit_def(code: &mut Code, def_bind: DefBind) {
    todo!()
}

fn emit_cmd(code: &mut Code, cmd: Cmd) {
    match cmd {
        Cmd::Print(expr) => {
            emit_expr(code, expr);
            code.add(OpPrint, 1);
            // stack + 0
        }
        Cmd::Create(name) => todo!(),
        Cmd::Resume(expr, args) => todo!(),
        Cmd::Yield(expr) => todo!(),
        Cmd::While(cond, body) => todo!(),
        Cmd::If(cond, then, alt) => todo!(),
        Cmd::Expr(expr) => {
            emit_expr(code, expr);
        }
    }
}

fn emit_expr(code: &mut Code, expr: Expr) {
    match expr {
        Expr::Block(binds) => todo!(),
        Expr::Group(inner) => {
            emit_cmd(code, *inner);
            // stack + 1
        }
        Expr::Ident(name) => todo!(),
        Expr::Lt(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpLt, 1);
            // stack - 1
        }
        Expr::Eq(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpEq, 1);
            // stack - 1
        }
        Expr::Add(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpAdd, 1);
            // stack - 1
        }
        Expr::Sub(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpSub, 1);
            // stack - 1
        }
        Expr::Mul(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpMul, 1);
            // stack - 1
        }
        Expr::Div(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpDiv, 1);
            // stack - 1
        }
        Expr::Neg(inner) => {
            emit_expr(code, *inner);
            code.add(OpNeg, 1);
            // stack + 0
        }
        Expr::Not(inner) => {
            emit_expr(code, *inner);
            code.add(OpNot, 1);
            // stack + 0
        }
        Expr::Bool(lit) => {
            let instr = if lit { OpTrue } else { OpFalse };
            code.add(instr, 1);
            // stack + 1
        }
        Expr::Num(lit) => {
            let val = Value::Num(lit);
            emit_const(code, val);
            // stack + 1
        }
        Expr::Str(lit) => {
            let val = Value::Str(lit);
            emit_const(code, val);
            // stack + 1
        }
        Expr::Unit => {
            code.add(OpUnit, 1);
            // stack + 1
        }
    }
}

fn emit_const(code: &mut Code, value: Value) {
    let idx = code.add_const(value);
    let instr = OpConst(idx);
    code.add(instr, 1);
}
