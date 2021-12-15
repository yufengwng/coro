//! A module that provides helper to debug Coro.

use crate::code::Code;
use crate::code::Instr::*;

pub fn print(code: &Code, name: &str) {
    eprintln!("== instr: {} ==", name);
    let mut idx = 0;
    while idx < code.len() {
        print_instr(code, idx);
        idx += 1;
    }
}

pub fn print_instr(code: &Code, idx: usize) {
    // index
    eprint!("{:04} ", idx);

    // line number
    if idx > 0 && code.line(idx) == code.line(idx - 1) {
        eprint!("   | ");
    } else {
        eprint!("{:4} ", code.line(idx));
    }

    // instruction
    let instr = code.instr(idx).clone();
    match instr {
        OpConst(idx) => {
            let val = code.constant(idx);
            eprintln!("{:?} {:?}", instr, val);
        }
        OpLoad(idx) => {
            let name = code.constant(idx);
            eprintln!("{:?} {:?}", instr, name);
        }
        OpStore(idx) => {
            let name = code.constant(idx);
            eprintln!("{:?} {:?}", instr, name);
        }
        OpDefine(idx) => {
            let def = code.constant(idx);
            eprintln!("{:?} {:?}", instr, def);
        }
        OpCreate(idx) => {
            let ident = code.constant(idx);
            eprintln!("{:?} {:?}", instr, ident);
        }
        _ => eprintln!("{:?}", instr),
    }
}
