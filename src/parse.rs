use pest::Parser;

#[derive(Parser)]
#[grammar = "coro.pest"]
struct PEGParser;

pub struct CoParser;

impl CoParser {
    pub fn parse(src: &str) {
        let res = PEGParser::parse(Rule::program, src);
        println!("{:?}", res);
    }
}
