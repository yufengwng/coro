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
            // stack + 1
        }
        Cmd::Create(name) => todo!(),
        Cmd::Resume(expr, args) => todo!(),
        Cmd::Yield(expr) => todo!(),
        Cmd::While(cond, body) => {
            emit_while(code, cond, body);
            // stack + 1
        }
        Cmd::If(cond, then, alt) => {
            emit_if(code, cond, then, alt);
            // stack + 1
        }
        Cmd::Expr(expr) => {
            emit_expr(code, expr);
            // stack + 1
        }
    }
}

fn emit_while(code: &mut Code, cond: Expr, body: Expr) {
    let cond_idx = code.len();
    emit_expr(code, cond);
    let exit_idx = code.add(OpBranch(0), 1);

    // If cond is true, then pop cond value and do body-expr.
    code.add(OpPop, 1);
    emit_expr(code, body);
    // Discard the value produced by body-expr.
    code.add(OpPop, 1);
    // Loop back up to the cond.
    emit_loop(code, cond_idx);

    // If cond is false, then we jump down here to the pop.
    patch_branch(code, exit_idx);
    code.add(OpPop, 1);

    // `while` produces a unit value.
    code.add(OpUnit, 1);
}

fn emit_if(code: &mut Code, cond: Expr, then: Expr, alt: Expr) {
    emit_expr(code, cond);
    let then_idx = code.add(OpBranch(0), 1);

    // If cond is true, then pop cond value and do then-expr.
    code.add(OpPop, 1);
    emit_expr(code, then);
    // Once then-expr is done, skip over the else-expr.
    let exit_idx = code.add(OpJump(0), 1);

    // If cond is false, then we jump down here to else-expr's pop.
    patch_branch(code, then_idx);
    code.add(OpPop, 1);
    emit_expr(code, alt);

    // The skip will come down here.
    patch_jump(code, exit_idx);

    // No pop since `if` produces a value.
}

fn emit_loop(code: &mut Code, target_idx: usize) {
    // IP will point to next instr, so need one more when going backward.
    let offset = code.len() - target_idx + 1;
    code.add(OpLoop(offset), 1);
}

fn patch_jump(code: &mut Code, idx: usize) {
    backpatch(code, idx, true);
}

fn patch_branch(code: &mut Code, idx: usize) {
    backpatch(code, idx, false);
}

fn backpatch(code: &mut Code, idx: usize, is_jump: bool) {
    // IP will point to next instr, so do one less when going forward.
    let offset = code.len() - idx - 1;
    let instr = if is_jump {
        OpJump(offset)
    } else {
        OpBranch(offset)
    };
    code.patch(idx, instr);
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
            // stack + 1
        }
        Expr::Eq(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpEq, 1);
            // stack + 1
        }
        Expr::Add(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpAdd, 1);
            // stack + 1
        }
        Expr::Sub(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpSub, 1);
            // stack + 1
        }
        Expr::Mul(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpMul, 1);
            // stack + 1
        }
        Expr::Div(lhs, rhs) => {
            emit_expr(code, *lhs);
            emit_expr(code, *rhs);
            code.add(OpDiv, 1);
            // stack + 1
        }
        Expr::Neg(inner) => {
            emit_expr(code, *inner);
            code.add(OpNeg, 1);
            // stack + 1
        }
        Expr::Not(inner) => {
            emit_expr(code, *inner);
            code.add(OpNot, 1);
            // stack + 1
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
