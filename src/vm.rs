use crate::code::Code;
use crate::code::Instr::*;
use crate::debug;

pub enum CoRes {
    Ok,
    CompileErr,
    RuntimeErr,
}

pub struct CoVM;

impl CoVM {
    pub fn run(&mut self, _src: &str) -> CoRes {
        if cfg!(feature = "debug") {
            let mut code = Code::new();
            code.add(OpUnit, 1);
            code.add(OpTrue, 1);
            code.add(OpFalse, 2);
            debug::print(&code, "code");
        }
        CoRes::Ok
    }
}
