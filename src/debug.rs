use crate::code::Code;

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
    eprintln!("{:?}", code.instr(idx));
}
