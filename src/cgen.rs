//! This module is for the code generator (codegen) that translates the high-level AST into
//! lower-level "linear instructions".
//!
//! The main task here is to traverse the syntax tree and "compile" each item to corresponding
//! instructions. We keep things simple by focusing on individual items at a time to ensure we
//! get the semantics correct. Therefore, we assume the AST is correct and type-checks.

use std::rc::Rc;

use crate::ast::*;
use crate::code::Code;
use crate::code::Instr::*;
use crate::debug;
use crate::value::FnDef;
use crate::value::Value;

/// Main entry point to compiling AST to instructions.
pub fn compile(ast: Ast) -> Code {
    let mut code = Code::new();
    if !ast.items.is_empty() {
        // An AST is mostly just a block.
        emit_block(&mut code, ast.items);
        code.add(OpRet, 1);
    }
    code
}

fn emit_block(code: &mut Code, block: Vec<Bind>) {
    let len = block.len();
    let mut iter = block.into_iter();

    for _ in 0..(len - 1) {
        // Compile and discard the value of each item except the last.
        let bind = iter.next().unwrap();
        emit_bind(code, bind);
        code.add(OpPop, 1);
    }

    // Block should have at least one item.
    let last = iter.next().unwrap();
    emit_bind(code, last);

    // Last value produced is the value of the block, so no pop.
}

fn emit_bind(code: &mut Code, bind: Bind) {
    match bind {
        Bind::Def(def_bind) => {
            emit_def(code, def_bind);
            // stack + 1
        }
        Bind::Let(let_bind) => {
            emit_let(code, let_bind);
            // stack + 1
        }
        Bind::Cmd(cmd) => {
            emit_cmd(code, cmd);
            // stack + 1
        }
    }
}

fn emit_def(code: &mut Code, def_bind: DefBind) {
    let mut def = FnDef::with(def_bind.name, def_bind.params);
    emit_cmd(&mut def.code, def_bind.body);
    def.code.add(OpRet, 1);

    if cfg!(feature = "instr") {
        debug::print(&def.code, def.name());
    }

    let val = Value::Fn(Rc::new(def));
    let idx = code.add_const(val);

    code.add(OpDefine(idx), 1);
}

fn emit_let(code: &mut Code, let_bind: LetBind) {
    emit_cmd(code, let_bind.init);
    let name = Value::Str(let_bind.name);
    let idx = code.add_const(name);
    code.add(OpStore(idx), 1);
}

fn emit_cmd(code: &mut Code, cmd: Cmd) {
    match cmd {
        Cmd::Print(expr) => {
            emit_expr(code, expr);
            code.add(OpPrint, 1);
            // stack + 1
        }
        Cmd::Create(name) => {
            emit_create(code, name);
            // stack + 1
        }
        Cmd::Resume(expr, args) => {
            emit_resume(code, expr, args);
            // stack + 1
        }
        Cmd::Yield(expr) => {
            emit_yield(code, expr);
            // stack + 1
        }
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

fn emit_create(code: &mut Code, name: String) {
    let name = Value::Str(name);
    let idx = code.add_const(name);
    code.add(OpCreate(idx), 1);
}

fn emit_resume(code: &mut Code, expr: Expr, args: Vec<Expr>) {
    emit_expr(code, expr);
    let num = args.len();
    for arg in args {
        emit_expr(code, arg);
    }
    code.add(OpResume(num), 1);
}

fn emit_yield(code: &mut Code, expr: Expr) {
    emit_expr(code, expr);
    code.add(OpYield, 1);
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
        Expr::Block(binds) => {
            emit_block(code, binds);
            // stack + 1
        }
        Expr::Group(inner) => {
            emit_cmd(code, *inner);
            // stack + 1
        }
        Expr::Ident(name) => {
            let name = Value::Str(name);
            let idx = code.add_const(name);
            code.add(OpLoad(idx), 1);
            // stack + 1
        }
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
