pub enum CoRes {
    Ok,
    CompileErr,
    RuntimeErr,
}

pub struct CoVM;

impl CoVM {
    pub fn run(&mut self, _src: &str) -> CoRes {
        CoRes::Ok
    }
}
