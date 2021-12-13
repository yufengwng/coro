use crate::code::Code;
use crate::code::Instr::*;

pub fn print(code: &Code, name: &str) {
    eprintln!("== {} ==", name);
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
        OpConst(const_idx) => {
            let val = code.constant(const_idx);
            eprintln!("{:?} {}", instr, val);
        }
        _ => eprintln!("{:?}", instr),
    }
}
